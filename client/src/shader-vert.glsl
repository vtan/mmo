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
