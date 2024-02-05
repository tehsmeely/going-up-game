use bevy::app::{App, Plugin};

mod game;

pub struct CoreGamePlugin;

impl Plugin for CoreGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(game::GamePlugin);
    }
}
