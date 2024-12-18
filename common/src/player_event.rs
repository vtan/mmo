use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::{
    client_config::ClientConfig,
    object::{Direction, ObjectId},
    room::RoomSync,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerEventEnvelope<T>
where
    T: AsRef<PlayerEvent>,
{
    pub events: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerEvent {
    Initial {
        self_id: ObjectId,
        client_config: ClientConfig,
    },
    Pong {
        sequence_number: u32,
    },
    RoomEntered {
        room: RoomSync,
    },
    PlayerMovementChanged {
        object_id: ObjectId,
        position: Vector2<f32>,
        direction: Option<Direction>,
    },
    PlayerDisappeared {
        object_id: ObjectId,
    },
}

impl AsRef<PlayerEvent> for PlayerEvent {
    fn as_ref(&self) -> &PlayerEvent {
        self
    }
}
