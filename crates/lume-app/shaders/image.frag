#version 310 es
precision mediump float;

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
    vec4 tex = texture(u_texture, v_uv) * v_color;
    vec4 color = tex + v_bg * (1.0 - tex.a);
    if (v_shape.z > 0.0) {
        float inset = 0.5 * v_shape.w;
        float d = round_rect_distance(v_local, v_shape.xy - inset, max(v_shape.z - inset, 0.0));
        color *= clamp(0.5 - d / v_shape.w, 0.0, 1.0);
    }
    o_color = color;
}
