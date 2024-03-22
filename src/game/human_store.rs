use crate::core::TransformTween;
use crate::game::floors::{FloorVestibule, Person, PersonSpawnTimer};
use crate::game::game::MAP_Z;
use crate::loading::TextureAssets;
use bevy::prelude::*;
use bevy::utils::tracing::dispatcher::with_default;
use bevy_ecs_tilemap::map::{TilemapGridSize, TilemapType};
use bevy_ecs_tilemap::prelude::TilePos;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use std::cmp::max;
use std::time::Duration;

#[derive(Clone, Debug, Component)]
pub struct HumanStore {
    pub spawn_timer: Timer,
    pub max_humans: usize,
    // TODO: Other config options here like: Human kind spawn chance, etc
}

#[derive(Clone, Debug, Component)]
pub struct Human;

#[derive(Clone, Debug, Component)]
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
    human_query: &Query<(&PositionIndex), (With<Human>)>,
    parent_entity: Entity,
    texture_assets: &Res<TextureAssets>,
    commands: &mut Commands,
) {
    println!("Adding human to store");
    let max_index = human_query
        .iter()
        .map(|(position_index)| position_index.0)
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

fn remove_human(
    human_query: &Query<(Entity, &PositionIndex, &Parent), (With<Human>)>,
    parent_entity: Entity,
    commands: &mut Commands,
) {
    let max_index = human_query
        .iter()
        .map(|(_, position_index, _)| position_index.0)
        .max()
        .unwrap_or(0);
    let mut entity_to_remove = None;
    for (entity, position_index, parent) in human_query.iter() {
        if position_index.0 == max_index && parent.get() == parent_entity {
            entity_to_remove = Some(entity);
        }
    }
    if let Some(entity_to_remove) = entity_to_remove {
        commands.entity(entity_to_remove).despawn_recursive();
    }
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
