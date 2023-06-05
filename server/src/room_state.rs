use std::collections::HashMap;

use mmo_common::{player_event::PlayerEvent, room::RoomSync};
use nalgebra::Vector2;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct RoomState {
    pub room: RoomSync,
    pub portals: Vec<Portal>,
    pub players: HashMap<u64, Player>,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: u64,
    pub connection: mpsc::Sender<PlayerEvent>,
    pub position: Vector2<f32>,
}

#[derive(Debug, Clone)]
pub struct Portal {
    pub position: Vector2<u32>,
    pub target_room_id: u64,
}

#[derive(Debug, Clone)]
pub struct RoomWriter {
    pub events: HashMap<u64, Vec<PlayerEvent>>,
    pub upstream_messages: Vec<UpstreamMessage>,
}

impl RoomWriter {
    pub fn new() -> Self {
        Self { events: HashMap::new(), upstream_messages: vec![] }
    }

    pub fn tell(&mut self, player_id: u64, event: PlayerEvent) {
        self.events.entry(player_id).or_default().push(event);
    }

    pub fn tell_many(&mut self, player_ids: impl Iterator<Item = u64>, event: PlayerEvent) {
        for player_id in player_ids {
            self.events.entry(player_id).or_default().push(event.clone());
        }
    }
}

#[derive(Debug, Clone)]
pub enum UpstreamMessage {
    // TODO: add target position
    PlayerLeftRoom { sender_room_id: u64, player_id: u64, target_room_id: u64 },
}
