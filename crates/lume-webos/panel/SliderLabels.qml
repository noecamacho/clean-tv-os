// The glyph, name and value of a slider in one colour. PanelSlider draws them twice: white over
// the glass and in ink inside the level's clip, so each letter takes the colour of what it is on.
import QtQuick 2.4

Item {
    id: labels

    property real u: 1
    property color color: "#ffffff"
    property string icon: ""
    property string label: ""
    property string value: ""
    property string textFamily: ""
    property string iconFamily: ""

    Text {
        x: 14 * labels.u
        width: 40 * labels.u
        height: labels.height
        text: labels.icon
        font.family: labels.iconFamily
        font.pixelSize: 24 * labels.u
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        color: labels.color
    }

    Text {
        x: 58 * labels.u
        height: labels.height
        text: labels.label
        font.family: labels.textFamily
        font.pixelSize: 19 * labels.u
        verticalAlignment: Text.AlignVCenter
        color: labels.color
    }

    Text {
        anchors.right: parent.right
        anchors.rightMargin: 20 * labels.u
        height: labels.height
        text: labels.value
        font.family: labels.textFamily
        font.pixelSize: 19 * labels.u
        verticalAlignment: Text.AlignVCenter
        color: labels.color
    }
}
