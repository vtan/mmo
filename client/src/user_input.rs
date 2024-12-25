use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::{prelude::*, JsValue};
use web_sys::{Document, KeyboardEvent, MouseEvent};

use crate::app_event::AppEvent;

pub fn setup_handlers(
    document: &Document,
    events: Rc<RefCell<Vec<AppEvent>>>,
) -> Result<(), JsValue> {
    let keydown_listener = {
        let events = events.clone();
        Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
            if !event.repeat() {
                let app_event = AppEvent::KeyDown { code: event.code() };
                (*events).borrow_mut().push(app_event);
            }
        })
        .into_js_value()
    };
    document.add_event_listener_with_callback("keydown", keydown_listener.unchecked_ref())?;

    let keyup_listener = {
        let events = events.clone();
        Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
            let app_event = AppEvent::KeyUp { code: event.code() };
            (*events).borrow_mut().push(app_event);
        })
        .into_js_value()
    };
    document.add_event_listener_with_callback("keyup", keyup_listener.unchecked_ref())?;

    let mousedown_listener = {
        let events = events.clone();
        Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
            let app_event = AppEvent::MouseDown {
                x: event.client_x(),
                y: event.client_y(),
                button: event.button().into(),
            };
            (*events).borrow_mut().push(app_event);
            event.prevent_default();
        })
        .into_js_value()
    };
    document.add_event_listener_with_callback("mousedown", mousedown_listener.unchecked_ref())?;

    Ok(())
}
