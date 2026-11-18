// "Now" card of the quick panel: what the picture and the sound are, as format badges in the
// style of the Apple TV app. Premium formats (Dolby Vision, Atmos) are filled; the rest outlined.
// The picture mode and the sound device sit at the right in a softer white.
import QtQuick 2.4

Item {
    id: card

    property Item panel // HomePanel, for the shared glass (PanelGlass)
    property real u: 1
    property var videoBadges: []
    property string videoMode: ""
    property var audioBadges: []
    property string audioDevice: ""
    property string textFamily: ""
    property string boldFamily: ""
    property string iconFamily: ""

    height: 112 * u

    PanelGlass {
        panel: card.panel
        focused: false
        radius: 34 * card.u
    }

    Column {
        x: 20 * card.u
        width: card.width - 40 * card.u
        anchors.verticalCenter: parent.verticalCenter
        spacing: 14 * card.u

        Repeater {
            model: [
                { icon: "\ue09d", badges: card.videoBadges, side: card.videoMode },
                { icon: "\ue1ab", badges: card.audioBadges, side: card.audioDevice }
            ]

            Item {
                width: parent.width
                height: 30 * card.u

                Text {
                    width: 26 * card.u
                    height: parent.height
                    text: modelData.icon
                    font.family: card.iconFamily
                    font.pixelSize: 20 * card.u
                    verticalAlignment: Text.AlignVCenter
                    color: "#ffffff"
                }

                Row {
                    x: 38 * card.u
                    height: parent.height
                    spacing: 6 * card.u

                    Repeater {
                        model: modelData.badges

                        Rectangle {
                            readonly property bool premium: /Dolby Vision|Atmos/.test(modelData)
                            anchors.verticalCenter: parent.verticalCenter
                            width: badge.implicitWidth + 16 * card.u
                            height: 24 * card.u
                            radius: 6 * card.u
                            color: premium ? "#f0f6f8fc" : "transparent"
                            border.width: premium ? 0 : 1.5
                            border.color: "#b3ffffff"

                            Text {
                                id: badge
                                anchors.centerIn: parent
                                // Formats in small caps; a frame rate keeps its usual lowercase "24p".
                                text: /^\d+p$/.test(modelData) ? String(modelData) : String(modelData).toUpperCase()
                                font.family: card.boldFamily
                                font.pixelSize: 12 * card.u
                                font.letterSpacing: 0.6 * card.u
                                color: parent.premium ? "#101620" : "#ffffff"
                            }
                        }
                    }

                    // Nothing known yet: a quiet dash rather than an empty line.
                    Text {
                        visible: modelData.badges.length === 0
                        anchors.verticalCenter: parent.verticalCenter
                        text: "—"
                        font.family: card.textFamily
                        font.pixelSize: 15 * card.u
                        color: "#80ffffff"
                    }
                }

                Text {
                    anchors.right: parent.right
                    height: parent.height
                    text: modelData.side
                    font.family: card.textFamily
                    font.pixelSize: 15 * card.u
                    verticalAlignment: Text.AlignVCenter
                    color: "#b3ffffff"
                }
            }
        }
    }
}
