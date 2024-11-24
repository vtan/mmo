use std::collections::HashMap;

use nalgebra::{Vector2, Vector4};
use serde::Deserialize;

use crate::vertex_buffer::VertexBuffer;

pub struct FontAtlas {
    atlas: Vector2<u32>,
    glyphs: HashMap<char, Glyph>,
}

impl FontAtlas {
    pub fn from_meta(meta: FontMeta) -> Self {
        let atlas = Vector2::new(meta.atlas.width, meta.atlas.height);
        let mut glyphs = HashMap::new();
        for glyph in meta.glyphs {
            if let Some(ch) = std::char::from_u32(glyph.unicode) {
                glyphs.insert(ch, glyph);
            }
        }
        Self { atlas, glyphs }
    }

    pub fn push_glyph(
        &self,
        ch: char,
        top_left: Vector2<f32>,
        height: f32,
        vertex_buffer: &mut VertexBuffer,
    ) {
        let glyph = if let Some(glyph) = self.glyphs.get(&ch) {
            glyph
        } else {
            return;
        };
        let glyph_bounds = if let Some(glyph_bounds) = glyph.atlas_bounds {
            glyph_bounds
        } else {
            return;
        };
        let atlas = self.atlas.cast();

        let extent = Vector2::new(
            glyph_bounds.right - glyph_bounds.left,
            glyph_bounds.top - glyph_bounds.bottom,
        );
        let screen_extent = extent * (height / extent.y);
        let texture_top_left =
            Vector2::new(glyph_bounds.left, atlas.y - glyph_bounds.top).component_div(&atlas);
        let texture_extent = extent.component_div(&atlas);
        let color = Vector4::new(1.0, 1.0, 1.0, 1.0);
        vertex_buffer.push_quad(
            top_left,
            screen_extent,
            texture_top_left,
            texture_extent,
            color,
        );
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontMeta {
    atlas: Atlas,
    glyphs: Vec<Glyph>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Atlas {
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Glyph {
    unicode: u32,
    atlas_bounds: Option<Rect>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Rect {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}
