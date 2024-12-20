use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

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
pub struct DirectionalAnimation(pub [Animation; 4]);

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
