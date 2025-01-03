use std::cell::RefCell;
use std::rc::Rc;

use game_state::PartialGameState;
use nalgebra::Vector2;
use vertex_buffer_renderer::VertexBufferRenderer;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext as GL;

use crate::app_state::{AppState, UniformLocations};
use crate::metrics::Metrics;

mod app_event;
mod app_state;
mod assets;
mod camera;
mod fetch;
mod font_atlas;
mod game_state;
mod metrics;
mod render;
mod shader;
mod texture;
mod update;
mod user_input;
mod util;
mod vertex_buffer;
mod vertex_buffer_renderer;
mod ws_connection;

static VERTEX_SHADER: &str = include_str!("shader-vert.glsl");
static FRAGMENT_SHADER: &str = include_str!("shader-frag.glsl");
static TEXT_FRAGMENT_SHADER: &str = include_str!("text-frag.glsl");

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    let client_git_sha = option_env!("VERGEN_GIT_SHA").unwrap_or("???");

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
    gl.bind_attrib_location(
        &program,
        vertex_buffer_renderer::ATTRIB_LOC_TEXTURE_INDEX,
        "textureIndex",
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
    gl.bind_attrib_location(
        &text_program,
        vertex_buffer_renderer::ATTRIB_LOC_TEXTURE_INDEX,
        "textureIndex",
    );

    let uniform_locations = UniformLocations {
        view_projection: gl
            .get_uniform_location(&program, "viewProjection")
            .ok_or("No uniform location")?,
        sampler: gl
            .get_uniform_location(&program, "sampler")
            .ok_or("No uniform location")?,
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

    let metrics = Rc::new(RefCell::new(Metrics::new(&window)));

    let mut app_state = AppState {
        client_git_sha,
        gl,
        program,
        text_program,
        uniform_locations,
        assets: None,
        vertex_buffer_renderer,
        metrics: metrics.clone(),
        viewport: Vector2::new(canvas.client_width() as u32, canvas.client_height() as u32),
        events: Rc::new(RefCell::new(vec![])),
        game_state: Err(PartialGameState::new()),
    };

    let ws = ws_connection::connect(app_state.events.clone(), metrics)?;

    user_input::setup_handlers(&document, app_state.events.clone())?;

    render::init(&mut app_state);

    start_self_referential_closure(
        move |f| {
            window.request_animation_frame(f).unwrap();
        },
        move || {
            update_time(&mut app_state);
            update_canvas_size(&canvas, &mut app_state);

            let events = (*app_state.events).take();
            update::update(&mut app_state, events);

            if let Ok(ref mut game_state) = &mut app_state.game_state {
                let room_id = game_state.room.room_id;
                let ws_commands = std::mem::take(&mut game_state.ws_commands);
                if !ws_commands.is_empty() {
                    ws_connection::send(
                        &ws,
                        room_id,
                        ws_commands,
                        &mut app_state.metrics.borrow_mut(),
                    )
                    .unwrap();
                }
            }

            render::render(&mut app_state);
            app_state.metrics.borrow_mut().record_frame_end();
        },
    );

    Ok(())
}

fn start_self_referential_closure(
    mut consume: impl FnMut(&js_sys::Function) + 'static + Clone,
    mut f: impl FnMut() + 'static,
) {
    let x = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let y = x.clone();

    let mut consume_inner = consume.clone();
    *y.borrow_mut() = Some(Closure::new(move || {
        f();
        consume_inner(x.borrow().as_ref().unwrap().as_ref().unchecked_ref());
    }));
    consume(y.borrow().as_ref().unwrap().as_ref().unchecked_ref());
}

fn update_time(app_state: &mut AppState) {
    let now = app_state.metrics.borrow_mut().record_frame_start();
    let now = (1e-3 * now) as f32;

    let time = match app_state.game_state {
        Ok(ref mut game_state) => &mut game_state.time,
        Err(ref mut partial) => &mut partial.time,
    };
    let prev_time = time.now;
    time.now = now;
    time.frame_delta = now - prev_time;
}

fn update_canvas_size(canvas: &web_sys::HtmlCanvasElement, app_state: &mut AppState) {
    let client_width = canvas.client_width() as u32;
    let client_height = canvas.client_height() as u32;
    if (canvas.width(), canvas.height()) != (client_width, client_height) {
        canvas.set_width(client_width);
        canvas.set_height(client_height);
        app_state
            .gl
            .viewport(0, 0, client_width as i32, client_height as i32);
        app_state.viewport = Vector2::new(client_width, client_height);
    }
}
