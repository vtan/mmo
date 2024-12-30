use nalgebra::{Vector2, Vector4};

#[repr(C)]
pub struct TexturedVertex {
    pub position: Vector2<f32>,
    pub texture_position: Vector2<f32>,
    pub color: Vector4<u8>,
    pub texture_index: u32,
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
        color: Vector4<u8>,
        texture_index: u32,
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
            texture_index,
        });
        vs.push(TexturedVertex {
            position: top_left + dx,
            texture_position: texture_top_left + du,
            color,
            texture_index,
        });
        vs.push(TexturedVertex {
            position: top_left + dy,
            texture_position: texture_top_left + dv,
            color,
            texture_index,
        });

        // second triangle
        vs.push(TexturedVertex {
            position: top_left + dx,
            texture_position: texture_top_left + du,
            color,
            texture_index,
        });
        vs.push(TexturedVertex {
            position: top_left + dx + dy,
            texture_position: texture_top_left + du + dv,
            color,
            texture_index,
        });
        vs.push(TexturedVertex {
            position: top_left + dy,
            texture_position: texture_top_left + dv,
            color,
            texture_index,
        });
    }
}

pub struct TileVertexBuffer {
    pub vertex_buffer: VertexBuffer,
    pub tile_size_on_texture: Vector2<f32>,
    pub texture_columns: u32,
}

impl TileVertexBuffer {
    pub fn new(tile_counts: Vector2<u32>) -> Self {
        Self {
            vertex_buffer: VertexBuffer::new(),
            tile_size_on_texture: Vector2::new(1.0, 1.0).component_div(&tile_counts.cast()),
            texture_columns: tile_counts.x,
        }
    }

    pub fn push_tile(&mut self, top_left: Vector2<f32>, tile_index: u32, texture_index: u32) {
        self.push_tile_multi(top_left, Vector2::new(1, 1), tile_index, texture_index);
    }

    pub fn push_tile_multi(
        &mut self,
        top_left: Vector2<f32>,
        tile_extent: Vector2<u32>,
        tile_index: u32,
        texture_index: u32,
    ) {
        let u = (tile_index % self.texture_columns) as f32;
        let v = (tile_index / self.texture_columns) as f32;
        let texture_top_left = Vector2::new(
            u * self.tile_size_on_texture.x,
            v * self.tile_size_on_texture.y,
        );
        let tile_extent = tile_extent.cast();
        let texture_extent = self.tile_size_on_texture.component_mul(&tile_extent);
        self.vertex_buffer.push_quad(
            top_left,
            tile_extent,
            texture_top_left,
            texture_extent,
            Vector4::new(0xff, 0xff, 0xff, 0xff),
            texture_index,
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

    pub fn push_line(&mut self, start: Vector2<f32>, end: Vector2<f32>, color: Vector4<u8>) {
        self.vertex_buffer.vertices.push(TexturedVertex {
            position: start,
            texture_position: Vector2::new(0.0, 0.0),
            color,
            texture_index: 0,
        });
        self.vertex_buffer.vertices.push(TexturedVertex {
            position: end,
            texture_position: Vector2::new(0.0, 0.0),
            color,
            texture_index: 0,
        });
    }

    pub fn push_rect(&mut self, top_left: Vector2<f32>, extent: Vector2<f32>, color: Vector4<u8>) {
        self.push_line(top_left, top_left + Vector2::new(extent.x, 0.0), color);
        self.push_line(
            top_left + Vector2::new(extent.x, 0.0),
            top_left + extent,
            color,
        );
        self.push_line(
            top_left + extent,
            top_left + Vector2::new(0.0, extent.y),
            color,
        );
        self.push_line(top_left + Vector2::new(0.0, extent.y), top_left, color);
    }
}
