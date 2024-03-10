use bevy::prelude::Reflect;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum InputAction {
    Up,
    Down,
    MouseMove,
    MouseLClick,
}
