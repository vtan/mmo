use bincode::{Decode, Encode};
use nalgebra::Vector2;

use crate::room::RoomSync;

#[derive(Debug, Clone, Encode, Decode)]
pub enum PlayerEvent {
    Ping {
        sequence_number: u32,
        sent_at: u64,
    },
    SyncRoom {
        room: RoomSync,
        #[bincode(with_serde)]
        players: Vec<(u64, Vector2<f32>)>,
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
