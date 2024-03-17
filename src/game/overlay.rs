use crate::core::{InScreenSpaceLocation, ScreenSpaceAnchor};
use crate::game::game::{Lift, LinearVelocity};
use crate::GameState;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;

pub struct OverlayPlugin;

impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), (LiftSpeedText::setup))
            .add_systems(
                Update,
                (LiftSpeedText::update_system).run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component, Debug, Reflect)]
struct LiftSpeedText;

impl LiftSpeedText {
    fn setup(mut commands: Commands) {
        commands
            .spawn(Text2dBundle {
                text: Text::from_section(
                    "",
                    TextStyle {
                        font_size: 60.0,
                        ..default()
                    },
                ),
                transform: Transform::from_translation(Vec3::new(-300.0, 200.0, 0.0)),
                ..default()
            })
            .insert(RenderLayers::layer(crate::camera::RENDER_LAYER_OVERLAY))
            .insert(InScreenSpaceLocation::new(ScreenSpaceAnchor::Bottom, 20.0))
            .insert(Self);
    }
    fn update_system(
        lift_query: Query<(&LinearVelocity), With<Lift>>,
        mut text_query: Query<(&mut Text), (With<LiftSpeedText>)>,
    ) {
        for (lift_velocity) in lift_query.iter() {
            for (mut text) in text_query.iter_mut() {
                text.sections[0].value = lift_velocity.velocity.to_string();
            }
        }
    }
}
