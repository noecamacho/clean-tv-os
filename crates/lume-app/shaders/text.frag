#version 310 es
precision mediump float;

layout(set = 0, binding = 0) uniform sampler2D u_texture;

layout(location = 0) in vec2 v_uv;
layout(location = 1) in vec4 v_color;

layout(location = 0) out vec4 o_color;

void main() {
    // The atlas is one 8-bit channel.
    o_color = v_color * texture(u_texture, v_uv).r;
}
