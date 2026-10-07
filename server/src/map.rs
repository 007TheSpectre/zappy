use crate::{
    args::Config,
    gui_parser::{player_spawn, plv_reply},
    server::{Client, ClientState, Game},
};
use rand::RngExt;

#[derive(Default, Clone)]
pub struct Tile {
    pub food: u32,
    pub resources: Resources,
    pub ais: Vec<u32>,
}

#[derive(Default, Clone)]
pub struct Resources {
    pub linemate: u32,
    pub deraumere: u32,
    pub sibur: u32,
    pub mendiane: u32,
    pub phiras: u32,
    pub thystame: u32,
}

#[derive(Default)]
pub struct Egg {
    pub x: u32,
    pub y: u32,
    pub id: u32,
    pub is_starting: bool,
    pub player_id: u32,
    pub team: String,
}

pub fn create_map(config: &Config) -> Vec<Vec<Tile>> {
    let map = vec![vec![Tile::default(); config.width as usize]; config.height as usize];
    map
}

pub fn spawn_resources(config: &Config, map: &mut [Vec<Tile>]) {
    let tiles = (config.width * config.height) as f64;
    let counts: [(f64, u8); 7] = [
        (0.5, 0),  // food
        (0.3, 1),  // linemate
        (0.15, 2), // deraumere
        (0.1, 3),  // sibur
        (0.1, 4),  // mendiane
        (0.08, 5), // phiras
        (0.05, 6), // thystame
    ];
    for (density, kind) in counts {
        let total = (tiles * density).round() as u32;
        for _ in 0..total {
            let x = rand::rng().random_range(0..config.width) as usize;
            let y = rand::rng().random_range(0..config.height) as usize;
            match kind {
                0 => map[y][x].food += 1,
                1 => map[y][x].resources.linemate += 1,
                2 => map[y][x].resources.deraumere += 1,
                3 => map[y][x].resources.sibur += 1,
                4 => map[y][x].resources.mendiane += 1,
                5 => map[y][x].resources.phiras += 1,
                _ => map[y][x].resources.thystame += 1,
            }
        }
    }
}

pub fn msz_reply(config: &Config) -> String {
    let mut string =
        String::with_capacity(8 + config.width.ilog10() as usize + config.height.ilog10() as usize);
    string.push_str("msz ");
    string.push_str(&config.width.to_string());
    string.push(' ');
    string.push_str(&config.height.to_string());
    string.push('\n');
    string
}

pub fn sgt_reply(config: &Config) -> String {
    let mut string = String::with_capacity(6 + config.frequency.ilog10() as usize);
    string.push_str("sgt ");
    string.push_str(&config.frequency.to_string());
    string.push('\n');
    string
}

pub fn bct_reply(x: u32, y: u32, map: &[Vec<Tile>], config: &Config) -> Option<String> {
    if x >= config.width || y >= config.height {
        return None;
    }
    let mut string = String::with_capacity(
        17 + x.checked_ilog10().unwrap_or(0) as usize
            + y.checked_ilog10().unwrap_or(0) as usize
            + map[y as usize][x as usize]
                .food
                .checked_ilog10()
                .unwrap_or(0) as usize
            + map[y as usize][x as usize]
                .resources
                .linemate
                .checked_ilog10()
                .unwrap_or(0) as usize
            + map[y as usize][x as usize]
                .resources
                .deraumere
                .checked_ilog10()
                .unwrap_or(0) as usize
            + map[y as usize][x as usize]
                .resources
                .sibur
                .checked_ilog10()
                .unwrap_or(0) as usize
            + map[y as usize][x as usize]
                .resources
                .mendiane
                .checked_ilog10()
                .unwrap_or(0) as usize
            + map[y as usize][x as usize]
                .resources
                .phiras
                .checked_ilog10()
                .unwrap_or(0) as usize
            + map[y as usize][x as usize]
                .resources
                .thystame
                .checked_ilog10()
                .unwrap_or(0) as usize,
    );
    string.push_str("bct ");
    string.push_str(&x.to_string());
    string.push(' ');
    string.push_str(&y.to_string());
    string.push(' ');
    string.push_str(&map[y as usize][x as usize].food.to_string());
    string.push(' ');
    string.push_str(&map[y as usize][x as usize].resources.linemate.to_string());
    string.push(' ');
    string.push_str(&map[y as usize][x as usize].resources.deraumere.to_string());
    string.push(' ');
    string.push_str(&map[y as usize][x as usize].resources.sibur.to_string());
    string.push(' ');
    string.push_str(&map[y as usize][x as usize].resources.mendiane.to_string());
    string.push(' ');
    string.push_str(&map[y as usize][x as usize].resources.phiras.to_string());
    string.push(' ');
    string.push_str(&map[y as usize][x as usize].resources.thystame.to_string());
    string.push('\n');
    Some(string)
}

pub fn tna_reply(config: &Config) -> String {
    let mut string = String::new();
    for team in config.teams.iter() {
        string.push_str("tna ");
        string.push_str(team);
        string.push('\n');
    }
    string
}

pub fn mct_reply(config: &Config, map: &[Vec<Tile>]) -> Option<String> {
    let mut map_string = String::new();
    for y in 0..config.height {
        for x in 0..config.width {
            match bct_reply(x, y, map, config) {
                Some(string) => map_string.push_str(&string),
                None => return None,
            }
        }
    }
    Some(map_string)
}

pub fn create_gui_connection_string(game: &Game, clients: &mut [Client]) -> Option<String> {
    let mut map_string = String::new();

    map_string.push_str(&msz_reply(&game.config));
    map_string.push_str(&sgt_reply(&game.config));
    match mct_reply(&game.config, &game.map) {
        Some(string) => map_string.push_str(&string),
        None => return None,
    }
    map_string.push_str(&tna_reply(&game.config));
    for client in clients {
        let player = match &client.state {
            ClientState::Ai { player, .. } => player.clone(),
            _ => continue,
        };
        map_string.push_str(&player_spawn(client));
        map_string.push_str(&plv_reply(&player));
    }
    for egg in &game.eggs {
        map_string.push_str(&enw_string(egg));
    }
    Some(map_string)
}

pub fn enw_string(egg: &Egg) -> String {
    let mut string = String::new();
    string.push_str("enw #");
    string.push_str(&egg.id.to_string());
    string.push_str(" #");
    if egg.is_starting {
        string.push_str("-1");
    } else {
        string.push_str(&egg.player_id.to_string());
    }
    string.push(' ');
    string.push_str(&egg.x.to_string());
    string.push(' ');
    string.push_str(&egg.y.to_string());
    string.push('\n');
    string
}

pub fn refill_resources(config: &Config, map: &mut [Vec<Tile>]) {
    let mut current = [0u32; 7];
    for row in map.iter() {
        for tile in row.iter() {
            current[0] += tile.food;
            current[1] += tile.resources.linemate;
            current[2] += tile.resources.deraumere;
            current[3] += tile.resources.sibur;
            current[4] += tile.resources.mendiane;
            current[5] += tile.resources.phiras;
            current[6] += tile.resources.thystame;
        }
    }

    let tiles = (config.width * config.height) as f64;
    let targets: [u32; 7] = [
        (tiles * 0.5).round() as u32,
        (tiles * 0.3).round() as u32,
        (tiles * 0.15).round() as u32,
        (tiles * 0.1).round() as u32,
        (tiles * 0.1).round() as u32,
        (tiles * 0.08).round() as u32,
        (tiles * 0.05).round() as u32,
    ];

    for kind in 0..7usize {
        if current[kind] >= targets[kind] {
            continue;
        }
        let to_add = targets[kind] - current[kind];
        for _ in 0..to_add {
            let x = rand::rng().random_range(0..config.width);
            let y = rand::rng().random_range(0..config.height);
            match kind {
                0 => map[y as usize][x as usize].food += 1,
                1 => map[y as usize][x as usize].resources.linemate += 1,
                2 => map[y as usize][x as usize].resources.deraumere += 1,
                3 => map[y as usize][x as usize].resources.sibur += 1,
                4 => map[y as usize][x as usize].resources.mendiane += 1,
                5 => map[y as usize][x as usize].resources.phiras += 1,
                _ => map[y as usize][x as usize].resources.thystame += 1,
            }
        }
    }
}

pub fn start_spawn_eggs(config: &Config) -> Vec<Egg> {
    let mut eggs = Vec::new();
    let mut id = 0u32;
    for team in &config.teams {
        for _ in 0..config.clients_per_team {
            eggs.push(Egg {
                x: rand::rng().random_range(0..config.width),
                y: rand::rng().random_range(0..config.height),
                id,
                player_id: 0,
                is_starting: true,
                team: team.clone(),
            });
            id += 1;
        }
    }
    eggs
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn enw_basic() {
        assert_eq!(
            enw_string(&Egg {
                x: 5,
                y: 6,
                id: 0,
                player_id: 0,
                is_starting: false,
                team: String::new(),
            }),
            "enw #0 #0 5 6\n"
        );
    }

    #[test]
    fn enw_starting() {
        assert_eq!(
            enw_string(&Egg {
                x: 5,
                y: 6,
                id: 0,
                is_starting: true,
                player_id: 0,
                team: String::new(),
            }),
            "enw #0 #-1 5 6\n"
        );
    }

    #[test]
    fn refill_tops_up_to_target_densities() {
        let config = Config {
            port: 0,
            width: 4,
            height: 4,
            teams: vec![],
            clients_per_team: 0,
            frequency: 0,
        };
        let mut map = create_map(&config);

        let total = |map: &[Vec<Tile>]| {
            let mut sum = 0u32;
            for row in map {
                for t in row {
                    sum += t.food
                        + t.resources.linemate
                        + t.resources.deraumere
                        + t.resources.sibur
                        + t.resources.mendiane
                        + t.resources.phiras
                        + t.resources.thystame;
                }
            }
            sum
        };

        // Empty map: refill adds resources up to the target densities.
        refill_resources(&config, &mut map);
        let after_first = total(&map);
        assert!(after_first > 0);

        // Already at target densities: a second refill adds nothing.
        refill_resources(&config, &mut map);
        assert_eq!(total(&map), after_first);
    }

    #[test]
    fn tna_basic() {
        assert_eq!(
            tna_reply(&Config {
                clients_per_team: 0,
                port: 0,
                width: 0,
                height: 0,
                teams: vec!["team1".to_string(), "team2".to_string()],
                frequency: 0
            }),
            "tna team1\ntna team2\n"
        );
    }

    #[test]
    fn sgt_basic() {
        assert_eq!(
            sgt_reply(&Config {
                port: 0,
                width: 0,
                height: 0,
                teams: vec![],
                clients_per_team: 0,
                frequency: 10
            }),
            "sgt 10\n"
        );
    }

    #[test]
    fn msz_basic() {
        assert_eq!(
            msz_reply(&Config {
                port: 0,
                width: 10,
                height: 5,
                teams: vec![],
                clients_per_team: 0,
                frequency: 0
            }),
            "msz 10 5\n"
        );
    }
}
