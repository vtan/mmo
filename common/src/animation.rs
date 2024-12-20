use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::object::Direction;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct SpriteIndex(pub u16);

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum AnimationAction {
    Attack,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnimationSet {
    pub sprite_size: Vector2<u32>,
    pub anchor: Vector2<f32>,
    pub idle: DirectionalAnimation,
    pub walk: DirectionalAnimation,
    pub attack: DirectionalAnimation,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DirectionalAnimation {
    pub right: Animation,
    pub down: Animation,
    pub left: Animation,
    pub up: Animation,
}

impl DirectionalAnimation {
    pub fn get(&self, direction: Direction) -> &Animation {
        match direction {
            Direction::Right => &self.right,
            Direction::Down => &self.down,
            Direction::Left => &self.left,
            Direction::Up => &self.up,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Animation {
    pub total_length: f32,
    pub frames: Vec<AnimationFrame>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct AnimationFrame {
    pub start: f32,
    pub sprite_index: SpriteIndex,
}
