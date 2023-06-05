use bincode::{Decode, Encode};
use nalgebra::Vector2;

#[derive(Debug, Clone, Encode, Decode)]
pub struct RoomSync {
    pub room_id: u64,
    #[bincode(with_serde)]
    pub size: Vector2<u32>,
    pub tiles: Vec<Tile>,
}

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct Tile {
    #[bincode(with_serde)]
    pub position: Vector2<u32>,
    pub tile_index: TileIndex,
}

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct TileIndex(pub u8);
