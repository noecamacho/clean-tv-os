// The Liquid Glass material of a quick-panel control (docs/design/panel-rapido-lg.md, «Variante
// Liquid Glass»): one small shader draws the rounded shape, the tint, the lens band at the edge,
// the top-lit specular rim, an inner shadow and the depth shadow around it.
//
// Over graphics (`panel.frost` = 1) it shows the panel's shared blurred snapshot of what is
// behind, read at this control's place. Over video (frost 0) the picture is on a hardware plane
// the compositor cannot read, so there is no blur: a darker tint, the rim and the shadows.
// Colours are passed premultiplied by hand (vector4d), not as QColor.
import QtQuick 2.4

ShaderEffect {
    id: glass

    property Item panel             // HomePanel: backdropTexture, backdropRect, frost, night, u
    property bool focused: false    // the white capsule of focus (buttons and selector)
    property real radius: parent.height / 2

    readonly property real u: panel ? panel.u : 1
    readonly property real margin: 16 * u
    readonly property real frost: panel ? panel.frost : 0
    readonly property real rim: panel && panel.night ? 0.55 : 1
    readonly property real shadow: frost > 0 ? 0.18 : 0.34
    readonly property size size: Qt.size(parent.width, parent.height)

    // Tint, premultiplied: 44 % over graphics, a 70 → 80 % gradient over video; focus is white.
    function pre(r, g, b, a) { return Qt.vector4d(r * a, g * a, b * a, a); }
    readonly property vector4d tintTop: focused ? (panel && panel.night ? pre(0.85, 0.87, 0.89, 0.92) : pre(0.965, 0.973, 0.988, 0.94))
                                                : frost > 0 ? pre(0.047, 0.071, 0.11, 0.44) : pre(0.078, 0.10, 0.15, 0.70)
    readonly property vector4d tintBottom: focused ? tintTop : frost > 0 ? tintTop : pre(0.035, 0.047, 0.078, 0.80)

    // Where this control sits in the shared snapshot (normalized). Re-evaluated as the column
    // moves (opening rise) and when focus scales the control.
    readonly property var source: panel && panel.backdropTexture ? panel.backdropTexture : dummy
    readonly property vector4d area: {
        if (!panel || !panel.backdropRect.width) return Qt.vector4d(0, 0, 1, 1);
        var dep = parent.x + parent.y + panel.rise + parent.scale; // dependencies only
        var p = parent.mapToItem(panel, 0, 0);
        var r = panel.backdropRect;
        return Qt.vector4d((p.x - r.x) / r.width, (p.y - r.y) / r.height,
                           parent.width / r.width, parent.height / r.height);
    }

    x: -margin
    y: -margin
    width: parent.width + 2 * margin
    height: parent.height + 2 * margin

    // A 1×1 transparent texture while there is no snapshot (over video, or before the first).
    ShaderEffectSource {
        id: dummy
        hideSource: true
        visible: false
        sourceItem: Item { width: 1; height: 1 }
    }

    fragmentShader: "
        varying highp vec2 qt_TexCoord0;
        uniform sampler2D source;
        uniform highp vec4 area;
        uniform highp vec2 size;
        uniform highp float radius;
        uniform highp float margin;
        uniform highp float frost;
        uniform highp float rim;
        uniform highp float shadow;
        uniform lowp vec4 tintTop;
        uniform lowp vec4 tintBottom;
        uniform lowp float qt_Opacity;
        highp float sdf(highp vec2 p) {
            highp vec2 q = abs(p - size * 0.5) - (size * 0.5 - vec2(radius));
            return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - radius;
        }
        void main() {
            highp vec2 p = qt_TexCoord0 * (size + 2.0 * margin) - vec2(margin);
            highp float d = sdf(p);
            lowp float a = clamp(0.5 - d, 0.0, 1.0);
            highp vec2 c = clamp(p, vec2(radius), size - vec2(radius));
            highp vec2 v = p - c;
            highp vec2 n = v / max(length(v), 0.001);
            highp float edge = clamp(1.0 + d / (0.22 * size.y), 0.0, 1.0);
            highp vec2 t = p / size;
            highp vec2 uv = area.xy + area.zw * (t - n * edge * edge * 0.06);
            lowp vec4 bg = texture2D(source, uv) * frost;
            bg.rgb *= 1.0 + 0.25 * edge * frost;
            lowp vec4 tn = mix(tintTop, tintBottom, clamp(t.y, 0.0, 1.0));
            lowp vec4 col = bg * (1.0 - tn.a) + tn;
            col.rgb += 0.10 * (1.0 - smoothstep(0.0, 0.46, t.y)) * col.a;
            col.rgb *= 1.0 - 0.22 * edge * clamp(n.y, 0.0, 1.0);
            lowp float spec = (1.0 - smoothstep(0.0, 1.6, abs(d + 0.8))) * (0.25 + 0.75 * clamp(-n.y, 0.0, 1.0));
            col += vec4(vec3(0.85 * spec * rim), 0.85 * spec * rim) * (1.0 - col.a * 0.5);
            highp float ds = sdf(p - vec2(0.0, 0.35 * margin));
            lowp float sh = shadow * (1.0 - smoothstep(0.0, margin, ds)) * (1.0 - a);
            gl_FragColor = (col * a + vec4(0.0, 0.0, 0.0, sh)) * qt_Opacity;
        }"
}
