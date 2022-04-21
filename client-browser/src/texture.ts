export function loadTexture(gl: WebGL2RenderingContext, uri: string): Promise<WebGLTexture> {
  const level = 0;
  const internalFormat = gl.RGBA;
  const srcFormat = gl.RGBA;
  const srcType = gl.UNSIGNED_BYTE;

  return new Promise((resolve, reject) => {
    const image = new Image();
    image.src = uri;
    image.onload = () => {
      const texture = gl.createTexture();
      if (texture === null) {
        reject("null texture");
      } else {
        gl.bindTexture(gl.TEXTURE_2D, texture);
        gl.texImage2D(gl.TEXTURE_2D, level, internalFormat, srcFormat, srcType, image);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        resolve(texture);
      }
    };
    image.onerror = () => {
      reject("image error");
    };
  });
}
