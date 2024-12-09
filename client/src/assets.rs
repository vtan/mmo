use crate::{
    fetch,
    font_atlas::FontAtlas,
    texture::{self, Texture},
};
use mmo_common::client_config::AssetPaths;

use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext as GL;

pub struct Assets {
    pub tileset: Texture,
    pub charset: Texture,
    pub font: Texture,
    pub white: Texture,
    pub font_atlas: FontAtlas,
}

pub async fn load(gl: &GL, asset_paths: &AssetPaths) -> Result<Assets, JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let white = texture::create_white_texture(gl)?;

    let tileset = texture::load_texture(gl, &asset_paths.tileset, GL::NEAREST).await?;
    let charset = texture::load_texture(gl, &asset_paths.charset, GL::NEAREST).await?;
    let font = texture::load_texture(gl, &asset_paths.font, GL::LINEAR).await?;

    let font_meta = fetch::fetch_json(&window, &asset_paths.font_meta).await?;
    let font_atlas = FontAtlas::from_meta(font_meta);

    Ok(Assets { tileset, charset, font, white, font_atlas })
}
