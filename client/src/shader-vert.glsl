#version 300 es
in vec2 position;
in vec2 texturePosition;
in vec4 color;
in float textureIndex;

out vec2 vertTexturePosition;
out vec4 vertColor;
flat out float vertTextureIndex;

uniform mat3 viewProjection;

void main() {
    vec3 position = viewProjection * vec3(position, 1.0);
    gl_Position = vec4(position.x, position.y, 0.0, position.z);
    vertTexturePosition = texturePosition;
    vertColor = color;
    vertTextureIndex = textureIndex;
}
