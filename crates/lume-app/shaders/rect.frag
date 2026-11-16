#version 310 es
precision highp float;

layout(location = 0) in vec3 v_local;
layout(location = 1) flat in vec4 v_shape;
layout(location = 2) flat in vec4 v_misc;
layout(location = 3) flat in vec4 v_stops;
layout(location = 4) flat in vec4 v_c0;
layout(location = 5) flat in vec4 v_c1;
layout(location = 6) flat in vec4 v_c2;

layout(location = 0) out vec4 o_color;

float round_rect_distance(vec2 p, vec2 half_size, float radius) {
    vec2 q = abs(p) - half_size + radius;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - radius;
}

float dither() {
    vec2 p = mod(floor(gl_FragCoord.xy), 4.0);
    vec2 low = mod(p, 2.0);
    vec2 high = floor(p / 2.0);
    float level = 4.0 * mod(2.0 * low.x + 3.0 * low.y, 4.0) + mod(2.0 * high.x + 3.0 * high.y, 4.0);
    return ((level + 0.5) / 16.0 - 0.5) / 255.0;
}

void main() {
    float d = round_rect_distance(v_local.xy, v_shape.xy, v_shape.z);
    float unit = v_local.z;
    float alpha;
    if (v_misc.x > 0.0) {
        alpha = 1.0 - smoothstep(-v_misc.x, v_misc.x, d);
    } else {
        alpha = clamp(0.5 - d / unit, 0.0, 1.0);
        if (v_shape.w > 0.0) {
            alpha -= clamp(0.5 - (d + v_shape.w) / unit, 0.0, 1.0);
        }
    }
    vec4 color = v_c0;
    if (v_misc.w > 0.5) {
        float t = clamp(dot(v_local.xy, v_misc.yz) + 0.5, 0.0, 1.0);
        color = mix(v_c0, v_c1, clamp((t - v_stops.x) / max(v_stops.y - v_stops.x, 0.0001), 0.0, 1.0));
        color = mix(color, v_c2, clamp((t - v_stops.y) / max(v_stops.z - v_stops.y, 0.0001), 0.0, 1.0));
        color.rgb += dither() * color.a;
    }
    o_color = color * alpha;
}
