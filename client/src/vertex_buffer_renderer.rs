use std::mem::{offset_of, size_of};

use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext as GL;
use web_sys::{WebGlBuffer, WebGlVertexArrayObject};

use crate::vertex_buffer::{TexturedVertex, VertexBuffer};

pub const ATTRIB_LOC_POSITION: u32 = 0;
pub const ATTRIB_LOC_TEXTURE_POSITION: u32 = 1;
pub const ATTRIB_LOC_COLOR: u32 = 2;
pub const ATTRIB_LOC_TEXTURE_INDEX: u32 = 3;

pub struct VertexBufferRenderer {
    pub vao: WebGlVertexArrayObject,
    pub vbo: WebGlBuffer,
}

impl VertexBufferRenderer {
    pub fn new(gl: &GL) -> Result<Self, JsValue> {
        let vao = gl
            .create_vertex_array()
            .ok_or("Could not create vertex array object")?;
        gl.bind_vertex_array(Some(&vao));

        let vbo = gl.create_buffer().ok_or("Could not create buffer")?;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo));

        let stride = size_of::<TexturedVertex>() as i32;
        {
            let num_components = 2;
            let typ = GL::FLOAT;
            let normalize = false;
            let offset = offset_of!(TexturedVertex, position) as i32;
            gl.vertex_attrib_pointer_with_i32(
                ATTRIB_LOC_POSITION,
                num_components,
                typ,
                normalize,
                stride,
                offset,
            );
            gl.enable_vertex_attrib_array(ATTRIB_LOC_POSITION);
        }
        {
            let num_components = 2;
            let typ = GL::FLOAT;
            let normalize = false;
            let offset = offset_of!(TexturedVertex, texture_position) as i32;
            gl.vertex_attrib_pointer_with_i32(
                ATTRIB_LOC_TEXTURE_POSITION,
                num_components,
                typ,
                normalize,
                stride,
                offset,
            );
            gl.enable_vertex_attrib_array(ATTRIB_LOC_TEXTURE_POSITION);
        }
        {
            let num_components = 4;
            let typ = GL::UNSIGNED_BYTE;
            let normalize = true;
            let offset = offset_of!(TexturedVertex, color) as i32;
            gl.vertex_attrib_pointer_with_i32(
                ATTRIB_LOC_COLOR,
                num_components,
                typ,
                normalize,
                stride,
                offset,
            );
            gl.enable_vertex_attrib_array(ATTRIB_LOC_COLOR);
        }
        {
            let num_components = 1;
            let typ = GL::UNSIGNED_INT;
            let normalize = false;
            let offset = offset_of!(TexturedVertex, texture_index) as i32;
            gl.vertex_attrib_pointer_with_i32(
                ATTRIB_LOC_TEXTURE_INDEX,
                num_components,
                typ,
                normalize,
                stride,
                offset,
            );
            gl.enable_vertex_attrib_array(ATTRIB_LOC_TEXTURE_INDEX);
        }

        Ok(Self { vao, vbo })
    }

    pub fn render_triangles(&mut self, vertex_buffer: &VertexBuffer, gl: &GL) {
        self.prepare_buffer(vertex_buffer, gl);
        gl.draw_arrays(GL::TRIANGLES, 0, vertex_buffer.vertices.len() as i32);
    }

    pub fn render_lines(&mut self, vertex_buffer: &VertexBuffer, gl: &GL) {
        self.prepare_buffer(vertex_buffer, gl);
        gl.draw_arrays(GL::LINES, 0, vertex_buffer.vertices.len() as i32);
    }

    fn prepare_buffer(&self, vertex_buffer: &VertexBuffer, gl: &GL) {
        gl.bind_vertex_array(Some(&self.vao));
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.vbo));

        // Unsafe: do not allocate memory until the view is dropped
        unsafe {
            let byte_slice = vertex_buffer.byte_slice();
            let buffer_view = js_sys::Uint8Array::view(byte_slice);
            gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &buffer_view, GL::DYNAMIC_DRAW);
        }
    }
}
