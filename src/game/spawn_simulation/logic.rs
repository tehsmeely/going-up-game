use crate::game::floors::FloorNum;
use crate::game::game_clock::{GameTime, TimeOfDay};
use bevy::time::Time;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::time::Duration;

/// Represents contiguous time ranges
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeRange {
    Morning,
    Midday,
    Afternoon,
}

#[derive(Clone, Debug)]
pub enum SinkOrSource {
    Sink,
    Source,
    // TODO: Re-add this wildcard later
    //Random,
}

pub struct RawFloorConfig {
    sink_or_source: Box<dyn Fn(TimeRange) -> SinkOrSource>,
    strength: Box<dyn Fn(TimeRange) -> usize>,
}

impl Debug for RawFloorConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawFloorConfig")
            .field("sink_or_source", &"<elided fn>")
            .field("strength", &"<elided fn>")
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct ResolvedFloorConfig {
    sink_or_source: SinkOrSource,
    strength: usize,
    floor_num: FloorNum,
}

impl TimeRange {
    pub fn of_time_ofday(time_ofday: &TimeOfDay) -> Self {
        match time_ofday.hour {
            0..=10 => Self::Morning,
            11..=14 => Self::Midday,
            15.. => Self::Afternoon,
        }
    }
}

impl RawFloorConfig {
    pub fn new(
        sink_or_source: Box<dyn Fn(TimeRange) -> SinkOrSource>,
        strength: Box<dyn Fn(TimeRange) -> usize>,
    ) -> Self {
        Self {
            sink_or_source,
            strength,
        }
    }
    pub fn resolve(&self, time_range: TimeRange, floor_num: FloorNum) -> ResolvedFloorConfig {
        let sink_or_source = (self.sink_or_source)(time_range);
        let strength = (self.strength)(time_range);
        ResolvedFloorConfig {
            sink_or_source,
            strength,
            floor_num,
        }
    }
}

pub struct FloorSpawnManager {
    floor_spawn_rates: FloorSpawnRates,
    raw_floors: HashMap<FloorNum, RawFloorConfig>,
}

fn resolve_all(
    floors: &HashMap<FloorNum, RawFloorConfig>,
    time_range: TimeRange,
) -> Vec<ResolvedFloorConfig> {
    floors
        .iter()
        .map(|(floor_num, raw)| raw.resolve(time_range, *floor_num))
        .collect()
}

impl FloorSpawnManager {
    pub fn new(raw_floors: HashMap<FloorNum, RawFloorConfig>) -> Self {
        let resolved = resolve_all(&raw_floors, TimeRange::Morning);
        let floor_spawn_rates = FloorSpawnRates::get_rates(resolved, TimeRange::Morning);
        Self {
            floor_spawn_rates,
            raw_floors,
        }
    }
    pub fn tick<R: Rng>(
        &mut self,
        game_time: &GameTime,
        delta: Duration,
        rng: &mut R,
    ) -> Vec<(FloorNum, FloorNum)> {
        let time_range = TimeRange::of_time_ofday(&game_time.to_game_time_of_day());
        if time_range != self.floor_spawn_rates.resolved_for_time_range {
            // Re-resolve
            println!("Re-Resolving, time range changed (to: {:?})", time_range);
            let resolved: Vec<ResolvedFloorConfig> = resolve_all(&self.raw_floors, time_range);
            self.floor_spawn_rates = FloorSpawnRates::get_rates(resolved, time_range);
        }
        self.floor_spawn_rates.tick(game_time, delta, rng)
    }
}

pub struct FloorSpawnRates {
    floors_with_rates: HashMap<FloorNum, SpawnRate>,
    sinks: Sinks,
    resolved_for_time_range: TimeRange,
}
// The rough concept is: Using configs, they generate a *spawn rate*
// That spawn rate is resolved back to the chance to spawn, for a given time span
// Then a die is rolled on that chance, with a spawn happening if so.
// The target is then resolved from sink floors
impl FloorSpawnRates {
    pub fn get_rates(floors: Vec<ResolvedFloorConfig>, time_range: TimeRange) -> Self {
        let (mut sinks, mut sources): (Vec<_>, Vec<_>) =
            floors
                .into_iter()
                .partition(|floor| match floor.sink_or_source {
                    SinkOrSource::Sink => true,
                    SinkOrSource::Source => false,
                });
        // Sort sinks so strongest is first
        sinks.sort_by_key(|floor| floor.strength);
        sinks.reverse();
        let mut floors_with_rates = HashMap::new();
        // Add as sinks with zero rates
        for sink in sinks.iter() {
            floors_with_rates.insert(sink.floor_num, SpawnRate(0.0));
        }
        for source in sources {
            let rate = source.strength as f32 / sinks.len() as f32;
            floors_with_rates.insert(source.floor_num, SpawnRate(rate));
        }
        let sinks = Sinks(sinks);

        Self {
            floors_with_rates,
            sinks,
            resolved_for_time_range: time_range,
        }
    }

    pub fn tick<R: Rng>(
        &self,
        game_time: &GameTime,
        delta: Duration,
        rng: &mut R,
    ) -> Vec<(FloorNum, FloorNum)> {
        let mut spawn_floors: Vec<FloorNum> = Vec::new();
        // chance a person spawns in this span, is humans/hour * factor_of_hours
        let delta_hrs = game_time.to_hrs_f32(&delta);
        for (floor, rate) in self.floors_with_rates.iter() {
            let chance = rate.0 * delta_hrs;
            let roll: f32 = rng.gen();
            if roll < chance {
                spawn_floors.push(*floor);
            }
        }

        // Now decide destination, for any floors that will spawn
        let mut floor_and_dest = Vec::new();
        for spawn_floor in spawn_floors.iter() {
            let sink = self
                .sinks
                .0
                .choose_weighted(rng, |floor| {
                    let dfloor = (floor.floor_num.0 - spawn_floor.0).abs();
                    if dfloor > 1 {
                        floor.strength as f64
                    } else {
                        0.5f64
                    }
                })
                .unwrap();
            floor_and_dest.push((*spawn_floor, sink.floor_num));
        }
        floor_and_dest
    }
}

// The unit is "people per hour"
pub struct SpawnRate(pub f32);

// A vec of sink floors, sorted by strength order
pub struct Sinks(pub Vec<ResolvedFloorConfig>);
