use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use game_state::GameState;
use js_sys::{ArrayBuffer, Uint8Array};
use nalgebra::Vector2;
use texture::load_texture;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{KeyboardEvent, MessageEvent, WebGl2RenderingContext as GL, WebSocket};

use crate::app_event::AppEvent;
use crate::app_state::{AppState, AttribLocations, Buffers, Textures, UniformLocations, Vaos};
use crate::fps_counter::FpsCounter;

mod app_event;
mod app_state;
mod fps_counter;
mod game_state;
mod render;
mod shader;
mod texture;
mod update;

static VERTEX_SHADER: &str = include_str!("shader-vert.glsl");
static FRAGMENT_SHADER: &str = include_str!("shader-frag.glsl");

static QUAD_VERTICES: [f32; 8] = [1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0];

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    let bincode_config = bincode::config::standard().with_limit::<32_768>();

    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;
    let canvas = document.get_element_by_id("canvas").ok_or("No canvas")?;
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let gl = {
        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"alpha".into(), &JsValue::FALSE).unwrap();
        canvas
            .get_context_with_context_options("webgl2", &options)?
            .ok_or("No webgl")?
            .dyn_into::<GL>()?
    };
    gl.enable(GL::BLEND);
    gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);

    let vert_shader = shader::compile(&gl, GL::VERTEX_SHADER, VERTEX_SHADER)?;
    let frag_shader = shader::compile(&gl, GL::FRAGMENT_SHADER, FRAGMENT_SHADER)?;
    let program = shader::link(&gl, &vert_shader, &frag_shader)?;
    gl.use_program(Some(&program));

    let attrib_locations = AttribLocations {
        position: gl.get_attrib_location(&program, "position") as u32,
        instance_translation: gl.get_attrib_location(&program, "instanceTranslation") as u32,
        instance_texture_coord_offset: gl
            .get_attrib_location(&program, "instanceTextureCoordOffset")
            as u32,
        instance_texture_index: gl.get_attrib_location(&program, "instanceTextureIndex") as u32,
    };
    let uniform_locations = UniformLocations {
        view_projection: gl
            .get_uniform_location(&program, "viewProjection")
            .ok_or("No uniform location")?,
        sampler: gl.get_uniform_location(&program, "sampler").ok_or("No uniform location")?,
    };
    let textures = Textures {
        tileset: load_texture(&gl, "/assets/tileset.png").await?,
        charset: load_texture(&gl, "/assets/charset.png").await?,
    };
    let buffers = Buffers {
        quad_vertex: gl.create_buffer().ok_or("Failed to create buffer")?,
        tile_attrib: gl.create_buffer().ok_or("Failed to create buffer")?,
        tile_attrib_data: vec![],
    };

    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffers.quad_vertex));
    // Unsafe: do not allocate memory until the view is dropped
    unsafe {
        let buffer_view = js_sys::Float32Array::view(&QUAD_VERTICES);
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &buffer_view, GL::STATIC_DRAW);
    }

    let vaos = Vaos {
        tile: render::create_tile_vao(&gl, &buffers, &attrib_locations)?,
    };

    let mut app_state = AppState {
        gl,
        program,
        attrib_locations,
        uniform_locations,
        textures,
        vaos,
        buffers,
        game_state: GameState {
            connection: None,
            room_id: 0,
            tiles: vec![],
            player_position: Vector2::new(0.0, 0.0),
            other_positions: HashMap::new(),
        },
    };
    let events = Rc::new(RefCell::new(vec![]));

    let mut fps_counter = FpsCounter::new(&window);

    // TODO: construct URL from window.location
    let ws = WebSocket::new("ws://localhost:8081/api/ws")?;
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let ws_onopen = {
        let events = events.clone();
        let ws = ws.clone();
        Closure::once_into_js(move || {
            let sender = Box::new(move |command| {
                let bytes = bincode::encode_to_vec(command, bincode_config).unwrap();
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
            if let Ok(buf) = ws_event.data().dyn_into::<ArrayBuffer>() {
                let bytes = Uint8Array::new(&buf).to_vec();
                let (message, _) = bincode::decode_from_slice(&bytes, bincode_config).unwrap();
                let app_event = AppEvent::WebsocketMessage { message };
                (*events).borrow_mut().push(app_event);
            } else {
                web_sys::console::warn_1(&"Unexpected websocket message type".into());
            }
        })
        .into_js_value()
    };
    ws.set_onmessage(Some(ws_onmessage.unchecked_ref()));

    let key_listener = {
        let events = events.clone();
        Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
            if !event.repeat() {
                let app_event = AppEvent::KeyDown { code: event.code() };
                (*events).borrow_mut().push(app_event);
            }
        })
        .into_js_value()
    };
    document.add_event_listener_with_callback("keydown", key_listener.unchecked_ref())?;

    let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let g = f.clone();

    let w = window.clone();
    *g.borrow_mut() = Some(Closure::new(move || {
        fps_counter.record_start();

        let events = (*events).take();

        render::render(&mut app_state);
        update::update(&mut app_state, events);

        w.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();

        fps_counter.record_end();
    }));
    window
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();

    Ok(())
}
