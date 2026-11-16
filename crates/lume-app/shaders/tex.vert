#version 310 es
// Textured quads (glyphs, icons, glass): TEX_VERTEX of lume-core for Vulkan.
layout(location = 0) in vec4 a_pos_uv;
layout(location = 1) in vec4 a_color;
layout(location = 2) in vec4 a_local;
layout(location = 3) in vec4 a_params;
layout(location = 4) in vec4 a_bg;

layout(push_constant) uniform Push { vec4 view; vec4 surface; } pc;

layout(location = 0) out vec2 v_uv;
layout(location = 1) out vec4 v_color;
layout(location = 2) flat out vec4 v_shape;
layout(location = 3) out vec2 v_local;
layout(location = 4) flat out vec4 v_bg;

void main() {
    vec2 device = a_pos_uv.xy * pc.view.x + pc.view.yz;
    float w = 1.0 + a_params.y;
    gl_Position = vec4((device.x / pc.surface.x * 2.0 - 1.0) * w, (device.y / pc.surface.y * 2.0 - 1.0) * w, 0.0, w);
    v_uv = a_pos_uv.zw;
    v_color = a_color;
    v_shape = vec4(a_local.zw, a_params.x, pc.view.w / pc.view.x);
    v_local = a_local.xy;
    v_bg = a_bg;
}
