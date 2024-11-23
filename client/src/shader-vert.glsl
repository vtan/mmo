#version 300 es
in vec2 position;
in vec2 texturePosition;

out vec2 vertTexturePosition;

uniform mat4 viewProjection;

void main() {
    gl_Position = viewProjection * vec4(position, 0.0, 1.0);
    vertTexturePosition = texturePosition;
}
