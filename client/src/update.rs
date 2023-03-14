use mmo_common::player_command::PlayerCommand;
use mmo_common::player_event::PlayerEvent;
use nalgebra::Vector2;

use crate::app_event::AppEvent;
use crate::app_state::AppState;
use crate::game_state::GameState;

pub fn update(state: &mut AppState, events: Vec<AppEvent>) {
    let move_player = |state: &mut AppState, delta: Vector2<f32>| {
        state.game_state.player_position += delta;
        if let Some(ws_sender) = &state.game_state.connection {
            ws_sender(PlayerCommand::Move { position: state.game_state.player_position });
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
            AppEvent::WebsocketConnected { sender } => state.game_state.connection = Some(sender),
            AppEvent::WebsocketDisconnected => state.game_state.connection = None,
            AppEvent::WebsocketMessage { message } => {
                update_server_event(&mut state.game_state, message)
            }
        }
    }
}

fn update_server_event(game_state: &mut GameState, event: PlayerEvent) {
    match event {
        PlayerEvent::SyncRoom { room_id, tiles } => {
            game_state.room_id = room_id;
            game_state.tiles = tiles;
        }
        PlayerEvent::PlayerMoved { player_id, position } => {
            game_state.other_positions.insert(player_id, position);
        }
        PlayerEvent::PlayerDisappeared { player_id } => {
            game_state.other_positions.remove(&player_id);
        }
    }
}
