use std::mem::size_of;

use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext as GL;
use web_sys::{WebGlBuffer, WebGlVertexArrayObject};

use crate::vertex_buffer::VertexBuffer;

pub const ATTRIB_LOC_POSITION: u32 = 0;
pub const ATTRIB_LOC_TEXTURE_POSITION: u32 = 1;

pub struct VertexBufferRenderer {
    pub vao: WebGlVertexArrayObject,
    pub vbo: WebGlBuffer,
}

impl VertexBufferRenderer {
    pub fn new(gl: &GL) -> Result<Self, JsValue> {
        let vao = gl.create_vertex_array().ok_or("Could not create vertex array object")?;
        gl.bind_vertex_array(Some(&vao));

        let vbo = gl.create_buffer().ok_or("Could not create buffer")?;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo));

        let stride = 4 * size_of::<f32>() as i32;
        {
            let num_components = 2;
            let typ = GL::FLOAT;
            let normalize = false;
            let offset = 0;
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
            let offset = 2 * size_of::<f32>() as i32;
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

        Ok(Self { vao, vbo })
    }

    pub fn render(&mut self, vertex_buffer: &VertexBuffer, gl: &GL) {
        gl.bind_vertex_array(Some(&self.vao));
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.vbo));

        // Unsafe: do not allocate memory until the view is dropped
        unsafe {
            let byte_slice = vertex_buffer.byte_slice();
            let buffer_view = js_sys::Uint8Array::view(byte_slice);
            gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &buffer_view, GL::DYNAMIC_DRAW);
        }

        gl.draw_arrays(GL::TRIANGLES, 0, vertex_buffer.vertices.len() as i32);
    }
}
