use crate::camera::{CameraTrack, RENDER_LAYER_MAIN};
use crate::game::floors::{
    floor_spawn_process_system, human_store_spawn_humans_system, FloorLatchYPositions, FloorNum,
    FloorRegular, FloorShaft, FloorVestibule, Floors, LiftLimits, PersonSpawnTimer, ShaftCentreX,
    SpawnHumansEvent,
};
use crate::game::human_store;
use crate::game::human_store::{
    FloorDesire, HowMany, Human, HumanStore, PositionIndex, Unavailable,
};
use crate::game::lift::LiftHumanStore;
use crate::game::speed_selector::TargetVelocity;
use crate::game::world_gen::Floor;
use crate::game::{floors, lift};
use crate::history_store::HistoryStore;
use crate::input_action::InputAction;
use crate::loading::TextureAssets;
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

pub const MAP_Z: f32 = 0.5;
const LIFT_Z: f32 = 210.0;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::PlayingDay),
            (floors::build_floor_map, setup_game, setup_background).chain(),
        )
        .add_systems(
            Update,
            (
                lift_gizmo_system,
                debug_lift_mode_text,
                floor_proximity_system,
                proximity_timer_display_system,
                floor_proximity_effect_system.after(floor_proximity_system),
                floor_spawn_process_system,
                human_store_spawn_humans_system,
                human_store::floor_desire_system,
                human_store::human_marker_component_system,
                lift::LiftHumanStore::update_system,
            )
                .run_if(in_state(GameState::PlayingDay)),
        )
        .add_systems(
            FixedUpdate,
            ((lift_latch_system, move_lift_system).chain(),)
                .run_if(in_state(GameState::PlayingDay)),
        )
        .insert_resource(VelocityLog(HistoryStore::new(512, 1024, 60)))
        .insert_resource(ObservedVelocityLog(HistoryStore::new(512, 1024, 60)))
        .insert_resource(AccelerationLog(HistoryStore::new(512, 1024, 60)))
        .insert_resource(PersonSpawnTimer(Timer::from_seconds(
            5.0,
            TimerMode::Repeating,
        )))
        .add_event::<SpawnHumansEvent>()
        .register_type::<LiftMode>()
        .register_type::<LinearVelocity>()
        .register_type::<FloorProximity>()
        .register_type::<FloorProximitySensor>()
        .register_type::<FloorShaft>()
        .register_type::<FloorVestibule>()
        .register_type::<FloorRegular>()
        .register_type::<FloorNum>()
        .register_type::<HumanStore>()
        .register_type::<PositionIndex>()
        .register_type::<Human>()
        .register_type::<FloorDesire>()
        .register_type::<LiftLimits>()
        .register_type::<Floors>()
        .register_type::<FloorLatchYPositions>();

        super::lift::add(app);
    }
}

#[derive(Resource, Debug)]
pub struct VelocityLog(pub HistoryStore<(f32, f32)>);
#[derive(Resource, Debug)]
pub struct ObservedVelocityLog(pub HistoryStore<(f32, f32)>);
#[derive(Resource, Debug)]
pub struct AccelerationLog(pub HistoryStore<(f32, f32)>);

fn setup_background(mut commands: Commands, assets: Res<TextureAssets>) {
    commands.spawn(SpriteBundle {
        texture: assets.city_background_1.clone(),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..default()
    });
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("Setting up game");
    let mut input_map = InputMap::default();
    input_map.insert(InputAction::Up, KeyCode::KeyW);
    input_map.insert(InputAction::Down, KeyCode::KeyS);
    input_map.insert(InputAction::MouseMove, SingleAxis::mouse_motion_y());
    input_map.insert(InputAction::MouseLClick, Mouse(MouseButton::Left));
    input_map.insert(InputAction::ZoomIn, KeyCode::KeyQ);
    input_map.insert(InputAction::ZoomOut, KeyCode::KeyE);
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
        .insert(ObservedVelocity(0.0))
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
    mut lift_query: Query<(&mut Transform, &mut LinearVelocity, &mut ObservedVelocity), With<Lift>>,
    time: Res<Time>,
    shaft_centre_x: Res<ShaftCentreX>,
    lift_limits: Res<LiftLimits>,
    target_velocity: Res<TargetVelocity>,
    mut velocity_log: ResMut<VelocityLog>,
    mut observed_velocity_log: ResMut<ObservedVelocityLog>,
    mut acceleration_log: ResMut<AccelerationLog>,
) {
    let (mut lift_transform, mut actual_velocity, mut observed_velocity) = lift_query.single_mut();

    let accel_this_tick = actual_velocity.update(target_velocity.0, time.delta());

    let new_y = f32::clamp(
        lift_transform.translation.y + (actual_velocity.velocity * time.delta_seconds()),
        lift_limits.min,
        lift_limits.max,
    );
    let dy = (new_y - lift_transform.translation.y).abs();
    observed_velocity.0 = dy / time.delta_seconds();
    lift_transform.translation.y = new_y;
    lift_transform.translation.x = shaft_centre_x.0;
    velocity_log
        .0
        .push((time.elapsed_seconds(), actual_velocity.velocity));
    acceleration_log
        .0
        .push((time.elapsed_seconds(), accel_this_tick));
    observed_velocity_log
        .0
        .push((time.elapsed_seconds(), observed_velocity.0));
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

#[derive(Component, Debug, Default, Reflect)]
pub struct ObservedVelocity(f32);

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
        &mut LiftMode,
    )>,
    floors: Res<Floors>,
    time: Res<Time>,
) {
    for (entity, lift_transform, velocity, sensor, mut proximity, mut lift_mode) in
        lift_query.iter_mut()
    {
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
                    *lift_mode = if floor_proximity.time_in_proximity.finished() {
                        LiftMode::Open
                    } else {
                        LiftMode::Opening
                    };
                } else {
                    floor_proximity.floor_num = closest_floor;
                    floor_proximity.time_in_proximity.reset();
                    *lift_mode = LiftMode::Free;
                }
            } else {
                commands.entity(entity).insert(FloorProximity::new(
                    closest_floor,
                    Timer::new(sensor.floor_timer_duration, TimerMode::Once),
                ));
            }
        } else {
            *lift_mode = LiftMode::Free
        }
    }
}

fn floor_proximity_effect_system(
    query: Query<&FloorProximity>,
    human_store_query: Query<(Entity, &FloorNum), With<HumanStore>>,
    human_query: Query<
        (Entity, &FloorDesire, &PositionIndex, &Parent),
        (With<Human>, Without<Unavailable>),
    >,
    mut commands: Commands,
    mut held_humans: ResMut<LiftHumanStore>,
) {
    for proximity in query.iter() {
        if proximity.time_in_proximity.finished() {
            println!(
                "Collecting from and delivering to Floor {}!!",
                proximity.floor_num
            );
            let humans_desiring_this_floor = held_humans.take_for_floor(proximity.floor_num);
            println!("Delivered humans: {:?}", humans_desiring_this_floor);

            // Looping the second query inside here seems like it'd be O(n^2) but in practice
            // there will only ever be one floor proximity at a time, so it's fine.
            for (store_entity, floor_num) in human_store_query.iter() {
                if floor_num.0 == proximity.floor_num {
                    let capacity = held_humans.free_capacity();
                    let picked_up_floor_desires = human_store::remove_humans(
                        &human_query,
                        store_entity,
                        &mut commands,
                        HowMany::N(capacity),
                    );
                    println!(
                        "Picked up {} humies ({:?})",
                        picked_up_floor_desires.len(),
                        picked_up_floor_desires
                    );
                    held_humans.add(picked_up_floor_desires);
                }
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
