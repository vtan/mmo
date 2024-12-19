#version 300 es
precision mediump float;

in vec2 vertTexturePosition;
in vec4 vertColor;
flat in float vertTextureIndex;

out vec4 fragColor;

uniform sampler2D sampler[2];

void main() {
    switch (int(vertTextureIndex)) {
        case 1:
            fragColor = texture(sampler[1], vertTexturePosition);
            break;
        default:
            fragColor = texture(sampler[0], vertTexturePosition);
            break;
    }
    fragColor = vertColor * fragColor;
}
