use crate::server::Game;
use std::time::{Duration, Instant};

mod args;
mod command;
mod gui_parser;
mod map;
mod server;
mod time_manager;

fn main() -> std::io::Result<()> {
    let config = args::parse().unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(84);
    });

    let mut server = server::start(config.clone())?;

    let mut map = map::create_map(&config);
    map::spawn_resources(&config, &mut map);

    let mut game = Game {
        next_egg_id: config.clients_per_team * config.teams.len() as u32,
        config: config.clone(),
        map,
        eggs: map::start_spawn_eggs(&config),
        next_player_id: 0,
    };

    let mut current_frequency = config.frequency;
    let mut food_interval = food_interval_for(current_frequency);
    let mut spawn_interval = spawn_interval_for(current_frequency);
    let mut next_spawn = Instant::now() + spawn_interval;
    let mut next_food_tick = Instant::now() + food_interval;

    loop {
        server.execute_ready_commands(&config, &mut game);
        if let Some(winner) = server.check_win_condition() {
            server.end_game(&winner);
        }
        if server.is_game_over() {
            break;
        }

        let new_frequency = game.config.frequency;
        if new_frequency != current_frequency {
            let now = Instant::now();
            food_interval = food_interval_for(new_frequency);
            spawn_interval = spawn_interval_for(new_frequency);
            next_food_tick = now + food_interval;
            next_spawn = now + spawn_interval;
            server.set_frequency(new_frequency);
            current_frequency = new_frequency;
        }

        let now = Instant::now();
        let ms_to_spawn = next_spawn
            .saturating_duration_since(now)
            .as_millis()
            .min(i32::MAX as u128) as i32;
        let ms_to_food = next_food_tick
            .saturating_duration_since(now)
            .as_millis()
            .min(i32::MAX as u128) as i32;
        let timeout_ms = match server.next_poll_timeout_ms() {
            -1 => ms_to_spawn.min(ms_to_food),
            t => t.min(ms_to_spawn).min(ms_to_food),
        };

        let ready = server.poll(timeout_ms)?;
        server.accept(&ready)?;
        let read = server.read(&ready, &mut game)?;
        server.cleanup(ready.disconnected.into_iter().chain(read.disconnected));

        let now = Instant::now();
        // Apply every food tick whose deadline has passed. At high frequencies a
        // single tick can be sub-millisecond, so several may be due per loop pass;
        // batching them keeps the food economy correct without forcing the poll
        // timeout down to a 1 ms floor (the old `.max(1)` both burned CPU and
        // distorted food drain relative to action speed above ~1000 Hz).
        let mut batched = 0u32;
        while now >= next_food_tick && batched < MAX_BATCHED_FOOD_TICKS {
            server.decrement_food();
            next_food_tick += food_interval;
            batched += 1;
        }
        if now >= next_food_tick {
            // Still behind after the batch cap (extreme frequency or a long stall):
            // resync rather than spin forever trying to catch up tick by tick.
            next_food_tick = now + food_interval;
        }

        if now >= next_spawn {
            // The reference server tops the map back up to its target densities
            // but does not push tile updates to the GUI on refill, so we don't
            // either — the GUI stays in sync through action-driven bct/pgt/pdr.
            // Refill is an idempotent top-up, so resync to "now" instead of
            // replaying every missed interval.
            map::refill_resources(&game.config, &mut game.map);
            next_spawn = now + spawn_interval;
        }
    }
    Ok(())
}

/// Wall-clock spacing between successive food decrements: one game time-unit,
/// i.e. `1 / frequency` seconds, computed at nanosecond precision so high
/// frequencies are not flattened by integer-millisecond rounding. Floored at
/// 1 ns so the batching loop always makes progress.
fn food_interval_for(frequency: u32) -> Duration {
    Duration::from_nanos((1_000_000_000u64 / frequency.max(1) as u64).max(1))
}

/// Wall-clock spacing between resource refills: 20 game time-units.
fn spawn_interval_for(frequency: u32) -> Duration {
    Duration::from_nanos((20_000_000_000u64 / frequency.max(1) as u64).max(1))
}

/// Upper bound on food ticks applied in a single loop iteration, so a long stall
/// or an extreme frequency can never trap the loop in the catch-up `while`.
const MAX_BATCHED_FOOD_TICKS: u32 = 4096;
