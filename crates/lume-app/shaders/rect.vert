#version 310 es
// Rounded rectangles: see RECT_VERTEX in lume-core's shaders.rs, which this mirrors for Vulkan.
layout(location = 0) in vec4 a_pos_local;
layout(location = 1) in vec4 a_shape;
layout(location = 2) in vec4 a_misc;
layout(location = 3) in vec4 a_stops;
layout(location = 4) in vec4 a_c0;
layout(location = 5) in vec4 a_c1;
layout(location = 6) in vec4 a_c2;

layout(push_constant) uniform Push { vec4 view; vec4 surface; } pc;

layout(location = 0) out vec3 v_local;
// The same on the four vertices of a quad: flat spares the GPU their interpolation.
layout(location = 1) flat out vec4 v_shape;
layout(location = 2) flat out vec4 v_misc;
layout(location = 3) flat out vec4 v_stops;
layout(location = 4) flat out vec4 v_c0;
layout(location = 5) flat out vec4 v_c1;
layout(location = 6) flat out vec4 v_c2;

void main() {
    vec2 device = a_pos_local.xy * pc.view.x + pc.view.yz;
    float w = 1.0 + a_stops.w;
    // Vulkan's clip space has y down, as the canvas does.
    gl_Position = vec4((device.x / pc.surface.x * 2.0 - 1.0) * w, (device.y / pc.surface.y * 2.0 - 1.0) * w, 0.0, w);
    v_local = vec3(a_pos_local.zw, pc.view.w / pc.view.x);
    v_shape = a_shape;
    v_misc = a_misc;
    v_stops = a_stops;
    v_c0 = a_c0;
    v_c1 = a_c1;
    v_c2 = a_c2;
}
