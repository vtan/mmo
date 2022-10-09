use mmo_common::MoveCommand;

use crate::app_event::AppEvent;
use crate::app_state::AppState;

pub fn update(state: &mut AppState, events: Vec<AppEvent>) {
    state.ticks += 1;

    let move_player = |state: &mut AppState, dx, dy| {
        state.player_position.x += dx;
        state.player_position.y += dy;
        if let Some(ws_sender) = &state.connection {
            ws_sender(MoveCommand { x: state.player_position.x, y: state.player_position.y });
        }
    };
    for event in events {
        match event {
            AppEvent::KeyDown { code } => match code.as_str() {
                "ArrowLeft" => move_player(state, -1.0, 0.0),
                "ArrowRight" => move_player(state, 1.0, 0.0),
                "ArrowUp" => move_player(state, 0.0, -1.0),
                "ArrowDown" => move_player(state, 0.0, 1.0),
                _ => (),
            },
            AppEvent::WebsocketConnected { sender } => state.connection = Some(sender),
            AppEvent::WebsocketDisconnected => state.connection = None,
            AppEvent::WebsocketMessage { message } => todo!(),
        }
    }
}
