use crate::game::game::{AccelerationLog, ObservedVelocity, ObservedVelocityLog, VelocityLog};
use crate::game::lift::LiftHumanStore;
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
        app.add_systems(
            Update,
            (
                recalculate_plot_points,
                show_ui,
                GameCentralInfo::update_system,
                LiftHumanStore::draw_system,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .init_resource::<ShowUiState>()
        .insert_resource(GameCentralInfo::new())
        .insert_resource(TrueVelocityPlotPoints(vec![]))
        .insert_resource(ObservedVelocityPlotPoints(vec![]))
        .insert_resource(AccelerationPlotPoints(vec![]));
    }
}

#[derive(Resource)]
struct ShowUiState {
    show_velocity: bool,
    show_acceleration: bool,
    show_smooth_acceleration: bool,
    show_observed_velocity: bool,
}
impl Default for ShowUiState {
    fn default() -> Self {
        Self {
            show_velocity: true,
            show_acceleration: true,
            show_smooth_acceleration: true,
            show_observed_velocity: true,
        }
    }
}

fn show_ui(
    mut contexts: EguiContexts,
    target_velocity_plot_points: Res<TrueVelocityPlotPoints>,
    observed_velocity_plot_points: Res<ObservedVelocityPlotPoints>,
    acceleration_plot_points: Res<AccelerationPlotPoints>,
    mut show_state: ResMut<ShowUiState>,
) {
    egui::Window::new("Ui").show(contexts.ctx_mut(), |ui| {
        ui.checkbox(&mut show_state.show_velocity, "Show Velocity");
        ui.checkbox(&mut show_state.show_acceleration, "Show Acceleration");
        ui.checkbox(
            &mut show_state.show_smooth_acceleration,
            "Show Smooth Acceleration",
        );
        ui.checkbox(&mut show_state.show_observed_velocity, "Show Obs Velocity");
        Plot::new("Plot").view_aspect(2.0).show(ui, |ui| {
            if show_state.show_velocity {
                let velocity_points = PlotPoints::new(target_velocity_plot_points.0.clone());
                ui.line(
                    Line::new(velocity_points)
                        .name("Velocity")
                        .color(Color32::GREEN),
                );
            }
            if show_state.show_acceleration {
                let accel_points = PlotPoints::new(acceleration_plot_points.0.clone());
                ui.line(
                    Line::new(accel_points)
                        .name("Acceleration")
                        .color(Color32::RED),
                );
            }
            if show_state.show_smooth_acceleration {
                let accel_points = {
                    let mut prev: Option<[f64; 2]> = None;
                    let points: Vec<[f64; 2]> = acceleration_plot_points
                        .0
                        .iter()
                        .map(|point| match prev {
                            Some(prev_point) => {
                                let x = (prev_point[0] + point[0]) / 2.0;
                                let y = (prev_point[1] + point[1]) / 2.0;
                                prev = Some(*point);
                                [x, y]
                            }
                            None => {
                                prev = Some(*point);
                                *point
                            }
                        })
                        .collect();
                    PlotPoints::new(points)
                };
                ui.line(
                    Line::new(accel_points)
                        .name("Smoothed Acceleration")
                        .color(Color32::YELLOW),
                );
            }
            if show_state.show_observed_velocity {
                let obs_velocity_points = PlotPoints::new(observed_velocity_plot_points.0.clone());
                ui.line(
                    Line::new(obs_velocity_points)
                        .name("Observed Velocity")
                        .color(Color32::DARK_GREEN),
                );
            }
        });
    });
}

fn recalculate_plot_points(
    velocity_log: Res<VelocityLog>,
    observed_velocity_log: Res<ObservedVelocityLog>,
    acceleration_log: Res<AccelerationLog>,
    mut velocity_plot: ResMut<TrueVelocityPlotPoints>,
    mut obs_velocity_plot: ResMut<ObservedVelocityPlotPoints>,
    mut acceleration_plot: ResMut<AccelerationPlotPoints>,
    mut count: Local<usize>,
) {
    // This means plot update is framerate linked, but meh
    *count = (*count + 1usize) % 3;
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
    } else if *count == 2 {
        let points: Vec<[f64; 2]> = observed_velocity_log
            .0
            .iter_primary()
            .map(|(time, val)| [*time as f64, *val as f64])
            .collect();
        obs_velocity_plot.0 = points;
    }
}

#[derive(Resource, Debug)]
pub struct TrueVelocityPlotPoints(Vec<[f64; 2]>);
#[derive(Resource, Debug)]
pub struct ObservedVelocityPlotPoints(Vec<[f64; 2]>);
#[derive(Resource, Debug)]
pub struct AccelerationPlotPoints(Vec<[f64; 2]>);

pub fn default_frame() -> Frame {
    Frame::default()
        .inner_margin(4.0)
        .fill(Color32::DARK_GRAY)
        .rounding(Rounding {
            nw: 5.0,
            ..default()
        })
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
