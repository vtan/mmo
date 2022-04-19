import { compileShaders } from "./shader";

const canvas = document.getElementById("canvas") as HTMLCanvasElement | null;
const gl = canvas?.getContext("webgl2") as WebGL2RenderingContext | null;

if (canvas === null || gl === null) {
  throw Error("Failed to initialize WebGL 2 context");
}

const vertexShader = `
#version 300 es

in vec4 position;
uniform float angle;

void main() {
  gl_Position = mat4(mat2(cos(angle), sin(angle), -sin(angle), cos(angle))) * position;
}
`.trim();

const fragmentShader = `
#version 300 es
precision mediump float;

out vec4 fragColor;

void main() {
  fragColor = vec4(1.0, 1.0, 1.0, 1.0);
}
`.trim();

const shaderProgram = compileShaders(gl, vertexShader, fragmentShader);
const attribLocations = {
  position: gl.getAttribLocation(shaderProgram, "position")
};
const uniformLocations = {
  angle: gl.getUniformLocation(shaderProgram, "angle")
};

const positionBuffer = gl.createBuffer();
gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
gl.bufferData(
  gl.ARRAY_BUFFER,
  new Float32Array([1, 1, -1, 1, 1, -1, -1, -1,]),
  gl.STATIC_DRAW
);

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
    gl.vertexAttribPointer(
      attribLocations.position,
      numComponents,
      type,
      normalize,
      stride,
      offset
    );
    gl.enableVertexAttribArray(attribLocations.position);
  }

  gl.useProgram(shaderProgram);
  gl.uniform1f(uniformLocations.angle, now);
  {
    const offset = 0;
    const vertexCount = 3;
    gl.drawArrays(gl.TRIANGLE_STRIP, offset, vertexCount);
  }

  window.requestAnimationFrame(renderFrame);
}
