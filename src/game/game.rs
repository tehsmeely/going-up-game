use crate::camera::{CameraTrack, RENDER_LAYER_MAIN};
use crate::game::floors;
use crate::game::floors::{
    spawn_person_system, FloorLatchYPositions, FloorRegular, FloorShaft, FloorVestibule, Floors,
    LiftLimits, PersonSpawnTimer, ShaftCentreX,
};
use crate::game::speed_selector::TargetVelocity;
use crate::game::world_gen::Floor;
use crate::history_store::HistoryStore;
use crate::input_action::InputAction;
use crate::{camera, GameState};
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_ecs_tilemap::prelude::*;
use bevy_inspector_egui::inspector_options::Target;
use derive_new::new;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::user_input::InputKind::Mouse;
use rand::{thread_rng, Rng};
use std::cmp::Ordering;
use std::f32::consts::TAU;
use std::time::Duration;

pub struct GamePlugin;

pub const MAP_Z: f32 = 0.0;
const LIFT_Z: f32 = 210.0;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            (floors::build_floor_map, setup_game).chain(),
        )
        .add_systems(
            Update,
            (
                camera::camera_track_system,
                lift_gizmo_system,
                debug_lift_mode_text,
                floor_proximity_system,
                proximity_timer_display_system,
                spawn_person_system,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            FixedUpdate,
            ((lift_latch_system, move_lift_system).chain(),).run_if(in_state(GameState::Playing)),
        )
        .insert_resource(VelocityLog(HistoryStore::new(512, 1024, 60)))
        .insert_resource(AccelerationLog(HistoryStore::new(512, 1024, 60)))
        .insert_resource(PersonSpawnTimer(Timer::from_seconds(
            5.0,
            TimerMode::Repeating,
        )))
        .register_type::<LiftMode>()
        .register_type::<LinearVelocity>()
        .register_type::<FloorProximity>()
        .register_type::<FloorProximitySensor>()
        .register_type::<FloorShaft>()
        .register_type::<FloorVestibule>()
        .register_type::<FloorRegular>();
    }
}

#[derive(Resource, Debug)]
pub struct VelocityLog(pub HistoryStore<(f32, f32)>);
#[derive(Resource, Debug)]
pub struct AccelerationLog(pub HistoryStore<(f32, f32)>);

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("Setting up game");
    let mut input_map = InputMap::default();
    input_map.insert(KeyCode::W, InputAction::Up);
    input_map.insert(KeyCode::S, InputAction::Down);
    input_map.insert(SingleAxis::mouse_motion_y(), InputAction::MouseMove);
    input_map.insert(Mouse(MouseButton::Left), InputAction::MouseLClick);
    let texture = asset_server.load("textures/lift.png");
    commands
        .spawn(SpriteBundle {
            texture,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, LIFT_Z)),
            ..Default::default()
        })
        .insert(Name::new("Lift"))
        .insert(Lift)
        .insert(LiftMode::Free)
        .insert(LinearVelocity::new((-100.0, 100.0), 100.0))
        .insert(CameraTrack { y_threshold: 50.0 })
        .insert(InputManagerBundle::<InputAction> {
            input_map,
            ..Default::default()
        })
        .insert(RenderLayers::layer(RENDER_LAYER_MAIN))
        .insert(FloorProximitySensor {
            abs_distance_threshold: 10.0,
            max_velocity: 2.5,
            floor_timer_duration: Duration::from_secs(2),
        });

    commands
        .spawn(Text2dBundle {
            text: Text::from_section("", TextStyle::default()),
            ..default()
        })
        .insert(LiftModeDebugText);
}

fn lift_latch_system(
    mut lift_query: Query<(&Transform, &mut LinearVelocity), With<Lift>>,
    latch_y_positions: Res<FloorLatchYPositions>,
    mut gizmos: Gizmos,
) {
    // TODO: Implement me!
}

fn move_lift_system(
    mut lift_query: Query<(&mut Transform, &mut LinearVelocity), With<Lift>>,
    time: Res<Time>,
    shaft_centre_x: Res<ShaftCentreX>,
    lift_limits: Res<LiftLimits>,
    target_velocity: Res<TargetVelocity>,
    mut velocity_log: ResMut<VelocityLog>,
    mut acceleration_log: ResMut<AccelerationLog>,
) {
    let (mut lift_transform, mut actual_velocity) = lift_query.single_mut();

    let accel_this_tick = actual_velocity.update(target_velocity.0, time.delta());

    lift_transform.translation.y = f32::clamp(
        lift_transform.translation.y + (actual_velocity.velocity * time.delta_seconds()),
        lift_limits.min,
        lift_limits.max,
    );
    lift_transform.translation.x = shaft_centre_x.0;
    velocity_log
        .0
        .push((time.elapsed_seconds(), actual_velocity.velocity));
    acceleration_log
        .0
        .push((time.elapsed_seconds(), accel_this_tick));
}

fn lift_gizmo_system(
    floor_latch_ypositions: Res<FloorLatchYPositions>,
    shaft_centre_x: Res<ShaftCentreX>,
    lift_query: Query<&Transform, With<Lift>>,
    mut gizmos: Gizmos,
) {
    let shaft_centre_x = shaft_centre_x.0;
    for y in floor_latch_ypositions.0.iter() {
        gizmos.line_2d(
            Vec2::new(shaft_centre_x - 10.0, *y),
            Vec2::new(shaft_centre_x + 10.0, *y),
            Color::BLUE,
        );
    }

    for lift in lift_query.iter() {
        gizmos.circle_2d(lift.translation.truncate(), 10.0, Color::RED);
    }
}

/// Marker component for the lift
#[derive(Component, Debug, Reflect)]
pub struct Lift;

#[derive(Component, Debug, Reflect)]
enum LiftMode {
    Free,
    Opening,
    Open,
    Closing,
}

impl LiftMode {
    fn as_str(&self) -> &str {
        match self {
            Self::Free => "free",
            Self::Opening => "opening",
            Self::Open => "open",
            Self::Closing => "closing",
        }
    }
}

#[derive(Component, Debug, Reflect)]
struct LiftModeDebugText;

// TODO: Consider splitting in two for transform and text content changes
fn debug_lift_mode_text(
    lift_query: Query<(&GlobalTransform, &LiftMode), With<Lift>>,
    mut text_query: Query<(&mut Transform, &mut Text), (Without<Lift>, With<LiftModeDebugText>)>,
) {
    for (lift_transform, lift_mode) in lift_query.iter() {
        for (mut text_transform, mut text) in text_query.iter_mut() {
            text_transform.translation = lift_transform.translation();
            text_transform.translation.y += 10.0;
            text_transform.translation.z = LIFT_Z + 1.0;
            text.sections[0].value = lift_mode.as_str().to_string();
        }
    }
}

#[derive(Component, Debug, Default, Reflect)]
pub struct LinearVelocity {
    pub velocity: f32,
    bounds: (f32, f32),
    max_accel: f32,
}

impl LinearVelocity {
    fn new(bounds: (f32, f32), max_accel: f32) -> Self {
        Self {
            bounds,
            max_accel,
            velocity: 0.0,
        }
    }
    /// Update self to match target_x, with a maximum change of max_accel
    /// Emits the true acceleration applied
    fn update(&mut self, target_x: f32, delta: Duration) -> f32 {
        // v = u + at
        // solve for a
        // a = (v - u) / t
        // if a is above max_accel, use max_accel instead and resolve
        let target_accel = (target_x - self.velocity) / delta.as_secs_f32();
        if target_accel.abs() > self.max_accel {
            self.velocity += self.max_accel * target_accel.signum() * delta.as_secs_f32();
            self.max_accel * target_accel.signum()
        } else {
            self.velocity = target_x;
            target_accel
        }
    }
}

#[derive(Clone, Debug, Reflect, Component, new)]
struct FloorProximity {
    floor_num: i32,
    time_in_proximity: Timer,
}
#[derive(Clone, Debug, Reflect, Component, new)]
struct FloorProximitySensor {
    abs_distance_threshold: f32,
    max_velocity: f32,
    floor_timer_duration: Duration,
}

fn floor_proximity_system(
    mut commands: Commands,
    mut lift_query: Query<(
        Entity,
        &Transform,
        &LinearVelocity,
        &FloorProximitySensor,
        Option<&mut FloorProximity>,
    )>,
    floors: Res<Floors>,
    time: Res<Time>,
) {
    for (entity, lift_transform, velocity, sensor, mut proximity) in lift_query.iter_mut() {
        if let Some((closest_floor, closest_floor_y)) =
            floors.closest_floor(lift_transform.translation.y)
        {
            let close_enough = (lift_transform.translation.y - closest_floor_y).abs()
                < sensor.abs_distance_threshold;
            let slow_enough = velocity.velocity.abs() < sensor.max_velocity;

            // If all matches up, tick the timer, otherwise reset
            if let Some(mut floor_proximity) = proximity {
                let same_floor = floor_proximity.floor_num == closest_floor;
                if same_floor && close_enough && slow_enough {
                    floor_proximity.time_in_proximity.tick(time.delta());
                } else {
                    floor_proximity.floor_num = closest_floor;
                    floor_proximity.time_in_proximity.reset();
                }
            } else {
                commands.entity(entity).insert(FloorProximity::new(
                    closest_floor,
                    Timer::new(sensor.floor_timer_duration, TimerMode::Once),
                ));
            }
        }
    }
}

fn proximity_timer_display_system(
    query: Query<(Option<&FloorProximity>, &FloorProximitySensor, &Transform)>,
    mut gizmos: Gizmos,
) {
    for (proximity, sensor, transform) in query.iter() {
        if let Some(proximity) = proximity {
            let fill_pct = proximity.time_in_proximity.elapsed().as_secs_f32()
                / sensor.floor_timer_duration.as_secs_f32();
            gizmos.arc_2d(
                transform.translation.truncate(),
                0.0,
                TAU * fill_pct,
                15.0,
                Color::BLUE,
            );
        }
    }
}
