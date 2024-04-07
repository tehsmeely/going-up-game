use bevy::app::{App, Plugin};

mod floors;
mod game;
mod human_store;
mod lift;
mod overlay;
mod speed_selector;
mod ui;
mod world_gen;

pub struct CoreGamePlugin;

impl Plugin for CoreGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            game::GamePlugin,
            speed_selector::SpeedSelectorPlugin,
            ui::GameUiPlugin,
            overlay::OverlayPlugin,
        ));
    }
}
