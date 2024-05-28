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
    use crate::game::spawn_simulation::{
        FloorSpawnManager, HourOfDay, ResolvedFloorConfig, SinkOrSource,
    };
    use crate::game::{spawn_simulation, FloorNum};
    use rand::thread_rng;
    use std::fs::File;
    use std::io::{BufWriter, Write};
    use std::time::Duration;

    mod prefabs {
        use crate::game::spawn_simulation::{HourOfDay, SinkOrSource};
        use bevy::utils::tracing::span_enabled;
        use std::ops::Range;

        type Sr = Vec<(Range<u8>, SinkOrSource)>;

        pub fn ground_floor_source() -> Sr {
            vec![
                (0..10, SinkOrSource::Source),
                (10..16, SinkOrSource::Sink),
                (16..17, SinkOrSource::Source),
                (17..24, SinkOrSource::Sink),
            ]
        }

        pub fn always<T: Copy>(val: T) -> Vec<(Range<u8>, T)> {
            vec![(0..24, val)]
        }

        pub fn morning_sink() -> Sr {
            vec![(0..12, SinkOrSource::Sink), (12..24, SinkOrSource::Source)]
        }

        pub fn afternoon_sink() -> Sr {
            vec![(0..12, SinkOrSource::Source), (12..24, SinkOrSource::Sink)]
        }

        pub fn alternating_sink(mode: bool) -> Sr {
            let (a, b) = match mode {
                true => (SinkOrSource::Sink, SinkOrSource::Source),
                false => (SinkOrSource::Source, SinkOrSource::Sink),
            };
            vec![
                (0..2, a),
                (2..4, b),
                (4..6, a),
                (6..8, b),
                (8..10, a),
                (10..12, b),
                (12..14, a),
                (14..16, b),
                (16..18, a),
                (18..20, b),
                (20..22, a),
                (22..24, b),
            ]
        }
    }

    pub fn run_spawn_example() {
        let mut game_clock = game::game_clock::GameTime::new();
        let floors = vec![
            spawn_simulation::RawFloorConfig::new(
                prefabs::ground_floor_source(),
                prefabs::always(10),
            ),
            spawn_simulation::RawFloorConfig::new(prefabs::afternoon_sink(), prefabs::always(1)),
            spawn_simulation::RawFloorConfig::new(prefabs::afternoon_sink(), prefabs::always(1)),
            spawn_simulation::RawFloorConfig::new(prefabs::morning_sink(), prefabs::always(1)),
            spawn_simulation::RawFloorConfig::new(
                prefabs::alternating_sink(true),
                prefabs::always(1),
            ),
            spawn_simulation::RawFloorConfig::new(
                prefabs::alternating_sink(true),
                prefabs::always(1),
            ),
            spawn_simulation::RawFloorConfig::new(prefabs::morning_sink(), prefabs::always(1)),
            spawn_simulation::RawFloorConfig::new(prefabs::morning_sink(), prefabs::always(1)),
            spawn_simulation::RawFloorConfig::new(
                prefabs::always(SinkOrSource::Sink),
                prefabs::always(1),
            ),
            spawn_simulation::RawFloorConfig::new(
                prefabs::alternating_sink(false),
                prefabs::always(1),
            ),
        ];

        let floors = floors
            .into_iter()
            .enumerate()
            .map(|(i, raw)| (FloorNum(i as i32), raw.unwrap()))
            .collect();
        let mut manager = FloorSpawnManager::new(floors);
        /*
        let resolved: Vec<ResolvedFloorConfig> = floors
            .iter()
            .enumerate()
            .map(|(i, raw)| raw.resolve(time_range, FloorNum(i as i32)))
            .collect();

        let mut floor_spawn_rates =
            spawn_simulation::FloorSpawnRates::get_rates(resolved.clone(), time_range);
         */

        let mut rng = thread_rng();
        let output_file = File::create("spawn_output.csv").unwrap();
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
        println!("Done. Ran for {} ticks", num_ticks);
    }
}
