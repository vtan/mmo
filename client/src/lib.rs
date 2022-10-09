mod app_event;
mod fps_counter;
mod shader;
mod texture;

use std::cell::RefCell;
use std::mem::size_of;
use std::rc::Rc;

use fps_counter::FpsCounter;
use app_event::AppEvent;
use mmo_common::MoveCommand;
use nalgebra::Orthographic3;
use nalgebra::Scale3;
use nalgebra::Vector2;
use texture::load_texture;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use web_sys::WebGl2RenderingContext as GL;
use web_sys::WebGlBuffer;
use web_sys::WebGlProgram;
use web_sys::WebGlTexture;
use web_sys::WebGlUniformLocation;
use web_sys::WebGlVertexArrayObject;
use web_sys::WebSocket;

static VERTEX_SHADER: &str = include_str!("shader-vert.glsl");
static FRAGMENT_SHADER: &str = include_str!("shader-frag.glsl");

static QUAD_VERTICES: [f32; 8] = [1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0];

struct AppState {
    gl: GL,
    program: WebGlProgram,
    attrib_locations: AttribLocations,
    uniform_locations: UniformLocations,
    textures: Textures,
    vaos: Vaos,
    buffers: Buffers,
    ticks: u64,
    connection: Option<Box<dyn Fn(MoveCommand)>>,
    player_position: Vector2<f32>,
}

struct AttribLocations {
    position: u32,
    instance_translation: u32,
    instance_texture_coord_offset: u32,
}

struct UniformLocations {
    view_projection: WebGlUniformLocation,
    sampler: WebGlUniformLocation,
}

struct Textures {
    tileset: WebGlTexture,
}

struct Vaos {
    tile: WebGlVertexArrayObject,
}

struct Buffers {
    quad_vertex: WebGlBuffer,
    tile_attrib: WebGlBuffer,
    tile_attrib_data: Vec<f32>,
}

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    let bincode_config = bincode::config::standard().with_limit::<32_768>();

    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;
    let canvas = document.get_element_by_id("canvas").ok_or("No canvas")?;
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let gl = canvas.get_context("webgl2")?.ok_or("No webgl")?.dyn_into::<GL>()?;

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
    };
    let uniform_locations = UniformLocations {
        view_projection: gl
            .get_uniform_location(&program, "viewProjection")
            .ok_or("No uniform location")?,
        sampler: gl.get_uniform_location(&program, "sampler").ok_or("No uniform location")?,
    };
    let textures = Textures { tileset: load_texture(&gl, "/assets/tileset.png").await? };
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

    let vaos = Vaos { tile: create_tile_vao(&gl, &buffers, &attrib_locations)? };

    let mut app_state = AppState {
        gl,
        program,
        attrib_locations,
        uniform_locations,
        textures,
        vaos,
        buffers,
        ticks: 0,
        connection: None,
        player_position: Vector2::new(0.0, 0.0),
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

    let key_listener = {
        let events = events.clone();
        Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
            if !event.repeat() {
                let game_event = AppEvent::KeyDown { code: event.code() };
                (*events).borrow_mut().push(game_event);
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
        render(&mut app_state, events);

        w.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();

        fps_counter.record_end();
    }));
    window
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();

    Ok(())
}

fn create_tile_vao(
    gl: &GL,
    buffers: &Buffers,
    attrib_locations: &AttribLocations,
) -> Result<WebGlVertexArrayObject, JsValue> {
    let vao = gl.create_vertex_array().ok_or("Could not create vertex array object")?;
    gl.bind_vertex_array(Some(&vao));

    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffers.quad_vertex));
    {
        let num_components = 2;
        let typ = GL::FLOAT;
        let normalize = false;
        let stride = 0;
        let offset = 0;
        gl.vertex_attrib_pointer_with_i32(
            attrib_locations.position,
            num_components,
            typ,
            normalize,
            stride,
            offset,
        );
        gl.enable_vertex_attrib_array(attrib_locations.position);
    }

    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffers.tile_attrib));
    {
        let num_components = 2;
        let typ = GL::FLOAT;
        let normalize = false;
        let stride = 4 * size_of::<f32>() as i32;
        let offset = 0;
        gl.vertex_attrib_pointer_with_i32(
            attrib_locations.instance_translation,
            num_components,
            typ,
            normalize,
            stride,
            offset,
        );
        gl.vertex_attrib_divisor(attrib_locations.instance_translation, 1);
        gl.enable_vertex_attrib_array(attrib_locations.instance_translation);
    }
    {
        let num_components = 2;
        let typ = GL::FLOAT;
        let normalize = false;
        let stride = 4 * size_of::<f32>() as i32;
        let offset = 2 * size_of::<f32>() as i32;
        gl.vertex_attrib_pointer_with_i32(
            attrib_locations.instance_texture_coord_offset,
            num_components,
            typ,
            normalize,
            stride,
            offset,
        );
        gl.vertex_attrib_divisor(attrib_locations.instance_texture_coord_offset, 1);
        gl.enable_vertex_attrib_array(attrib_locations.instance_texture_coord_offset);
    }
    Ok(vao)
}

fn render_tile_vao(state: &AppState) {
    let gl = &state.gl;
    gl.bind_vertex_array(Some(&state.vaos.tile));
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&state.buffers.tile_attrib));

    // Unsafe: do not allocate memory until the view is dropped
    unsafe {
        let buffer_view = js_sys::Float32Array::view(&state.buffers.tile_attrib_data);
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &buffer_view, GL::DYNAMIC_DRAW);
    }

    let offset = 0;
    let count = 4;
    let instance_count = state.buffers.tile_attrib_data.len() as i32 / 4;
    gl.draw_arrays_instanced(GL::TRIANGLE_STRIP, offset, count, instance_count);
}

fn render(state: &mut AppState, events: Vec<AppEvent>) {
    state.ticks += 1;

    let move_player = |state: &mut AppState, dx, dy| {
        state.player_position.x += dx;
        state.player_position.y += dy;
        if let Some(ws_sender) = &state.connection {
            ws_sender(MoveCommand { x: state.player_position.x, y: state.player_position.y });
        }
    };
    for event in events {
        match event {
            AppEvent::KeyDown { code } => match code.as_str() {
                "ArrowLeft" => move_player(state, -1.0, 0.0),
                "ArrowRight" => move_player(state, 1.0, 0.0),
                "ArrowUp" => move_player(state, 0.0, -1.0),
                "ArrowDown" => move_player(state, 0.0, 1.0),
                _ => (),
            },
            AppEvent::WebsocketConnected { sender } => state.connection = Some(sender),
            AppEvent::WebsocketDisconnected => state.connection = None,
            AppEvent::WebsocketMessage { message } => todo!(),
        }
    }

    /*
    if state.ticks % (3 * 60) == 0 {
        let i = state.ticks / (3 * 60);
        let x = (i / 16) as f32;
        let y = (i % 16) as f32;
        let attribs = [x, y, 5.0 / 16.0, 6.0 / 16.0];
        state.buffers.tile_attrib_data.extend_from_slice(&attribs);
    }
    */

    let gl = &state.gl;
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(GL::COLOR_BUFFER_BIT);

    if state.connection.is_none() {
        return;
    }

    state.buffers.tile_attrib_data =
        vec![state.player_position.x, state.player_position.y, 0.0, 0.0];

    gl.use_program(Some(&state.program));

    let projection = Orthographic3::new(0.0, 320.0, 180.0, 0.0, -1.0, 1.0).to_homogeneous();
    let view = Scale3::new(16.0, 16.0, 16.0).to_homogeneous();
    let view_projection = projection * view;
    gl.uniform_matrix4fv_with_f32_array(
        Some(&state.uniform_locations.view_projection),
        false,
        view_projection.as_slice(),
    );

    gl.active_texture(GL::TEXTURE0);
    gl.bind_texture(GL::TEXTURE_2D, Some(&state.textures.tileset));
    gl.uniform1i(Some(&state.uniform_locations.sampler), 0);

    render_tile_vao(state);
}
