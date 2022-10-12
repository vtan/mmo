use mmo_common::{PlayerCommand, PlayerEvent};
use nalgebra::Vector2;

use crate::app_event::AppEvent;
use crate::app_state::AppState;

pub fn update(state: &mut AppState, events: Vec<AppEvent>) {
    state.ticks += 1;

    let move_player = |state: &mut AppState, delta: Vector2<f32>| {
        state.player_position += delta;
        if let Some(ws_sender) = &state.connection {
            ws_sender(PlayerCommand::Move { position: state.player_position });
        }
    };
    for event in events {
        match event {
            AppEvent::KeyDown { code } => match code.as_str() {
                "ArrowLeft" => move_player(state, Vector2::new(-1.0, 0.0)),
                "ArrowRight" => move_player(state, Vector2::new(1.0, 0.0)),
                "ArrowUp" => move_player(state, Vector2::new(0.0, -1.0)),
                "ArrowDown" => move_player(state, Vector2::new(0.0, 1.0)),
                _ => (),
            },
            AppEvent::WebsocketConnected { sender } => state.connection = Some(sender),
            AppEvent::WebsocketDisconnected => state.connection = None,
            AppEvent::WebsocketMessage {
                message: PlayerEvent::PlayerMoved { player_id, position },
            } => {
                state.other_positions.insert(player_id, position);
            }
            AppEvent::WebsocketMessage {
                message: PlayerEvent::PlayerDisappeared { player_id },
            } => {
                state.other_positions.remove(&player_id);
            }
        }
    }
}
