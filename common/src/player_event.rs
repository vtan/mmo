use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::{
    client_config::ClientConfig,
    object::{Direction4, Direction8, ObjectId, ObjectType},
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
        object_type: ObjectType,
        animation_id: u32,
        velocity: f32,
        health: i32,
        max_health: i32,
    },
    ObjectDisappeared {
        object_id: ObjectId,
    },
    ObjectMovementChanged {
        object_id: ObjectId,
        position: Vector2<f32>,
        direction: Option<Direction8>,
        look_direction: Direction4,
    },
    ObjectAnimationAction {
        object_id: ObjectId,
        animation_index: u8,
    },
    ObjectHealthChanged {
        object_id: ObjectId,
        change: i32,
        health: i32,
    },
    AttackTargeted {
        position: Vector2<f32>,
        radius: f32,
        length: f32,
    },
}

impl AsRef<PlayerEvent> for PlayerEvent {
    fn as_ref(&self) -> &PlayerEvent {
        self
    }
}
