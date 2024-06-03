use crate::game::spawn_simulation::{RawFloorConfig, SinkOrSource};
use std::ops::Range;

type Sr = Vec<(Range<u8>, SinkOrSource)>;

pub fn ground_floor_source() -> Sr {
    vec![
        (0..10, SinkOrSource::Source),
        (10..16, SinkOrSource::Sink),
        (16..17, SinkOrSource::Source),
        (17..24, SinkOrSource::Sink),
    ]
}

pub fn normal_hours(out_of_hours: usize, in_hours: usize) -> Vec<(Range<u8>, usize)> {
    vec![
        (0..9, out_of_hours),
        (9..18, in_hours),
        (18..24, out_of_hours),
    ]
}

pub fn always<T: Copy>(val: T) -> Vec<(Range<u8>, T)> {
    vec![(0..24, val)]
}

pub fn morning_sink() -> Sr {
    vec![(0..12, SinkOrSource::Sink), (12..24, SinkOrSource::Source)]
}

pub fn afternoon_sink() -> Sr {
    vec![(0..12, SinkOrSource::Source), (12..24, SinkOrSource::Sink)]
}

pub fn alternating_sink(mode: bool) -> Sr {
    let (a, b) = match mode {
        true => (SinkOrSource::Sink, SinkOrSource::Source),
        false => (SinkOrSource::Source, SinkOrSource::Sink),
    };
    vec![
        (0..2, a),
        (2..4, b),
        (4..6, a),
        (6..8, b),
        (8..10, a),
        (10..12, b),
        (12..14, a),
        (14..16, b),
        (16..18, a),
        (18..20, b),
        (20..22, a),
        (22..24, b),
    ]
}

pub fn generate_config_of_floor_num(floor_num: i32) -> RawFloorConfig {
    let (sink_or_source, strength) = match floor_num {
        ..=-1 => (afternoon_sink(), normal_hours(1, 2)),
        0 => (ground_floor_source(), normal_hours(2, 15)),
        1..=12 => (alternating_sink(floor_num % 2 == 0), normal_hours(2, 3)),
        13.. => (afternoon_sink(), normal_hours(1, 2)),
    };
    RawFloorConfig::new(sink_or_source, strength).unwrap()
}
