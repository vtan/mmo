use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

static VIEW_PROJECTION: [f32; 16] = [
    0.1, 0.0, 0.0, 0.0,
    0.0, -0.17777777777777778, 0.0, 0.0,
    0.0, 0.0, -16.0, 0.0,
    -1.0, 1.0, 0.0, 1.0,
];

#[wasm_bindgen]
pub fn render(gl: WebGl2RenderingContext) {
    let offset = 0;
    let count = 4;
    let instance_count = 2;
    gl.draw_arrays_instanced(WebGl2RenderingContext::TRIANGLE_STRIP, offset, count, instance_count);
}
