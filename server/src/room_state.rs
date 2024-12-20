use std::{collections::HashMap, sync::Arc};

use crate::{player::PlayerConnection, server_context::ServerContext};

use mmo_common::{
    object::{Direction, ObjectId},
    player_event::PlayerEvent,
    room::{ForegroundTile, RoomId, RoomSync, TileIndex},
};
use nalgebra::Vector2;
use tokio::time::Instant;

#[derive(Debug, Clone)]
pub struct RoomState {
    pub server_context: Arc<ServerContext>,
    pub map: Arc<RoomMap>,
    pub room: RoomSync,
    pub players: HashMap<ObjectId, Player>,
}

#[derive(Debug, Clone)]
pub struct RoomMap {
    pub size: Vector2<u32>,
    pub bg_dense_layers: Vec<Vec<TileIndex>>,
    pub bg_sparse_layer: Vec<(Vector2<u32>, TileIndex)>,
    pub fg_sparse_layer: Vec<ForegroundTile>,
    pub collisions: Vec<bool>,
    pub portals: Vec<Portal>,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: ObjectId,
    pub connection: PlayerConnection,
    pub local_movement: LocalMovement,
    pub remote_movement: RemoteMovement,
}

#[derive(Debug, Clone, Copy)]
pub struct LocalMovement {
    pub position: Vector2<f32>,
    pub updated_at: Instant,
}

#[derive(Debug, Clone, Copy)]
pub struct RemoteMovement {
    pub position: Vector2<f32>,
    pub direction: Option<Direction>,
    pub look_direction: Direction,
    pub received_at: Instant,
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

    pub fn broadcast_many(
        &mut self,
        player_ids: impl Iterator<Item = ObjectId>,
        events: &[PlayerEvent],
    ) {
        let events = events.iter().map(|event| Arc::new(event.clone())).collect::<Vec<_>>();
        for player_id in player_ids {
            self.events.entry(player_id).or_default().extend(events.iter().cloned());
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
