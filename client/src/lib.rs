use std::cell::RefCell;
use std::rc::Rc;

use app_state::Timestamps;
use font_atlas::FontAtlas;
use game_state::PartialGameState;
use js_sys::{ArrayBuffer, Uint8Array};
use vertex_buffer_renderer::VertexBufferRenderer;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{KeyboardEvent, MessageEvent, WebGl2RenderingContext as GL, WebSocket};

use crate::app_event::AppEvent;
use crate::app_state::{AppState, Textures, UniformLocations};
use crate::fps_counter::FpsCounter;

mod app_event;
mod app_state;
mod fetch;
mod font_atlas;
mod fps_counter;
mod game_state;
mod render;
mod shader;
mod texture;
mod update;
mod vertex_buffer;
mod vertex_buffer_renderer;

static VERTEX_SHADER: &str = include_str!("shader-vert.glsl");
static FRAGMENT_SHADER: &str = include_str!("shader-frag.glsl");
static TEXT_FRAGMENT_SHADER: &str = include_str!("text-frag.glsl");

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;
    let canvas = document.get_element_by_id("canvas").ok_or("No canvas")?;
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let gl = {
        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"antialias".into(), &JsValue::FALSE).unwrap();
        js_sys::Reflect::set(&options, &"alpha".into(), &JsValue::FALSE).unwrap();
        canvas
            .get_context_with_context_options("webgl2", &options)?
            .ok_or("No webgl")?
            .dyn_into::<GL>()?
    };
    gl.enable(GL::BLEND);
    gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);

    let program = {
        let vert_shader = shader::compile(&gl, GL::VERTEX_SHADER, VERTEX_SHADER)?;
        let frag_shader = shader::compile(&gl, GL::FRAGMENT_SHADER, FRAGMENT_SHADER)?;
        shader::link(&gl, &vert_shader, &frag_shader)?
    };
    gl.bind_attrib_location(
        &program,
        vertex_buffer_renderer::ATTRIB_LOC_POSITION,
        "position",
    );
    gl.bind_attrib_location(
        &program,
        vertex_buffer_renderer::ATTRIB_LOC_TEXTURE_POSITION,
        "texturePosition",
    );

    let text_program = {
        let vert_shader = shader::compile(&gl, GL::VERTEX_SHADER, VERTEX_SHADER)?;
        let frag_shader = shader::compile(&gl, GL::FRAGMENT_SHADER, TEXT_FRAGMENT_SHADER)?;
        shader::link(&gl, &vert_shader, &frag_shader)?
    };
    // FIXME: duplication
    gl.bind_attrib_location(
        &text_program,
        vertex_buffer_renderer::ATTRIB_LOC_POSITION,
        "position",
    );
    gl.bind_attrib_location(
        &text_program,
        vertex_buffer_renderer::ATTRIB_LOC_TEXTURE_POSITION,
        "texturePosition",
    );

    let uniform_locations = UniformLocations {
        view_projection: gl
            .get_uniform_location(&program, "viewProjection")
            .ok_or("No uniform location")?,
        sampler: gl.get_uniform_location(&program, "sampler").ok_or("No uniform location")?,
        text_view_projection: gl
            .get_uniform_location(&text_program, "viewProjection")
            .ok_or("No uniform location")?,
        text_sampler: gl
            .get_uniform_location(&text_program, "sampler")
            .ok_or("No uniform location")?,
    };
    let textures = Textures {
        tileset: texture::load_texture(&gl, "/assets/tileset.png", GL::NEAREST).await?,
        charset: texture::load_texture(&gl, "/assets/charset.png", GL::NEAREST).await?,
        font: texture::load_texture(&gl, "/assets/notosans.png", GL::LINEAR).await?,
        white: texture::create_white_texture(&gl)?,
    };

    let font_atlas =
        FontAtlas::from_meta(fetch::fetch_json(&window, "/assets/notosans.json").await?);

    let vertex_buffer_renderer = VertexBufferRenderer::new(&gl)?;

    let time = Timestamps { now_ms: 0.0, now: 0.0, frame_delta: 0.0 };

    let mut app_state = AppState {
        gl,
        program,
        text_program,
        uniform_locations,
        textures,
        font_atlas,
        vertex_buffer_renderer,
        time,
        game_state: Err(PartialGameState::new()),
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
            if let Ok(buf) = ws_event.data().dyn_into::<ArrayBuffer>() {
                let bytes = Uint8Array::new(&buf).to_vec();
                let message = postcard::from_bytes(&bytes).unwrap();
                let app_event = AppEvent::WebsocketMessage { message };
                (*events).borrow_mut().push(app_event);
            } else {
                web_sys::console::warn_1(&"Unexpected websocket message type".into());
            }
        })
        .into_js_value()
    };
    ws.set_onmessage(Some(ws_onmessage.unchecked_ref()));

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

    let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let g = f.clone();

    let w = window.clone();
    *g.borrow_mut() = Some(Closure::new(move || {
        let prev_time_ms = app_state.time.now_ms;
        app_state.time.now_ms = fps_counter.record_start();
        app_state.time.now = (0.001 * app_state.time.now_ms) as f32;
        app_state.time.frame_delta = (0.001 * (app_state.time.now_ms - prev_time_ms)) as f32;

        let events = (*events).take();

        update::update(&mut app_state, events);
        render::render(&mut app_state);

        w.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();

        fps_counter.record_end();
    }));
    window
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();

    Ok(())
}
