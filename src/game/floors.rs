use crate::game::game::MAP_Z;
use crate::game::game_clock::GameTime;
use crate::game::human_store;
use crate::game::human_store::{Human, HumanStore, HumanStoreBundle, PositionIndex};
use crate::game::spawn_simulation::{prefabs, FloorSpawnManager, RawFloorConfig};
use crate::game::ui::GameCentralInfo;
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
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Formatter;

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
#[derive(Debug, Default, Reflect, Component)]
pub struct FloorBorder {
    floor_num: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FloorKind {
    Vestibule,
    Shaft,
    Regular,
    LeftWall,
    RightWall,
    Bottom,
    BottomLeftCorner,
    BottomRightCorner,
    Roof,
    TopRightCorner,
    TopLeftCorner,
    ShaftRoof,
}

impl FloorKind {
    fn insert_marker_component(&self, floor_num: i32, commands: &mut EntityCommands) {
        match self {
            Self::Vestibule => commands.insert(FloorVestibule { floor_num }),
            Self::Shaft => commands.insert(FloorShaft { floor_num }),
            Self::Regular => commands.insert(FloorRegular { floor_num }),
            _ => commands.insert(FloorBorder { floor_num }),
        };
    }

    fn texture_index(&self, rng: &mut impl Rng) -> TileTextureIndex {
        TileTextureIndex(match self {
            Self::Vestibule => 2,
            Self::Shaft => 3,
            Self::Regular => rng.gen_range(0..2),
            Self::LeftWall => 4,
            Self::RightWall => 5,
            Self::Bottom => 6,
            Self::BottomLeftCorner => 7,
            Self::BottomRightCorner => 8,
            Self::Roof => 9,
            Self::TopRightCorner => 10,
            Self::TopLeftCorner => 11,
            Self::ShaftRoof => 12,
        })
    }

    fn name(&self) -> Name {
        match self {
            Self::Vestibule => Name::new("Vestibule"),
            Self::Shaft => Name::new("Shaft"),
            Self::Regular => Name::new("Regular"),
            Self::LeftWall => Name::new("LeftWall"),
            Self::RightWall => Name::new("RightWall"),
            Self::Bottom => Name::new("Bottom"),
            Self::BottomLeftCorner => Name::new("BottomLeftCorner"),
            Self::BottomRightCorner => Name::new("BottomRightCorner"),
            Self::Roof => Name::new("Roof"),
            Self::TopRightCorner => Name::new("TopRightCorner"),
            Self::TopLeftCorner => Name::new("TopLeftCorner"),
            Self::ShaftRoof => Name::new("ShaftRoof"),
        }
    }
}

pub fn floor_num_pretty_str(floor_num: i32) -> String {
    match floor_num.cmp(&0i32) {
        Ordering::Equal => "G".to_string(),
        Ordering::Greater => format!("{}F", floor_num),
        Ordering::Less => format!("B{}", -floor_num),
    }
}

fn make_regular_row(row_size: usize, shaft_x: usize) -> Vec<FloorKind> {
    let mut row = Vec::new();
    for i in 0..row_size {
        let kind = if i == 0 {
            FloorKind::LeftWall
        } else if i == row_size - 1 {
            FloorKind::RightWall
        } else if i == shaft_x {
            FloorKind::Shaft
        } else if i == shaft_x - 1 {
            FloorKind::Vestibule
        } else {
            FloorKind::Regular
        };
        row.push(kind);
    }
    row
}
fn make_top_row(row_size: usize, shaft_x: usize) -> Vec<FloorKind> {
    let mut row = Vec::new();
    for i in 0..row_size {
        if i == 0 {
            row.push(FloorKind::TopLeftCorner);
        } else if i == row_size - 1 {
            row.push(FloorKind::TopRightCorner);
        } else if i == shaft_x {
            row.push(FloorKind::ShaftRoof);
        } else {
            row.push(FloorKind::Roof);
        }
    }
    row
}

fn make_bottom_row(row_size: usize) -> Vec<FloorKind> {
    let mut row = Vec::new();
    for i in 0..row_size {
        if i == 0 {
            row.push(FloorKind::BottomLeftCorner);
        } else if i == row_size - 1 {
            row.push(FloorKind::BottomRightCorner);
        } else {
            row.push(FloorKind::Bottom);
        }
    }
    row
}

fn position_of_tilepos(tilepos: &TilePos, tile_size: Vec2, tilemap_transform: &Transform) -> Vec2 {
    let pos = Vec2::new(tilepos.x as f32, tilepos.y as f32) * tile_size;
    pos + tilemap_transform.translation.truncate()
}

pub fn build_floor_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    array_texture_loader: Res<ArrayTextureLoader>,
) {
    println!("Building floor map");
    let texture: Handle<Image> = asset_server.load("textures/floor_tile.spritesheet.png");
    let tilemap_entity = commands.spawn_empty().id();
    let num_regular_tiles_per_row = 10;
    // Each row is, left wall, N regular tiles, vestibule, shaft, right wall
    let row_width = num_regular_tiles_per_row + 4;
    let shaft_x = num_regular_tiles_per_row + 2;
    // There is a base floor, and a roof
    let num_regular_floors = 10;
    let num_rows = num_regular_floors + 2;
    let map_size = TilemapSize {
        x: row_width as u32,
        y: num_rows as u32,
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
    let mut floor_configs = HashMap::new();
    for floor_num in 0..num_rows {
        let row = if floor_num == 0 {
            make_bottom_row(row_width)
        } else if floor_num == num_rows - 1 {
            make_top_row(row_width, shaft_x)
        } else {
            make_regular_row(row_width, shaft_x)
        };
        floor_latch_y_positions.push(floor_num as f32 * tile_size.y);
        for (x, floor_kind) in row.iter().enumerate() {
            let tile_pos = TilePos::new(x as u32, floor_num);
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
            if floor_kind == &FloorKind::Vestibule {
                let pos = tile_size_vec2 * Vec2::new(x as f32, floor_num as f32);
                vestibule_locations.push((floor_num as i32, pos));
            }
            tile_storage.set(&tile_pos, tile_entity);
            child_tiles.push(tile_entity);
        }
        // TODO: Think about how to pick the config here
        let floor_config = prefabs::generate_config_of_floor_num(floor_num as i32);
        floor_configs.insert(FloorNum(floor_num as i32), floor_config);
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
    for (floor_num, vestibule_pos) in vestibule_locations {
        let pos = vestibule_pos + tilemap_transform.translation.truncate();
        commands
            .spawn(HumanStoreBundle::new(
                HumanStore { max_humans: 3 },
                floor_num,
                pos.extend(MAP_Z + 1.0),
            ))
            .with_children(|parent| {
                parent
                    .spawn(Text2dBundle {
                        text: Text::from_section(
                            floor_num_pretty_str(floor_num),
                            TextStyle {
                                font_size: 25.0,
                                color: Color::WHITE,
                                ..Default::default()
                            },
                        )
                        .with_justify(JustifyText::Center),
                        transform: Transform::from_translation(Vec3::new(6.6, 18.1, 0.1)),
                        ..default()
                    })
                    .insert(RenderLayers::layer(crate::camera::RENDER_LAYER_MAIN));
            });
    }

    // Adjust floor latch y positions to be in world space
    let floor_latch_y_positions_raw: Vec<f32> = floor_latch_y_positions
        .iter()
        .map(|y| y + tilemap_transform.translation.y)
        .collect();
    let floor_latch_y_positions = FloorLatchYPositions(floor_latch_y_positions_raw.clone());
    println!("Inserting {:?}", floor_latch_y_positions);
    commands.insert_resource(floor_latch_y_positions);

    let floors: Vec<(i32, f32)> = floor_latch_y_positions_raw
        .into_iter()
        .enumerate()
        .map(|(i, y)| (i as i32, y))
        .collect();
    let floors = Floors {
        floor_y_positions: floors,
    };
    println!("Inserting {:?}", floors);
    commands.insert_resource(floors);

    let shaft_centre_x = (shaft_x as f32 * tile_size.x) + tilemap_transform.translation.x;
    let shaft_centre_x = ShaftCentreX(shaft_centre_x);
    println!("Inserting {:?}", shaft_centre_x);
    commands.insert_resource(shaft_centre_x);

    let lift_limits = {
        // TODO: This hardcoded "1" and "-2" are fragile and assume a 1 floor padding above and
        // below
        let lowest_floor = position_of_tilepos(
            &TilePos::new(shaft_x as u32, 1),
            tile_size_vec2,
            &tilemap_transform,
        );
        let top_floor = position_of_tilepos(
            &TilePos::new(shaft_x as u32, map_size.y - 2),
            tile_size_vec2,
            &tilemap_transform,
        );
        LiftLimits {
            min: lowest_floor.y,
            max: top_floor.y,
        }
    };
    println!("Inserting {:?}", lift_limits);
    commands.insert_resource(lift_limits);

    let floor_spawn_manager = FloorSpawnManager::new(floor_configs);
    println!("Inserting Floor Spawn Manager: {:?}", floor_spawn_manager);
    commands.insert_resource(floor_spawn_manager);
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

#[derive(Clone, Debug, Event)]
pub struct SpawnHumansEvent {
    spawn_at_floor: FloorNum,
    destination_floor: FloorNum,
}

pub fn floor_spawn_process_system(
    mut floor_spawn_manager: ResMut<FloorSpawnManager>,
    mut spawn_humans_event_writer: EventWriter<SpawnHumansEvent>,
    time: Res<Time>,
    game_central_info: Res<GameCentralInfo>,
) {
    let mut rand = thread_rng();
    // TODO: Handle rng properly
    let spawns = floor_spawn_manager.tick(&game_central_info.time, time.delta(), &mut rand);
    for (spawn_at_floor, destination_floor) in spawns {
        let event = SpawnHumansEvent {
            spawn_at_floor,
            destination_floor,
        };
        println!("Sending: {:?}", event);
        spawn_humans_event_writer.send(event);
    }
}

pub fn human_store_spawn_humans_system(
    mut spawn_humans_event_reader: EventReader<SpawnHumansEvent>,
    mut query: Query<(Entity, &mut HumanStore, &FloorNum, Option<&Children>)>,
    human_query: Query<(&PositionIndex, &Parent), With<Human>>,
    texture_assets: Res<TextureAssets>,
    mut commands: Commands,
) {
    for SpawnHumansEvent {
        spawn_at_floor,
        destination_floor,
    } in spawn_humans_event_reader.read()
    {
        for (entity, mut human_store, floor_num, children) in query.iter_mut() {
            if floor_num == spawn_at_floor {
                let has_capacity = children.map_or(0, |c| c.len()) < human_store.max_humans;
                if has_capacity {
                    human_store::add_human_to_store(
                        &human_query,
                        entity,
                        &texture_assets,
                        *destination_floor,
                        &mut commands,
                    );
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Component, Reflect, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct FloorNum(pub i32);

impl std::fmt::Display for FloorNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FloorNum({})", self.0)
    }
}
