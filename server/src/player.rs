use std::sync::Arc;

use mmo_common::player_event::PlayerEvent;
use tokio::sync::mpsc;

pub type PlayerConnection = mpsc::Sender<Vec<Arc<PlayerEvent>>>;
