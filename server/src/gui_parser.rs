use crate::{
    map::{Egg, bct_reply, mct_reply, msz_reply, sgt_reply, tna_reply},
    server::{Client, ClientState, Game, Orientation, Player, Server},
};
use std::io::Write;

impl Server {
    fn gui_error(client: &mut Client, err: &str) -> bool {
        let _ = client.stream.write_all(err.as_bytes()).is_err();
        false
    }

    pub fn msz_parse(parts: Vec<&str>, client: &mut Client, game: &Game) -> bool {
        if parts.len() > 1 {
            return Self::gui_error(client, "sbp\n");
        }
        client
            .stream
            .write_all(msz_reply(&game.config).as_bytes())
            .is_err()
    }

    pub fn bct_parse(parts: Vec<&str>, client: &mut Client, game: &Game) -> bool {
        if parts.len() != 3 || parts[1].parse::<u32>().is_err() || parts[2].parse::<u32>().is_err()
        {
            return Self::gui_error(client, "sbp\n");
        }
        match bct_reply(
            parts[1].parse::<u32>().unwrap(),
            parts[2].parse::<u32>().unwrap(),
            &game.map,
            &game.config,
        ) {
            Some(string) => client.stream.write_all(string.as_bytes()).is_err(),
            None => Self::gui_error(client, "smg Failed to create bct reply\n"),
        }
    }

    pub fn mct_parse(parts: Vec<&str>, client: &mut Client, game: &Game) -> bool {
        if parts.len() > 1 {
            return Self::gui_error(client, "sbp\n");
        }
        match mct_reply(&game.config, &game.map) {
            Some(string) => client.stream.write_all(string.as_bytes()).is_err(),
            None => {
                let _ = client
                    .stream
                    .write_all("smg Failed to create mct reply\n".as_bytes())
                    .is_err();
                false
            }
        }
    }

    pub fn tna_parse(parts: Vec<&str>, client: &mut Client, game: &Game) -> bool {
        if parts.len() > 1 {
            return Self::gui_error(client, "sbp\n");
        }
        client
            .stream
            .write_all(tna_reply(&game.config).as_bytes())
            .is_err()
    }

    pub fn ppo_parse(parts: Vec<&str>, clients: &mut [Client], client_id: u32) -> bool {
        if parts.len() != 2 || !parts[1].starts_with('#') {
            return Self::gui_error(&mut clients[client_id as usize], "sbp\n");
        }
        let player_id = match parts[1][1..].parse::<u32>() {
            Ok(id) => id,
            Err(_) => return Self::gui_error(&mut clients[client_id as usize], "sbp\n"),
        };
        let reply = clients.iter().find_map(|c| match &c.state {
            ClientState::Ai { player, .. } if player.id == player_id => Some(ppo_reply(player)),
            _ => None,
        });
        match reply {
            Some(msg) => clients[client_id as usize]
                .stream
                .write_all(msg.as_bytes())
                .is_err(),
            None => Self::gui_error(&mut clients[client_id as usize], "sbp\n"),
        }
    }

    pub fn plv_parse(parts: Vec<&str>, clients: &mut [Client], client_id: u32) -> bool {
        if parts.len() != 2 || !parts[1].starts_with('#') {
            return Self::gui_error(&mut clients[client_id as usize], "sbp\n");
        }
        let player_id = match parts[1][1..].parse::<u32>() {
            Ok(id) => id,
            Err(_) => return Self::gui_error(&mut clients[client_id as usize], "sbp\n"),
        };
        let reply = clients.iter().find_map(|c| match &c.state {
            ClientState::Ai { player, .. } if player.id == player_id => Some(plv_reply(player)),
            _ => None,
        });
        match reply {
            Some(msg) => clients[client_id as usize]
                .stream
                .write_all(msg.as_bytes())
                .is_err(),
            None => Self::gui_error(&mut clients[client_id as usize], "sbp\n"),
        }
    }

    pub fn pin_parse(parts: Vec<&str>, clients: &mut [Client], client_id: u32) -> bool {
        if parts.len() != 2 || !parts[1].starts_with('#') {
            return Self::gui_error(&mut clients[client_id as usize], "sbp\n");
        }
        let player_id = match parts[1][1..].parse::<u32>() {
            Ok(id) => id,
            Err(_) => return Self::gui_error(&mut clients[client_id as usize], "sbp\n"),
        };
        let reply = clients.iter().find_map(|c| match &c.state {
            ClientState::Ai { player, .. } if player.id == player_id => Some(pin_reply(player)),
            _ => None,
        });
        match reply {
            Some(msg) => clients[client_id as usize]
                .stream
                .write_all(msg.as_bytes())
                .is_err(),
            None => Self::gui_error(&mut clients[client_id as usize], "sbp\n"),
        }
    }

    pub fn sgt_parse(parts: Vec<&str>, client: &mut Client, game: &Game) -> bool {
        if parts.len() > 1 {
            return Self::gui_error(client, "sbp\n");
        }
        client
            .stream
            .write_all(sgt_reply(&game.config).as_bytes())
            .is_err()
    }

    pub fn sst_parse(parts: Vec<&str>, client: &mut Client, game: &mut Game) -> bool {
        // Frequency must be a strictly positive integer: the main loop and the
        // time manager divide tick/command durations by it, so 0 would panic the
        // whole server. Reject bad arity, non-numeric, or zero with `sbp`.
        if parts.len() != 2 {
            return Self::gui_error(client, "sbp\n");
        }
        let frequency = match parts[1].parse::<u32>() {
            Ok(freq) if freq > 0 => freq,
            _ => return Self::gui_error(client, "sbp\n"),
        };
        game.config.frequency = frequency;
        client
            .stream
            .write_all(("sst ".to_string() + &game.config.frequency.to_string() + "\n").as_bytes())
            .is_err()
    }

    pub fn bad_command(parts: Vec<&str>, client: &mut Client) -> bool {
        eprintln!("unknown command: {}", parts[0]);
        Self::gui_error(client, "suc\n")
    }
}

pub fn ppo_reply(player: &Player) -> String {
    let mut string = String::new();

    string.push_str("ppo #");
    string.push_str(&player.id.to_string());
    string.push(' ');
    string.push_str(&player.x.to_string());
    string.push(' ');
    string.push_str(&player.y.to_string());
    string.push(' ');
    match player.orientation {
        Orientation::North => string.push('1'),
        Orientation::East => string.push('2'),
        Orientation::South => string.push('3'),
        Orientation::West => string.push('4'),
    }
    string.push('\n');
    string
}

pub fn plv_reply(player: &Player) -> String {
    let mut string = String::new();

    string.push_str("plv #");
    string.push_str(&player.id.to_string());
    string.push(' ');
    string.push_str(&player.level.to_string());
    string.push('\n');
    string
}

pub fn pin_reply(player: &Player) -> String {
    let mut string = String::new();

    string.push_str("pin #");
    string.push_str(&player.id.to_string());
    string.push(' ');
    string.push_str(&player.x.to_string());
    string.push(' ');
    string.push_str(&player.y.to_string());
    string.push(' ');
    string.push_str(&player.food.to_string());
    string.push(' ');
    string.push_str(&player.inventory.linemate.to_string());
    string.push(' ');
    string.push_str(&player.inventory.deraumere.to_string());
    string.push(' ');
    string.push_str(&player.inventory.sibur.to_string());
    string.push(' ');
    string.push_str(&player.inventory.mendiane.to_string());
    string.push(' ');
    string.push_str(&player.inventory.phiras.to_string());
    string.push(' ');
    string.push_str(&player.inventory.thystame.to_string());
    string.push('\n');
    string
}

pub fn pnw_reply(player: &Player, team: &str) -> String {
    let mut string = String::new();

    string.push_str("pnw #");
    string.push_str(&player.id.to_string());
    string.push(' ');
    string.push_str(&player.x.to_string());
    string.push(' ');
    string.push_str(&player.y.to_string());
    string.push(' ');
    match player.orientation {
        Orientation::North => string.push('1'),
        Orientation::East => string.push('2'),
        Orientation::South => string.push('3'),
        Orientation::West => string.push('4'),
    }
    string.push(' ');
    string.push_str(&player.level.to_string());
    string.push(' ');
    string.push_str(team);
    string.push('\n');
    string
}

pub fn pex_reply(player: &Player) -> String {
    let mut string = String::new();

    string.push_str("pex #");
    string.push_str(&player.id.to_string());
    string.push('\n');
    string
}

pub fn pbc_reply(player: &Player, broadcast: &str) -> String {
    let mut string = String::new();

    string.push_str("pbc #");
    string.push_str(&player.id.to_string());
    string.push(' ');
    string.push_str(broadcast);
    string.push('\n');
    string
}

pub fn pic_reply(source: &Player, list: &Vec<&Player>) -> String {
    let mut string = String::new();

    string.push_str("pic #");
    string.push_str(&source.id.to_string());
    string.push(' ');
    string.push_str(&source.x.to_string());
    string.push(' ');
    string.push_str(&source.y.to_string());
    string.push(' ');
    string.push_str(&source.level.to_string());
    for player in list {
        string.push_str(" #");
        string.push_str(&player.id.to_string());
    }
    string.push('\n');
    string
}

pub fn pie_reply(x: u32, y: u32, success: bool) -> String {
    let mut string = String::new();

    string.push_str("pie ");
    string.push_str(&x.to_string());
    string.push(' ');
    string.push_str(&y.to_string());
    string.push(' ');
    string.push_str(if success { "1" } else { "0" });
    string.push('\n');
    string
}

pub fn pfk_reply(player: &Player) -> String {
    let mut string = String::new();

    string.push_str("pfk #");
    string.push_str(&player.id.to_string());
    string.push('\n');
    string
}

pub fn pdr_reply(player: &Player, ressource: u8) -> String {
    let mut string = String::new();

    string.push_str("pdr #");
    string.push_str(&player.id.to_string());
    string.push(' ');
    string.push_str(&ressource.to_string());
    string.push('\n');
    string
}

pub fn pgt_reply(player: &Player, ressource: u8) -> String {
    let mut string = String::new();

    string.push_str("pgt #");
    string.push_str(&player.id.to_string());
    string.push(' ');
    string.push_str(&ressource.to_string());
    string.push('\n');
    string
}

pub fn pdi_reply(player: &Player) -> String {
    let mut string = String::new();

    string.push_str("pdi #");
    string.push_str(&player.id.to_string());
    string.push('\n');
    string
}

pub fn ebo_reply(egg: &Egg) -> String {
    let mut string = String::new();

    string.push_str("ebo #");
    string.push_str(&egg.id.to_string());
    string.push('\n');
    string
}

pub fn edi_reply(egg: &Egg) -> String {
    let mut string = String::new();

    string.push_str("edi #");
    string.push_str(&egg.id.to_string());
    string.push('\n');
    string
}

pub fn seg_reply(team: &str) -> String {
    let mut string = String::new();

    string.push_str("seg ");
    string.push_str(team);
    string.push('\n');
    string
}

pub fn player_spawn(client: &mut Client) -> String {
    let (team, player) = match &client.state {
        ClientState::Ai { team, player } => (team.clone(), player.clone()),
        _ => return "ko\n".to_string(),
    };
    let mut string = String::new();
    string.push_str(&pnw_reply(&player, &team));
    string.push_str(&pin_reply(&player));
    string
}

#[cfg(test)]
mod tests {

    use crate::map::Resources;

    use super::*;

    #[test]
    fn seg_basic() {
        assert_eq!(seg_reply("Zapfrites"), "seg Zapfrites\n");
    }

    #[test]
    fn edi_basic() {
        assert_eq!(
            edi_reply(&Egg {
                x: 1,
                y: 2,
                id: 1,
                player_id: 0,
                is_starting: false,
                team: "Zapfrites".to_string()
            }),
            "edi #1\n"
        );
    }

    #[test]
    fn ebo_basic() {
        assert_eq!(
            ebo_reply(&Egg {
                x: 1,
                y: 2,
                id: 1,
                player_id: 0,
                is_starting: false,
                team: "Zapfrites".to_string()
            }),
            "ebo #1\n"
        );
    }

    #[test]
    fn pdi_basic() {
        assert_eq!(
            pdi_reply(&Player {
                x: 5,
                y: 8,
                id: 10,
                orientation: Orientation::North,
                food: 10,
                level: 2,
                inventory: Resources {
                    deraumere: 0,
                    linemate: 0,
                    mendiane: 0,
                    thystame: 0,
                    phiras: 0,
                    sibur: 0
                },
                time_left: 0,
            }),
            "pdi #10\n"
        );
    }

    #[test]
    fn pgt_basic() {
        assert_eq!(
            pgt_reply(
                &Player {
                    x: 5,
                    y: 8,
                    id: 10,
                    orientation: Orientation::North,
                    food: 10,
                    level: 2,
                    inventory: Resources {
                        deraumere: 0,
                        linemate: 0,
                        mendiane: 0,
                        thystame: 0,
                        phiras: 0,
                        sibur: 0
                    },
                    time_left: 0,
                },
                2
            ),
            "pgt #10 2\n"
        );
    }

    #[test]
    fn pdr_basic() {
        assert_eq!(
            pdr_reply(
                &Player {
                    x: 5,
                    y: 8,
                    id: 10,
                    orientation: Orientation::North,
                    food: 10,
                    level: 2,
                    inventory: Resources {
                        deraumere: 0,
                        linemate: 0,
                        mendiane: 0,
                        thystame: 0,
                        phiras: 0,
                        sibur: 0
                    },
                    time_left: 0,
                },
                2
            ),
            "pdr #10 2\n"
        );
    }

    #[test]
    fn pfk_basic() {
        assert_eq!(
            pfk_reply(&Player {
                x: 5,
                y: 8,
                id: 10,
                orientation: Orientation::North,
                food: 10,
                level: 2,
                inventory: Resources {
                    deraumere: 0,
                    linemate: 0,
                    mendiane: 0,
                    thystame: 0,
                    phiras: 0,
                    sibur: 0
                },
                time_left: 0,
            }),
            "pfk #10\n"
        );
    }

    #[test]
    fn pie_basic() {
        assert_eq!(pie_reply(5, 80, true), "pie 5 80 true\n");
    }
}
