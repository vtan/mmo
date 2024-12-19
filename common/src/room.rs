use std::num::NonZeroU16;

use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::rle::Rle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct RoomId(pub u64);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RoomSync {
    pub room_id: RoomId,
    pub size: Vector2<u32>,
    pub bg_dense_layers: Vec<Rle<TileIndex>>,
    pub bg_sparse_layer: Vec<(Vector2<u32>, TileIndex)>,
    pub fg_sparse_layer: Vec<(Vector2<u32>, TileIndex)>,
    pub collisions: Rle<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct TileIndex(pub Option<NonZeroU16>);

impl TileIndex {
    pub fn empty() -> Self {
        Self(None)
    }
}

pub fn collision_at(size: Vector2<u32>, collisions: &[bool], position: Vector2<f32>) -> bool {
    let x = position.x as u32;
    let y = position.y as u32;
    if position.x < 0.0 || position.y < 0.0 || x >= size.x || y >= size.y {
        true
    } else {
        let index = y * size.x + x;
        collisions[index as usize]
    }
}
