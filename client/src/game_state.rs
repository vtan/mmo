use std::{collections::HashMap, rc::Rc};

use mmo_common::{
    client_config::ClientConfig, movement::Direction, player_command::PlayerCommand, room::RoomSync,
};
use nalgebra::Vector2;

pub struct GameState {
    pub connection: Rc<Box<dyn Fn(PlayerCommand)>>, // TODO: delete from here
    pub ws_commands: Vec<PlayerCommand>,
    pub last_ping: Option<LastPing>,
    pub ping_rtt: f32,
    pub player_id: u64,
    pub client_config: ClientConfig,
    pub room: RoomSync,
    pub self_movement: Movement,
    pub other_positions: HashMap<u64, RemoteMovement>,
}

pub struct Movement {
    pub position: Vector2<f32>,
    pub direction: Option<Direction>,
}

pub struct RemoteMovement {
    pub position: Vector2<f32>,
    pub direction: Option<Direction>,
    pub started_at: f32,
    pub velocity: f32,
}

pub struct LastPing {
    pub sequence_number: u32,
    pub sent_at: f32,
}

pub struct PartialGameState {
    pub connection: Option<Rc<Box<dyn Fn(PlayerCommand)>>>,
    pub player_id: Option<u64>,
    pub client_config: Option<ClientConfig>,
    pub room: Option<RoomSync>,
}

impl PartialGameState {
    pub fn new() -> Self {
        Self {
            connection: None,
            player_id: None,
            client_config: None,
            room: None,
        }
    }

    pub fn to_full(&self) -> Option<GameState> {
        let connection = self.connection.clone()?;
        let player_id = self.player_id?;
        let client_config = self.client_config.clone()?;
        let room = self.room.clone()?;
        Some(GameState {
            connection,
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
