use std::collections::HashMap;

use mmo_common::{
    client_config::ClientConfig, movement::Direction, player_command::PlayerCommand, room::RoomSync,
};
use nalgebra::Vector2;

pub struct GameState {
    pub time: Timestamps,
    pub ws_commands: Vec<PlayerCommand>,
    pub last_ping: Option<LastPing>,
    pub ping_rtt: f32,
    pub player_id: u64,
    pub client_config: ClientConfig,
    pub room: RoomSync,
    pub self_movement: Movement,
    pub other_positions: HashMap<u64, RemoteMovement>,
}

#[derive(Debug, Clone, Copy)]
pub struct Timestamps {
    pub now: f32,
    pub frame_delta: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Movement {
    pub position: Vector2<f32>,
    pub direction: Option<Direction>,
}

#[derive(Debug, Clone, Copy)]
pub struct RemoteMovement {
    pub position: Vector2<f32>,
    pub direction: Option<Direction>,
    pub started_at: f32,
    pub velocity: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct LastPing {
    pub sequence_number: u32,
    pub sent_at: f32,
}

pub struct PartialGameState {
    pub time: Timestamps,
    pub player_id: Option<u64>,
    pub client_config: Option<ClientConfig>,
    pub room: Option<RoomSync>,
}

impl PartialGameState {
    pub fn new() -> Self {
        Self {
            time: Timestamps { now: 0.0, frame_delta: 0.0 },
            player_id: None,
            client_config: None,
            room: None,
        }
    }

    pub fn to_full(&self) -> Option<GameState> {
        let player_id = self.player_id?;
        let client_config = self.client_config.clone()?;
        let room = self.room.clone()?;
        Some(GameState {
            time: self.time,
            ws_commands: Vec::new(),
            last_ping: None,
            ping_rtt: 0.0,
            player_id,
            client_config,
            room,
            self_movement: Movement { position: Vector2::new(0.0, 0.0), direction: None },
            other_positions: HashMap::new(),
        })
    }
}
