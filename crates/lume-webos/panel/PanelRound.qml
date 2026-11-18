// A round button of the quick panel's last row (Night mode, Screen off, More settings), as
// HOME's round controls: a 69-unit glass disc with a glyph. Focus is the white disc with the
// halo; "on" is a filled inner disc (white on the glass, ink on the focused white) — never the
// white of focus. Its name is shown by the row's caption, not here.
import QtQuick 2.4

Item {
    id: round

    property Item panel // HomePanel, for the shared glass (PanelGlass)
    property real u: 1
    property string icon: ""
    property bool on: false
    property bool focused: false
    property bool night: false
    property string iconFamily: ""

    signal clicked()
    signal hovered()

    width: 69 * u
    height: 69 * u
    scale: focused ? 1.03 : 1
    Behavior on scale { NumberAnimation { duration: 120; easing.type: Easing.BezierSpline; easing.bezierCurve: [0.2, 0, 0, 1, 1, 1] } }

    PanelFocus { u: round.u; shown: round.focused }

    PanelGlass {
        panel: round.panel
        focused: round.focused
    }

    Rectangle {
        visible: round.on
        anchors.centerIn: parent
        width: 50 * round.u
        height: width
        radius: width / 2
        color: round.focused ? "#101620" : "#f0f6f8fc"
    }

    Text {
        anchors.fill: parent
        text: round.icon
        font.family: round.iconFamily
        font.pixelSize: 24 * round.u
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        color: round.on ? (round.focused ? "#ffffff" : "#101620") : (round.focused ? "#101620" : "#ffffff")
    }

    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        onEntered: round.hovered()
        onClicked: round.clicked()
    }
}
