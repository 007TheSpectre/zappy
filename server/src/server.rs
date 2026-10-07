use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::fd::AsRawFd;
use std::os::raw::{c_int, c_short, c_ulong};

use crate::args::Config;
use crate::command::{
    Command, eject, fork, get_inventory, incantation, look, move_forward, send_ai_command_to_gui,
    send_string_to_all_guis, set, take, turn_left, turn_right,
};
use crate::gui_parser::{ebo_reply, pdi_reply, pie_reply, pin_reply, player_spawn, seg_reply};
use crate::map::{Egg, Tile};
use crate::map::{Resources, create_gui_connection_string};
use crate::time_manager::TimeManager;

type Nfds = c_ulong;

const POLLIN: c_short = 0x0001;
const POLLERR: c_short = 0x0008;
const POLLHUP: c_short = 0x0010;
const POLLNVAL: c_short = 0x0020;

#[repr(C)]
struct PollFd {
    fd: c_int,
    events: c_short,
    revents: c_short,
}

unsafe extern "C" {
    fn poll(fds: *mut PollFd, nfds: Nfds, timeout: c_int) -> c_int;
}

#[derive(Clone, Default)]
pub enum Orientation {
    #[default]
    North = 1,
    East = 2,
    South = 3,
    West = 4,
}

#[derive(Clone, Default)]
pub struct Player {
    pub x: u32,
    pub y: u32,
    pub id: u32,
    pub orientation: Orientation,
    pub level: u8,
    pub inventory: Resources,
    pub food: u32,
    pub time_left: u8,
}

#[derive(Clone)]
pub enum ClientState {
    AwaitingTeamName,
    AwaitingEgg { team: String },
    Ai { team: String, player: Player },
    Gui,
}

pub struct Client {
    id: u32,
    pub stream: TcpStream,
    addr: SocketAddr,
    pub state: ClientState,
    pending: String,
}

#[derive(Default)]
pub struct Game {
    pub map: Vec<Vec<Tile>>,
    pub config: Config,
    pub eggs: Vec<Egg>,
    pub next_egg_id: u32,
    pub next_player_id: u32,
}

pub struct Server {
    listener: TcpListener,
    clients: Vec<Client>,
    team_slots: Vec<(String, u32)>,
    time_manager: TimeManager,
    next_client_id: u32,
    pub game_over: bool,
}

pub struct Ready {
    pub accept: bool,
    pub readable: Vec<u32>,
    pub disconnected: Vec<u32>,
}

pub struct ReadOutcome {
    pub disconnected: Vec<u32>,
}

pub fn start(config: Config) -> io::Result<Server> {
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr)?;
    println!("Listening on {addr}");

    let team_slots = config
        .teams
        .into_iter()
        .map(|name| (name, config.clients_per_team))
        .collect();

    let time_manager = TimeManager::new(config.frequency);

    Ok(Server {
        listener,
        clients: Vec::new(),
        team_slots,
        time_manager,
        next_client_id: 0,
        game_over: false,
    })
}

impl Server {
    pub fn next_poll_timeout_ms(&self) -> c_int {
        self.time_manager.next_timeout_ms().unwrap_or(-1)
    }

    /// Propagate a runtime frequency change (e.g. from a GUI `sst`) to command
    /// scheduling, keeping execution speed in sync with the food/spawn cadence.
    pub fn set_frequency(&mut self, frequency: u32) {
        self.time_manager.set_frequency(frequency);
    }

    pub fn execute_ready_commands(&mut self, config: &Config, game: &mut Game) {
        let ready = self.time_manager.tick();
        for scheduled in ready {
            if command_logging_enabled() {
                println!(
                    "EXECUTE: client {} - {}",
                    scheduled.client_id, scheduled.command
                );
            }

            let idx = match self
                .clients
                .iter()
                .position(|c| c.id == scheduled.client_id)
            {
                Some(i) => i,
                None => {
                    eprintln!("Client {} not found for execution", scheduled.client_id);
                    continue;
                }
            };

            let mut incantation_result = false;

            let response = match &scheduled.command {
                Command::Forward => move_forward(&mut self.clients[idx], config, game),
                Command::Right => turn_right(&mut self.clients[idx]),
                Command::Left => turn_left(&mut self.clients[idx]),
                Command::Look => look(&self.clients[idx], &game.map),
                Command::Inventory => get_inventory(&self.clients[idx]),
                Command::ConnectNbr => {
                    if let ClientState::Ai { ref team, .. } = self.clients[idx].state {
                        if let Some(slot) = self.team_slots.iter().find(|(name, _)| name == team) {
                            format!("{}\n", slot.1)
                        } else {
                            "0\n".to_string()
                        }
                    } else {
                        "0\n".to_string()
                    }
                }
                Command::Take(resource) => take(&mut self.clients[idx], game, resource),
                Command::Set(resource) => set(&mut self.clients[idx], game, resource),
                Command::Fork => fork(&mut self.clients[idx], game, &mut self.team_slots),
                Command::Eject => eject(
                    &mut self.clients,
                    idx as u32,
                    game,
                    config,
                    &mut self.team_slots,
                ),
                Command::Broadcast(msg) => {
                    let (ex, ey, eid) = match &self.clients[idx].state {
                        ClientState::Ai { player, .. } => (player.x, player.y, player.id),
                        _ => continue,
                    };
                    let _ = self.clients[idx].stream.write_all(b"ok\n");
                    let pbc = format!("pbc #{eid} {msg}\n");
                    let w = config.width;
                    let h = config.height;
                    for other in self.clients.iter_mut() {
                        let line = match &other.state {
                            // The emitter never receives its own broadcast.
                            ClientState::Ai { player, .. } if player.id == eid => None,
                            ClientState::Ai { player, .. } => {
                                let k = broadcast_k(
                                    ex,
                                    ey,
                                    player.x,
                                    player.y,
                                    &player.orientation,
                                    w,
                                    h,
                                );
                                Some(format!("message {k}, {msg}\n"))
                            }
                            ClientState::Gui => Some(pbc.clone()),
                            _ => None,
                        };
                        if let Some(l) = line {
                            let _ = other.stream.write_all(l.as_bytes());
                        }
                    }
                    continue;
                }
                Command::Incantation => {
                    let string = incantation(&mut self.clients, idx, &mut game.map);
                    if string != "ko\n" {
                        incantation_result = true;
                    }
                    string
                }
            };

            let _ = self.clients[idx].stream.write_all(response.as_bytes());

            if let ClientState::Ai { player, .. } = &self.clients[idx].state
                && (response != "ko\n" || scheduled.command == Command::Incantation)
            {
                let player = player.clone();
                send_ai_command_to_gui(
                    scheduled.command,
                    &player,
                    incantation_result,
                    &mut self.clients,
                    game,
                );
            }
        }
    }

    pub fn check_win_condition(&self) -> Option<String> {
        use std::collections::HashMap;
        let mut team_level8: HashMap<&str, u32> = HashMap::new();
        for client in &self.clients {
            if let ClientState::Ai { team, player } = &client.state
                && player.level >= 8
            {
                *team_level8.entry(team.as_str()).or_insert(0) += 1;
            }
        }
        team_level8
            .into_iter()
            .find(|(_, count)| *count >= 6)
            .map(|(team, _)| team.to_string())
    }

    pub fn end_game(&mut self, winning_team: &str) {
        let msg = seg_reply(winning_team);
        send_string_to_all_guis(&mut self.clients, msg);
        self.game_over = true;
        println!("Game over — {winning_team} wins!");
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn poll(&self, timeout_ms: c_int) -> io::Result<Ready> {
        let mut fds = Vec::with_capacity(self.clients.len() + 1);
        fds.push(PollFd {
            fd: self.listener.as_raw_fd(),
            events: POLLIN | POLLERR | POLLHUP | POLLNVAL,
            revents: 0,
        });

        for client in &self.clients {
            fds.push(PollFd {
                fd: client.stream.as_raw_fd(),
                events: POLLIN | POLLERR | POLLHUP | POLLNVAL,
                revents: 0,
            });
        }

        loop {
            let result = unsafe { poll(fds.as_mut_ptr(), fds.len() as Nfds, timeout_ms) };
            if result >= 0 {
                break;
            }

            let err = io::Error::last_os_error();
            if err.kind() == io::ErrorKind::Interrupted {
                continue;
            }

            return Err(err);
        }

        let mut readable = Vec::new();
        let mut disconnected = Vec::new();

        for (index, fd) in fds.iter().enumerate().skip(1) {
            let revents = fd.revents;
            let index = (index - 1) as u32;

            if revents & (POLLERR | POLLHUP | POLLNVAL) != 0 {
                disconnected.push(index);
                continue;
            }

            if revents & POLLIN != 0 {
                readable.push(index);
            }
        }

        Ok(Ready {
            accept: fds[0].revents & POLLIN != 0,
            readable,
            disconnected,
        })
    }

    pub fn accept(&mut self, ready: &Ready) -> io::Result<()> {
        if !ready.accept {
            return Ok(());
        }

        let (mut stream, addr) = self.listener.accept()?;
        stream.write_all(b"WELCOME\n")?;
        println!("Client connected: {addr}");

        let client_id = self.next_client_id;
        self.next_client_id += 1;
        self.clients.push(Client {
            id: client_id,
            stream,
            addr,
            state: ClientState::AwaitingTeamName,
            pending: String::new(),
        });

        Ok(())
    }

    pub fn read(&mut self, ready: &Ready, game: &mut Game) -> io::Result<ReadOutcome> {
        let mut disconnected = ready.disconnected.clone();
        let mut buffer = [0u8; 1024];

        for &index in &ready.readable {
            if index >= self.clients.len() as u32 {
                disconnected.push(index);
                continue;
            }

            let client_idx = index as usize;
            let client_id = self.clients[client_idx].id;
            let drop = Self::read_client(
                &mut self.clients,
                &mut buffer,
                &mut self.team_slots,
                game,
                &mut self.time_manager,
                client_idx,
                client_id,
            );

            if drop {
                disconnected.push(index);
            }
        }

        Ok(ReadOutcome { disconnected })
    }

    pub fn cleanup(&mut self, disconnected: impl IntoIterator<Item = u32>) {
        let mut disconnected = disconnected.into_iter().collect::<Vec<_>>();
        disconnected.sort_unstable();
        disconnected.dedup();

        for index in disconnected.into_iter().rev() {
            if index < self.clients.len() as u32 {
                let client = self.clients.swap_remove(index as usize);
                match &client.state {
                    ClientState::Ai { player, .. } => {
                        // Active/dead AI keeps its slot consumed (a player existed).
                        send_string_to_all_guis(&mut self.clients, pdi_reply(player));
                    }
                    ClientState::AwaitingEgg { team } => {
                        // Reserved a slot but never hatched — return it to the team.
                        if let Some(slot) =
                            self.team_slots.iter_mut().find(|(name, _)| name == team)
                        {
                            slot.1 += 1;
                        }
                    }
                    _ => {}
                }
                println!("Client disconnected: {}", client.addr);
            }
        }
    }

    pub fn decrement_food(&mut self) {
        let mut to_remove: Vec<usize> = Vec::new();
        let mut pin_notification: Vec<String> = Vec::new();
        let mut death_notifications: Vec<String> = Vec::new();

        for (i, client) in self.clients.iter_mut().enumerate() {
            if let ClientState::Ai { ref mut player, .. } = client.state {
                player.time_left = player.time_left.saturating_sub(1);
                if player.time_left == 0 {
                    player.food = player.food.saturating_sub(1);
                    pin_notification.push(pin_reply(player));
                    if player.food == 0 {
                        println!("Client {} starved", client.addr);
                        death_notifications.push(pdi_reply(player));
                        let _ = client.stream.write_all("dead\n".as_bytes());
                        let _ = client.stream.shutdown(std::net::Shutdown::Both);
                        to_remove.push(i);
                    } else {
                        player.time_left = 126;
                    }
                }
            }
        }

        for msg in pin_notification {
            send_string_to_all_guis(&mut self.clients, msg);
        }

        for msg in death_notifications {
            send_string_to_all_guis(&mut self.clients, msg);
        }

        for i in to_remove.into_iter().rev() {
            self.clients.remove(i);
        }
    }

    fn read_client(
        clients: &mut [Client],
        buffer: &mut [u8; 1024],
        team_slots: &mut [(String, u32)],
        game: &mut Game,
        time_manager: &mut TimeManager,
        client_idx: usize,
        client_id: u32,
    ) -> bool {
        let n = match clients[client_idx].stream.read(buffer) {
            Ok(0) => {
                println!(
                    "Client {} sent EOF (closed connection)",
                    clients[client_idx].addr
                );
                return true;
            }
            Ok(n) => n,
            Err(err) => {
                eprintln!("Read error from {}: {err}", clients[client_idx].addr);
                return true;
            }
        };

        clients[client_idx]
            .pending
            .push_str(&String::from_utf8_lossy(&buffer[..n]));

        while let Some(pos) = clients[client_idx].pending.find('\n') {
            let line = clients[client_idx].pending[..pos]
                .trim_end_matches('\r')
                .to_string();
            clients[client_idx].pending.drain(..=pos);

            if Self::handle_line(
                clients,
                &line,
                team_slots,
                game,
                time_manager,
                client_idx,
                client_id,
            ) {
                return true;
            }
        }

        false
    }

    fn handle_line(
        clients: &mut [Client],
        line: &str,
        team_slots: &mut [(String, u32)],
        game: &mut Game,
        time_manager: &mut TimeManager,
        client_idx: usize,
        client_id: u32,
    ) -> bool {
        let state = clients[client_idx].state.clone();
        match state {
            ClientState::AwaitingTeamName => {
                Self::handle_team_name(clients, client_idx, line, team_slots, game)
            }
            ClientState::Ai { player, .. } => Self::handle_ai_command(
                clients,
                line,
                time_manager,
                client_idx,
                client_id,
                &player,
                game,
            ),
            // Client is waiting for an egg to be laid — ignore all input until hatched
            ClientState::AwaitingEgg { .. } => false,
            ClientState::Gui => Self::handle_gui_command(clients, client_idx, line, game),
        }
    }

    fn handle_gui_command(
        clients: &mut [Client],
        client_idx: usize,
        line: &str,
        game: &mut Game,
    ) -> bool {
        let trimmed = line.trim();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();

        if parts.is_empty() {
            eprintln!(
                "Parse error from {}: empty command",
                clients[client_idx].addr
            );
            return clients[client_idx]
                .stream
                .write_all("ko\n".as_bytes())
                .is_err();
        }
        // ppo/plv/pin look up the target player by logical ID, then reply to the
        // requesting GUI by its current vec index (ids are not vec indices after disconnects).
        let gui_idx = client_idx as u32;
        match parts[0] {
            "msz" => Self::msz_parse(parts, &mut clients[client_idx], game),
            "bct" => Self::bct_parse(parts, &mut clients[client_idx], game),
            "mct" => Self::mct_parse(parts, &mut clients[client_idx], game),
            "tna" => Self::tna_parse(parts, &mut clients[client_idx], game),
            "ppo" => Self::ppo_parse(parts, clients, gui_idx),
            "plv" => Self::plv_parse(parts, clients, gui_idx),
            "pin" => Self::pin_parse(parts, clients, gui_idx),
            "sgt" => Self::sgt_parse(parts, &mut clients[client_idx], game),
            "sst" => Self::sst_parse(parts, &mut clients[client_idx], game),
            _ => Self::bad_command(parts, &mut clients[client_idx]),
        }
    }

    fn handle_ai_command(
        clients: &mut [Client],
        line: &str,
        time_manager: &mut TimeManager,
        client_idx: usize,
        client_id: u32,
        player: &Player,
        game: &Game,
    ) -> bool {
        // Enforce the per-client command buffer cap *before* parsing, so a command
        // we are going to drop never produces side effects (e.g. parsing an
        // Incantation emits a GUI `pic`). Dropped silently, matching the protocol.
        if time_manager.is_client_queue_full(client_id) {
            if command_logging_enabled() {
                eprintln!(
                    "AI command from {} dropped: command buffer full (max {})",
                    clients[client_idx].addr,
                    crate::time_manager::MAX_PENDING_PER_CLIENT
                );
            }
            return false;
        }

        // Only `Command::parse`'s Incantation branch needs the full player list
        // (to build the GUI `pic`). Build it lazily for that one command instead
        // of deep-cloning every player on every single command line.
        let owned_players: Vec<Player> = if line.split_whitespace().next() == Some("Incantation") {
            clients
                .iter()
                .filter_map(|c| match &c.state {
                    ClientState::Ai { player, .. } => Some(player.clone()),
                    _ => None,
                })
                .collect()
        } else {
            Vec::new()
        };
        let list: Vec<&Player> = owned_players.iter().collect();
        let addr = clients[client_idx].addr;

        match Command::parse(line, &list, client_idx, player, clients, game) {
            Ok(cmd) => {
                let execute_at = time_manager.schedule(cmd.clone(), client_id);
                if command_logging_enabled() {
                    println!(
                        "AI command from {}: {} (execute at {:?})",
                        addr, cmd, execute_at
                    );
                }
                false
            }
            Err(err) => {
                if err == "ko\n" {
                    send_string_to_all_guis(clients, pie_reply(player.x, player.y, false));
                } else {
                    eprintln!("Parse error from {}: {}", addr, err);
                }
                clients[client_idx].stream.write_all(b"ko\n").is_err()
            }
        }
    }

    fn handle_team_name(
        clients: &mut [Client],
        idx: usize,
        name: &str,
        team_slots: &mut [(String, u32)],
        game: &mut Game,
    ) -> bool {
        if name == "GRAPHIC" {
            clients[idx].state = ClientState::Gui;
            println!("{} identified as GUI", clients[idx].addr);
            match create_gui_connection_string(game, clients) {
                Some(message) => {
                    return clients[idx].stream.write_all(message.as_bytes()).is_err();
                }
                None => return false,
            }
        }

        let slot = team_slots
            .iter_mut()
            .find(|(team, slots)| team == name && *slots > 0);

        match slot {
            Some(entry) => {
                entry.1 -= 1;
                let remaining = entry.1;

                if let Some(egg_idx) = game.eggs.iter().position(|egg| egg.team == name) {
                    // An egg is available: hatch it immediately
                    let egg = game.eggs.swap_remove(egg_idx);

                    let spawn_id = game.next_player_id;
                    clients[idx].state = ClientState::Ai {
                        team: name.to_string(),
                        player: Player {
                            x: egg.x,
                            y: egg.y,
                            id: spawn_id,
                            orientation: Orientation::North,
                            level: 1,
                            inventory: Resources {
                                linemate: 0,
                                deraumere: 0,
                                sibur: 0,
                                mendiane: 0,
                                phiras: 0,
                                thystame: 0,
                            },
                            food: 9,
                            time_left: 126,
                        },
                    };
                    game.map[egg.y as usize][egg.x as usize].ais.push(spawn_id);
                    game.next_player_id += 1;

                    println!(
                        "{} hatched egg at ({}, {}) for team {name} ({remaining} slots left)",
                        clients[idx].addr, egg.x, egg.y
                    );
                    let string = player_spawn(&mut clients[idx]);
                    send_string_to_all_guis(clients, string);
                    send_string_to_all_guis(clients, ebo_reply(&egg));
                } else {
                    // No egg yet: park the client until one is laid
                    clients[idx].state = ClientState::AwaitingEgg {
                        team: name.to_string(),
                    };
                    println!(
                        "{} waiting for an egg for team {name} ({remaining} slots left)",
                        clients[idx].addr
                    );
                }

                let response = format!(
                    "{remaining}\n{} {}\n",
                    game.config.width, game.config.height
                );
                clients[idx].stream.write_all(response.as_bytes()).is_err()
            }
            None => {
                let response = "ko\n".to_string();
                clients[idx].stream.write_all(response.as_bytes()).is_err()
            }
        }
    }
}

/// Per-command logging (`EXECUTE` / `AI command`) is the server's biggest source
/// of stdout I/O at high frequency. Setting `ZAPPY_QUIET=1` suppresses just those
/// two lines (read once); win / hatch / starve / disconnect logging is unaffected.
/// Default is unchanged (logging on) so this is a pure opt-in perf toggle.
fn command_logging_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("ZAPPY_QUIET").is_none())
}

fn broadcast_k(
    ex: u32,
    ey: u32,
    rx: u32,
    ry: u32,
    recv_orientation: &Orientation,
    width: u32,
    height: u32,
) -> u8 {
    let w = width as i32;
    let h = height as i32;
    let mut dx = (ex as i32 - rx as i32).rem_euclid(w);
    if dx > w / 2 {
        dx -= w;
    }
    let mut dy = (ey as i32 - ry as i32).rem_euclid(h);
    if dy > h / 2 {
        dy -= h;
    }
    if dx == 0 && dy == 0 {
        return 0;
    }
    // (forward_x, forward_y, left_x, left_y) in screen coords (y grows down)
    let (fx, fy, lx, ly): (i32, i32, i32, i32) = match recv_orientation {
        Orientation::North => (0, -1, -1, 0),
        Orientation::East => (1, 0, 0, -1),
        Orientation::South => (0, 1, 1, 0),
        Orientation::West => (-1, 0, 0, 1),
    };
    let f = dx * fx + dy * fy;
    let l = dx * lx + dy * ly;
    match (f > 0, f < 0, l > 0, l < 0) {
        (true, false, false, false) => 1, // front
        (true, false, true, false) => 2,  // front-left
        (false, false, true, false) => 3, // left
        (false, true, true, false) => 4,  // back-left
        (false, true, false, false) => 5, // back
        (false, true, false, true) => 6,  // back-right
        (false, false, false, true) => 7, // right
        (true, false, false, true) => 8,  // front-right
        _ => 0,
    }
}
