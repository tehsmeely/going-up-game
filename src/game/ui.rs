use crate::game::game::{AccelerationLog, VelocityLog};
use crate::GameState;
use bevy::prelude::*;
use bevy_egui::egui::Color32;
use bevy_egui::{egui, EguiContexts};
use egui_plot::{Line, Plot, PlotPoints};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (recalculate_plot_points, show_ui).run_if(in_state(GameState::Playing)),
        )
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
