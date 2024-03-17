use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;

pub const RENDER_LAYER_MAIN: u8 = 0;
pub const RENDER_LAYER_OVERLAY: u8 = 1;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_cameras))
            .add_systems(FixedUpdate, (camera_track_system));
    }
}
fn setup_cameras(mut commands: Commands) {
    let camera_scale = 0.5;
    commands
        .spawn(Camera2dBundle {
            camera: Camera {
                order: 0,
                ..default()
            },
            projection: OrthographicProjection {
                scale: camera_scale,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1000.0)),
            ..default()
        })
        .insert(RenderLayers::layer(RENDER_LAYER_MAIN))
        .insert(MainCamera)
        .insert(Name::new("MainCamera"));

    commands
        .spawn(Camera2dBundle {
            camera: Camera {
                order: 1,
                ..default()
            },
            camera_2d: Camera2d {
                // no "background color", we need to see the main camera's output
                clear_color: ClearColorConfig::None,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1000.0)),
            ..default()
        })
        .insert(RenderLayers::layer(RENDER_LAYER_OVERLAY))
        .insert(OverlayCamera)
        .insert(Name::new("OverlayCamera"));
}

#[derive(Component, Debug, Default)]
pub struct MainCamera;
#[derive(Component, Debug, Default)]
pub struct OverlayCamera;

#[derive(Component, Debug, Default)]
pub struct CameraTrack {
    pub y_threshold: f32,
}

pub fn camera_track_system(
    mut camera_query: Query<(&mut Transform), (With<Camera>, With<MainCamera>)>,
    tracked_query: Query<(&Transform, &CameraTrack), Without<Camera>>,
) {
    for (transform, camera_track) in tracked_query.iter() {
        for (mut camera_transform) in camera_query.iter_mut() {
            let y = f32::clamp(
                camera_transform.translation.y,
                transform.translation.y - camera_track.y_threshold,
                transform.translation.y + camera_track.y_threshold,
            );
            camera_transform.translation.y = y;
            camera_transform.translation.x = transform.translation.x - 200.0;
        }
    }
}
