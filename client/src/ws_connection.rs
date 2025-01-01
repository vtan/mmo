use std::cell::RefCell;
use std::rc::Rc;

use js_sys::{ArrayBuffer, Uint8Array};
use mmo_common::player_command::PlayerCommand;
use mmo_common::player_command::PlayerCommandEnvelope;
use mmo_common::player_command::PlayerHandshake;
use mmo_common::player_event::PlayerEvent;
use mmo_common::player_event::PlayerEventEnvelope;
use mmo_common::room::RoomId;
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket};

use crate::app_event::AppEvent;
use crate::console_error;
use crate::console_warn;
use crate::metrics::Metrics;

pub fn connect(
    events: Rc<RefCell<Vec<AppEvent>>>,
    metrics: Rc<RefCell<Metrics>>,
) -> Result<WebSocket, JsValue> {
    let window = web_sys::window().expect("No window");
    let performance = window.performance().expect("No performance");

    let location_origin = window.location().origin()?;
    let url = format!("{location_origin}/api/ws");
    let ws = WebSocket::new(&url)?;
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let ws_onopen = {
        let events = events.clone();
        let ws = ws.clone();
        Closure::once_into_js(move || {
            send_serde(&ws, PlayerHandshake::new()).unwrap();
            (*events).borrow_mut().push(AppEvent::WebsocketConnected);
        })
    };
    ws.set_onopen(Some(ws_onopen.unchecked_ref()));

    let ws_onclose = {
        let events = events.clone();
        Closure::<dyn FnMut()>::new(move || {
            console_error!("Websocket disconnected");
            (*events).borrow_mut().push(AppEvent::WebsocketDisconnected);
        })
        .into_js_value()
    };
    ws.set_onclose(Some(ws_onclose.unchecked_ref()));

    let ws_onerror = {
        let events = events.clone();
        Closure::<dyn FnMut()>::new(move || {
            console_error!("Websocket error");
            (*events).borrow_mut().push(AppEvent::WebsocketDisconnected);
        })
        .into_js_value()
    };
    ws.set_onerror(Some(ws_onerror.unchecked_ref()));

    let ws_onmessage = {
        let events = events.clone();
        Closure::<dyn FnMut(_)>::new(move |ws_event: MessageEvent| {
            let received_at = (performance.now() * 1e-3) as f32;
            if let Ok(buf) = ws_event.data().dyn_into::<ArrayBuffer>() {
                let bytes = Uint8Array::new(&buf).to_vec();
                let len = bytes.len();
                let message: PlayerEventEnvelope<PlayerEvent> =
                    postcard::from_bytes(&bytes).expect("Failed to deserialize message");
                let event_count = message.events.len();
                let app_event = AppEvent::WebsocketMessage {
                    message,
                    received_at,
                };
                (*events).borrow_mut().push(app_event);
                metrics
                    .borrow_mut()
                    .record_net_event(len as u32, event_count as u32);
            } else {
                console_warn!("Unexpected websocket message type");
            }
        })
        .into_js_value()
    };
    ws.set_onmessage(Some(ws_onmessage.unchecked_ref()));

    Ok(ws)
}

pub fn send(
    ws: &WebSocket,
    room_id: RoomId,
    commands: Vec<PlayerCommand>,
    metrics: &mut Metrics,
) -> Result<(), JsValue> {
    let command_count = commands.len();
    let envelope = PlayerCommandEnvelope { room_id, commands };
    let len = send_serde(ws, envelope)?;
    metrics.record_net_command(len as u32, command_count as u32);
    Ok(())
}

fn send_serde<T: Serialize>(ws: &WebSocket, message: T) -> Result<usize, JsValue> {
    let bytes = postcard::to_stdvec(&message).map_err(|e| e.to_string())?;
    let len = bytes.len();
    ws.send_with_u8_array(&bytes)?;
    Ok(len)
}
