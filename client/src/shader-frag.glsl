#version 300 es
precision mediump float;

in vec2 fragTextureCoord;
flat in float fragTextureIndex;

out vec4 fragColor;

uniform sampler2D sampler[2];

void main() {
    if (int(fragTextureIndex) == 0) {
        fragColor = texture(sampler[0], fragTextureCoord);
    } else {
        fragColor = texture(sampler[1], fragTextureCoord);
    }
}
