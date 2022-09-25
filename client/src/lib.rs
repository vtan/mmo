mod shader;
mod texture;

use nalgebra::Orthographic3;
use nalgebra::Scale3;
use texture::load_texture;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext as GL};

static VERTEX_SHADER: &str = r##"#version 300 es
    in vec2 position;
    in vec2 instanceTranslation;
    in vec2 instanceTextureCoordOffset;

    out vec2 fragTextureCoord;

    uniform mat4 viewProjection;

    void main() {
        gl_Position = viewProjection * vec4(position + instanceTranslation, 0.0, 1.0);
        fragTextureCoord = instanceTextureCoordOffset + position / 16.0;
    }
"##;

static FRAGMENT_SHADER: &str = r##"#version 300 es
    precision mediump float;

    in vec2 fragTextureCoord;

    out vec4 fragColor;

    uniform sampler2D sampler;

    void main() {
        fragColor = texture(sampler, fragTextureCoord);
    }
"##;

static QUAD_VERTICES: [f32; 8] = [1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0];

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    let document = web_sys::window().ok_or("No window")?.document().ok_or("No document")?;
    let canvas = document.get_element_by_id("canvas").ok_or("No canvas")?;
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let gl = canvas.get_context("webgl2")?.ok_or("No webgl")?.dyn_into::<GL>()?;

    let texture = load_texture(&gl, "/assets/tileset.png").await?;

    let vert_shader = shader::compile(&gl, GL::VERTEX_SHADER, VERTEX_SHADER)?;
    let frag_shader = shader::compile(&gl, GL::FRAGMENT_SHADER, FRAGMENT_SHADER)?;
    let program = shader::link(&gl, &vert_shader, &frag_shader)?;
    gl.use_program(Some(&program));

    let position_location = gl.get_attrib_location(&program, "position");
    let instance_translation_location = gl.get_attrib_location(&program, "instanceTranslation");
    let instance_texture_coord_offset_location =
        gl.get_attrib_location(&program, "instanceTextureCoordOffset");

    let view_projection_location = gl
        .get_uniform_location(&program, "viewProjection")
        .ok_or("No uniform location")?;
    let sampler_location =
        gl.get_uniform_location(&program, "sampler").ok_or("No uniform location")?;

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
            position_location as u32,
            num_components,
            typ,
            normalize,
            stride,
            offset,
        );
        gl.enable_vertex_attrib_array(position_location as u32);
    }
    {
        let num_components = 2;
        let typ = GL::FLOAT;
        let normalize = false;
        let stride = 0;
        let offset = 0;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&translation_buffer));
        gl.vertex_attrib_pointer_with_i32(
            instance_translation_location as u32,
            num_components,
            typ,
            normalize,
            stride,
            offset,
        );
        gl.vertex_attrib_divisor(instance_translation_location as u32, 1);
        gl.enable_vertex_attrib_array(instance_translation_location as u32);
    }
    {
        let num_components = 2;
        let typ = GL::FLOAT;
        let normalize = false;
        let stride = 0;
        let offset = 0;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&texture_coord_offset_buffer));
        gl.vertex_attrib_pointer_with_i32(
            instance_texture_coord_offset_location as u32,
            num_components,
            typ,
            normalize,
            stride,
            offset,
        );
        gl.vertex_attrib_divisor(instance_texture_coord_offset_location as u32, 1);
        gl.enable_vertex_attrib_array(instance_texture_coord_offset_location as u32);
    }

    let projection = Orthographic3::new(0.0, 320.0, 180.0, 0.0, -1.0, 1.0).to_homogeneous();
    let view = Scale3::new(16.0, 16.0, 16.0).to_homogeneous();
    let view_projection = projection * view;
    gl.uniform_matrix4fv_with_f32_array(
        Some(&view_projection_location),
        false,
        view_projection.as_slice(),
    );

    gl.active_texture(GL::TEXTURE0);
    gl.bind_texture(GL::TEXTURE_2D, Some(&texture));
    gl.uniform1i(Some(&sampler_location), 0);

    draw(&gl);

    Ok(())
}

fn draw(gl: &GL) {
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(GL::COLOR_BUFFER_BIT);

    let offset = 0;
    let count = 4;
    let instance_count = 2;
    gl.draw_arrays_instanced(GL::TRIANGLE_STRIP, offset, count, instance_count);
}
