use crate::core::TransformTween;
use crate::game::floors::{FloorNum, FloorVestibule, Person, PersonSpawnTimer};
use crate::game::game::MAP_Z;
use crate::loading::TextureAssets;
use bevy::prelude::*;
use std::time::Duration;

#[derive(Clone, Debug, Component, Reflect)]
pub struct HumanStore {
    pub spawn_timer: Timer,
    pub max_humans: usize,
    // TODO: Other config options here like: Human kind spawn chance, etc
}

#[derive(Debug, Bundle)]
pub struct HumanStoreBundle {
    human_store: HumanStore,
    spatial_bundle: SpatialBundle,
    floor_num: FloorNum,
    name: Name,
}

impl HumanStoreBundle {
    pub fn new(human_store: HumanStore, floor_num: i32, translation: Vec3) -> Self {
        HumanStoreBundle {
            human_store,
            spatial_bundle: SpatialBundle::from_transform(Transform::from_translation(translation)),
            floor_num: FloorNum(floor_num),
            name: Name::new(format!("Human Store : {}", floor_num)),
        }
    }
}

#[derive(Clone, Debug, Component, Reflect)]
pub struct Human;

#[derive(Clone, Debug, Component, Reflect)]
pub struct FloorDesire {
    pub floor_num: i32,
}

#[derive(Clone, Debug, Component, Reflect)]
pub struct PositionIndex(usize);

impl PositionIndex {
    fn to_translation(&self) -> Vec3 {
        let start_x = 14.0;
        let x_offset = self.0 as f32 * 23.0;
        let z_offset = self.0 as f32 * 0.1;
        Vec3::new(start_x - x_offset, -10.0, MAP_Z + 0.1 + z_offset)
    }

    fn default_translation() -> Vec3 {
        Vec3::new(-100.0, -10.0, MAP_Z + 0.1)
    }
}

pub fn add_human_to_store(
    human_query: &Query<(&PositionIndex, &Parent), (With<Human>)>,
    parent_entity: Entity,
    texture_assets: &Res<TextureAssets>,
    desired_floor: i32,
    commands: &mut Commands,
) {
    println!("Adding human to store");
    let max_index = human_query
        .iter()
        .filter_map(|(position_index, parent)| {
            if parent.get() == parent_entity {
                Some(position_index.0)
            } else {
                None
            }
        })
        .max();
    let position_index = match max_index {
        Some(i) => PositionIndex(i + 1),
        None => PositionIndex(0),
    };
    let initial_transform = Transform::from_translation(PositionIndex::default_translation());
    let final_transform = Transform::from_translation(position_index.to_translation());
    commands
        .spawn(SpriteBundle {
            texture: texture_assets.human.clone(),
            transform: initial_transform.clone(),
            ..Default::default()
        })
        .insert(Human)
        .insert(position_index)
        .insert(TransformTween::new(
            initial_transform,
            final_transform,
            Duration::from_secs(1),
        ))
        .insert(FloorDesire {
            floor_num: desired_floor,
        })
        .insert(Name::new("Human"))
        .set_parent(parent_entity);
}

pub enum HowMany {
    All,
    N(usize),
}
pub fn remove_humans(
    human_query: &Query<(Entity, &FloorDesire, &PositionIndex, &Parent), (With<Human>)>,
    parent_entity: Entity,
    commands: &mut Commands,
    num_humans: HowMany,
) -> Vec<i32> {
    let mut indices: Vec<usize> = human_query
        .iter()
        .map(|(_, _, position_index, _)| position_index.0)
        .collect();
    indices.sort();
    indices.reverse();
    let slice_len = match num_humans {
        HowMany::All => indices.len(),
        HowMany::N(n) => n.min(indices.len()),
    };
    let indices_to_remove = &indices[..slice_len];

    let mut entities_to_remove = vec![];
    let mut removed_desired_floors = vec![];
    for (entity, floor_desire, position_index, parent) in human_query.iter() {
        if indices_to_remove.contains(&position_index.0) && parent.get() == parent_entity {
            entities_to_remove.push(entity);
            removed_desired_floors.push(floor_desire.floor_num);
        }
    }
    for entity_to_remove in entities_to_remove.iter() {
        commands.entity(*entity_to_remove).despawn_recursive();
    }
    removed_desired_floors
}

//
fn human_positioning_system(
    mut human_query: Query<
        (&PositionIndex, &mut Transform),
        (Changed<PositionIndex>, Without<TransformTween>),
    >,
) {
    for (position_index, mut transform) in human_query.iter_mut() {
        transform.translation = position_index.to_translation();
    }
}

pub fn floor_desire_system(
    desire_query: Query<(Entity, &FloorDesire, Option<&Children>), Without<Text>>,
    mut text_query: Query<(&mut Text), With<Text>>,
    mut commands: Commands,
) {
    for (entity, floor_desire, maybe_children) in desire_query.iter() {
        let mut text_found = false;
        if let Some(children) = maybe_children {
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(*child) {
                    text.sections[0].value = format!("{}", floor_desire.floor_num);
                    text_found = true;
                }
            }
        }
        if !text_found {
            let text = Text::from_section(
                format!("{}", floor_desire.floor_num),
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..Default::default()
                },
            );
            commands
                .spawn(Text2dBundle {
                    text,
                    transform: Transform::from_translation(Vec3::new(10.0, 10.0, 0.0)),
                    ..Default::default()
                })
                .set_parent(entity);
        }
    }
}
