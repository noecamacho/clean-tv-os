#version 310 es
precision highp float;

layout(set = 0, binding = 0) uniform sampler2D u_texture;
layout(set = 0, binding = 1) uniform sampler2D u_bayer;
// view.xy: a quarter of a destination pixel, in source texture coordinates.
layout(push_constant) uniform Push { vec4 view; vec4 surface; } pc;

layout(location = 0) in vec2 v_uv;
layout(location = 1) in vec2 v_bayer;

layout(location = 0) out vec4 o_color;

void main() {
    vec2 s = pc.view.xy;
    vec3 color = texture(u_texture, v_uv + vec2(-s.x, -s.y)).rgb
        + texture(u_texture, v_uv + vec2(s.x, -s.y)).rgb
        + texture(u_texture, v_uv + vec2(-s.x, s.y)).rgb
        + texture(u_texture, v_uv + vec2(s.x, s.y)).rgb;
    color = color * 0.25 + (texture(u_bayer, v_bayer).r - 0.5) / 255.0;
    o_color = vec4(color, 1.0);
}
