use js_sys::Promise;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlImageElement, WebGl2RenderingContext as GL, WebGlTexture};

pub async fn load_texture(gl: &GL, uri: &str) -> Result<WebGlTexture, JsValue> {
    let level = 0;
    let internal_format = GL::RGBA;
    let src_format = GL::RGBA;
    let src_type = GL::UNSIGNED_BYTE;
    let image = HtmlImageElement::new()?;
    image.set_src(uri);

    let promise = Promise::new(&mut |resolve, reject| {
        let onload = Closure::<dyn Fn()>::new(move || {
            resolve.call0(&JsValue::NULL).unwrap();
        });
        let onerror = Closure::<dyn Fn()>::new(move || {
            reject.call0(&JsValue::NULL).unwrap();
        });
        image.set_onload(Some(onload.as_ref().unchecked_ref()));
        image.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onload.forget();
        onerror.forget();
    });
    JsFuture::from(promise).await?;

    let texture = gl.create_texture().ok_or("Could not create texture")?;
    gl.bind_texture(GL::TEXTURE_2D, Some(&texture));
    gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
        GL::TEXTURE_2D,
        level,
        internal_format as i32,
        src_format,
        src_type,
        &image,
    )?;
    gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32);
    gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32);
    Ok(texture)
}
