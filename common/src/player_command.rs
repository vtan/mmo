use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::{object::Direction, room::RoomId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCommandEnvelope {
    pub commands: Vec<PlayerCommand>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PlayerCommand {
    GlobalCommand { command: GlobalCommand },
    RoomCommand { room_id: RoomId, command: RoomCommand },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum GlobalCommand {
    Ping { sequence_number: u32 },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum RoomCommand {
    Move {
        position: Vector2<f32>,
        direction: Option<Direction>,
        look_direction: Direction,
    },
}
