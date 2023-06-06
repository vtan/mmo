use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::room::RoomSync;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerEvent {
    Ping { sequence_number: u32, sent_at: u64 },
    SyncRoom { room: RoomSync, players: Vec<(u64, Vector2<f32>)> },
    PlayerMoved { player_id: u64, position: Vector2<f32> },
    PlayerDisappeared { player_id: u64 },
}
