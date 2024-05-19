use crate::camera::OverlayCamera;
use crate::core::{InScreenSpaceLocation, ScreenSpaceAnchor, With2DScale};
use crate::{GameState, InputAction};
use bevy::asset::AssetLoader;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
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
const TARGET_VELOCITY_FACTOR: f32 = -50.0;

const SCALE: f32 = 2.0;

pub struct SpeedSelectorPlugin;

#[derive(Default, Reflect, GizmoConfigGroup)]
struct OverlayGizmos {}

impl Plugin for SpeedSelectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::PlayingDay),
            (spawn_selector, OverlayGizmos::setup),
        )
        .add_systems(
            Update,
            (
                update_selector,
                handle_selector_input,
                cursor_position_system,
                mouse_selection_rect_debug_gizmo,
                position_cursor_selection_rect_system,
            )
                .run_if(in_state(GameState::PlayingDay)),
        )
        .init_gizmo_group::<OverlayGizmos>()
        .insert_resource(TargetVelocity(0.0))
        .insert_resource(SelectionEnabled(false))
        .register_type::<Rotation>()
        .register_type::<SelectionEnabled>()
        .register_type::<MouseSelectionRect>();
    }
}

impl OverlayGizmos {
    fn setup(mut config_store: ResMut<GizmoConfigStore>) {
        let (config, _) = config_store.config_mut::<Self>();
        config.render_layers = RenderLayers::layer(crate::camera::RENDER_LAYER_OVERLAY);
    }
}

fn spawn_selector(mut commands: Commands, asset_server: Res<AssetServer>) {
    let render_layers = RenderLayers::layer(crate::camera::RENDER_LAYER_OVERLAY);
    let dial_handle = asset_server.load("textures/speed-dial.png");
    let handle_handle = asset_server.load("textures/speed-handle.png");
    commands
        .spawn(SpriteBundle {
            texture: dial_handle,
            transform: Transform::from_translation(Vec2::default().extend(DIAL_Z)),
            ..default()
        })
        .insert(MouseSelectionRect::new(
            Vec2::new(190.0, 370.0),
            Rect::new(0.0, 0.0, 0.0, 0.0),
        ))
        .insert(Name::from("Speed Dial"))
        .insert(SpeedDial)
        .insert(render_layers.clone())
        .insert(With2DScale::new(2.0))
        .insert(InScreenSpaceLocation::new(ScreenSpaceAnchor::Right, 80.0))
        .with_children(|parent| {
            parent
                .spawn(SpriteBundle {
                    texture: handle_handle,
                    transform: Transform::from_translation(Vec2::default().extend(HANDLE_Z)),
                    ..default()
                })
                .insert(Name::from("Speed Handle"))
                .insert(Rotation::default())
                .insert(render_layers.clone())
                .insert(With2DScale::new(1.0))
                .insert(SpeedHandle);
        });
}

fn update_selector(
    mut query: Query<(&mut Rotation, &mut Transform), (With<SpeedHandle>)>,
    mut target_velocity: ResMut<TargetVelocity>,
) {
    for (mut rotation, mut transform) in query.iter_mut() {
        let diff = rotation.update();
        transform.rotate_around(Vec3::new(23.0, 0.0, 0.0), Quat::from_rotation_z(diff));
        target_velocity.0 = (rotation.actual.abs() * rotation.actual.abs())
            * TARGET_VELOCITY_FACTOR
            * rotation.actual.signum();
    }
}

fn mouse_selection_rect_debug_gizmo(
    selector_query: Query<(&MouseSelectionRect), With<SpeedDial>>,
    mut gizmos: Gizmos<OverlayGizmos>,
) {
    for (selector_rect) in selector_query.iter() {
        gizmos.rect_2d(
            selector_rect.world_rect.center(),
            0.0,
            selector_rect.world_rect.half_size(),
            Color::RED,
        );
    }
}

#[derive(Debug, Component, Reflect)]
struct MouseSelectionRect {
    size: Vec2,
    world_rect: Rect,
}
impl MouseSelectionRect {
    fn new(size: Vec2, world_rect: Rect) -> Self {
        Self { size, world_rect }
    }

    fn set_middle(&mut self, middle: Vec2) {
        self.world_rect = Rect::new(
            middle.x - self.size.x / 2.0,
            middle.y - self.size.y / 2.0,
            middle.x + self.size.x / 2.0,
            middle.y + self.size.y / 2.0,
        );
    }
}
#[derive(Debug, Resource, Reflect)]
struct SelectionEnabled(bool);

fn cursor_position_system(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<OverlayCamera>>,
    selector_query: Query<(&MouseSelectionRect), With<SpeedDial>>,
    mut selection_enabled: ResMut<SelectionEnabled>,
) {
    let (camera, camera_transform) = camera_query.single();
    let mut window = window_query.single_mut();
    let selector_rect = selector_query.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        selection_enabled.0 = selector_rect.world_rect.contains(world_position);
        window.cursor.icon = match selection_enabled.0 {
            true => bevy::window::CursorIcon::Pointer,
            false => bevy::window::CursorIcon::Default,
        };
    }
}

fn position_cursor_selection_rect_system(
    mut selector_query: Query<
        (&Transform, &mut MouseSelectionRect),
        (With<SpeedDial>, Changed<Transform>),
    >,
) {
    for (transform, mut selector_rect) in selector_query.iter_mut() {
        selector_rect.set_middle(transform.translation.truncate());
    }
}

fn handle_selector_input(
    inputs: Query<&ActionState<InputAction>>,
    mut handle_query: Query<(&mut Rotation), With<SpeedHandle>>,
    mut mouse_held: Local<bool>,
    selection_enabled: Res<SelectionEnabled>,
) {
    let inputs = inputs.single();
    let mouse = inputs.value(&InputAction::MouseMove);

    if !*mouse_held && inputs.just_pressed(&InputAction::MouseLClick) && selection_enabled.0 {
        *mouse_held = true;
    } else if *mouse_held && inputs.just_released(&InputAction::MouseLClick) {
        *mouse_held = false;
    }

    if mouse != 0.0 && *mouse_held {
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
