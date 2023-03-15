use bincode::{Decode, Encode};
use nalgebra::Vector2;

#[derive(Debug, Clone, Encode, Decode)]
pub enum PlayerEvent {
    Ping {
        sequence_number: u32,
        sent_at: u64,
    },
    SyncRoom {
        room_id: u64,
        tiles: Vec<(i32, i32)>,
    },
    PlayerMoved {
        player_id: u64,
        #[bincode(with_serde)]
        position: Vector2<f32>,
    },
    PlayerDisappeared {
        player_id: u64,
    },
}
