use std::{fmt, io::Write};

use crate::{
    args::Config,
    gui_parser::{
        edi_reply, pbc_reply, pdr_reply, pex_reply, pfk_reply, pgt_reply, pic_reply, pie_reply,
        pin_reply, ppo_reply,
    },
    map::{self, Egg, Resources, bct_reply, enw_string},
    server::{Client, ClientState, Game, Orientation, Player},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Forward,
    Right,
    Left,
    Look,
    Inventory,
    Broadcast(String),
    ConnectNbr,
    Fork,
    Eject,
    Take(String),
    Set(String),
    Incantation,
}

impl Command {
    /// Parse a command from a string
    /// Returns (Command, time_cost) or error message
    pub fn parse(
        input: &str,
        list: &Vec<&Player>,
        idx: usize,
        source: &Player,
        clients: &mut [Client],
        game: &Game,
    ) -> Result<Self, String> {
        let trimmed = input.trim();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();

        if parts.is_empty() {
            return Err("empty command".to_string());
        }

        match parts[0] {
            "Forward" => Ok(Command::Forward),
            "Right" => Ok(Command::Right),
            "Left" => Ok(Command::Left),
            "Look" => Ok(Command::Look),
            "Inventory" => Ok(Command::Inventory),
            "Broadcast" => {
                if parts.len() < 2 {
                    return Err("Broadcast requires a message".to_string());
                }
                let message = parts[1..].join(" ");
                Ok(Command::Broadcast(message))
            }
            "Connect_nbr" => Ok(Command::ConnectNbr),
            "Fork" => Ok(Command::Fork),
            "Eject" => Ok(Command::Eject),
            "Take" => {
                if parts.len() < 2 {
                    return Err("Take requires an object".to_string());
                }
                let object = parts[1..].join(" ");
                Ok(Command::Take(object))
            }
            "Set" => {
                if parts.len() < 2 {
                    return Err("Set requires an object".to_string());
                }
                let object = parts[1..].join(" ");
                Ok(Command::Set(object))
            }
            "Incantation" => {
                send_string_to_all_guis(clients, pic_reply(source, list));
                let (x, y, level) = match &clients[idx].state {
                    ClientState::Ai { player, .. } => {
                        (player.x as usize, player.y as usize, player.level)
                    }
                    _ => return Err("ko\n".to_string()),
                };

                if requirements_for(level).is_none() {
                    // Already at max level (8), or invalid level.
                    return Err("ko\n".to_string());
                }

                if !meets_requirements(clients, &game.map[y][x], level) {
                    return Err("ko\n".to_string());
                }
                Ok(Command::Incantation)
            }
            _ => Err(format!("unknown command: {}", parts[0])),
        }
    }

    /// Get the base execution time for this command (in deciseconds)
    pub fn execution_time(&self) -> u32 {
        match self {
            Command::Forward => 7,
            Command::Right => 7,
            Command::Left => 7,
            Command::Look => 7,
            Command::Inventory => 1,
            Command::Broadcast(_) => 7,
            Command::Fork => 42,
            Command::Eject => 7,
            Command::Take(_) => 7,
            Command::Set(_) => 7,
            Command::Incantation => 300,
            Command::ConnectNbr => 0,
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Forward => write!(f, "Forward"),
            Command::Right => write!(f, "Right"),
            Command::Left => write!(f, "Left"),
            Command::Look => write!(f, "Look"),
            Command::Inventory => write!(f, "Inventory"),
            Command::Broadcast(msg) => write!(f, "Broadcast {}", msg),
            Command::ConnectNbr => write!(f, "Connect_nbr"),
            Command::Fork => write!(f, "Fork"),
            Command::Eject => write!(f, "Eject"),
            Command::Take(obj) => write!(f, "Take {}", obj),
            Command::Set(obj) => write!(f, "Set {}", obj),
            Command::Incantation => write!(f, "Incantation"),
        }
    }
}

fn convert_ressource_name_to_id(string: &str) -> u8 {
    match string {
        "food" => 0,
        "linemate" => 1,
        "deraumere" => 2,
        "sibur" => 3,
        "mendiane" => 4,
        "phiras" => 5,
        "thystame" => 6,
        // Any other (invalid) object name — kept distinct from food so that
        // `Take <garbage>` / `Set <garbage>` fall through to `ko`.
        _ => u8::MAX,
    }
}

pub fn send_string_to_all_guis(clients: &mut [Client], msg: String) {
    for client in clients.iter_mut() {
        match client.state {
            ClientState::Gui => {
                let _ = client.stream.write_all(msg.as_bytes()).is_err();
                continue;
            }
            _ => continue,
        }
    }
}

pub fn send_ai_command_to_gui(
    cmd: Command,
    player: &Player,
    incantation_success: bool,
    clients: &mut [Client],
    game: &Game,
) {
    match cmd {
        Command::Forward | Command::Left | Command::Right => {
            send_string_to_all_guis(clients, ppo_reply(player))
        }
        Command::Eject => send_string_to_all_guis(clients, pex_reply(player)),
        Command::Broadcast(string) => send_string_to_all_guis(clients, pbc_reply(player, &string)),
        Command::Incantation => {
            send_string_to_all_guis(clients, pie_reply(player.x, player.y, incantation_success))
        }
        Command::Fork => {
            send_string_to_all_guis(clients, pfk_reply(player));
            // Notify the GUI of the egg this fork just laid (highest id among this
            // player's eggs) rather than assuming it is the last egg pushed.
            if let Some(egg) = game
                .eggs
                .iter()
                .filter(|e| e.player_id == player.id)
                .max_by_key(|e| e.id)
            {
                send_string_to_all_guis(clients, enw_string(egg));
            }
        }
        Command::Take(string) => {
            send_string_to_all_guis(
                clients,
                pgt_reply(player, convert_ressource_name_to_id(&string)),
            );
            send_string_to_all_guis(clients, pin_reply(player));
            let string = bct_reply(player.x, player.y, &game.map, &game.config);
            if let Some(content) = string {
                send_string_to_all_guis(clients, content);
            }
        }
        Command::Set(string) => {
            send_string_to_all_guis(
                clients,
                pdr_reply(player, convert_ressource_name_to_id(&string)),
            );
            send_string_to_all_guis(clients, pin_reply(player));
            let string = bct_reply(player.x, player.y, &game.map, &game.config);
            if let Some(content) = string {
                send_string_to_all_guis(clients, content);
            }
        }
        _ => (),
    }
}

pub fn take(client: &mut Client, game: &mut Game, resource: &str) -> String {
    let player = match &mut client.state {
        ClientState::Ai { player, .. } => player,
        _ => return "ko\n".to_string(),
    };
    let resource_id = convert_ressource_name_to_id(resource);
    match resource_id {
        1 => {
            if game.map[player.y as usize][player.x as usize]
                .resources
                .linemate
                == 0
            {
                return "ko\n".to_string();
            }
            game.map[player.y as usize][player.x as usize]
                .resources
                .linemate -= 1;
            player.inventory.linemate += 1;
            "ok\n".to_string()
        }
        2 => {
            if game.map[player.y as usize][player.x as usize]
                .resources
                .deraumere
                == 0
            {
                return "ko\n".to_string();
            }
            game.map[player.y as usize][player.x as usize]
                .resources
                .deraumere -= 1;
            player.inventory.deraumere += 1;
            "ok\n".to_string()
        }
        3 => {
            if game.map[player.y as usize][player.x as usize]
                .resources
                .sibur
                == 0
            {
                return "ko\n".to_string();
            }
            game.map[player.y as usize][player.x as usize]
                .resources
                .sibur -= 1;
            player.inventory.sibur += 1;
            "ok\n".to_string()
        }
        4 => {
            if game.map[player.y as usize][player.x as usize]
                .resources
                .mendiane
                == 0
            {
                return "ko\n".to_string();
            }
            game.map[player.y as usize][player.x as usize]
                .resources
                .mendiane -= 1;
            player.inventory.mendiane += 1;
            "ok\n".to_string()
        }
        5 => {
            if game.map[player.y as usize][player.x as usize]
                .resources
                .phiras
                == 0
            {
                return "ko\n".to_string();
            }
            game.map[player.y as usize][player.x as usize]
                .resources
                .phiras -= 1;
            player.inventory.phiras += 1;
            "ok\n".to_string()
        }
        6 => {
            if game.map[player.y as usize][player.x as usize]
                .resources
                .thystame
                == 0
            {
                return "ko\n".to_string();
            }
            game.map[player.y as usize][player.x as usize]
                .resources
                .thystame -= 1;
            player.inventory.thystame += 1;
            "ok\n".to_string()
        }
        0 => {
            // Food is collectable too: move one unit from the tile into the
            // player's food reserve (extends survival).
            if game.map[player.y as usize][player.x as usize].food == 0 {
                return "ko\n".to_string();
            }
            game.map[player.y as usize][player.x as usize].food -= 1;
            player.food += 1;
            "ok\n".to_string()
        }
        _ => "ko\n".to_string(),
    }
}

pub fn set(client: &mut Client, game: &mut Game, resource: &str) -> String {
    let player = match &mut client.state {
        ClientState::Ai { player, .. } => player,
        _ => return "ko\n".to_string(),
    };
    let resource_id = convert_ressource_name_to_id(resource);
    match resource_id {
        1 => {
            if player.inventory.linemate == 0 {
                return "ko\n".to_string();
            }
            game.map[player.y as usize][player.x as usize]
                .resources
                .linemate += 1;
            player.inventory.linemate -= 1;
            "ok\n".to_string()
        }
        2 => {
            if player.inventory.deraumere == 0 {
                return "ko\n".to_string();
            }
            game.map[player.y as usize][player.x as usize]
                .resources
                .deraumere += 1;
            player.inventory.deraumere -= 1;
            "ok\n".to_string()
        }
        3 => {
            if player.inventory.sibur == 0 {
                return "ko\n".to_string();
            }
            game.map[player.y as usize][player.x as usize]
                .resources
                .sibur += 1;
            player.inventory.sibur -= 1;
            "ok\n".to_string()
        }
        4 => {
            if player.inventory.mendiane == 0 {
                return "ko\n".to_string();
            }
            game.map[player.y as usize][player.x as usize]
                .resources
                .mendiane += 1;
            player.inventory.mendiane -= 1;
            "ok\n".to_string()
        }
        5 => {
            if player.inventory.phiras == 0 {
                return "ko\n".to_string();
            }
            game.map[player.y as usize][player.x as usize]
                .resources
                .phiras += 1;
            player.inventory.phiras -= 1;
            "ok\n".to_string()
        }
        6 => {
            if player.inventory.thystame == 0 {
                return "ko\n".to_string();
            }
            game.map[player.y as usize][player.x as usize]
                .resources
                .thystame += 1;
            player.inventory.thystame -= 1;
            "ok\n".to_string()
        }
        0 => {
            // Drop one unit of food from the player's reserve onto the tile.
            if player.food == 0 {
                return "ko\n".to_string();
            }
            game.map[player.y as usize][player.x as usize].food += 1;
            player.food -= 1;
            "ok\n".to_string()
        }
        _ => "ko\n".to_string(),
    }
}

pub fn fork(client: &mut Client, game: &mut Game, team_slots: &mut Vec<(String, u32)>) -> String {
    let player = match &client.state {
        ClientState::Ai { player, .. } => player,
        _ => return "ko\n".to_string(),
    };
    let team = match &client.state {
        ClientState::Ai { team, .. } => team,
        _ => return "ko\n".to_string(),
    };

    game.eggs.push(Egg {
        x: player.x,
        y: player.y,
        id: game.next_egg_id,
        player_id: player.id,
        is_starting: false,
        team: team.clone(),
    });
    game.next_egg_id += 1;
    if let Some(slot) = team_slots.iter_mut().find(|(name, _)| name == team) {
        slot.1 += 1;
    } else {
        team_slots.push((team.to_string(), 1));
    }
    "ok\n".to_string()
}

pub fn eject(
    clients: &mut [Client],
    idx: u32,
    game: &mut Game,
    config: &Config,
    team_slots: &mut [(String, u32)],
) -> String {
    let (ejector_id, ejector_x, ejector_y, dest_x, dest_y) = {
        let player = match &clients[idx as usize].state {
            ClientState::Ai { player, .. } => player,
            _ => return "ko\n".to_string(),
        };

        let orientation = &player.orientation;
        let dx = match orientation {
            Orientation::West => {
                if player.x == 0 {
                    config.width - 1
                } else {
                    player.x - 1
                }
            }
            Orientation::East => {
                if player.x >= config.width - 1 {
                    0
                } else {
                    player.x + 1
                }
            }
            _ => player.x,
        };
        let dy = match orientation {
            Orientation::North => {
                if player.y == 0 {
                    config.height - 1
                } else {
                    player.y - 1
                }
            }
            Orientation::South => {
                if player.y >= config.height - 1 {
                    0
                } else {
                    player.y + 1
                }
            }
            _ => player.y,
        };

        (player.id, player.x, player.y, dx, dy)
    };

    game.map[ejector_y as usize][ejector_x as usize]
        .ais
        .retain(|&id| id == ejector_id);
    for client in clients.iter_mut() {
        let player_to_move = match &mut client.state {
            ClientState::Ai { player, .. } => player,
            _ => continue,
        };
        if player_to_move.id != ejector_id
            && player_to_move.x == ejector_x
            && player_to_move.y == ejector_y
        {
            player_to_move.x = dest_x;
            player_to_move.y = dest_y;
            game.map[dest_y as usize][dest_x as usize]
                .ais
                .push(player_to_move.id);
        }
    }

    let mut to_remove = Vec::<u32>::new();

    for (i, egg) in game.eggs.iter().enumerate() {
        if egg.x == ejector_x && egg.y == ejector_y {
            send_string_to_all_guis(clients, edi_reply(egg));
            if let Some(slot) = team_slots.iter_mut().find(|(name, _)| *name == egg.team) {
                slot.1 -= 1;
            }
            to_remove.push(i as u32);
        }
    }

    for i in to_remove.iter().rev() {
        game.eggs.remove(*i as usize);
    }

    "ok\n".to_string()
}

struct ElevationRequirement {
    nb_players: usize,
    resources: Resources,
}

const ELEVATION_REQUIREMENTS: [ElevationRequirement; 7] = [
    ElevationRequirement {
        nb_players: 1,
        resources: Resources {
            linemate: 1,
            deraumere: 0,
            sibur: 0,
            mendiane: 0,
            phiras: 0,
            thystame: 0,
        },
    },
    ElevationRequirement {
        nb_players: 2,
        resources: Resources {
            linemate: 1,
            deraumere: 1,
            sibur: 1,
            mendiane: 0,
            phiras: 0,
            thystame: 0,
        },
    },
    ElevationRequirement {
        nb_players: 2,
        resources: Resources {
            linemate: 2,
            deraumere: 0,
            sibur: 1,
            mendiane: 0,
            phiras: 2,
            thystame: 0,
        },
    },
    ElevationRequirement {
        nb_players: 4,
        resources: Resources {
            linemate: 1,
            deraumere: 1,
            sibur: 2,
            mendiane: 0,
            phiras: 1,
            thystame: 0,
        },
    },
    ElevationRequirement {
        nb_players: 4,
        resources: Resources {
            linemate: 1,
            deraumere: 2,
            sibur: 1,
            mendiane: 3,
            phiras: 0,
            thystame: 0,
        },
    },
    ElevationRequirement {
        nb_players: 6,
        resources: Resources {
            linemate: 1,
            deraumere: 2,
            sibur: 3,
            mendiane: 0,
            phiras: 1,
            thystame: 0,
        },
    },
    ElevationRequirement {
        nb_players: 6,
        resources: Resources {
            linemate: 2,
            deraumere: 2,
            sibur: 2,
            mendiane: 2,
            phiras: 2,
            thystame: 1,
        },
    },
];

fn requirements_for(level: u8) -> Option<&'static ElevationRequirement> {
    if level == 0 || level as usize > ELEVATION_REQUIREMENTS.len() {
        return None;
    }
    Some(&ELEVATION_REQUIREMENTS[(level - 1) as usize])
}

fn players_of_level_on_tile(clients: &[Client], tile: &map::Tile, level: u8) -> Vec<u32> {
    tile.ais
        .iter()
        .copied()
        .filter(|id| {
            clients.iter().any(|c| match &c.state {
                ClientState::Ai { player, .. } => player.id == *id && player.level == level,
                _ => false,
            })
        })
        .collect()
}

fn meets_requirements(clients: &[Client], tile: &map::Tile, level: u8) -> bool {
    let Some(req) = requirements_for(level) else {
        return false;
    };

    let present = players_of_level_on_tile(clients, tile, level).len();

    present >= req.nb_players
        && tile.resources.linemate >= req.resources.linemate
        && tile.resources.deraumere >= req.resources.deraumere
        && tile.resources.sibur >= req.resources.sibur
        && tile.resources.mendiane >= req.resources.mendiane
        && tile.resources.phiras >= req.resources.phiras
        && tile.resources.thystame >= req.resources.thystame
}

fn consume_resources(tile: &mut map::Tile, level: u8) {
    if let Some(req) = requirements_for(level) {
        tile.resources.linemate -= req.resources.linemate;
        tile.resources.deraumere -= req.resources.deraumere;
        tile.resources.sibur -= req.resources.sibur;
        tile.resources.mendiane -= req.resources.mendiane;
        tile.resources.phiras -= req.resources.phiras;
        tile.resources.thystame -= req.resources.thystame;
    }
}

pub fn incantation(clients: &mut [Client], idx: usize, map: &mut [Vec<map::Tile>]) -> String {
    let (x, y, level, initiator_id) = match &clients[idx].state {
        ClientState::Ai { player, .. } => (
            player.x as usize,
            player.y as usize,
            player.level,
            player.id,
        ),
        _ => return "ko\n".to_string(),
    };

    if requirements_for(level).is_none() {
        // Already at max level (8), or invalid level.
        return "ko\n".to_string();
    }

    let participant_ids = players_of_level_on_tile(clients, &map[y][x], level);

    if !meets_requirements(clients, &map[y][x], level) {
        return "ko\n".to_string();
    }

    consume_resources(&mut map[y][x], level);

    let new_level = level + 1;
    let msg = format!("Current level: {new_level}\n");
    for id in &participant_ids {
        if let Some(c) = clients.iter_mut().find(|c| match &c.state {
            ClientState::Ai { player, .. } => player.id == *id,
            _ => false,
        }) {
            if let ClientState::Ai { player, .. } = &mut c.state {
                player.level += 1;
            }
            if *id != initiator_id {
                let _ = c.stream.write_all(msg.as_bytes());
            }
        }
    }
    msg
}

pub fn move_forward(client: &mut Client, config: &Config, game: &mut Game) -> String {
    let old_x;
    let old_y;
    let player_id;
    let new_x;
    let new_y;
    match &mut client.state {
        ClientState::Ai { player, .. } => {
            old_x = player.x;
            old_y = player.y;
            player_id = player.id;
            match player.orientation {
                Orientation::North => {
                    if player.y == 0 {
                        player.y = config.height - 1;
                    } else {
                        player.y -= 1;
                    }
                }
                Orientation::East => player.x = (player.x + 1) % config.width,
                Orientation::South => player.y = (player.y + 1) % config.height,
                Orientation::West => {
                    if player.x == 0 {
                        player.x = config.width - 1;
                    } else {
                        player.x -= 1;
                    }
                }
            }
            new_x = player.x;
            new_y = player.y;
        }
        _ => return "ko\n".to_string(),
    }
    game.map[old_y as usize][old_x as usize]
        .ais
        .retain(|x| *x != player_id);
    game.map[new_y as usize][new_x as usize].ais.push(player_id);
    "ok\n".to_string()
}

pub fn turn_right(client: &mut Client) -> String {
    match &mut client.state {
        ClientState::Ai { player, .. } => {
            player.orientation = match player.orientation {
                Orientation::East => Orientation::South,
                Orientation::North => Orientation::East,
                Orientation::South => Orientation::West,
                Orientation::West => Orientation::North,
            }
        }
        _ => return "ko\n".to_string(),
    }
    "ok\n".to_string()
}

pub fn turn_left(client: &mut Client) -> String {
    match &mut client.state {
        ClientState::Ai { player, .. } => {
            player.orientation = match player.orientation {
                Orientation::East => Orientation::North,
                Orientation::North => Orientation::West,
                Orientation::South => Orientation::East,
                Orientation::West => Orientation::South,
            }
        }
        _ => return "ko\n".to_string(),
    }
    "ok\n".to_string()
}

fn forward(o: &Orientation) -> (i32, i32) {
    match o {
        Orientation::North => (0, -1),
        Orientation::East => (1, 0),
        Orientation::South => (0, 1),
        Orientation::West => (-1, 0),
    }
}

fn right(o: &Orientation) -> (i32, i32) {
    match o {
        Orientation::North => (1, 0),
        Orientation::East => (0, 1),
        Orientation::South => (-1, 0),
        Orientation::West => (0, -1),
    }
}

fn wrap(v: i32, max: i32) -> usize {
    (((v % max) + max) % max) as usize
}

fn tile_content(tile: &map::Tile) -> String {
    let mut items: Vec<&str> = Vec::new();

    items.extend(std::iter::repeat_n("player", tile.ais.len()));
    items.extend(std::iter::repeat_n("food", tile.food as usize));
    items.extend(std::iter::repeat_n(
        "linemate",
        tile.resources.linemate as usize,
    ));
    items.extend(std::iter::repeat_n(
        "deraumere",
        tile.resources.deraumere as usize,
    ));
    items.extend(std::iter::repeat_n("sibur", tile.resources.sibur as usize));
    items.extend(std::iter::repeat_n(
        "mendiane",
        tile.resources.mendiane as usize,
    ));
    items.extend(std::iter::repeat_n(
        "phiras",
        tile.resources.phiras as usize,
    ));
    items.extend(std::iter::repeat_n(
        "thystame",
        tile.resources.thystame as usize,
    ));

    items.join(" ")
}

pub fn look(client: &Client, map: &[Vec<map::Tile>]) -> String {
    let player = match &client.state {
        ClientState::Ai { player, .. } => player,
        _ => return "ko\n".to_string(),
    };
    let height = map.len() as i32;
    let width = if height > 0 { map[0].len() as i32 } else { 0 };
    if height == 0 || width == 0 {
        return "ko\n".to_string();
    }

    let view_distance = player.level as i32;

    let px = player.x as i32;
    let py = player.y as i32;

    let (fx, fy) = forward(&player.orientation);
    let (rx, ry) = right(&player.orientation);

    let mut cells: Vec<String> = Vec::new();
    {
        let x = wrap(px, width);
        let y = wrap(py, height);
        cells.push(tile_content(&map[y][x]));
    }
    for d in 1..=view_distance {
        for offset in -d..=d {
            let tx = px + fx * d + rx * offset;
            let ty = py + fy * d + ry * offset;
            let x = wrap(tx, width);
            let y = wrap(ty, height);
            cells.push(tile_content(&map[y][x]));
        }
    }

    format!("[ {} ]\n", cells.join(", "))
}

pub fn get_inventory(client: &Client) -> String {
    match &client.state {
        ClientState::Ai { player, .. } => {
            let mut inventory_string = String::new();
            inventory_string.push_str(&format!(
                "[food {}, linemate {}, deraumere {}, sibur {}, mendiane {}, phiras {}, thystame {}]\n",
                player.food,
                player.inventory.linemate,
                player.inventory.deraumere,
                player.inventory.sibur,
                player.inventory.mendiane,
                player.inventory.phiras,
                player.inventory.thystame
            ));
            inventory_string
        }
        _ => "ko\n".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_id_mapping_food_vs_invalid() {
        // food is object id 0; the six stones are 1..=6.
        assert_eq!(convert_ressource_name_to_id("food"), 0);
        assert_eq!(convert_ressource_name_to_id("linemate"), 1);
        assert_eq!(convert_ressource_name_to_id("thystame"), 6);
        // An invalid name must map to the sentinel u8::MAX, not collapse onto
        // food's id, otherwise `Take <garbage>` would pick food up off the tile.
        assert_eq!(convert_ressource_name_to_id("banana"), u8::MAX);
    }

    #[test]
    fn parse_simple_commands() {
        assert_eq!(
            Command::parse(
                "Forward",
                &Vec::new(),
                0,
                &Player::default(),
                &mut Vec::new(),
                &Game::default()
            )
            .unwrap(),
            Command::Forward
        );
        assert_eq!(
            Command::parse(
                "Right",
                &Vec::new(),
                0,
                &Player::default(),
                &mut Vec::new(),
                &Game::default()
            )
            .unwrap(),
            Command::Right
        );
        assert_eq!(
            Command::parse(
                "Left",
                &Vec::new(),
                0,
                &Player::default(),
                &mut Vec::new(),
                &Game::default()
            )
            .unwrap(),
            Command::Left
        );
        assert_eq!(
            Command::parse(
                "Look",
                &Vec::new(),
                0,
                &Player::default(),
                &mut Vec::new(),
                &Game::default()
            )
            .unwrap(),
            Command::Look
        );
        assert_eq!(
            Command::parse(
                "Inventory",
                &Vec::new(),
                0,
                &Player::default(),
                &mut Vec::new(),
                &Game::default()
            )
            .unwrap(),
            Command::Inventory
        );
        assert_eq!(
            Command::parse(
                "Fork",
                &Vec::new(),
                0,
                &Player::default(),
                &mut Vec::new(),
                &Game::default()
            )
            .unwrap(),
            Command::Fork
        );
        assert_eq!(
            Command::parse(
                "Eject",
                &Vec::new(),
                0,
                &Player::default(),
                &mut Vec::new(),
                &Game::default()
            )
            .unwrap(),
            Command::Eject
        );
    }

    #[test]
    fn parse_commands_with_args() {
        assert_eq!(
            Command::parse(
                "Broadcast hello world",
                &Vec::new(),
                0,
                &Player::default(),
                &mut Vec::new(),
                &Game::default()
            )
            .unwrap(),
            Command::Broadcast("hello world".to_string())
        );
        assert_eq!(
            Command::parse(
                "Take linemate",
                &Vec::new(),
                0,
                &Player::default(),
                &mut Vec::new(),
                &Game::default()
            )
            .unwrap(),
            Command::Take("linemate".to_string())
        );
        assert_eq!(
            Command::parse(
                "Set food",
                &Vec::new(),
                0,
                &Player::default(),
                &mut Vec::new(),
                &Game::default()
            )
            .unwrap(),
            Command::Set("food".to_string())
        );
    }

    #[test]
    fn parse_whitespace_handling() {
        assert_eq!(
            Command::parse(
                "  Forward  ",
                &Vec::new(),
                0,
                &Player::default(),
                &mut Vec::new(),
                &Game::default()
            )
            .unwrap(),
            Command::Forward
        );
        assert_eq!(
            Command::parse(
                "Broadcast   message with spaces",
                &Vec::new(),
                0,
                &Player::default(),
                &mut Vec::new(),
                &Game::default()
            )
            .unwrap(),
            Command::Broadcast("message with spaces".to_string())
        );
    }

    #[test]
    fn parse_invalid_commands() {
        assert!(
            Command::parse(
                "Unknown",
                &Vec::new(),
                0,
                &Player::default(),
                &mut Vec::new(),
                &Game::default()
            )
            .is_err()
        );
        assert!(
            Command::parse(
                "",
                &Vec::new(),
                0,
                &Player::default(),
                &mut Vec::new(),
                &Game::default()
            )
            .is_err()
        );
        assert!(
            Command::parse(
                "Broadcast",
                &Vec::new(),
                0,
                &Player::default(),
                &mut Vec::new(),
                &Game::default()
            )
            .is_err()
        );
        assert!(
            Command::parse(
                "Take",
                &Vec::new(),
                0,
                &Player::default(),
                &mut Vec::new(),
                &Game::default()
            )
            .is_err()
        );
    }

    #[test]
    fn execution_times() {
        assert_eq!(Command::Forward.execution_time(), 7);
        assert_eq!(Command::Inventory.execution_time(), 1);
        assert_eq!(Command::Fork.execution_time(), 42);
        assert_eq!(Command::Incantation.execution_time(), 300);
    }

    #[test]
    fn command_display() {
        assert_eq!(format!("{}", Command::Forward), "Forward");
        assert_eq!(
            format!("{}", Command::Broadcast("test".to_string())),
            "Broadcast test"
        );
    }
}
