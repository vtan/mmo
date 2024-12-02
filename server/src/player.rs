use std::sync::Arc;

use mmo_common::{client_config::ClientConfig, player_event::PlayerEvent};
use tokio::sync::mpsc;

pub type PlayerConnection = mpsc::Sender<Vec<Arc<PlayerEvent>>>;

pub const CLIENT_CONFIG: ClientConfig = ClientConfig { player_velocity: 3.0 };
