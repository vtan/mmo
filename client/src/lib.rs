use std::cell::RefCell;
use std::rc::Rc;

use game_state::PartialGameState;
use vertex_buffer_renderer::VertexBufferRenderer;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{KeyboardEvent, WebGl2RenderingContext as GL};

use crate::app_event::AppEvent;
use crate::app_state::{AppState, UniformLocations};
use crate::fps_counter::FpsCounter;

mod app_event;
mod app_state;
mod assets;
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
mod ws_connection;

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
        text_distance_range: gl
            .get_uniform_location(&text_program, "distanceRange")
            .ok_or("No uniform location")?,
    };

    let vertex_buffer_renderer = VertexBufferRenderer::new(&gl)?;

    let fps_counter = FpsCounter::new(&window);

    let mut app_state = AppState {
        gl,
        program,
        text_program,
        uniform_locations,
        assets: None,
        vertex_buffer_renderer,
        fps_counter,
        events: Rc::new(RefCell::new(vec![])),
        game_state: Err(PartialGameState::new()),
    };

    let ws = ws_connection::connect(app_state.events.clone())?;

    let keydown_listener = {
        let events = app_state.events.clone();
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
        let events = app_state.events.clone();
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
        update_time(&mut app_state);
        update_canvas_size(&canvas, &mut app_state.gl);

        let events = (*app_state.events).take();
        update::update(&mut app_state, events);
        render::render(&mut app_state);

        if let Ok(ref mut game_state) = &mut app_state.game_state {
            ws_connection::send(&ws, &game_state.ws_commands).unwrap();
            game_state.ws_commands.clear();
        }

        w.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();

        app_state.fps_counter.record_end();
    }));
    window
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();

    Ok(())
}

fn update_time(app_state: &mut AppState) {
    let now = app_state.fps_counter.record_start();
    let now = (1e-3 * now) as f32;

    let time = match app_state.game_state {
        Ok(ref mut game_state) => &mut game_state.time,
        Err(ref mut partial) => &mut partial.time,
    };
    let prev_time = time.now;
    time.now = now;
    time.frame_delta = now - prev_time;
}

fn update_canvas_size(canvas: &web_sys::HtmlCanvasElement, gl: &mut GL) {
    let client_width = canvas.client_width() as u32;
    let client_height = canvas.client_height() as u32;
    if (canvas.width(), canvas.height()) != (client_width, client_height) {
        canvas.set_width(client_width);
        canvas.set_height(client_height);
        gl.viewport(0, 0, client_width as i32, client_height as i32);
    }
}
