use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::{
    object::{Direction4, Direction8},
    room::RoomId,
};

const HANDSHAKE_MAGIC: [u8; 8] = [111, 197, 49, 147, 243, 227, 34, 189];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerHandshake {
    pub magic: [u8; 8],
}

impl PlayerHandshake {
    pub fn new() -> Self {
        Self {
            magic: HANDSHAKE_MAGIC,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == HANDSHAKE_MAGIC
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCommandEnvelope {
    pub room_id: RoomId,
    pub commands: Vec<PlayerCommand>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PlayerCommand {
    GlobalCommand(GlobalCommand),
    RoomCommand(RoomCommand),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum GlobalCommand {
    Ping { sequence_number: u32 },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum RoomCommand {
    Move {
        position: Vector2<f32>,
        direction: Option<Direction8>,
        look_direction: Direction4,
    },
    Attack,
}

impl From<GlobalCommand> for PlayerCommand {
    fn from(command: GlobalCommand) -> Self {
        Self::GlobalCommand(command)
    }
}

impl From<RoomCommand> for PlayerCommand {
    fn from(command: RoomCommand) -> Self {
        Self::RoomCommand(command)
    }
}
