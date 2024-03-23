use crate::game::game::{AccelerationLog, VelocityLog};
use crate::GameState;
use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy::utils::hashbrown::HashMap;
use bevy_egui::egui::{Align2, Color32, Frame, RichText, Rounding};
use bevy_egui::{egui, EguiContexts};
use egui_extras::{Column, TableBuilder};
use egui_plot::{Line, Plot, PlotPoints};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        let mut humans = HashMap::new();
        humans.insert(10, 1);
        humans.insert(5, 2);
        humans.insert(7, 3);
        app.add_systems(
            Update,
            (
                recalculate_plot_points,
                show_ui,
                GameCentralInfo::update_system,
                HeldHumans::update_system,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .insert_resource(GameCentralInfo::new())
        .insert_resource(HeldHumans { humans })
        .insert_resource(TrueVelocityPlotPoints(vec![]))
        .insert_resource(AccelerationPlotPoints(vec![]));
    }
}

fn show_ui(
    mut contexts: EguiContexts,
    target_velocity_plot_points: Res<TrueVelocityPlotPoints>,
    acceleration_plot_points: Res<AccelerationPlotPoints>,
) {
    egui::Window::new("Ui").show(contexts.ctx_mut(), |ui| {
        Plot::new("Plot").view_aspect(2.0).show(ui, |ui| {
            let velocity_points = PlotPoints::new(target_velocity_plot_points.0.clone());
            ui.line(
                Line::new(velocity_points)
                    .name("Velocity")
                    .color(Color32::GREEN),
            );
            let accel_points = PlotPoints::new(acceleration_plot_points.0.clone());
            ui.line(
                Line::new(accel_points)
                    .name("Acceleration")
                    .color(Color32::RED),
            );
        });
    });
}

fn recalculate_plot_points(
    velocity_log: Res<VelocityLog>,
    acceleration_log: Res<AccelerationLog>,
    mut velocity_plot: ResMut<TrueVelocityPlotPoints>,
    mut acceleration_plot: ResMut<AccelerationPlotPoints>,
    mut count: Local<usize>,
) {
    // This means plot update is framerate linked, but meh
    *count = (*count + 1usize) % 2;
    if *count == 0 {
        let points: Vec<[f64; 2]> = velocity_log
            .0
            .iter_primary()
            .map(|(time, val)| [*time as f64, *val as f64])
            .collect();
        velocity_plot.0 = points;
    } else if *count == 1 {
        let points: Vec<[f64; 2]> = acceleration_log
            .0
            .iter_primary()
            .map(|(time, val)| [*time as f64, *val as f64])
            .collect();
        acceleration_plot.0 = points;
    }
}

#[derive(Resource, Debug)]
pub struct TrueVelocityPlotPoints(Vec<[f64; 2]>);
#[derive(Resource, Debug)]
pub struct AccelerationPlotPoints(Vec<[f64; 2]>);

fn default_frame() -> Frame {
    Frame::default()
        .inner_margin(4.0)
        .fill(Color32::DARK_GRAY)
        .rounding(Rounding {
            nw: 5.0,
            ..default()
        })
}
#[derive(Resource, Debug, Reflect)]
pub struct HeldHumans {
    humans: HashMap<i32, usize>,
}

impl HeldHumans {
    fn update_system(mut humans: Res<Self>, mut contexts: EguiContexts) {
        let frame = default_frame();
        let text_color = Color32::WHITE;
        let size = 24.0;
        egui::Window::new("Held Humans")
            .movable(false)
            .resizable(false)
            .anchor(Align2::RIGHT_TOP, bevy_egui::egui::Vec2::ZERO)
            .title_bar(false)
            .frame(frame)
            .show(contexts.ctx_mut(), |ui| {
                TableBuilder::new(ui)
                    .column(Column::auto())
                    .column(Column::auto())
                    .body(|mut body| {
                        for (dest_floor, count) in humans.humans.iter() {
                            body.row(30.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(
                                        RichText::new(dest_floor.to_string())
                                            .color(text_color)
                                            .size(size),
                                    );
                                });
                                row.col(|ui| {
                                    ui.label(
                                        RichText::new(count.to_string())
                                            .color(text_color)
                                            .size(size),
                                    );
                                });
                            })
                        }
                    })
            });
    }
}

#[derive(Resource, Debug, Reflect)]
pub struct GameCentralInfo {
    money: f32,
    day: usize,
    time: Stopwatch,
}

impl GameCentralInfo {
    pub fn new() -> Self {
        Self {
            money: 0.0,
            day: 1,
            time: Stopwatch::new(),
        }
    }

    fn update_system(mut info: ResMut<Self>, mut contexts: EguiContexts, time: Res<Time>) {
        info.time.tick(time.delta());
        let frame = default_frame();
        let text_color = Color32::WHITE;
        let size = 24.0;
        egui::Window::new("Game Info")
            .movable(false)
            .resizable(false)
            .anchor(Align2::RIGHT_BOTTOM, bevy_egui::egui::Vec2::ZERO)
            .title_bar(false)
            .frame(frame)
            .show(contexts.ctx_mut(), |ui| {
                ui.label(
                    RichText::new(format!("Day: {}", info.day))
                        .color(text_color)
                        .size(size),
                );
                ui.label(
                    RichText::new(format!("Money: {}", info.money))
                        .color(text_color)
                        .size(size),
                );
                ui.label(
                    RichText::new(format!("Time: {}", info.time.elapsed().as_secs()))
                        .color(text_color)
                        .size(size),
                );
            });
    }
}
