#version 300 es
precision mediump float;

in vec2 vertTexturePosition;
in vec4 vertColor;

out vec4 fragColor;

uniform sampler2D sampler;

float median(vec3 v) {
    return max(min(v.x, v.y), min(max(v.x, v.y), v.z));
}

float screenPxRange() {
    float distanceFieldRangePx = 2.0;
    vec2 unitRange = vec2(distanceFieldRangePx) / vec2(textureSize(sampler, 0));
    vec2 screenTexSize = vec2(1.0) / fwidth(vertTexturePosition);
    return max(0.5 * dot(unitRange, screenTexSize), 1.0);
}

void main() {
    vec4 texel = texture(sampler, vertTexturePosition);
    float dist = median(texel.rgb);

    float pxDist = screenPxRange() * (dist - 0.5);
    float opacity = clamp(pxDist + 0.5, 0.0, 1.0);

    fragColor = vec4(1.0, 1.0, 1.0, opacity) * vertColor;
}
