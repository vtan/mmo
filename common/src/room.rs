use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RoomSync {
    pub room_id: u64,
    pub size: Vector2<u32>,
    pub tiles: Vec<Tile>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Tile {
    pub position: Vector2<u32>,
    pub tile_index: TileIndex,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct TileIndex(pub u8);
