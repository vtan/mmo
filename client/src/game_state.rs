use std::collections::HashMap;

use mmo_common::player_command::PlayerCommand;
use nalgebra::Vector2;

pub struct GameState {
    pub connection: Option<Box<dyn Fn(PlayerCommand)>>,
    pub room_id: u64,
    pub tiles: Vec<(i32, i32)>,
    pub player_position: Vector2<f32>,
    pub other_positions: HashMap<u64, Vector2<f32>>,
}
