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
            name: Name::new("Human Store"),
        }
    }
}

#[derive(Clone, Debug, Component, Reflect)]
pub struct Human;

#[derive(Clone, Debug, Component, Reflect)]
pub struct PositionIndex(usize);

impl PositionIndex {
    fn to_translation(&self) -> Vec3 {
        let start_x = 15.0;
        let x_offset = self.0 as f32 * 9.0;
        let z_offset = self.0 as f32 * 0.1;
        Vec3::new(start_x - x_offset, 0.0, MAP_Z + 0.1 + z_offset)
    }

    fn default_translation() -> Vec3 {
        Vec3::new(-100.0, 0.0, MAP_Z + 0.1)
    }
}

pub fn add_human_to_store(
    human_query: &Query<(&PositionIndex, &Parent), (With<Human>)>,
    parent_entity: Entity,
    texture_assets: &Res<TextureAssets>,
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
        .set_parent(parent_entity);
}

pub enum HowMany {
    All,
    N(usize),
}
pub fn remove_humans(
    human_query: &Query<(Entity, &PositionIndex, &Parent), (With<Human>)>,
    parent_entity: Entity,
    commands: &mut Commands,
    num_humans: HowMany,
) -> usize {
    let mut indices: Vec<usize> = human_query
        .iter()
        .map(|(_, position_index, _)| position_index.0)
        .collect();
    indices.sort();
    indices.reverse();
    let slice_len = match num_humans {
        HowMany::All => indices.len(),
        HowMany::N(n) => n.min(indices.len()),
    };
    let indices_to_remove = &indices[..slice_len];

    let mut entities_to_remove = vec![];
    for (entity, position_index, parent) in human_query.iter() {
        if indices_to_remove.contains(&position_index.0) && parent.get() == parent_entity {
            entities_to_remove.push(entity);
        }
    }
    for entity_to_remove in entities_to_remove.iter() {
        commands.entity(*entity_to_remove).despawn_recursive();
    }
    slice_len
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

pub fn human_store_gizmo_system(
    mut human_store_query: Query<(&Transform), (With<HumanStore>)>,
    mut gizmos: Gizmos,
) {
    for human_store_transform in human_store_query.iter_mut() {
        for radius in 1..3 {
            gizmos.circle_2d(
                human_store_transform.translation.truncate(),
                3.0 * (radius as f32),
                Color::WHITE,
            );
        }
    }
}
