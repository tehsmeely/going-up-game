use crate::GameState;
use crate::GameState::PlayingMenu;
use bevy::prelude::*;
use bevy_egui::egui::Layout;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiSettings};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, day_menu.run_if(in_state(PlayingMenu)));
    }
}
fn day_menu(mut contexts: EguiContexts, mut next_state: ResMut<NextState<GameState>>) {
    let ctx = contexts.ctx_mut();

    egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        ui.style_mut().text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(20.0, egui::FontFamily::Proportional),
        );
        ui.label("Day 1");
    });
    egui::TopBottomPanel::bottom("bottom_panel_two").show(ctx, |ui| {
        ui.style_mut().text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(20.0, egui::FontFamily::Proportional),
        );
        ui.vertical_centered(|ui| {
            ui.label("Home");
        });
    });
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::Frame::none().inner_margin(200.0).show(ui, |ui| {
            ui.style_mut().text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::new(23.0, egui::FontFamily::Proportional),
            );
            ui.horizontal_centered(|ui| {
                ui.vertical_centered(|ui| {
                    let start_day_button = ui.button("Start Day");
                    if start_day_button.clicked() {
                        info!("Start Day");
                        next_state.set(GameState::PlayingDay);
                    }
                    let quit_button = ui.button("Quit");
                    if quit_button.clicked() {
                        info!("Quit");
                        next_state.set(GameState::MainMenu);
                    }
                });
            });
        });
    });
}
