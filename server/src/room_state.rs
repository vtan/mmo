use std::{collections::HashMap, sync::Arc};

use crate::player::PlayerConnection;

use mmo_common::{
    object::ObjectId,
    player_event::PlayerEvent,
    room::{RoomId, RoomSync},
};
use nalgebra::Vector2;

#[derive(Debug, Clone)]
pub struct RoomState {
    pub room: RoomSync,
    pub portals: Vec<Portal>,
    pub players: HashMap<ObjectId, Player>,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: ObjectId,
    pub connection: PlayerConnection,
    pub position: Vector2<f32>,
}

#[derive(Debug, Clone)]
pub struct Portal {
    pub position: Vector2<u32>,
    pub target_room_id: RoomId,
    pub target_position: Vector2<f32>,
}

#[derive(Debug, Clone)]
pub struct RoomWriter {
    pub events: HashMap<ObjectId, Vec<Arc<PlayerEvent>>>,
    pub upstream_messages: Vec<UpstreamMessage>,
}

impl RoomWriter {
    pub fn new() -> Self {
        Self { events: HashMap::new(), upstream_messages: vec![] }
    }

    pub fn tell(&mut self, player_id: ObjectId, event: PlayerEvent) {
        self.tell_many(player_id, &[event]);
    }

    pub fn tell_many(&mut self, player_id: ObjectId, events: &[PlayerEvent]) {
        for event in events {
            self.events.entry(player_id).or_default().push(Arc::new(event.clone()));
        }
    }

    pub fn broadcast(&mut self, player_ids: impl Iterator<Item = ObjectId>, event: PlayerEvent) {
        let event = Arc::new(event);
        for player_id in player_ids {
            self.events.entry(player_id).or_default().push(event.clone());
        }
    }
}

#[derive(Debug, Clone)]
pub enum UpstreamMessage {
    PlayerLeftRoom {
        sender_room_id: RoomId,
        player_id: ObjectId,
        target_room_id: RoomId,
        target_position: Vector2<f32>,
    },
}
