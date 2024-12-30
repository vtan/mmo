use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::object::Direction4;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct SpriteIndex(pub u16);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnimationSet {
    pub sprite_size: Vector2<u32>,
    pub anchor: Vector2<f32>,
    pub idle: DirectionalAnimation,
    pub walk: DirectionalAnimation,
    pub custom: Vec<DirectionalAnimation>,
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
    pub fn get(&self, direction: Direction4, time: f32) -> Option<SpriteIndex> {
        let frames = match direction {
            Direction4::Right => &self.right,
            Direction4::Down => &self.down,
            Direction4::Left => &self.left,
            Direction4::Up => &self.up,
        };
        if self.total_length == 0.0 {
            frames.first().copied()
        } else {
            let rel_time = time % self.total_length;
            let i = self
                .start_times
                .iter()
                .take_while(|t| **t <= rel_time)
                .count();
            if i > 0 {
                frames.get(i - 1).copied()
            } else {
                None
            }
        }
    }
}
