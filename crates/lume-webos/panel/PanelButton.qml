// A button of the quick panel, as HOME's tiles: a glyph in a disc, a label and an optional second
// line. "On" and "focus" never share the white: focus is the white capsule; on is a filled disc
// (white on the glass, ink on the focused capsule) and the word in `detail`.
import QtQuick 2.4

Item {
    id: button

    property real u: 1
    property string icon: ""
    property string label: ""
    property string detail: "" // a second, softer line ("Activado", "22:00–07:00")
    property bool on: false
    property bool focused: false
    property bool night: false // the focused capsule dims at night
    property string textFamily: ""
    property string iconFamily: ""

    signal clicked()
    signal hovered()

    readonly property color ink: focused ? "#101620" : "#ffffff"

    height: 69 * u
    scale: focused ? 1.03 : 1
    Behavior on scale { NumberAnimation { duration: 120; easing.type: Easing.BezierSpline; easing.bezierCurve: [0.2, 0, 0, 1, 1, 1] } }

    PanelFocus { u: button.u; shown: button.focused }

    Rectangle {
        anchors.fill: parent
        radius: height / 2
        color: button.focused ? (button.night ? "#d9dfe4ea" : "#f0f6f8fc") : "#c70e141e"
        border.width: 1
        border.color: "#50ffffff"
        Behavior on color { ColorAnimation { duration: 120 } }
    }

    Rectangle {
        x: 12 * button.u
        y: (button.height - height) / 2
        width: 44 * button.u
        height: width
        radius: width / 2
        color: button.on ? (button.focused ? "#101620" : "#f0f6f8fc")
                         : (button.focused ? "#1a101620" : "#26ffffff")
        Behavior on color { ColorAnimation { duration: 120 } }
    }

    Text {
        x: 12 * button.u
        width: 44 * button.u
        height: button.height
        text: button.icon
        font.family: button.iconFamily
        font.pixelSize: 22 * button.u
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        // Ink on a white disc, white on an ink disc.
        color: button.on ? (button.focused ? "#ffffff" : "#101620") : button.ink
    }

    Text {
        x: 66 * button.u
        width: button.width - x - 14 * button.u
        height: button.detail ? button.height * 0.56 : button.height
        text: button.label
        elide: Text.ElideRight
        font.family: button.textFamily
        font.pixelSize: 17 * button.u
        verticalAlignment: button.detail ? Text.AlignBottom : Text.AlignVCenter
        color: button.ink
    }

    Text {
        visible: button.detail !== ""
        x: 66 * button.u
        y: button.height * 0.56
        width: button.width - x - 14 * button.u
        text: button.detail
        elide: Text.ElideRight
        font.family: button.textFamily
        font.pixelSize: 15 * button.u
        color: button.focused ? "#b3101620" : "#c7ffffff"
    }

    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        onEntered: button.hovered()
        onClicked: button.clicked()
    }
}
