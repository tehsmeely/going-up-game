#![allow(clippy::type_complexity)]

mod audio;
mod camera;
mod core;
mod game;
mod game_menu;
mod helpers;
mod history_store;
mod input_action;
mod loading;
mod menu;
mod ui_widgets;

use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use leafwing_input_manager::prelude::*;

use crate::camera::CameraPlugin;
use crate::game::CoreGamePlugin;
use crate::input_action::InputAction;
use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    #[default]
    Loading,
    PlayingDay,
    MainMenu,
    PlayingMenu,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>().add_plugins((
            LoadingPlugin,
            MenuPlugin,
            CoreGamePlugin,
            InternalAudioPlugin,
            CameraPlugin,
            core::CorePlugin,
            game_menu::GameMenuPlugin,
        ));
        app.add_plugins((
            InputManagerPlugin::<InputAction>::default(),
            TilemapPlugin,
            EguiPlugin,
            WorldInspectorPlugin::new(),
        ));
        #[cfg(feature = "frame-time-diagnostics")]
        {
            app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }
}

pub fn run_spawn_example() {
    spawn_example::run_spawn_example();
}

mod spawn_example {
    use crate::game;
    use crate::game::spawn_simulation::{ResolvedFloorConfig, SinkOrSource, TimeRange};
    use crate::game::{spawn_simulation, FloorNum};
    use rand::thread_rng;
    use std::fs::File;
    use std::io::{BufWriter, Write};
    use std::time::Duration;

    pub fn run_spawn_example() {
        let mut game_clock = game::game_clock::GameTime::new();
        let floors = vec![
            spawn_simulation::RawFloorConfig::new(
                Box::new(|time_range| match time_range {
                    TimeRange::Morning => SinkOrSource::Source,
                    _ => SinkOrSource::Sink,
                }),
                Box::new(|_time_range| 10),
            ),
            spawn_simulation::RawFloorConfig::new(
                Box::new(|time_range| match time_range {
                    TimeRange::Morning => SinkOrSource::Sink,
                    _ => SinkOrSource::Source,
                }),
                Box::new(|_time_range| 1),
            ),
            spawn_simulation::RawFloorConfig::new(
                Box::new(|time_range| match time_range {
                    TimeRange::Morning => SinkOrSource::Sink,
                    _ => SinkOrSource::Source,
                }),
                Box::new(|_time_range| 1),
            ),
            spawn_simulation::RawFloorConfig::new(
                Box::new(|time_range| match time_range {
                    TimeRange::Morning => SinkOrSource::Sink,
                    _ => SinkOrSource::Source,
                }),
                Box::new(|_time_range| 1),
            ),
            spawn_simulation::RawFloorConfig::new(
                Box::new(|time_range| match time_range {
                    TimeRange::Morning => SinkOrSource::Sink,
                    _ => SinkOrSource::Source,
                }),
                Box::new(|_time_range| 1),
            ),
        ];

        let time_range = TimeRange::of_time_ofday(&game_clock.to_game_time_of_day());
        let resolved: Vec<ResolvedFloorConfig> = floors
            .iter()
            .enumerate()
            .map(|(i, raw)| raw.resolve(time_range, FloorNum(i as i32)))
            .collect();

        let mut floor_spawn_rates =
            spawn_simulation::FloorSpawnRates::get_rates(resolved.clone(), time_range);

        let mut rng = thread_rng();
        let output_file = File::create("spawn_output.csv").unwrap();
        let mut output_writer = BufWriter::new(output_file);
        let tick_size = Duration::from_secs(1);
        let mut num_ticks = 0;
        loop {
            num_ticks += 1;
            let spawns = floor_spawn_rates.tick(&game_clock, tick_size, &mut rng);
            for (from, to_) in spawns {
                let line = format!("{},{},{}\n", game_clock.to_string_secs(), from.0, to_.0);
                output_writer.write_all(line.as_bytes()).unwrap()
            }
            if game_clock.tick(tick_size) {
                break;
            }
        }
        println!("Done. Ran for {} ticks", num_ticks);
    }
}
