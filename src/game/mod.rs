use bevy::app::{App, Plugin};

mod floors;
mod game;
pub mod game_clock;
mod human_store;
mod lift;
mod overlay;
pub mod spawn_simulation;
mod speed_selector;
mod ui;
mod ui_b;
mod world_gen;

pub use floors::FloorNum;

pub struct CoreGamePlugin;

impl Plugin for CoreGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            game::GamePlugin,
            speed_selector::SpeedSelectorPlugin,
            ui::GameUiPlugin,
            ui_b::UIBPlugin,
            overlay::OverlayPlugin,
        ));
    }
}
