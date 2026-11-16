#version 310 es
layout(location = 0) in vec4 a_pos_uv;
layout(location = 1) in vec4 a_color;

layout(push_constant) uniform Push { vec4 view; vec4 surface; } pc;

layout(location = 0) out vec2 v_uv;
layout(location = 1) out vec4 v_color;
layout(location = 2) out vec2 v_bayer;

void main() {
    vec2 device = a_pos_uv.xy * pc.view.x + pc.view.yz;
    gl_Position = vec4(device.x / pc.surface.x * 2.0 - 1.0, device.y / pc.surface.y * 2.0 - 1.0, 0.0, 1.0);
    v_uv = a_pos_uv.zw;
    v_color = a_color;
    v_bayer = device / 4.0;
}
