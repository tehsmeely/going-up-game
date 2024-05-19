use bevy::app::{App, Plugin};
use bevy::prelude::IntoSystemConfigs;

mod game_menu;

pub struct GameMenuPlugin;

impl Plugin for GameMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(game_menu::MenuPlugin);
    }
}
