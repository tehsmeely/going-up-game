use crate::GameState;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (show_ui).run_if(in_state(GameState::Playing)));
    }
}

fn show_ui(mut contexts: EguiContexts) {
    egui::Window::new("Ui").show(contexts.ctx_mut(), |ui| {
        ui.label("Label");
    });
}
