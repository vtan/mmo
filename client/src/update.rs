use mmo_common::movement::Direction;
use mmo_common::player_command::{GlobalCommand, PlayerCommand, RoomCommand};
use mmo_common::player_event::{PlayerEvent, PlayerEventEnvelope};

use crate::app_event::AppEvent;
use crate::app_state::AppState;
use crate::game_state::{GameState, PartialGameState};

pub fn update(state: &mut AppState, events: Vec<AppEvent>) {
    let move_player = |state: &mut AppState, direction: Direction| {
        if let Ok(game_state) = &mut state.game_state {
            if game_state.self_movement.direction != Some(direction) {
                game_state.self_movement.direction = Some(direction);

                let command = PlayerCommand::RoomCommand {
                    room_id: game_state.room.room_id,
                    command: RoomCommand::Move {
                        position: game_state.self_movement.position,
                        direction: Some(direction),
                    },
                };
                (game_state.connection)(command);
            }
        }
    };
    let stop_moving = |state: &mut AppState, direction: Direction| {
        if let Ok(game_state) = &mut state.game_state {
            if game_state.self_movement.direction == Some(direction) {
                game_state.self_movement.direction = None;

                let command = PlayerCommand::RoomCommand {
                    room_id: game_state.room.room_id,
                    command: RoomCommand::Move {
                        position: game_state.self_movement.position,
                        direction: None,
                    },
                };
                (game_state.connection)(command);
            }
        }
    };

    for event in events {
        match event {
            AppEvent::KeyDown { code } => match code.as_str() {
                "KeyW" => move_player(state, Direction::Up),
                "KeyA" => move_player(state, Direction::Left),
                "KeyS" => move_player(state, Direction::Down),
                "KeyD" => move_player(state, Direction::Right),
                _ => (),
            },
            AppEvent::KeyUp { code } => match code.as_str() {
                "KeyW" => stop_moving(state, Direction::Up),
                "KeyA" => stop_moving(state, Direction::Left),
                "KeyS" => stop_moving(state, Direction::Down),
                "KeyD" => stop_moving(state, Direction::Right),
                _ => (),
            },
            AppEvent::WebsocketConnected { sender } => match &mut state.game_state {
                Ok(_) => unreachable!(),
                Err(partial) => {
                    partial.connection = Some(sender.into());
                    if let Some(full) = partial.to_full() {
                        state.game_state = Ok(full);
                    }
                }
            },
            AppEvent::WebsocketDisconnected => state.game_state = Err(PartialGameState::new()),
            AppEvent::WebsocketMessage { message } => match &mut state.game_state {
                Ok(game_state) => update_server_events(game_state, message),
                Err(partial) => {
                    update_partial(partial, message);
                    if let Some(full) = partial.to_full() {
                        state.game_state = Ok(full);
                    }
                }
            },
        }
    }

    if let Ok(game_state) = &mut state.game_state {
        if let Some(direction) = game_state.self_movement.direction {
            game_state.self_movement.position += 0.01 * direction.to_vector();
        }
    }
}

fn update_partial(partial: &mut PartialGameState, events: PlayerEventEnvelope<Box<PlayerEvent>>) {
    for event in events.events {
        match *event {
            PlayerEvent::Ping { sequence_number, sent_at } => {
                if let Some(ws_sender) = &partial.connection {
                    let command = PlayerCommand::GlobalCommand {
                        command: GlobalCommand::Pong { sequence_number, ping_sent_at: sent_at },
                    };
                    ws_sender(command);
                }
            }
            PlayerEvent::Initial { player_id } => {
                partial.player_id = Some(player_id);
            }
            PlayerEvent::SyncRoom { room } => {
                partial.room = Some(room);
            }
            PlayerEvent::PlayerMoved { .. } | PlayerEvent::PlayerDisappeared { .. } => {}
        }
    }
}

fn update_server_events(game_state: &mut GameState, events: PlayerEventEnvelope<Box<PlayerEvent>>) {
    for event in events.events {
        update_server_event(game_state, *event);
    }
}

fn update_server_event(game_state: &mut GameState, event: PlayerEvent) {
    match event {
        PlayerEvent::Ping { sequence_number, sent_at } => {
            let command = PlayerCommand::GlobalCommand {
                command: GlobalCommand::Pong { sequence_number, ping_sent_at: sent_at },
            };
            (game_state.connection)(command);
        }
        PlayerEvent::Initial { .. } => {}
        PlayerEvent::SyncRoom { room } => {
            game_state.room = room;
        }
        PlayerEvent::PlayerMoved { player_id, position, direction: _ } => {
            game_state.other_positions.insert(player_id, position);
        }
        PlayerEvent::PlayerDisappeared { player_id } => {
            game_state.other_positions.remove(&player_id);
        }
    }
}
