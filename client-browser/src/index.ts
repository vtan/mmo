import { glMatrix, mat4, vec3 } from "gl-matrix";

import { compileShaders } from "./shader";
import { loadTexture } from "./texture";
import * as wasm from "./wasm-bindgen/mmo_client";

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

in vec2 position;
in vec2 instanceTranslation;
in vec2 instanceTextureCoordOffset;

out vec2 fragTextureCoord;

uniform mat4 viewProjection;

void main() {
  gl_Position = viewProjection * vec4(position + instanceTranslation, 0.0, 1.0);
  fragTextureCoord = instanceTextureCoordOffset + position / 16.0;
}
`.trim();

const fragmentShader = `
#version 300 es
precision mediump float;

in vec2 fragTextureCoord;

out vec4 fragColor;

uniform sampler2D sampler;

void main() {
  fragColor = texture(sampler, fragTextureCoord);
}
`.trim();

const shaderProgram = compileShaders(gl, vertexShader, fragmentShader);
const attribLocations = {
  position: gl.getAttribLocation(shaderProgram, "position"),
  instanceTranslation: gl.getAttribLocation(shaderProgram, "instanceTranslation"),
  instanceTextureCoordOffset: gl.getAttribLocation(shaderProgram, "instanceTextureCoordOffset"),
};
const uniformLocations = {
  viewProjection: gl.getUniformLocation(shaderProgram, "viewProjection"),
  sampler: gl.getUniformLocation(shaderProgram, "sampler")
};

const positionBuffer = gl.createBuffer();
gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
gl.bufferData(
  gl.ARRAY_BUFFER,
  new Float32Array([1, 1, 0, 1, 1, 0, 0, 0,]),
  gl.STATIC_DRAW
);

const translationBuffer = gl.createBuffer();
gl.bindBuffer(gl.ARRAY_BUFFER, translationBuffer);
gl.bufferData(
  gl.ARRAY_BUFFER,
  new Float32Array([1, 0, 3, 1]),
  gl.STATIC_DRAW
);

const textureCoordOffsetBuffer = gl.createBuffer();
gl.bindBuffer(gl.ARRAY_BUFFER, textureCoordOffsetBuffer);
gl.bufferData(
  gl.ARRAY_BUFFER,
  new Float32Array([0, 0, 5/16, 1/16]),
  gl.STATIC_DRAW
);

const projection = mat4.create();
mat4.ortho(projection, 0, 320, 180, 0, -1, 1);
const view = mat4.create();
mat4.fromScaling(view, vec3.fromValues(16, 16, 16));
const viewProjection = mat4.create();
mat4.multiply(viewProjection, projection, view);

wasm.default().then(_ => {
  window.requestAnimationFrame(renderFrame);
});

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
    const numComponents = 2;
    const type = gl.FLOAT;
    const normalize = false;
    const stride = 0;
    const offset = 0;
    gl.bindBuffer(gl.ARRAY_BUFFER, translationBuffer);
    gl.vertexAttribPointer(attribLocations.instanceTranslation, numComponents, type, normalize, stride, offset);
    gl.vertexAttribDivisor(attribLocations.instanceTranslation, 1);
    gl.enableVertexAttribArray(attribLocations.instanceTranslation);
  }
  {
    const numComponents = 2;
    const type = gl.FLOAT;
    const normalize = false;
    const stride = 0;
    const offset = 0;
    gl.bindBuffer(gl.ARRAY_BUFFER, textureCoordOffsetBuffer);
    gl.vertexAttribPointer(attribLocations.instanceTextureCoordOffset, numComponents, type, normalize, stride, offset);
    gl.vertexAttribDivisor(attribLocations.instanceTextureCoordOffset, 1);
    gl.enableVertexAttribArray(attribLocations.instanceTextureCoordOffset);
  }

  gl.useProgram(shaderProgram);
  gl.uniformMatrix4fv(uniformLocations.viewProjection, false, viewProjection);
  gl.activeTexture(gl.TEXTURE0);
  gl.bindTexture(gl.TEXTURE_2D, texture);
  gl.uniform1i(uniformLocations.sampler, 0);

  wasm.render(gl);

  window.requestAnimationFrame(renderFrame);
}
