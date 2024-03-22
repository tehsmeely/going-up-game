use crate::game::game::MAP_Z;
use crate::game::human_store;
use crate::game::human_store::{Human, HumanStore, PositionIndex};
use crate::loading::TextureAssets;
use bevy::ecs::system::EntityCommands;
use bevy::hierarchy::BuildChildren;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_ecs_tilemap::map::{
    TilemapGridSize, TilemapId, TilemapSize, TilemapTexture, TilemapTileSize, TilemapType,
};
use bevy_ecs_tilemap::prelude::{
    get_tilemap_center_transform, ArrayTextureLoader, TileBundle, TilePos, TileStorage,
    TileTextureIndex, TilemapArrayTexture,
};
use bevy_ecs_tilemap::TilemapBundle;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::cmp::Ordering;

#[derive(Resource, Debug, Default, Reflect)]
pub struct FloorLatchYPositions(pub Vec<f32>);

#[derive(Resource, Debug, Default, Reflect)]
pub struct Floors {
    pub floor_y_positions: Vec<(i32, f32)>,
    //TODO: Add more here
}

#[derive(Resource, Debug, Default, Reflect)]
pub struct ShaftCentreX(pub f32);

impl Floors {
    pub fn closest_floor(&self, lift_y: f32) -> Option<(i32, f32)> {
        self.floor_y_positions
            .iter()
            .map(|(floor, floor_y)| (*floor, floor_y, (floor_y - lift_y).abs()))
            .min_by(|(_, _, a), (_, _, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .map(|(floor, floor_y, _)| (floor, *floor_y))
    }
}

/// Marker for the tilemap floor segment of the vestibule (i.e. entrance to lift)
#[derive(Debug, Default, Reflect, Component)]
pub struct FloorVestibule {
    floor_num: i32,
}

/// Marker for the tilemap floor segment of the shaft (i.e. where the lift runs)
#[derive(Debug, Default, Reflect, Component)]
pub struct FloorShaft {
    floor_num: i32,
}

/// Marker for the tilemap floor segment of regular floors
#[derive(Debug, Default, Reflect, Component)]
pub struct FloorRegular {
    floor_num: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FloorKind {
    Vestibule,
    Shaft,
    Regular,
}

impl FloorKind {
    fn insert_marker_component(&self, floor_num: i32, commands: &mut EntityCommands) {
        match self {
            Self::Vestibule => commands.insert(FloorVestibule { floor_num }),
            Self::Shaft => commands.insert(FloorShaft { floor_num }),
            Self::Regular => commands.insert(FloorRegular { floor_num }),
        };
    }

    fn texture_index(&self, rng: &mut impl Rng) -> TileTextureIndex {
        TileTextureIndex(match self {
            Self::Vestibule => 2,
            Self::Shaft => 3,
            Self::Regular => rng.gen_range(0..2),
        })
    }

    fn name(&self) -> Name {
        match self {
            Self::Vestibule => Name::new("Vestibule"),
            Self::Shaft => Name::new("Shaft"),
            Self::Regular => Name::new("Regular"),
        }
    }
}

pub fn build_floor_map(
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

    let tile_size_vec2 = Vec2::new(60.0, 60.0);
    let tile_size = TilemapTileSize {
        x: tile_size_vec2.x,
        y: tile_size_vec2.y,
    };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    let mut rng = thread_rng();

    // Initially populated with raw positions, then will be mapped with the tilemap transform after
    let mut floor_latch_y_positions = Vec::new();
    let mut vestibule_locations = Vec::new();

    let mut child_tiles = Vec::new();
    for x in 0..map_size.x {
        for floor_num in 0..map_size.y {
            let tile_pos = TilePos::new(x, floor_num);
            let floor_kind = if x == shaft_x {
                FloorKind::Shaft
            } else if x == (shaft_x - 1) {
                FloorKind::Vestibule
            } else {
                FloorKind::Regular
            };
            let texture_index = floor_kind.texture_index(&mut rng);
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index,
                    ..default()
                })
                .insert(floor_kind.name())
                .id();
            floor_kind.insert_marker_component(floor_num as i32, &mut commands.entity(tile_entity));
            floor_latch_y_positions.push(floor_num as f32 * tile_size.y);
            if floor_kind == FloorKind::Vestibule {
                let pos = tile_size_vec2 * Vec2::new(x as f32, floor_num as f32);
                vestibule_locations.push(pos);
            }
            tile_storage.set(&tile_pos, tile_entity);
            child_tiles.push(tile_entity);
        }
    }
    commands.entity(tilemap_entity).push_children(&child_tiles);

    let tilemap_transform = get_tilemap_center_transform(&map_size, &grid_size, &map_type, MAP_Z);

    // Spawn full tilemap
    commands
        .entity(tilemap_entity)
        .insert(TilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture.clone()),
            tile_size,
            transform: tilemap_transform,
            ..Default::default()
        })
        .insert(Name::new("TileMap"));
    array_texture_loader.add(TilemapArrayTexture {
        texture: TilemapTexture::Single(texture),
        tile_size,
        ..Default::default()
    });

    // Spawn human stores
    for vestibule_pos in vestibule_locations {
        let pos = vestibule_pos + tilemap_transform.translation.truncate();
        commands
            .spawn(SpatialBundle::from_transform(Transform::from_translation(
                pos.extend(MAP_Z + 1.0),
            )))
            .insert(HumanStore {
                max_humans: 5,
                spawn_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            })
            .insert(Name::new("HumanStore"));
    }

    // Adjust floor latch y positions to be in world space
    let floor_latch_y_positions: Vec<f32> = floor_latch_y_positions
        .iter()
        .map(|y| y + tilemap_transform.translation.y)
        .collect();
    commands.insert_resource(FloorLatchYPositions(floor_latch_y_positions.clone()));

    let floors: Vec<(i32, f32)> = floor_latch_y_positions
        .into_iter()
        .enumerate()
        .map(|(i, y)| (i as i32, y))
        .collect();
    commands.insert_resource(Floors {
        floor_y_positions: floors,
    });

    let shaft_centre_x = (shaft_x as f32 * tile_size.x) + tilemap_transform.translation.x;
    commands.insert_resource(ShaftCentreX(shaft_centre_x));

    let lift_limits = LiftLimits {
        min: tilemap_transform.translation.y,
        max: tilemap_transform.translation.y + ((map_size.y - 1) as f32 * tile_size.y),
    };
    commands.insert_resource(lift_limits);
}

#[derive(Resource, Debug, Default, Reflect)]
pub struct LiftLimits {
    pub min: f32,
    pub max: f32,
}

#[derive(Clone, Debug, Component)]
pub struct Person;

#[derive(Clone, Debug, Resource, Deref, DerefMut)]
pub struct PersonSpawnTimer(pub Timer);

fn spawn_person(commands: &mut Commands, asset_server: &Res<AssetServer>, translation: Vec3) {
    let person_texture: Handle<Image> = asset_server.load("textures/person.png");
    commands
        .spawn(SpriteBundle {
            texture: person_texture,
            transform: Transform::from_translation(translation),
            ..Default::default()
        })
        .insert(Person)
        .insert(Name::new("Person"))
        .insert(RenderLayers::layer(crate::camera::RENDER_LAYER_MAIN));
}

pub fn human_store_spawn_humans_system(
    mut query: Query<(Entity, &mut HumanStore, Option<&Children>)>,
    human_query: Query<(&PositionIndex), With<Human>>,
    time: Res<Time>,
    texture_assets: Res<TextureAssets>,
    mut commands: Commands,
) {
    for (entity, mut human_store, children) in query.iter_mut() {
        human_store.spawn_timer.tick(time.delta());
        let num_children = children.map_or(0, |c| c.len());
        if human_store.spawn_timer.just_finished() && num_children < human_store.max_humans {
            human_store::add_human_to_store(&human_query, entity, &texture_assets, &mut commands);
        }
    }
}

// TODO: This system needs to generally be a bit more complicated. Floor spawns will be different
// per floor but also need to be centrally orchestrated. We also will want to spawn multiple, different
// people and display them accordingly
// See design doc for more details
/*
pub fn spawn_person_system(
    query: Query<(&FloorVestibule, &TilePos, Option<&Person>)>,
    tilemap_query: Query<(&TilemapGridSize, &TilemapType, &Transform)>,
    asset_server: Res<AssetServer>,
    mut person_spawn_timer: ResMut<PersonSpawnTimer>,
    time: Res<Time>,
    mut commands: Commands,
) {
    person_spawn_timer.tick(time.delta());
    if person_spawn_timer.just_finished() {
        println!("Attempting to spawn person");
        let possible_spawn_floors: Vec<i32> = query
            .iter()
            .filter_map(
                |(floor_vestibule, _tile_pos, maybe_person)| match maybe_person.is_some() {
                    true => None,
                    false => Some(floor_vestibule.floor_num),
                },
            )
            .collect();
        let mut rng = thread_rng();
        if let Some(spawn_floor) = (&possible_spawn_floors).choose(&mut rng) {
            println!("Spawning person on floor {}", spawn_floor);
            for (floor_vestibule, tile_pos, _) in query.iter() {
                if floor_vestibule.floor_num == *spawn_floor {
                    let (tilemap_grid_size, tilemap_map_size, tilemap_transform) =
                        tilemap_query.single();
                    let position = tile_pos
                        .center_in_world(tilemap_grid_size, tilemap_map_size)
                        .extend(MAP_Z + 1.0)
                        + tilemap_transform.translation;
                    println!("Actually spawning person at position : {:?}", position);
                    spawn_person(&mut commands, &asset_server, position);
                }
            }
        }
    }
}

 */
