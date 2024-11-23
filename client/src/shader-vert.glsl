#version 300 es
in vec2 position;
in vec2 texturePosition;
in vec4 color;

out vec2 vertTexturePosition;
out vec4 vertColor;

uniform mat4 viewProjection;

void main() {
    gl_Position = viewProjection * vec4(position, 0.0, 1.0);
    vertTexturePosition = texturePosition;
    vertColor = color;
}
