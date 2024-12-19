#version 300 es
precision mediump float;

in vec2 vertTexturePosition;
in vec4 vertColor;
flat in float vertTextureIndex;

out vec4 fragColor;

uniform sampler2D sampler[2];
uniform float distanceRange;

float median(vec3 v) {
    return max(min(v.x, v.y), min(max(v.x, v.y), v.z));
}

float screenPxRange(vec2 texSize) {
    vec2 unitRange = vec2(distanceRange) / texSize;
    vec2 screenTexSize = vec2(1.0) / fwidth(vertTexturePosition);
    return max(0.5 * dot(unitRange, screenTexSize), 1.0);
}

void main() {
    vec4 texel = vec4(0.0);
    vec2 texSize = vec2(0.0);
    switch (int(vertTextureIndex)) {
        case 1:
            texel = texture(sampler[1], vertTexturePosition);
            texSize = vec2(textureSize(sampler[1], 0));
            break;
        default:
            texel = texture(sampler[0], vertTexturePosition);
            texSize = vec2(textureSize(sampler[0], 0));
            break;
    }

    float dist = median(texel.rgb);

    float pxDist = screenPxRange(texSize) * (dist - 0.5);
    float opacity = clamp(pxDist + 0.5, 0.0, 1.0);

    fragColor = vec4(1.0, 1.0, 1.0, opacity) * vertColor;
}
