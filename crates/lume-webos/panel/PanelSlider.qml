// A level slider of the quick panel, as HOME draws them: the glass, the level filled over it,
// and what lies on the level in dark ink. Left/Right come from the filter; the pointer clicks
// or drags to a level.
import QtQuick 2.4

Item {
    id: slider

    property Item panel // HomePanel, for the shared glass (PanelGlass)
    property real u: 1
    property string icon: ""
    property string label: ""
    property int value: -1 // -1 until the TV answers
    property string prefix: "" // "Auto · " while the light sensor drives it
    property bool muted: false // shows `mutedText` and a dimmed level instead of the value
    property string mutedText: ""
    property string message: "" // replaces the value for a moment ("Primero desactiva Modo noche")
    property bool offline: false // the device does not answer: "Sin conexión", dimmed level
    property string offlineText: ""
    property bool night: false
    property bool focused: false
    property string textFamily: ""
    property string iconFamily: ""

    signal picked(int level)
    signal hovered()

    readonly property real fill: value < 0 ? 0 : value / 100
    readonly property color ink: focused ? "#101620" : "#ffffff"

    height: 69 * u
    scale: focused ? 1.03 : 1
    Behavior on scale { NumberAnimation { duration: 120; easing.type: Easing.BezierSpline; easing.bezierCurve: [0.2, 0, 0, 1, 1, 1] } }

    PanelFocus { u: slider.u; shown: slider.focused }

    // Glass. HOME's slider glass never turns white; the level does.
    PanelGlass {
        panel: slider.panel
        focused: false
    }

    readonly property string shown: message !== "" ? message : offline ? offlineText : muted ? mutedText : value < 0 ? "…" : prefix + value + " %"

    // White over the glass…
    SliderLabels {
        width: slider.width
        height: slider.height
        u: slider.u
        icon: slider.icon
        label: slider.label
        value: slider.shown
        textFamily: slider.textFamily
        iconFamily: slider.iconFamily
    }

    // …and the level, cut straight where it ends, with the same labels in ink inside it.
    Item {
        width: slider.width * slider.fill
        height: slider.height
        clip: true
        opacity: slider.muted || slider.offline ? 0.35 : 1
        Behavior on width { NumberAnimation { duration: 120; easing.type: Easing.BezierSpline; easing.bezierCurve: [0.2, 0, 0, 1, 1, 1] } }
        Rectangle {
            width: slider.width
            height: slider.height
            radius: height / 2
            color: slider.focused ? (slider.night ? "#d9dfe4ea" : "#f0f6f8fc") : "#46ffffff"
            Behavior on color { ColorAnimation { duration: 120 } }
        }
        SliderLabels {
            width: slider.width
            height: slider.height
            u: slider.u
            color: slider.ink
            icon: slider.icon
            label: slider.label
            value: slider.shown
            textFamily: slider.textFamily
            iconFamily: slider.iconFamily
        }
    }

    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        onEntered: slider.hovered()
        onPressed: slider.picked(Math.round(100 * Math.max(0, Math.min(1, mouse.x / width))))
        onPositionChanged: if (pressed) slider.picked(Math.round(100 * Math.max(0, Math.min(1, mouse.x / width))))
        onWheel: if (slider.value >= 0) slider.picked(Math.max(0, Math.min(100, slider.value + (wheel.angleDelta.y > 0 ? 5 : -5))))
    }
}
