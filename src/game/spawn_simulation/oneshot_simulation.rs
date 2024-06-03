use crate::game;
use crate::game::spawn_simulation::prefabs;
use crate::game::spawn_simulation::{FloorSpawnManager, SinkOrSource};
use crate::game::{spawn_simulation, FloorNum};
use rand::thread_rng;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Duration;

fn main() {
    run_spawn_example()
}
pub fn run_spawn_example() {
    let mut game_clock = game::game_clock::GameTime::new();
    let floors = vec![
        spawn_simulation::RawFloorConfig::new(
            prefabs::ground_floor_source(),
            prefabs::normal_hours(1, 10),
        ),
        spawn_simulation::RawFloorConfig::new(
            prefabs::afternoon_sink(),
            prefabs::normal_hours(1, 2),
        ),
        spawn_simulation::RawFloorConfig::new(
            prefabs::afternoon_sink(),
            prefabs::normal_hours(1, 2),
        ),
        spawn_simulation::RawFloorConfig::new(prefabs::morning_sink(), prefabs::normal_hours(1, 2)),
        spawn_simulation::RawFloorConfig::new(
            prefabs::alternating_sink(true),
            prefabs::normal_hours(1, 2),
        ),
        spawn_simulation::RawFloorConfig::new(
            prefabs::alternating_sink(true),
            prefabs::normal_hours(1, 2),
        ),
        spawn_simulation::RawFloorConfig::new(prefabs::morning_sink(), prefabs::normal_hours(1, 2)),
        spawn_simulation::RawFloorConfig::new(prefabs::morning_sink(), prefabs::normal_hours(1, 2)),
        spawn_simulation::RawFloorConfig::new(
            prefabs::always(SinkOrSource::Sink),
            prefabs::normal_hours(1, 2),
        ),
        spawn_simulation::RawFloorConfig::new(
            prefabs::alternating_sink(false),
            prefabs::normal_hours(1, 2),
        ),
    ];

    let floors = floors
        .into_iter()
        .enumerate()
        .map(|(i, raw)| (FloorNum(i as i32), raw.unwrap()))
        .collect();
    let mut manager = FloorSpawnManager::new(floors);
    let mut rng = thread_rng();
    let output_filename = "spawn_output.csv";
    let output_file = File::create(output_filename).unwrap();
    let mut output_writer = BufWriter::new(output_file);
    output_writer
        .write_all(game_clock.config().to_csv().as_bytes())
        .unwrap();
    let tick_size = Duration::from_secs(1);
    let mut num_ticks = 0;
    loop {
        num_ticks += 1;
        let spawns = manager.tick(&game_clock, tick_size, &mut rng);
        for (from, to_) in spawns {
            let line = format!("{},{},{}\n", game_clock.to_string_secs(), from.0, to_.0);
            output_writer.write_all(line.as_bytes()).unwrap()
        }
        if game_clock.tick(tick_size) {
            break;
        }
    }
    println!(
        "Done.\nRan for {} ticks.\nSaved to {}",
        num_ticks, output_filename
    );
}
