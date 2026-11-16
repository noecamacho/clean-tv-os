#version 310 es
precision mediump float;

layout(set = 0, binding = 0) uniform sampler2D u_texture;
layout(set = 0, binding = 1) uniform sampler2D u_bayer;

layout(location = 0) in vec2 v_uv;
layout(location = 1) in vec4 v_color;
layout(location = 2) in vec2 v_bayer;

layout(location = 0) out vec4 o_color;

void main() {
    vec4 color = texture(u_texture, v_uv) * v_color;
    color.rgb += (texture(u_bayer, v_bayer).r - 0.5) / 255.0;
    o_color = color;
}
