mod shader;
mod texture;

use std::cell::RefCell;
use std::rc::Rc;

use nalgebra::Orthographic3;
use nalgebra::Scale3;
use texture::load_texture;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext as GL;
use web_sys::WebGlProgram;
use web_sys::WebGlTexture;
use web_sys::WebGlUniformLocation;

static VERTEX_SHADER: &str = include_str!("shader-vert.glsl");
static FRAGMENT_SHADER: &str = include_str!("shader-frag.glsl");

static QUAD_VERTICES: [f32; 8] = [1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0];

struct AppState {
    gl: GL,
    program: WebGlProgram,
    attrib_locations: AttribLocations,
    uniform_locations: UniformLocations,
    tileset_atlas: WebGlTexture,
    ticks: u64,
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

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;
    let canvas = document.get_element_by_id("canvas").ok_or("No canvas")?;
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let gl = canvas.get_context("webgl2")?.ok_or("No webgl")?.dyn_into::<GL>()?;

    let tileset_atlas = load_texture(&gl, "/assets/tileset.png").await?;

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

    let position_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&position_buffer));
    // Unsafe: do not allocate memory until the view is dropped
    unsafe {
        let buffer_view = js_sys::Float32Array::view(&QUAD_VERTICES);
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &buffer_view, GL::STATIC_DRAW);
    }

    let translation_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&translation_buffer));
    // Unsafe: do not allocate memory until the view is dropped
    unsafe {
        let buffer = [1.0, 0.0, 3.0, 1.0];
        let buffer_view = js_sys::Float32Array::view(&buffer);
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &buffer_view, GL::STATIC_DRAW);
    }

    let texture_coord_offset_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&texture_coord_offset_buffer));
    // Unsafe: do not allocate memory until the view is dropped
    unsafe {
        let buffer = [0.0, 0.0, 5.0 / 16.0, 1.0 / 16.0];
        let buffer_view = js_sys::Float32Array::view(&buffer);
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &buffer_view, GL::STATIC_DRAW);
    }

    let vao = gl.create_vertex_array().ok_or("Could not create vertex array object")?;
    gl.bind_vertex_array(Some(&vao));

    {
        let num_components = 2;
        let typ = GL::FLOAT;
        let normalize = false;
        let stride = 0;
        let offset = 0;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&position_buffer));
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
    {
        let num_components = 2;
        let typ = GL::FLOAT;
        let normalize = false;
        let stride = 0;
        let offset = 0;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&translation_buffer));
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
        let stride = 0;
        let offset = 0;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&texture_coord_offset_buffer));
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

    let mut app_state = AppState {
        gl,
        program,
        attrib_locations,
        uniform_locations,
        tileset_atlas,
        ticks: 0,
    };

    // TODO: take some time to understand this
    let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let g = f.clone();

    let w = window.clone();
    *g.borrow_mut() = Some(Closure::new(move || {
        render(&mut app_state);
        w.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }));
    window
        .clone()
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();

    Ok(())
}

fn render(state: &mut AppState) {
    let gl = &state.gl;
    state.ticks += 1;

    gl.use_program(Some(&state.program));

    let n = (state.ticks % 120) as f32;
    let projection = Orthographic3::new(0.0, 320.0 + n, 180.0, 0.0, -1.0, 1.0).to_homogeneous();
    let view = Scale3::new(16.0, 16.0, 16.0).to_homogeneous();
    let view_projection = projection * view;
    gl.uniform_matrix4fv_with_f32_array(
        Some(&state.uniform_locations.view_projection),
        false,
        view_projection.as_slice(),
    );

    gl.active_texture(GL::TEXTURE0);
    gl.bind_texture(GL::TEXTURE_2D, Some(&state.tileset_atlas));
    gl.uniform1i(Some(&state.uniform_locations.sampler), 0);

    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(GL::COLOR_BUFFER_BIT);

    let offset = 0;
    let count = 4;
    let instance_count = 2;
    gl.draw_arrays_instanced(GL::TRIANGLE_STRIP, offset, count, instance_count);
}
