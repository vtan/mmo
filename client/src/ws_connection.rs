use std::cell::RefCell;
use std::rc::Rc;

use js_sys::{ArrayBuffer, Uint8Array};
use mmo_common::player_command::PlayerCommand;
use mmo_common::player_command::PlayerCommandEnvelope;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket};

use crate::app_event::AppEvent;

pub fn connect(events: Rc<RefCell<Vec<AppEvent>>>) -> Result<WebSocket, JsValue> {
    let window = web_sys::window().expect("No window");
    let performance = window.performance().expect("No performance");

    let location_origin = window.location().origin()?;
    let url = format!("{location_origin}/api/ws");
    let ws = WebSocket::new(&url)?;
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let ws_onopen = {
        let events = events.clone();
        Closure::once_into_js(move || {
            (*events).borrow_mut().push(AppEvent::WebsocketConnected);
        })
    };
    ws.set_onopen(Some(ws_onopen.unchecked_ref()));

    let ws_onclose = {
        let events = events.clone();
        Closure::<dyn FnMut()>::new(move || {
            web_sys::console::error_1(&"Websocket disconnected".into());
            (*events).borrow_mut().push(AppEvent::WebsocketDisconnected);
        })
        .into_js_value()
    };
    ws.set_onclose(Some(ws_onclose.unchecked_ref()));

    let ws_onerror = {
        let events = events.clone();
        Closure::<dyn FnMut()>::new(move || {
            web_sys::console::error_1(&"Websocket error".into());
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
                let message = postcard::from_bytes(&bytes).expect("Failed to deserialize message");
                let app_event = AppEvent::WebsocketMessage { message, received_at };
                (*events).borrow_mut().push(app_event);
            } else {
                web_sys::console::warn_1(&"Unexpected websocket message type".into());
            }
        })
        .into_js_value()
    };
    ws.set_onmessage(Some(ws_onmessage.unchecked_ref()));

    Ok(ws)
}

pub fn send(ws: &WebSocket, commands: Vec<PlayerCommand>) -> Result<(), JsValue> {
    let envelope = PlayerCommandEnvelope { commands };
    let bytes = postcard::to_stdvec(&envelope).map_err(|e| e.to_string())?;
    ws.send_with_u8_array(&bytes)?;
    Ok(())
}
