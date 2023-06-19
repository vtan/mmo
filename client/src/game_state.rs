use std::{collections::HashMap, rc::Rc};

use mmo_common::{movement::Direction, player_command::PlayerCommand, room::RoomSync};
use nalgebra::Vector2;

pub struct GameState {
    pub connection: Rc<Box<dyn Fn(PlayerCommand)>>,
    pub player_id: u64,
    pub room: RoomSync,
    pub self_movement: Movement,
    pub other_positions: HashMap<u64, Vector2<f32>>,
}

pub struct Movement {
    pub position: Vector2<f32>,
    pub direction: Option<Direction>,
}

pub struct PartialGameState {
    pub connection: Option<Rc<Box<dyn Fn(PlayerCommand)>>>,
    pub player_id: Option<u64>,
    pub room: Option<RoomSync>,
}

impl PartialGameState {
    pub fn new() -> Self {
        Self { connection: None, player_id: None, room: None }
    }

    pub fn to_full(&self) -> Option<GameState> {
        let connection = self.connection.clone()?;
        let player_id = self.player_id?;
        let room = self.room.clone()?;
        Some(GameState {
            connection,
            player_id,
            room,
            self_movement: Movement { position: Vector2::new(0.0, 0.0), direction: None },
            other_positions: HashMap::new(),
        })
    }
}
