use std::cell::RefCell;
use std::rc::Rc;

use js_sys::{ArrayBuffer, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket};

use crate::app_event::AppEvent;

pub fn connect(events: Rc<RefCell<Vec<AppEvent>>>) -> Result<WebSocket, JsValue> {
    let performance = web_sys::window().expect("No window").performance().expect("No performance");

    // TODO: construct URL from window.location
    let ws = WebSocket::new("ws://localhost:8081/api/ws")?;
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let ws_onopen = {
        let events = events.clone();
        let ws = ws.clone();
        Closure::once_into_js(move || {
            let sender = Box::new(move |command| {
                let bytes = postcard::to_stdvec(&command).unwrap();
                ws.send_with_u8_array(&bytes).unwrap();
            });
            (*events).borrow_mut().push(AppEvent::WebsocketConnected { sender });
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
                let message = postcard::from_bytes(&bytes).unwrap();
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
