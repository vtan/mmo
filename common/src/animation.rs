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
    pub total_length: f32,
    pub start_times: Vec<f32>,
    pub right: Vec<SpriteIndex>,
    pub down: Vec<SpriteIndex>,
    pub left: Vec<SpriteIndex>,
    pub up: Vec<SpriteIndex>,
}

impl DirectionalAnimation {
    pub fn get(&self, direction: Direction, time: f32) -> Option<SpriteIndex> {
        let frames = match direction {
            Direction::Right => &self.right,
            Direction::Down => &self.down,
            Direction::Left => &self.left,
            Direction::Up => &self.up,
        };
        if self.total_length == 0.0 {
            frames.first().copied()
        } else {
            let rel_time = time % self.total_length;
            let i = self.start_times.iter().take_while(|t| **t <= rel_time).count();
            if i > 0 {
                frames.get(i - 1).copied()
            } else {
                None
            }
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
