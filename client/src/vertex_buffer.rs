use nalgebra::{Vector2, Vector4};

pub const WHITE: Vector4<f32> = Vector4::new(1.0, 1.0, 1.0, 1.0);

#[repr(C)]
pub struct TexturedVertex {
    pub position: Vector2<f32>,
    pub texture_position: Vector2<f32>,
    pub color: Vector4<f32>,
}

pub struct VertexBuffer {
    pub vertices: Vec<TexturedVertex>,
}

impl VertexBuffer {
    pub fn new() -> Self {
        Self { vertices: vec![] }
    }

    pub unsafe fn byte_slice(&self) -> &[u8] {
        std::slice::from_raw_parts(
            self.vertices.as_ptr() as *const u8,
            self.vertices.len() * std::mem::size_of::<TexturedVertex>(),
        )
    }

    pub fn push_quad(
        &mut self,
        top_left: Vector2<f32>,
        extent: Vector2<f32>,
        texture_top_left: Vector2<f32>,
        texture_extent: Vector2<f32>,
        color: Vector4<f32>,
    ) {
        let vs = &mut self.vertices;
        let dx = Vector2::new(extent.x, 0.0);
        let dy = Vector2::new(0.0, extent.y);
        let du = Vector2::new(texture_extent.x, 0.0);
        let dv = Vector2::new(0.0, texture_extent.y);

        // first triangle
        vs.push(TexturedVertex {
            position: top_left,
            texture_position: texture_top_left,
            color,
        });
        vs.push(TexturedVertex {
            position: top_left + dx,
            texture_position: texture_top_left + du,
            color,
        });
        vs.push(TexturedVertex {
            position: top_left + dy,
            texture_position: texture_top_left + dv,
            color,
        });

        // second triangle
        vs.push(TexturedVertex {
            position: top_left + dx,
            texture_position: texture_top_left + du,
            color,
        });
        vs.push(TexturedVertex {
            position: top_left + dx + dy,
            texture_position: texture_top_left + du + dv,
            color,
        });
        vs.push(TexturedVertex {
            position: top_left + dy,
            texture_position: texture_top_left + dv,
            color,
        });
    }
}

pub struct TileVertexBuffer {
    pub vertex_buffer: VertexBuffer,
    pub tile_size_on_screen: Vector2<f32>,
    pub tile_size_on_texture: Vector2<f32>,
    pub texture_columns: u32,
}

impl TileVertexBuffer {
    pub fn new(tile_size_on_screen: Vector2<f32>, tile_counts: Vector2<u32>) -> Self {
        Self {
            vertex_buffer: VertexBuffer::new(),
            tile_size_on_screen,
            tile_size_on_texture: Vector2::new(1.0, 1.0).component_div(&tile_counts.cast()),
            texture_columns: tile_counts.x,
        }
    }

    pub fn push_tile(&mut self, top_left: Vector2<f32>, texture_index: u32) {
        let u = (texture_index % self.texture_columns) as f32;
        let v = (texture_index / self.texture_columns) as f32;
        let texture_top_left = Vector2::new(
            u * self.tile_size_on_texture.x,
            v * self.tile_size_on_texture.y,
        );
        self.vertex_buffer.push_quad(
            top_left,
            self.tile_size_on_screen,
            texture_top_left,
            self.tile_size_on_texture,
            Vector4::new(1.0, 1.0, 1.0, 1.0),
        );
    }
}

pub struct LineVertexBuffer {
    pub vertex_buffer: VertexBuffer,
}

impl LineVertexBuffer {
    pub fn new() -> Self {
        Self { vertex_buffer: VertexBuffer::new() }
    }

    pub fn push_line(&mut self, start: Vector2<f32>, end: Vector2<f32>, color: Vector4<f32>) {
        self.vertex_buffer.vertices.push(TexturedVertex {
            position: start,
            texture_position: Vector2::new(0.0, 0.0),
            color,
        });
        self.vertex_buffer.vertices.push(TexturedVertex {
            position: end,
            texture_position: Vector2::new(0.0, 0.0),
            color,
        });
    }
}
