use std::sync::Arc;

use mmo_common::{object::ObjectId, player_event::PlayerEvent};

use crate::room_state::UpstreamMessage;

#[derive(Debug, Clone)]
pub struct RoomWriter {
    pub events: Vec<RoomWriterEvent>,
    pub upstream_messages: Vec<UpstreamMessage>,
}

#[derive(Debug, Clone)]
pub struct RoomWriterEvent {
    pub target: RoomWriterTarget,
    pub event: Arc<PlayerEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomWriterTarget {
    Player(ObjectId),
    All,
    AllExcept(ObjectId),
}

impl RoomWriter {
    pub fn new() -> Self {
        Self {
            events: vec![],
            upstream_messages: vec![],
        }
    }

    pub fn tell(&mut self, target: RoomWriterTarget, event: PlayerEvent) {
        self.tell_many(target, &[event]);
    }

    pub fn tell_many(&mut self, target: RoomWriterTarget, events: &[PlayerEvent]) {
        for event in events {
            self.events.push(RoomWriterEvent {
                target,
                event: Arc::new(event.clone()),
            });
        }
    }
}
