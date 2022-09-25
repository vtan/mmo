#version 300 es
precision mediump float;

in vec2 fragTextureCoord;

out vec4 fragColor;

uniform sampler2D sampler;

void main() {
    fragColor = texture(sampler, fragTextureCoord);
}
