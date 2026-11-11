//! GLSL ES 1.00 sources. Only OpenGL ES 2.0 features are used.

/// Rounded rectangles from a signed distance function: fills (solid or gradient),
/// rings and soft shadows.
pub const RECT_VERTEX: &str = r#"#version 100
attribute vec4 a_pos_local; // xy: canvas position, zw: position relative to the center
attribute vec4 a_shape;     // xy: half size, z: radius, w: ring width
attribute vec4 a_misc;      // x: softness, yz: gradient axis, w: is gradient
attribute vec4 a_stops;     // xyz: gradient stop positions
attribute vec4 a_c0;
attribute vec4 a_c1;
attribute vec4 a_c2;

uniform vec4 u_view;        // x: scale, yz: offset (device px), w: edge softness (device px)
uniform vec2 u_surface;     // surface size in device px

varying vec3 v_local; // xy: position relative to the center, z: logical units of the edge ramp
varying vec4 v_shape;
varying vec4 v_misc;
varying vec4 v_stops;
varying vec4 v_c0;
varying vec4 v_c1;
varying vec4 v_c2;

void main() {
    vec2 device = a_pos_local.xy * u_view.x + u_view.yz;
    gl_Position = vec4(device.x / u_surface.x * 2.0 - 1.0, 1.0 - device.y / u_surface.y * 2.0, 0.0, 1.0);
    v_local = vec3(a_pos_local.zw, u_view.w / u_view.x);
    v_shape = a_shape;
    v_misc = a_misc;
    v_stops = a_stops;
    v_c0 = a_c0;
    v_c1 = a_c1;
    v_c2 = a_c2;
}
"#;

pub const RECT_FRAGMENT: &str = r#"#version 100
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif

varying vec3 v_local;
varying vec4 v_shape;
varying vec4 v_misc;
varying vec4 v_stops;
varying vec4 v_c0;
varying vec4 v_c1;
varying vec4 v_c2;

float round_rect_distance(vec2 p, vec2 half_size, float radius) {
    vec2 q = abs(p) - half_size + radius;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - radius;
}

// The 4x4 ordered (Bayer) threshold of this pixel, within +-0.5 of a color level and zero on
// average. floor/mod on gl_FragCoord stay exact enough in mediump, unlike fract(sin(...)).
float dither() {
    vec2 p = mod(floor(gl_FragCoord.xy), 4.0);
    vec2 low = mod(p, 2.0);
    vec2 high = floor(p / 2.0);
    float level = 4.0 * mod(2.0 * low.x + 3.0 * low.y, 4.0) + mod(2.0 * high.x + 3.0 * high.y, 4.0);
    return ((level + 0.5) / 16.0 - 0.5) / 255.0;
}

void main() {
    float d = round_rect_distance(v_local.xy, v_shape.xy, v_shape.z);
    float unit = v_local.z; // logical units of the edge ramp
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
        // A gradient would band once the TV raises the contrast.
        color.rgb += dither() * color.a;
    }
    gl_FragColor = color * alpha;
}
"#;

/// Textured quads (glyphs, icons and images). The vertex shader is shared.
pub const TEX_VERTEX: &str = r#"#version 100
attribute vec4 a_pos_uv; // xy: canvas position, zw: texture coordinates
attribute vec4 a_color;
attribute vec4 a_local;  // xy: position relative to the image center, zw: half size
attribute vec4 a_params; // x: radius
attribute vec4 a_bg;     // premultiplied color the image is composed over

uniform vec4 u_view;     // x: scale, yz: offset (device px), w: edge softness (device px)
uniform vec2 u_surface;

varying vec2 v_uv;
varying vec4 v_color;
varying vec4 v_shape;    // xy: half size, z: radius, w: logical units of the edge ramp
varying vec2 v_local;
varying vec4 v_bg;

void main() {
    vec2 device = a_pos_uv.xy * u_view.x + u_view.yz;
    gl_Position = vec4(device.x / u_surface.x * 2.0 - 1.0, 1.0 - device.y / u_surface.y * 2.0, 0.0, 1.0);
    v_uv = a_pos_uv.zw;
    v_color = a_color;
    v_shape = vec4(a_local.zw, a_params.x, u_view.w / u_view.x);
    v_local = a_local.xy;
    v_bg = a_bg;
}
"#;

/// The wallpaper: the full-screen quad of every frame, so its fragment shader is as short as it
/// can be. The dither comes from a 4x4 ordered (Bayer) texture repeated over the screen, read at
/// coordinates the vertex shader computes: no math per pixel, which the TV's GPU could not afford
/// on a million pixels a frame.
pub const WALLPAPER_VERTEX: &str = r#"#version 100
attribute vec4 a_pos_uv; // xy: canvas position, zw: texture coordinates
attribute vec4 a_color;

uniform vec4 u_view;
uniform vec2 u_surface;

varying vec2 v_uv;
varying vec4 v_color;
varying vec2 v_bayer;    // device pixels over the 4 of the dither pattern

void main() {
    vec2 device = a_pos_uv.xy * u_view.x + u_view.yz;
    gl_Position = vec4(device.x / u_surface.x * 2.0 - 1.0, 1.0 - device.y / u_surface.y * 2.0, 0.0, 1.0);
    v_uv = a_pos_uv.zw;
    v_color = a_color;
    v_bayer = device / 4.0;
}
"#;

/// The wallpaper times the vertex color, plus +-0.5 of a color level of ordered dither, so its
/// gradients do not band when dimmed or when the TV raises the contrast.
pub const WALLPAPER_FRAGMENT: &str = r#"#version 100
precision mediump float;

uniform sampler2D u_texture;
uniform sampler2D u_bayer;

varying vec2 v_uv;
varying vec4 v_color;
varying vec2 v_bayer;

void main() {
    gl_FragColor = texture2D(u_texture, v_uv) * v_color;
    gl_FragColor.rgb += (texture2D(u_bayer, v_bayer).r - 0.5) / 255.0;
}
"#;

/// Single-tap wallpaper refraction and an edge-weighted UV displacement for glass controls.
/// It samples the already uploaded wallpaper, avoiding an offscreen render target.
pub const GLASS_FRAGMENT: &str = r#"#version 100
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif

uniform sampler2D u_texture;
varying vec2 v_uv;
varying vec4 v_color;
varying vec4 v_shape;
varying vec2 v_local;
varying vec4 v_bg;

float round_rect_distance(vec2 p, vec2 half_size, float radius) {
    vec2 q = abs(p) - half_size + radius;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - radius;
}

void main() {
    vec2 edge = v_local / max(v_shape.xy, vec2(1.0));
    float strength = smoothstep(0.58, 1.0, max(abs(edge.x), abs(edge.y)));
    vec2 uv = v_uv - edge * strength * vec2(0.0017, 0.0030);
    vec3 refracted = texture2D(u_texture, uv).rgb;
    float inset = 0.5 * v_shape.w;
    float d = round_rect_distance(v_local, v_shape.xy - inset, max(v_shape.z - inset, 0.0));
    float coverage = clamp(0.5 - d / v_shape.w, 0.0, 1.0);
    gl_FragColor = vec4(refracted * v_color.rgb, v_color.a) * coverage;
}
"#;

/// Fills the destination of the wallpaper reduction (a framebuffer of the render buffer's size).
pub const REDUCE_VERTEX: &str = r#"#version 100
attribute vec2 a_pos;

uniform vec2 u_dest;     // destination size in pixels

varying vec2 v_uv;
varying vec2 v_bayer;    // destination pixels over the 4 of the dither pattern

void main() {
    vec2 unit = a_pos * 0.5 + 0.5;
    gl_Position = vec4(a_pos, 0.0, 1.0);
    v_uv = unit;
    v_bayer = unit * u_dest / 4.0;
}
"#;

/// Reduces the wallpaper to the render buffer's size once, when it is loaded: four linear taps spread
/// over the footprint of a destination pixel (a box of the size of the reduction, which is an area
/// filter), then the dither is made again at the final size, before the 8 bits.
pub const REDUCE_FRAGMENT: &str = r#"#version 100
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif

uniform sampler2D u_texture;
uniform sampler2D u_bayer;
uniform vec2 u_step;     // a quarter of a destination pixel, in source texture coordinates

varying vec2 v_uv;
varying vec2 v_bayer;

void main() {
    vec3 color = texture2D(u_texture, v_uv + vec2(-u_step.x, -u_step.y)).rgb
        + texture2D(u_texture, v_uv + vec2(u_step.x, -u_step.y)).rgb
        + texture2D(u_texture, v_uv + vec2(-u_step.x, u_step.y)).rgb
        + texture2D(u_texture, v_uv + vec2(u_step.x, u_step.y)).rgb;
    color = color * 0.25 + (texture2D(u_bayer, v_bayer).r - 0.5) / 255.0;
    gl_FragColor = vec4(color, 1.0);
}
"#;

/// The 4x4 ordered dither thresholds, `(level + 0.5) / 16` as bytes, row by row: the texture
/// [`WALLPAPER_FRAGMENT`] repeats over the screen.
pub const BAYER_4X4: [u8; 16] = {
    const LEVELS: [u8; 16] = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];
    let mut bytes = [0u8; 16];
    let mut i = 0;
    while i < 16 {
        bytes[i] = ((LEVELS[i] as u32 * 2 + 1) * 255 / 32) as u8;
        i += 1;
    }
    bytes
};

/// Glyph coverage in the alpha channel of an 8-bit atlas, tinted by the vertex color.
pub const TEXT_FRAGMENT: &str = r#"#version 100
precision mediump float;

uniform sampler2D u_texture;

varying vec2 v_uv;
varying vec4 v_color;

void main() {
    gl_FragColor = v_color * texture2D(u_texture, v_uv).a;
}
"#;

/// Premultiplied RGBA image multiplied by the vertex color and composed over the vertex background,
/// with optional rounded corner clipping: one antialiased edge for the image and the color under it.
pub const IMAGE_FRAGMENT: &str = r#"#version 100
precision mediump float;

uniform sampler2D u_texture;

varying vec2 v_uv;
varying vec4 v_color;
varying vec4 v_shape;
varying vec2 v_local;
varying vec4 v_bg;

float round_rect_distance(vec2 p, vec2 half_size, float radius) {
    vec2 q = abs(p) - half_size + radius;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - radius;
}

void main() {
    // The coordinates go to the texture untouched: on fp16-only GPUs (the TV's Mali-470) any math
    // on them would lose about a texel of the atlas.
    vec4 tex = texture2D(u_texture, v_uv) * v_color;
    vec4 color = tex + v_bg * (1.0 - tex.a);
    if (v_shape.z > 0.0) {
        // The shape sits half a ramp inside the image, so its whole antialiased edge is drawn
        // within the quad and over pixels of the image itself.
        float inset = 0.5 * v_shape.w;
        float d = round_rect_distance(v_local, v_shape.xy - inset, max(v_shape.z - inset, 0.0));
        color *= clamp(0.5 - d / v_shape.w, 0.0, 1.0);
    }
    gl_FragColor = color;
}
"#;
