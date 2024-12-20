use mmo_common::animation::AnimationAction;
use mmo_common::object::Direction;
use mmo_common::player_command::{GlobalCommand, PlayerCommand, RoomCommand};
use mmo_common::player_event::{PlayerEvent, PlayerEventEnvelope};
use mmo_common::room::RoomSync;
use mmo_common::{rle, room};

use crate::app_event::AppEvent;
use crate::app_state::AppState;
use crate::assets;
use crate::game_state::{
    GameState, LastPing, LocalMovement, MovementAction, PartialGameState, RemoteMovement, Room,
    SelfMovement,
};

pub fn update(state: &mut AppState, events: Vec<AppEvent>) {
    for event in events {
        match event {
            AppEvent::KeyDown { code } => {
                if let Ok(game_state) = &mut state.game_state {
                    match code.as_str() {
                        "KeyW" => start_moving(game_state, Direction::Up),
                        "KeyA" => start_moving(game_state, Direction::Left),
                        "KeyS" => start_moving(game_state, Direction::Down),
                        "KeyD" => start_moving(game_state, Direction::Right),
                        "Space" => start_attack(game_state),
                        _ => (),
                    }
                }
            }
            AppEvent::KeyUp { code } => {
                if let Ok(game_state) = &mut state.game_state {
                    match code.as_str() {
                        "KeyW" => stop_moving(game_state, Direction::Up),
                        "KeyA" => stop_moving(game_state, Direction::Left),
                        "KeyS" => stop_moving(game_state, Direction::Down),
                        "KeyD" => stop_moving(game_state, Direction::Right),
                        _ => (),
                    }
                }
            }
            AppEvent::WebsocketConnected => {}
            AppEvent::WebsocketDisconnected => state.game_state = Err(PartialGameState::new()),
            AppEvent::WebsocketMessage { message, received_at } => {
                update_async(state, &message);

                match &mut state.game_state {
                    Ok(game_state) => {
                        handle_server_events(game_state, received_at, message);
                    }
                    Err(partial) => {
                        update_partial(partial, message);
                        if let Some(mut full) = partial.to_full() {
                            for remaining in &partial.remaining_events {
                                handle_server_events(&mut full, received_at, remaining.clone());
                            }
                            state.game_state = Ok(full);
                        }
                    }
                }
            }
            AppEvent::AssetsLoaded { assets } => {
                state.assets = Some(assets);
            }
        }
    }

    if let Ok(game_state) = &mut state.game_state {
        game_state.local_movements.clear();
        update_self_movement(game_state);
        update_remote_movement(game_state);
        add_ping_if_needed(game_state);

        game_state
            .local_movements
            .sort_unstable_by(|a, b| a.position.y.partial_cmp(&b.position.y).expect("NaN"));
    }
}

fn update_async(state: &mut AppState, message: &PlayerEventEnvelope<PlayerEvent>) {
    for event in message.events.iter() {
        if let PlayerEvent::Initial { client_config, .. } = event {
            let gl = state.gl.clone();
            let events = state.events.clone();
            let asset_paths = client_config.asset_paths.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let assets = assets::load(&gl, &asset_paths).await.unwrap();
                (*events).borrow_mut().push(AppEvent::AssetsLoaded { assets });
            });
        }
    }
}

fn update_partial(partial: &mut PartialGameState, events: PlayerEventEnvelope<PlayerEvent>) {
    let mut remaining = events.clone();
    remaining.events.clear();

    for event in events.events {
        match event {
            PlayerEvent::Initial { self_id, client_config } => {
                partial.self_id = Some(self_id);
                partial.client_config = Some(*client_config);
            }
            PlayerEvent::RoomEntered { room } => {
                partial.room = Some(load_room_map(*room));
            }
            PlayerEvent::Pong { .. }
            | PlayerEvent::PlayerMovementChanged { .. }
            | PlayerEvent::PlayerAnimationAction { .. }
            | PlayerEvent::PlayerDisappeared { .. } => {
                remaining.events.push(event);
            }
        }
    }
    partial.remaining_events.push(remaining);
}

fn handle_server_events(
    game_state: &mut GameState,
    received_at: f32,
    events: PlayerEventEnvelope<PlayerEvent>,
) {
    for event in events.events {
        if !matches!(event, PlayerEvent::Pong { .. }) {
            web_sys::console::info_1(&format!("{event:?}").into());
        }
        handle_server_event(game_state, received_at, event);
    }
}

fn handle_server_event(game_state: &mut GameState, received_at: f32, event: PlayerEvent) {
    match event {
        PlayerEvent::Pong { sequence_number } => {
            if let Some(last_ping) = &mut game_state.last_ping {
                if sequence_number == last_ping.sequence_number {
                    game_state.ping_rtt = received_at - last_ping.sent_at;
                } else {
                    let msg = format!("Unexpected pong sequence number, received: {sequence_number}, expected: {}", last_ping.sequence_number).into();
                    web_sys::console::warn_1(&msg);
                }
            }
        }
        PlayerEvent::Initial { .. } => {}
        PlayerEvent::RoomEntered { room } => {
            game_state.room = load_room_map(*room);
            game_state.remote_movements.clear();
            game_state.local_movements.clear();
        }
        PlayerEvent::PlayerMovementChanged {
            object_id: player_id,
            position,
            direction,
            look_direction,
        } => {
            if player_id == game_state.self_id {
                let changed_at = game_state.time.now;
                game_state.self_movement = SelfMovement {
                    position,
                    direction,
                    look_direction,
                    changed_at,
                    action: game_state.self_movement.action,
                };
            } else {
                let started_at = game_state.time.now;
                let velocity = game_state.client_config.player_velocity;
                let action = game_state.remote_movements.get(&player_id).and_then(|m| m.action);
                let remote_movement = RemoteMovement {
                    position,
                    direction,
                    look_direction,
                    started_at,
                    velocity,
                    action,
                };
                game_state.remote_movements.insert(player_id, remote_movement);
            }
        }
        PlayerEvent::PlayerAnimationAction { object_id, action } => {
            // Assuming the event is not about us
            if let Some(m) = game_state.remote_movements.get_mut(&object_id) {
                m.action = Some(MovementAction { action, started_at: game_state.time.now })
            }
        }
        PlayerEvent::PlayerDisappeared { object_id: player_id } => {
            game_state.remote_movements.remove(&player_id);
        }
    }
}

fn start_moving(game_state: &mut GameState, direction: Direction) {
    if game_state.self_movement.direction != Some(direction) {
        game_state.self_movement = SelfMovement {
            position: game_state.self_movement.position,
            direction: Some(direction),
            look_direction: direction,
            changed_at: game_state.time.now,
            action: game_state.self_movement.action,
        };

        let command = PlayerCommand::RoomCommand {
            room_id: game_state.room.room_id,
            command: RoomCommand::Move {
                position: game_state.self_movement.position,
                direction: game_state.self_movement.direction,
                look_direction: game_state.self_movement.look_direction,
            },
        };
        game_state.ws_commands.push(command);
    }
}

fn stop_moving(game_state: &mut GameState, direction: Direction) {
    if game_state.self_movement.direction == Some(direction) {
        game_state.self_movement = SelfMovement {
            position: game_state.self_movement.position,
            direction: None,
            look_direction: direction,
            changed_at: game_state.time.now,
            action: game_state.self_movement.action,
        };

        let command = PlayerCommand::RoomCommand {
            room_id: game_state.room.room_id,
            command: RoomCommand::Move {
                position: game_state.self_movement.position,
                direction: game_state.self_movement.direction,
                look_direction: game_state.self_movement.look_direction,
            },
        };
        game_state.ws_commands.push(command);
    }
}

fn start_attack(game_state: &mut GameState) {
    game_state.self_movement.action = Some(MovementAction {
        action: AnimationAction::Attack,
        started_at: game_state.time.now,
    });
    let command = PlayerCommand::RoomCommand {
        room_id: game_state.room.room_id,
        command: RoomCommand::Attack,
    };
    game_state.ws_commands.push(command);
}

fn update_self_movement(game_state: &mut GameState) {
    let room = &game_state.room;
    if let Some(direction) = game_state.self_movement.direction {
        let delta = game_state.time.frame_delta
            * game_state.client_config.player_velocity
            * direction.to_vector();
        let target = game_state.self_movement.position + delta;

        if room::collision_at(room.size, &room.collisions, target) {
            game_state.self_movement.direction = None;
            game_state.ws_commands.push(PlayerCommand::RoomCommand {
                room_id: game_state.room.room_id,
                command: RoomCommand::Move {
                    position: game_state.self_movement.position,
                    direction: None,
                    look_direction: game_state.self_movement.look_direction,
                },
            });
        } else {
            game_state.self_movement.position = target;
        }
    }

    let animation_action = if let Some(action) = game_state.self_movement.action {
        let animation = match action.action {
            AnimationAction::Attack => &game_state.client_config.player_animation.attack,
        };
        let animation = &animation.0[game_state.self_movement.look_direction as usize];
        if game_state.time.now - action.started_at < animation.total_length {
            Some(action.action)
        } else {
            None
        }
    } else {
        None
    };

    let local_movement = LocalMovement {
        object_id: game_state.self_id,
        position: game_state.self_movement.position,
        direction: game_state.self_movement.direction,
        look_direction: game_state.self_movement.look_direction,
        animation_action,
        animation_time: game_state.time.now - game_state.self_movement.changed_at,
    };
    game_state.local_movements.push(local_movement);
}

fn update_remote_movement(game_state: &mut GameState) {
    for (object_id, remote_movement) in game_state.remote_movements.iter() {
        let current_position = match remote_movement.direction {
            Some(dir) => {
                let mov_distance =
                    remote_movement.velocity * (game_state.time.now - remote_movement.started_at);
                remote_movement.position + mov_distance * dir.to_vector()
            }
            None => remote_movement.position,
        };

        // TODO: deduplicate with update_self_movement
        let animation_action = if let Some(action) = remote_movement.action {
            let animation = match action.action {
                AnimationAction::Attack => &game_state.client_config.player_animation.attack,
            };
            let animation = &animation.0[remote_movement.look_direction as usize];
            if game_state.time.now - action.started_at < animation.total_length {
                Some(action.action)
            } else {
                None
            }
        } else {
            None
        };

        let local_movement = LocalMovement {
            object_id: *object_id,
            position: current_position,
            direction: remote_movement.direction,
            look_direction: remote_movement.look_direction,
            animation_action,
            animation_time: game_state.time.now - remote_movement.started_at,
        };
        game_state.local_movements.push(local_movement);
    }
}

fn add_ping_if_needed(gs: &mut GameState) {
    let should_send = if let Some(last_ping) = &gs.last_ping {
        let elapsed = gs.time.now - last_ping.sent_at;
        if elapsed >= 1.0 || (elapsed >= 0.5 && !gs.ws_commands.is_empty()) {
            Some(last_ping.sequence_number + 1)
        } else {
            None
        }
    } else {
        Some(0)
    };
    if let Some(sequence_number) = should_send {
        gs.ws_commands.push(PlayerCommand::GlobalCommand {
            command: GlobalCommand::Ping { sequence_number },
        });
        gs.last_ping = Some(LastPing { sequence_number, sent_at: gs.time.now });
    }
}

fn load_room_map(room_sync: RoomSync) -> Room {
    let bg_dense_layers = room_sync.bg_dense_layers.iter().map(rle::decode).collect();
    let collisions = rle::decode(&room_sync.collisions);
    Room {
        room_id: room_sync.room_id,
        size: room_sync.size,
        bg_dense_layers,
        bg_sparse_layer: room_sync.bg_sparse_layer,
        fg_sparse_layer: room_sync.fg_sparse_layer,
        collisions,
    }
}
