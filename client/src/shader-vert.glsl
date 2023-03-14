#version 300 es
in vec2 position;
in vec2 instanceTranslation;
in vec2 instanceTextureCoordOffset;
in float instanceTextureIndex;

out vec2 fragTextureCoord;
flat out float fragTextureIndex;

uniform mat4 viewProjection;

// TODO: do not hardcode
const float TILES_ON_TEXTURE = 16.0;

void main() {
    gl_Position = viewProjection * vec4(position + instanceTranslation, 0.0, 1.0);
    fragTextureCoord = instanceTextureCoordOffset + position / TILES_ON_TEXTURE;
    fragTextureIndex = instanceTextureIndex;
}
