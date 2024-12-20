use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::{
    animation::AnimationAction,
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
        client_config: Box<ClientConfig>,
    },
    Pong {
        sequence_number: u32,
    },
    RoomEntered {
        room: Box<RoomSync>,
    },
    ObjectAppeared {
        object_id: ObjectId,
        animation_id: u32,
        velocity: f32,
    },
    ObjectDisappeared {
        object_id: ObjectId,
    },
    ObjectMovementChanged {
        object_id: ObjectId,
        position: Vector2<f32>,
        direction: Option<Direction>,
        look_direction: Direction,
    },
    ObjectAnimationAction {
        object_id: ObjectId,
        action: AnimationAction,
    },
}

impl AsRef<PlayerEvent> for PlayerEvent {
    fn as_ref(&self) -> &PlayerEvent {
        self
    }
}
