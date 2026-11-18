// The focus halo shared by the panel's controls: a 3-unit ring outside the capsule, so focus
// reads by shape as well as by the white fill (and on bright video). The control itself scales
// to 1.03 (see `focusScale` in each control).
import QtQuick 2.4

Rectangle {
    property real u: 1
    property bool shown: false

    anchors.fill: parent
    anchors.margins: -3 * u
    radius: height / 2
    color: "transparent"
    border.width: 3 * u
    border.color: "#59ffffff"
    opacity: shown ? 1 : 0
    Behavior on opacity { NumberAnimation { duration: 120 } }
}
