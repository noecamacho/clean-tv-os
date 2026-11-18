// The picture-mode selector: glyph, name, and `‹ value ›` with position dots under it. Left/Right
// come from the filter (no wrap-around: the chevron of an end dims to 30 %). While a choice waits
// to be applied, the value is dimmed. The pointer clicks a chevron or turns the wheel.
import QtQuick 2.4

Item {
    id: choice

    property Item panel // HomePanel, for the shared glass (PanelGlass)
    property real u: 1
    property string icon: ""
    property string label: ""
    property string value: ""
    property bool pending: false
    property int index: -1 // position in the list, -1 while unknown
    property int count: 0
    property bool focused: false
    property bool night: false
    property string textFamily: ""
    property string iconFamily: ""

    signal step(int direction)
    signal hovered()

    readonly property color ink: focused ? "#101620" : "#ffffff"

    height: 69 * u
    scale: focused ? 1.03 : 1
    Behavior on scale { NumberAnimation { duration: 120; easing.type: Easing.BezierSpline; easing.bezierCurve: [0.2, 0, 0, 1, 1, 1] } }

    PanelFocus { u: choice.u; shown: choice.focused }

    PanelGlass {
        panel: choice.panel
        focused: choice.focused
    }

    Text {
        x: 14 * choice.u
        width: 40 * choice.u
        height: choice.height
        text: choice.icon
        font.family: choice.iconFamily
        font.pixelSize: 24 * choice.u
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        color: choice.ink
    }

    Text {
        x: 58 * choice.u
        height: choice.height
        text: choice.label
        font.family: choice.textFamily
        font.pixelSize: 19 * choice.u
        verticalAlignment: Text.AlignVCenter
        color: choice.ink
    }

    Item {
        id: picker
        anchors.right: parent.right
        anchors.rightMargin: 12 * choice.u
        width: row.width
        height: choice.height

        Row {
            id: row
            y: (choice.height - 26 * choice.u) / 2 - (choice.count > 1 ? 4 * choice.u : 0)
            height: 26 * choice.u
            spacing: 6 * choice.u

            Text {
                height: parent.height
                text: ""
                font.family: choice.iconFamily
                font.pixelSize: 20 * choice.u
                verticalAlignment: Text.AlignVCenter
                color: choice.ink
                opacity: choice.index <= 0 ? 0.3 : 0.8
                MouseArea { anchors.fill: parent; anchors.margins: -12 * choice.u; onClicked: choice.step(-1) }
            }
            Text {
                height: parent.height
                // The full name at once; while it waits to be applied it is only dimmed (an
                // ellipsis read as a cut-off name).
                text: choice.value || "…"
                font.family: choice.textFamily
                font.pixelSize: 19 * choice.u
                verticalAlignment: Text.AlignVCenter
                color: choice.ink
                opacity: choice.pending ? 0.6 : 1
                Behavior on opacity { NumberAnimation { duration: 120 } }
            }
            Text {
                height: parent.height
                text: ""
                font.family: choice.iconFamily
                font.pixelSize: 20 * choice.u
                verticalAlignment: Text.AlignVCenter
                color: choice.ink
                opacity: choice.index < 0 || choice.index >= choice.count - 1 ? 0.3 : 0.8
                MouseArea { anchors.fill: parent; anchors.margins: -12 * choice.u; onClicked: choice.step(1) }
            }
        }

        // Where the mode is in the list ("3 of 6"), as dots under the value.
        Row {
            visible: choice.count > 1
            anchors.horizontalCenter: row.horizontalCenter
            y: row.y + row.height + 5 * choice.u
            spacing: 4 * choice.u
            Repeater {
                model: choice.count
                Rectangle {
                    width: index === choice.index ? 10 * choice.u : 4 * choice.u
                    height: 4 * choice.u
                    radius: 2 * choice.u
                    color: choice.ink
                    opacity: index === choice.index ? 0.9 : 0.35
                    Behavior on width { NumberAnimation { duration: 120 } }
                }
            }
        }
    }

    MouseArea {
        anchors.fill: parent
        z: -1
        hoverEnabled: true
        onEntered: choice.hovered()
        onWheel: choice.step(wheel.angleDelta.y > 0 ? 1 : -1)
    }
}
