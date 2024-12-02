use std::collections::HashMap;

use nalgebra::{Vector2, Vector4};
use serde::Deserialize;

use crate::vertex_buffer::VertexBuffer;

pub struct FontAtlas {
    pub distance_range: f32,
    atlas_size: Vector2<u32>,
    metrics: Metrics,
    glyphs: HashMap<char, Glyph>,
}

impl FontAtlas {
    pub fn from_meta(meta: FontMeta) -> Self {
        let atlas_size = Vector2::new(meta.atlas.width, meta.atlas.height);
        let distance_range = meta.atlas.distance_range;
        let metrics = meta.metrics;
        let mut glyphs = HashMap::new();
        for glyph in meta.glyphs {
            if let Some(ch) = std::char::from_u32(glyph.unicode) {
                glyphs.insert(ch, glyph);
            }
        }
        Self { atlas_size, distance_range, metrics, glyphs }
    }

    pub fn push_text(
        &self,
        text: &str,
        top_left: Vector2<f32>,
        height: f32,
        color: Vector4<f32>,
        vertex_buffer: &mut VertexBuffer,
    ) {
        let mut cursor = top_left;
        for ch in text.chars() {
            if let Some(glyph) = self.glyphs.get(&ch) {
                self.push_glyph(glyph, cursor, height, color, vertex_buffer);
                cursor.x += height / self.metrics.line_height * glyph.advance;
            }
        }
    }

    fn push_glyph(
        &self,
        glyph: &Glyph,
        top_left: Vector2<f32>,
        height: f32,
        color: Vector4<f32>,
        vertex_buffer: &mut VertexBuffer,
    ) {
        let plane_bounds = if let Some(plane_bounds) = glyph.plane_bounds {
            plane_bounds
        } else {
            return;
        };
        let atlas_bounds = if let Some(atlas_bounds) = glyph.atlas_bounds {
            atlas_bounds
        } else {
            return;
        };

        let plane_to_px = height / self.metrics.line_height;

        let (screen_top_left, screen_extent) = {
            let top = self.metrics.ascender - plane_bounds.top;
            let bottom = self.metrics.ascender - plane_bounds.bottom;
            let left = plane_bounds.left;
            let right = plane_bounds.right;
            (
                top_left + Vector2::new(left, top) * plane_to_px,
                Vector2::new(right - left, bottom - top) * plane_to_px,
            )
        };

        let atlas = self.atlas_size.cast();

        let texture_top_left =
            Vector2::new(atlas_bounds.left, atlas.y - atlas_bounds.top).component_div(&atlas);
        let texture_extent = Vector2::new(
            atlas_bounds.right - atlas_bounds.left,
            atlas_bounds.top - atlas_bounds.bottom,
        )
        .component_div(&atlas);
        vertex_buffer.push_quad(
            screen_top_left,
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
    metrics: Metrics,
    glyphs: Vec<Glyph>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Atlas {
    width: u32,
    height: u32,
    distance_range: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Metrics {
    line_height: f32,
    ascender: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Glyph {
    unicode: u32,
    advance: f32,
    plane_bounds: Option<Rect>,
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
