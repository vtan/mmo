#version 300 es
precision mediump float;

in vec2 vertTexturePosition;
in vec4 vertColor;

out vec4 fragColor;

uniform sampler2D sampler;

void main() {
    fragColor = vertColor * texture(sampler, vertTexturePosition);
}
