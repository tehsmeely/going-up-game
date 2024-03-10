use crate::{GameState, InputAction};
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;
use std::thread::spawn;

const DIAL_Z: f32 = 100.0;
const HANDLE_Z: f32 = 101.0;
const ROTATION_MAX: f32 = 1.3;

// TODO: This should be user configurable
/// Factor by which mouse movement is turned into rotation radians
const MOUSE_MOVE_ROTATION_FACTOR: f32 = 0.005;

// TODO: Consider this factor being configurable and maybe non-linear?
/// Factor by which target velocity is determined by multiplying with rotation of handle
const TARGET_VELOCITY_FACTOR: f32 = 10.0;

pub struct SpeedSelectorPlugin;

impl Plugin for SpeedSelectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), (spawn_selector))
            .add_systems(
                Update,
                (update_selector, handle_selector_input).run_if(in_state(GameState::Playing)),
            )
            .insert_resource(TargetVelocity(0.0))
            .register_type::<Rotation>();
    }
}

fn spawn_selector(mut commands: Commands, asset_server: Res<AssetServer>) {
    let dial_handle = asset_server.load("textures/speed-dial.png");
    let handle_handle = asset_server.load("textures/speed-handle.png");
    commands
        .spawn(SpriteBundle {
            texture: dial_handle,
            transform: Transform::from_translation(Vec2::default().extend(DIAL_Z)),
            ..default()
        })
        .insert(Name::from("Speed Dial"))
        .insert(SpeedDial)
        .with_children(|parent| {
            parent
                .spawn(SpriteBundle {
                    texture: handle_handle,
                    transform: Transform::from_translation(Vec2::default().extend(HANDLE_Z)),
                    ..default()
                })
                .insert(Name::from("Speed Handle"))
                .insert(Rotation::default())
                .insert(SpeedHandle);
        });
}

fn update_selector(
    mut query: Query<(&mut Rotation, &mut Transform), (With<SpeedHandle>)>,
    mut target_velocity: ResMut<TargetVelocity>,
) {
    for (mut rotation, mut transform) in query.iter_mut() {
        let diff = rotation.update();
        transform.rotate_around(Vec3::new(18.0, 0.0, 0.0), Quat::from_rotation_z(diff));
        target_velocity.0 = rotation.actual * TARGET_VELOCITY_FACTOR;
    }
}

fn handle_selector_input(
    inputs: Query<&ActionState<InputAction>>,
    mut handle_query: Query<(&mut Rotation), With<SpeedHandle>>,
) {
    let inputs = inputs.single();
    let mouse = inputs.value(InputAction::MouseMove);

    let mouse_button_held = inputs.value(InputAction::MouseLClick) != 0.0;

    if mouse != 0.0 && mouse_button_held {
        let angle_change = mouse * MOUSE_MOVE_ROTATION_FACTOR;
        for mut rotation in handle_query.iter_mut() {
            let target = rotation.target + angle_change;
            rotation.set(target)
        }
    }
}

#[derive(Clone, Debug, Default, Component)]
struct SpeedHandle;

#[derive(Clone, Debug, Default, Component)]
struct SpeedDial;

#[derive(Clone, Debug, Component, Reflect)]
struct Rotation {
    actual: f32,
    target: f32,
}

impl Rotation {
    fn set(&mut self, target: f32) {
        self.target = f32::clamp(target, -ROTATION_MAX, ROTATION_MAX);
    }

    fn update(&mut self) -> f32 {
        let new = crate::helpers::lerp(self.actual, self.target, 0.2);
        let diff = new - self.actual;
        self.actual = new;
        return diff;
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Self {
            actual: 0.0,
            target: 0.0,
        }
    }
}

#[derive(Clone, Debug, Deref, DerefMut, Resource)]
pub struct TargetVelocity(pub f32);
