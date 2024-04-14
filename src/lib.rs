#![allow(clippy::type_complexity)]

mod audio;
mod camera;
mod core;
mod game;
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
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
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
