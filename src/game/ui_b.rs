use crate::GameState;
/// Bevy UI, versus egui ui , hence the B.
use bevy::prelude::*;

pub struct UIBPlugin;

impl Plugin for UIBPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::PlayingDay), setup);
    }
}

fn setup(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|p0| {
            p0.spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(100.0),
                    right: Val::Percent(100.0),
                    width: Val::Px(200.0),
                    height: Val::Px(200.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::ALICE_BLUE),
                ..default()
            })
            .with_children(|p1| {
                p1.spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        height: Val::Px(200.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::GOLD),
                    ..default()
                })
                .with_children(|p2| {
                    p2.spawn(TextBundle {
                        text: Text::from_section("Hello World", TextStyle::default()),
                        ..default()
                    });
                });
            });
        });
}
