use rand::Rng;
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Class {
    One,
    Two,
    Three,
    Four,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LevelKind {
    Hotel,
    //Office,
    //TODO: Other?
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FloorKind {
    Regular,
    Lobby,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Floor {
    kind: FloorKind,
    number: usize,
}

// TODO: Consider generic over floor where floor corresponds to kind
#[derive(Debug, PartialEq)]
pub struct Level {
    kind: LevelKind,
    floors: Vec<Floor>,
    class: Class,
}

impl LevelKind {
    fn to_floor_count_of_class(&self, class: Class) -> Range<usize> {
        match self {
            Self::Hotel => match class {
                Class::One => 5..10,
                Class::Two => 7..20,
                Class::Three => 17..50,
                Class::Four => 40..100,
            },
        }
    }
}

impl Floor {
    fn new(number: usize, kind: FloorKind) -> Self {
        Self { kind, number }
    }
}

impl Level {
    pub fn create(kind: LevelKind, class: Class, rng: &mut impl Rng) -> Self {
        let possible_floor_range = kind.to_floor_count_of_class(class);
        let floor_count = rng.gen_range(possible_floor_range);
        let mut floors = Vec::new();
        for i in 0..floor_count {
            floors.push(
                // Calculate floor kind here. Always running sequentially from the bottom might not
                // be well balanced though level-to-level so maybe this needs to be more complicated
                Floor::new(i, FloorKind::Regular),
            );
        }

        Self {
            kind,
            floors,
            class,
        }
    }
}

mod tests {
    use crate::game::world_gen::{Class, Floor, FloorKind, Level, LevelKind};
    use rand::SeedableRng;

    #[test]
    fn basic_test() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(1337);
        let level = Level::create(LevelKind::Hotel, Class::Two, &mut rng);

        let expected = {
            let num_floors = 15;
            let mut floors = Vec::new();
            for i in 0..num_floors {
                floors.push(Floor::new(i, FloorKind::Regular));
            }
            Level {
                kind: LevelKind::Hotel,
                floors,
                class: Class::Two,
            }
        };
        assert_eq!(level, expected);
    }
}
