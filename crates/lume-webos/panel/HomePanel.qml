// HOME OS quick panel, drawn by LG's compositor over any app (see home-panel.js).
//
// Layout from docs/design/panel-rapido-lg.md: HOME's Control Center column (crates/lume-core/src/
// app/control_center.rs) in its 1280x720 units scaled to the compositor (`u`), top to bottom:
// the "Now" card, Brightness, Volume, Picture mode, and a row of round buttons (Night mode,
// Screen off, More settings) with a caption. Over video there is no wallpaper to blur: the glass is a 78 % dark fill and a shade
// behind the column keeps it legible on bright pictures. Nothing is drawn while closed.
import QtQuick 2.4
import QtGraphicalEffects 1.0

Item {
    id: root
    objectName: "homeOsPanel"
    anchors.fill: parent
    z: 9000
    visible: false
    opacity: 0

    // Set by the filter. Focus: 0 brightness, 3 volume, 4 picture mode, and the last row of round
    // buttons 1 night mode, 5 screen off, 6 more settings (the numbers are kept from earlier
    // layouts; 2, the schedule button, was removed: the schedule runs on its own).
    property string dir: ""
    property var strings: ({})
    property int focusIndex: 0
    property int brightness: -1
    property int volume: -1
    property bool autoBrightness: false
    property bool muted: false
    property bool volumeOffline: false
    property string brightnessMessage: ""
    property string pictureMode: ""
    property bool modePending: false
    property int modeIndex: -1
    property int modeCount: 0
    property bool night: false
    property string nightDetail: ""
    property var videoBadges: []
    property var audioBadges: []
    property string audioDevice: ""

    // Liquid Glass: `frost` 1 over graphics (blurred snapshot behind the column), 0 over video
    // (set by the filter from videooutput). `backdropSource` is the compositor's views root, which
    // does not contain this panel, so the snapshot never shows the panel itself.
    property real frost: 0
    property Item backdropSource: null
    readonly property rect backdropRect: Qt.rect(column.x - 24 * u, column.y - 24 * u,
                                                 column.width + 48 * u, column.height + 48 * u)
    readonly property var backdropTexture: frost > 0 ? blurredTexture : null

    property bool shown: false
    readonly property real u: width / 1280
    property real rise: 0

    signal action(string name)
    signal levelPicked(string name, int value)

    function refreshBackdrop() {
        if (frost > 0) backdrop.scheduleUpdate();
    }

    function open() {
        if (frost > 0) backdrop.scheduleUpdate();
        closing.stop();
        shown = true;
        visible = true;
        opening.restart();
    }

    function close() {
        if (!shown) return;
        shown = false;
        opening.stop();
        closing.restart();
    }

    // HOME's OPEN: 240 ms, cubic-bezier(0.2, 0, 0, 1), rising 22 units.
    ParallelAnimation {
        id: opening
        NumberAnimation { target: root; property: "opacity"; to: 1; duration: 240; easing.type: Easing.BezierSpline; easing.bezierCurve: [0.2, 0, 0, 1, 1, 1] }
        NumberAnimation { target: root; property: "rise"; from: 22 * root.u; to: 0; duration: 240; easing.type: Easing.BezierSpline; easing.bezierCurve: [0.2, 0, 0, 1, 1, 1] }
    }
    // HOME's CLOSE: 160 ms, ease-in; then nothing is drawn.
    SequentialAnimation {
        id: closing
        ParallelAnimation {
            NumberAnimation { target: root; property: "opacity"; to: 0; duration: 160; easing.type: Easing.InQuad }
            NumberAnimation { target: root; property: "rise"; to: 22 * root.u; duration: 160; easing.type: Easing.InQuad }
        }
        ScriptAction { script: root.visible = false }
    }

    // What lies behind the column, at half resolution, taken only when asked (on opening and
    // once a second while open over graphics), blurred once and shared by every control.
    ShaderEffectSource {
        id: backdrop
        sourceItem: root.frost > 0 ? root.backdropSource : null
        sourceRect: root.backdropRect
        textureSize: Qt.size(Math.max(1, root.backdropRect.width / 2), Math.max(1, root.backdropRect.height / 2))
        live: false
        recursive: false
        visible: false
    }
    FastBlur {
        id: blurred
        width: Math.max(1, root.backdropRect.width / 2)
        height: Math.max(1, root.backdropRect.height / 2)
        source: backdrop
        radius: 32
        cached: true
        visible: false
    }
    ShaderEffectSource {
        id: blurredTexture
        sourceItem: blurred
        hideSource: true
        visible: false
    }
    Timer {
        interval: 1000
        repeat: true
        running: root.shown && root.frost > 0
        onTriggered: backdrop.scheduleUpdate()
    }

    FontLoader { id: textFont; source: root.dir ? root.dir + "/PlusJakartaSans-Medium.ttf" : "" }
    FontLoader { id: boldFont; source: root.dir ? root.dir + "/PlusJakartaSans-Bold.ttf" : "" }
    FontLoader { id: iconFont; source: root.dir ? root.dir + "/lucide-subset.ttf" : "" }

    // A click outside the controls closes the panel, as Back does.
    MouseArea {
        anchors.fill: parent
        onClicked: root.action("close")
    }

    Column {
        id: column
        width: 456 * root.u
        x: root.width - 54 * root.u - width
        y: 92 * root.u - root.rise
        spacing: 10 * root.u

        readonly property real half: (width - 10 * root.u) / 2

        PanelFormats {
            width: parent.width
            u: root.u
            panel: root
            videoBadges: root.videoBadges
            videoMode: "" // the mode has its own control below
            audioBadges: root.audioBadges
            audioDevice: root.audioDevice
            textFamily: textFont.name
            boldFamily: boldFont.name
            iconFamily: iconFont.name
        }

        PanelSlider {
            width: parent.width
            u: root.u
            panel: root
            icon: ""
            label: root.strings.brightness || ""
            value: root.brightness
            prefix: root.night ? (root.strings.nightPrefix || "") : root.autoBrightness ? "Auto · " : ""
            message: root.brightnessMessage
            focused: root.focusIndex === 0
            night: root.night
            textFamily: textFont.name
            iconFamily: iconFont.name
            onPicked: { root.focusIndex = 0; root.levelPicked("brightness", level); }
            onHovered: { root.focusIndex = 0; root.action("touch"); }
        }

        PanelSlider {
            width: parent.width
            u: root.u
            panel: root
            icon: ""
            label: root.strings.volume || ""
            value: root.volume
            muted: root.muted
            mutedText: root.strings.muted || ""
            offline: root.volumeOffline
            offlineText: root.strings.offline || ""
            focused: root.focusIndex === 3
            night: root.night
            textFamily: textFont.name
            iconFamily: iconFont.name
            onPicked: { root.focusIndex = 3; root.levelPicked("volume", level); }
            onHovered: { root.focusIndex = 3; root.action("touch"); }
        }

        PanelChoice {
            width: parent.width
            u: root.u
            panel: root
            icon: ""
            label: root.strings.pictureMode || ""
            value: root.pictureMode
            pending: root.modePending
            index: root.modeIndex
            count: root.modeCount
            focused: root.focusIndex === 4
            night: root.night
            textFamily: textFont.name
            iconFamily: iconFont.name
            onStep: { root.focusIndex = 4; root.action(direction > 0 ? "modeNext" : "modePrevious"); }
            onHovered: { root.focusIndex = 4; root.action("touch"); }
        }

        // Night mode, Screen off and More settings as round buttons, with a caption naming the
        // focused one and its state («Modo noche · Activado»).
        Row {
            spacing: 10 * root.u

            PanelRound {
                u: root.u
                panel: root
                icon: ""
                on: root.night
                focused: root.focusIndex === 1
                night: root.night
                iconFamily: iconFont.name
                onClicked: root.action("night")
                onHovered: { root.focusIndex = 1; root.action("touch"); }
            }
            PanelRound {
                u: root.u
                panel: root
                icon: ""
                focused: root.focusIndex === 5
                night: root.night
                iconFamily: iconFont.name
                onClicked: root.action("screenOff")
                onHovered: { root.focusIndex = 5; root.action("touch"); }
            }
            PanelRound {
                u: root.u
                panel: root
                icon: ""
                focused: root.focusIndex === 6
                night: root.night
                iconFamily: iconFont.name
                onClicked: root.action("settings")
                onHovered: { root.focusIndex = 6; root.action("touch"); }
            }

            Item { width: 6 * root.u; height: 1 } // QtQuick 2.4 has no Row/Column padding

            // The caption names the focused round button (and the night state); it only shows
            // while one of them has the focus or the pointer, sliding in from the buttons.
            Column {
                id: caption
                anchors.verticalCenter: parent.verticalCenter
                spacing: 2 * root.u
                readonly property bool active: root.focusIndex === 1 || root.focusIndex === 5 || root.focusIndex === 6
                property int which: 1
                opacity: active ? 1 : 0
                transform: Translate { x: caption.active ? 0 : -8 * root.u
                    Behavior on x { NumberAnimation { duration: 160; easing.type: Easing.BezierSpline; easing.bezierCurve: [0.2, 0, 0, 1, 1, 1] } } }
                Behavior on opacity { NumberAnimation { duration: 160 } }
                // Keep the last name while fading out, so the text does not jump as it goes.
                Connections {
                    target: root
                    onFocusIndexChanged: if (caption.active) caption.which = root.focusIndex
                }
                Text {
                    text: caption.which === 5 ? (root.strings.screenOff || "") : caption.which === 6 ? (root.strings.settings || "") : (root.strings.night || "")
                    font.family: textFont.name
                    font.pixelSize: 17 * root.u
                    color: "#ffffff"
                }
                Text {
                    visible: caption.which === 1
                    text: root.nightDetail
                    font.family: textFont.name
                    font.pixelSize: 15 * root.u
                    color: "#c7ffffff"
                }
            }
        }
    }
}
