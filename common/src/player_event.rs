use std::ops::Deref;

use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::{movement::Direction, room::RoomSync};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerEventEnvelope<T>
where
    T: Deref<Target = PlayerEvent>,
{
    pub events: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerEvent {
    Initial {
        player_id: u64,
    },
    Ping {
        sequence_number: u32,
        sent_at: u64,
    },
    SyncRoom {
        room: RoomSync,
    },
    PlayerMoved {
        player_id: u64,
        position: Vector2<f32>,
        direction: Option<Direction>,
    },
    PlayerDisappeared {
        player_id: u64,
    },
}
