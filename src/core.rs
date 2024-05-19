use bevy::math::Vec3;
use bevy::prelude::*;
use derive_new::new;
use std::time::Duration;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                With2DScale::apply_system,
                InScreenSpaceLocation::apply_position_system,
                TransformTween::update_system,
            ),
        )
        .add_event::<TweenCompleteEvent>()
        .register_type::<InScreenSpaceLocation>()
        .register_type::<TransformTween>()
        .register_type::<With2DScale>();
    }
}
#[derive(Clone, Debug, Reflect, Component)]
pub struct With2DScale {
    pub scale: f32,
}

impl With2DScale {
    pub fn new(scale: f32) -> Self {
        Self { scale }
    }

    fn apply_system(mut query: Query<(&mut Transform, &Self), Changed<Self>>) {
        for (mut transform, with_scale) in query.iter_mut() {
            transform.scale = Vec3::new(with_scale.scale, with_scale.scale, transform.scale.z);
        }
    }
}

#[derive(Clone, Debug, Reflect)]
pub enum ScreenSpaceAnchor {
    Top,
    Left,
    Right,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl ScreenSpaceAnchor {
    fn to_signum(&self) -> Vec2 {
        match self {
            Self::Top => Vec2::new(0.0, 1.0),
            Self::Left => Vec2::new(-1.0, 0.0),
            Self::Right => Vec2::new(1.0, 0.0),
            Self::Bottom => Vec2::new(0.0, -1.0),
            Self::TopLeft => Vec2::new(-1.0, 1.0),
            Self::TopRight => Vec2::new(1.0, 1.0),
            Self::BottomLeft => Vec2::new(-1.0, -1.0),
            Self::BottomRight => Vec2::new(1.0, -1.0),
        }
    }
    fn to_inverted_signum(&self) -> Vec2 {
        self.to_signum() * -1.0
    }
}
#[derive(Clone, Debug, Reflect, Component)]
pub struct InScreenSpaceLocation {
    anchor: ScreenSpaceAnchor,
    /// The impact of [offset] depends on where the anchor is. i.e. if `Left` then position will be
    /// ( -screen-width/2 + offset, 0)
    offset: f32,
}

impl InScreenSpaceLocation {
    pub fn new(anchor: ScreenSpaceAnchor, offset: f32) -> Self {
        Self { anchor, offset }
    }
    fn apply_position_system(
        mut query: Query<(&mut Transform, &Self)>,
        windows: Query<&Window, Or<(Changed<Window>, Changed<Self>)>>,
    ) {
        // TODO: Verify the Changed<Window> actually works
        if let Ok(window) = windows.get_single() {
            for (mut transform, screen_space_location) in query.iter_mut() {
                let raw_position = screen_space_location.anchor.to_signum()
                    * Vec2::new(window.width() / 2.0, window.height() / 2.0);
                let offset = screen_space_location.anchor.to_inverted_signum()
                    * Vec2::new(screen_space_location.offset, screen_space_location.offset);
                transform.translation = (raw_position + offset).extend(transform.translation.z);
            }
        }
    }
}

#[derive(Clone, Debug, Reflect, Event)]
pub enum TweenCompleteEvent {
    Finished(Entity),
}

#[derive(Clone, Debug, Reflect, Component)]
pub struct TransformTween {
    pub start: Transform,
    pub end: Transform,
    pub duration: f32,
    pub elapsed: f32,
}

impl TransformTween {
    pub fn new(start: Transform, end: Transform, duration: Duration) -> Self {
        Self {
            start,
            end,
            duration: duration.as_secs_f32(),
            elapsed: 0.0,
        }
    }
    fn update_system(
        time: Res<Time>,
        mut query: Query<(Entity, &mut Transform, &mut Self)>,
        mut commands: Commands,
        mut complete_event_writer: EventWriter<TweenCompleteEvent>,
    ) {
        for (entity, mut transform, mut tween) in query.iter_mut() {
            tween.elapsed += time.delta_seconds();
            let t = tween.elapsed / tween.duration;
            transform.translation = tween.start.translation.lerp(tween.end.translation, t);
            transform.rotation = tween.start.rotation.lerp(tween.end.rotation, t);
            transform.scale = tween.start.scale.lerp(tween.end.scale, t);
            if tween.elapsed >= tween.duration {
                commands.entity(entity).remove::<Self>();
                complete_event_writer.send(TweenCompleteEvent::Finished(entity));
            }
        }
    }
}

pub trait Lerpable {
    fn lerp(&self, other: Self, t: f32) -> Self;
}
#[derive(Clone, Debug, Reflect, Component)]
struct LerpTween<T> {
    from: T,
    to: T,
    elapsed: f32,
    duration: f32,
}

impl<T: Lerpable + Component + Copy> LerpTween<T> {
    fn new(from: T, to: T, duration: Duration) -> Self {
        Self {
            from,
            to,
            elapsed: 0.0,
            duration: duration.as_secs_f32(),
        }
    }
    fn update_system(
        time: Res<Time>,
        mut query: Query<(Entity, &mut T, &mut Self)>,
        mut commands: Commands,
    ) {
        for (entity, mut t, mut tween) in query.iter_mut() {
            tween.elapsed += time.delta_seconds();
            let s = tween.elapsed / tween.duration;
            let new_t = tween.from.lerp(tween.to, s);
            *t = new_t;
            if tween.elapsed >= tween.duration {
                commands.entity(entity).remove::<Self>();
            }
        }
    }
}
