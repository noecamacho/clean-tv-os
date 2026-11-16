#version 310 es
precision highp float;

layout(set = 0, binding = 0) uniform sampler2D u_texture;

layout(location = 0) in vec2 v_uv;
layout(location = 1) in vec4 v_color;
layout(location = 2) flat in vec4 v_shape;
layout(location = 3) in vec2 v_local;
layout(location = 4) flat in vec4 v_bg;

layout(location = 0) out vec4 o_color;

float round_rect_distance(vec2 p, vec2 half_size, float radius) {
    vec2 q = abs(p) - half_size + radius;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - radius;
}

void main() {
    float inset = 0.5 * v_shape.w;
    float d = round_rect_distance(v_local, v_shape.xy - inset, max(v_shape.z - inset, 0.0));
    vec2 outward = normalize(v_local / max(v_shape.xy, vec2(1.0)) + vec2(0.0001));
    float band = clamp(1.0 + d / 12.0, 0.0, 1.0);
    vec3 color = texture(u_texture, v_uv - outward * band * band * 0.012).rgb;
    float rim = clamp(1.0 + d / 1.5, 0.0, 1.0);
    float light = dot(outward, vec2(-0.7071, -0.7071));
    color += rim * (0.06 + 0.30 * light * light);
    float coverage = clamp(0.5 - d / v_shape.w, 0.0, 1.0);
    o_color = vec4(color * v_color.rgb, v_color.a) * coverage;
}
