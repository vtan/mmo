use std::ops::Deref;

use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::{client_config::ClientConfig, movement::Direction, room::RoomSync};

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
        client_config: ClientConfig,
    },
    Pong {
        sequence_number: u32,
    },
    RoomEntered {
        room: RoomSync,
    },
    PlayerMovementChanged {
        player_id: u64,
        position: Vector2<f32>,
        direction: Option<Direction>,
    },
    PlayerDisappeared {
        player_id: u64,
    },
}
