import { glMatrix, mat4 } from "gl-matrix";

import { compileShaders } from "./shader";
import { loadTexture } from "./texture";

const canvas = document.getElementById("canvas") as HTMLCanvasElement | null;
const gl = canvas?.getContext("webgl2") as WebGL2RenderingContext | null;


if (canvas === null || gl === null) {
  throw Error("Failed to initialize WebGL 2 context");
}
glMatrix.setMatrixArrayType(Array);

let texture: WebGLTexture | null = null;
loadTexture(gl, "/assets/tileset.png").then(loaded => texture = loaded);

const vertexShader = `
#version 300 es

in vec4 position;
in vec2 textureCoord;

out vec2 vertTextureCoord;

uniform mat4 mvp;

void main() {
  gl_Position = mvp * position;
  vertTextureCoord = textureCoord;
}
`.trim();

const fragmentShader = `
#version 300 es
precision mediump float;

in vec2 vertTextureCoord;

out vec4 fragColor;

uniform sampler2D sampler;

void main() {
  fragColor = texture(sampler, vertTextureCoord);
}
`.trim();

const shaderProgram = compileShaders(gl, vertexShader, fragmentShader);
const attribLocations = {
  position: gl.getAttribLocation(shaderProgram, "position"),
  textureCoord: gl.getAttribLocation(shaderProgram, "textureCoord")
};
const uniformLocations = {
  mvp: gl.getUniformLocation(shaderProgram, "mvp"),
  sampler: gl.getUniformLocation(shaderProgram, "sampler")
};

const positionBuffer = gl.createBuffer();
gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
gl.bufferData(
  gl.ARRAY_BUFFER,
  new Float32Array([1, 1, 0, 1, 1, 0, 0, 0,]),
  gl.STATIC_DRAW
);

const textureCoordBuffer = gl.createBuffer();
gl.bindBuffer(gl.ARRAY_BUFFER, textureCoordBuffer);
gl.bufferData(
  gl.ARRAY_BUFFER,
  new Float32Array([1/16, 1/16, 0, 1/16, 1/16, 0, 0, 0]),
  gl.STATIC_DRAW
);

const projection = mat4.create();
{
  const viewportW = 16;
  const viewportH = viewportW / 16 * 9;
  mat4.ortho(projection, 0, viewportW, viewportH, 0, -1, 1);
}

window.requestAnimationFrame(renderFrame);

function renderFrame(now: number) {
  now /= 1000;
  if (!gl) {
    return;
  }

  gl.clearColor(0, 0, 0, 1);
  gl.clear(gl.COLOR_BUFFER_BIT);

  {
    const numComponents = 2;
    const type = gl.FLOAT;
    const normalize = false;
    const stride = 0;
    const offset = 0;
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    gl.vertexAttribPointer(attribLocations.position, numComponents, type, normalize, stride, offset);
    gl.enableVertexAttribArray(attribLocations.position);
  }
  {
    const num = 2;
    const type = gl.FLOAT;
    const normalize = false;
    const stride = 0;
    const offset = 0;
    gl.bindBuffer(gl.ARRAY_BUFFER, textureCoordBuffer);
    gl.vertexAttribPointer(attribLocations.textureCoord, num, type, normalize, stride, offset);
    gl.enableVertexAttribArray(attribLocations.textureCoord);
  }

  gl.useProgram(shaderProgram);
  gl.uniformMatrix4fv(uniformLocations.mvp, false, projection);
  gl.activeTexture(gl.TEXTURE0);
  gl.bindTexture(gl.TEXTURE_2D, texture);
  gl.uniform1i(uniformLocations.sampler, 0);
  {
    const offset = 0;
    const vertexCount = 4;
    gl.drawArrays(gl.TRIANGLE_STRIP, offset, vertexCount);
  }

  window.requestAnimationFrame(renderFrame);
}
