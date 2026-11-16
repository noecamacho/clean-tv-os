#version 310 es
// One triangle over the whole destination; view.zw is its size in pixels.
layout(push_constant) uniform Push { vec4 view; vec4 surface; } pc;

layout(location = 0) out vec2 v_uv;
layout(location = 1) out vec2 v_bayer;

void main() {
    vec2 unit = vec2(float((gl_VertexIndex << 1) & 2), float(gl_VertexIndex & 2));
    gl_Position = vec4(unit * 2.0 - 1.0, 0.0, 1.0);
    v_uv = unit;
    v_bayer = unit * pc.view.zw / 4.0;
}
