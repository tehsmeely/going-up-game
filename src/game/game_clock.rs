use bevy::prelude::*;
use bevy::render::view::need_surface_configuration;
use bevy::time::Stopwatch;
use std::fmt::Formatter;
use std::time::Duration;

#[derive(Component, Debug, Reflect)]
pub struct GameTimeConfig {
    time_per_day: Duration,
}
impl Default for GameTimeConfig {
    fn default() -> Self {
        Self {
            time_per_day: Duration::from_secs(240),
        }
    }
}
#[derive(Component, Debug, Reflect)]
pub struct GameTime {
    time: Timer,
    config: GameTimeConfig,
}

impl GameTimeConfig {
    pub fn to_csv(&self) -> String {
        format!("{}\n", self.time_per_day.as_secs())
    }
}

impl GameTime {
    pub fn new() -> Self {
        let config = GameTimeConfig::default();
        Self {
            time: Timer::new(config.time_per_day, TimerMode::Once),
            config,
        }
    }

    /// Returns true if day is complete
    pub fn tick(&mut self, delta: Duration) -> bool {
        self.time.tick(delta);
        return self.time.elapsed() >= self.config.time_per_day;
    }

    pub fn to_string_secs(&self) -> String {
        self.time.elapsed().as_secs().to_string()
    }

    pub fn config(&self) -> &GameTimeConfig {
        &self.config
    }

    pub fn to_game_time_of_day(&self) -> TimeOfDay {
        // Based on configured time per day, convert time elapsed to a time of day
        let fraction_elapsed =
            self.time.elapsed().as_secs_f32() / self.config.time_per_day.as_secs_f32();
        let hour = (fraction_elapsed * 24.0) as u8;
        let minute = ((fraction_elapsed * 24.0) % 1.0 * 60.0) as u8;
        TimeOfDay { hour, minute }
    }

    pub fn to_hrs_f32(&self, duration: &Duration) -> f32 {
        let secs_per_hr = self.config.time_per_day.as_secs_f32() / 24.0;
        duration.as_secs_f32() / secs_per_hr
    }
}

#[derive(Clone, Copy, Debug, Default, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct TimeOfDay {
    pub hour: u8,
    pub minute: u8,
}

impl std::fmt::Display for TimeOfDay {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.minute)
    }
}
