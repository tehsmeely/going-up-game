use crate::input_action::InputAction;
use crate::{GameState, MainCamera};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use leafwing_input_manager::prelude::*;
use rand::{thread_rng, Rng};

pub struct GamePlugin;

const MAP_Z: f32 = 0.0;
const LIFT_Z: f32 = 10.0;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            (build_floor_map, setup_game, setup_camera).chain(),
        )
        .add_systems(
            Update,
            (move_lift_system, camera_track_system, lift_gizmo_system)
                .run_if(in_state(GameState::Playing)),
        )
        .register_type::<LinearVelocity>();
    }
}

#[derive(Resource, Debug, Default, Reflect)]
struct ShaftCentreX(f32);

#[derive(Resource, Debug, Default, Reflect)]
struct LiftLimits {
    min: f32,
    max: f32,
}

#[derive(Resource, Debug, Default, Reflect)]
struct FloorLatchYPositions(Vec<f32>);

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("Setting up game");
    let mut input_map = InputMap::default();
    input_map.insert(KeyCode::W, InputAction::Up);
    input_map.insert(KeyCode::S, InputAction::Down);
    let texture = asset_server.load("textures/lift.png");
    commands
        .spawn(SpriteBundle {
            texture,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, LIFT_Z)),
            ..Default::default()
        })
        .insert(Name::new("Lift"))
        .insert(Lift)
        .insert(LinearVelocity {
            max_y: Some((-100.0, 100.0)),
            ..default()
        })
        .insert(Acceleration(100.0))
        .insert(CameraTrack { y_threshold: 50.0 })
        .insert(InputManagerBundle::<InputAction> {
            input_map,
            ..Default::default()
        });
}

fn build_floor_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    array_texture_loader: Res<ArrayTextureLoader>,
) {
    println!("Building floor map");
    let texture: Handle<Image> = asset_server.load("textures/floor_tile.png");
    let tilemap_entity = commands.spawn_empty().id();
    let shaft_x = 10;
    let map_size = TilemapSize {
        x: shaft_x + 1,
        y: 10,
    };
    let mut tile_storage = TileStorage::empty(map_size);

    let tile_size = TilemapTileSize { x: 60.0, y: 60.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    let mut rng = thread_rng();

    // Intially populated with raw positions, then will be mapped with the tilemap transform after
    let mut floor_latch_y_positions = Vec::new();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos::new(x, y);
            let texture_index = {
                let texture_index = if x == shaft_x {
                    3
                } else if x == (shaft_x - 1) {
                    2
                } else {
                    rng.gen_range(0..2)
                };
                TileTextureIndex(texture_index)
            };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index,
                    ..default()
                })
                .id();
            floor_latch_y_positions.push(y as f32 * tile_size.y);
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tilemap_transform = get_tilemap_center_transform(&map_size, &grid_size, &map_type, MAP_Z);

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture.clone()),
        tile_size,
        transform: tilemap_transform,
        ..Default::default()
    });
    array_texture_loader.add(TilemapArrayTexture {
        texture: TilemapTexture::Single(texture),
        tile_size,
        ..Default::default()
    });

    let floor_latch_y_positions: Vec<f32> = floor_latch_y_positions
        .iter()
        .map(|y| y + tilemap_transform.translation.y)
        .collect();
    commands.insert_resource(FloorLatchYPositions(floor_latch_y_positions));

    let shaft_centre_x = (shaft_x as f32 * tile_size.x) + tilemap_transform.translation.x;
    commands.insert_resource(ShaftCentreX(shaft_centre_x));

    let lift_limits = LiftLimits {
        min: tilemap_transform.translation.y,
        max: tilemap_transform.translation.y + ((map_size.y - 1) as f32 * tile_size.y),
    };
    commands.insert_resource(lift_limits);
}

fn move_lift_system(
    inputs: Query<&ActionState<InputAction>>,
    mut lift_query: Query<(&mut Transform, &mut LinearVelocity, &Acceleration), With<Lift>>,
    time: Res<Time>,
    shaft_centre_x: Res<ShaftCentreX>,
    lift_limits: Res<LiftLimits>,
) {
    let inputs = inputs.single();
    let (mut lift_transform, mut lift_velocity, acceleration) = lift_query.single_mut();
    let velocity_change = if inputs.pressed(InputAction::Up) {
        time.delta_seconds() * acceleration.0
    } else if inputs.pressed(InputAction::Down) {
        time.delta_seconds() * acceleration.0 * -1.0
    } else {
        0.0
    };

    lift_velocity.add(0.0, velocity_change);

    // TODO: Reset velocity if we hit a limit
    lift_transform.translation.y = f32::clamp(
        lift_transform.translation.y + (lift_velocity.y * time.delta_seconds()),
        lift_limits.min,
        lift_limits.max,
    );
    lift_transform.translation.x = shaft_centre_x.0;
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
struct Lift;

#[derive(Component, Debug, Default, Reflect)]
struct LinearVelocity {
    x: f32,
    y: f32,
    max_x: Option<(f32, f32)>,
    max_y: Option<(f32, f32)>,
}

impl LinearVelocity {
    fn add(&mut self, x: f32, y: f32) {
        self.x = match self.max_x {
            Some((min, max)) => (self.x + x).clamp(min, max),
            None => self.x + x,
        };
        self.y = match self.max_y {
            Some((min, max)) => (self.y + y).clamp(min, max),
            None => self.y + x,
        };
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut)]
struct Acceleration(f32);

#[derive(Component, Debug, Default)]
struct CameraTrack {
    y_threshold: f32,
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

pub fn setup_camera(
    mut camera_query: Query<(&mut OrthographicProjection), (With<Camera>, With<MainCamera>)>,
) {
    for (mut camera) in camera_query.iter_mut() {
        camera.scale = 0.5;
    }
}
