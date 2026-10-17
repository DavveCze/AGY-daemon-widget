import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import org.kde.plasma.core as PlasmaCore
import org.kde.plasma.components as PlasmaComponents3

Item {
    id: compactRoot

    readonly property int activeCount: root.agyState?.stats?.total_active_agents ?? 0
    readonly property int busyCount: root.agyState?.stats?.busy_agents_count ?? 0
    readonly property bool isBusy: compactRoot.busyCount > 0
    readonly property bool hasActive: compactRoot.activeCount > 0

    Layout.minimumWidth: Kirigami.Units.iconSizes.small
    Layout.minimumHeight: Kirigami.Units.iconSizes.small
    Layout.preferredWidth: Kirigami.Units.iconSizes.medium
    Layout.preferredHeight: Kirigami.Units.iconSizes.medium

    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            root.expanded = !root.expanded;
        }

        PlasmaComponents3.ToolTip.visible: containsMouse
        PlasmaComponents3.ToolTip.text: {
            if (!root.isDaemonActive) {
                return "Antigravity Monitor\nDaemon starting...";
            }
            if (compactRoot.activeCount === 0) {
                return "Antigravity Monitor\nNo active CLI sessions\nModel: " + (root.agyState?.settings?.model ?? "Default");
            }
            return "Antigravity Monitor\nActive sessions: " + compactRoot.activeCount + " (" + compactRoot.busyCount + " working)\nRAM: " + (root.agyState?.stats?.total_memory_mb ?? 0) + " MB\nModel: " + (root.agyState?.settings?.model ?? "Default");
        }
    }

    Kirigami.Icon {
        id: mainIcon
        anchors.centerIn: parent
        width: Math.min(parent.width, parent.height) * 0.85
        height: width
        source: Qt.resolvedUrl("../assets/antigravity.svg")
        fallback: "utilities-system-monitor"
        opacity: compactRoot.hasActive ? 1.0 : 0.7
    }

    // Status Dot / Badge (Bottom Right)
    Rectangle {
        id: statusDot
        visible: compactRoot.hasActive || !root.isDaemonActive
        width: Math.max(8, parent.width * 0.35)
        height: width
        radius: width / 2
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: 1

        color: {
            if (!root.isDaemonActive) {
                return Kirigami.Theme.neutralTextColor; // Yellow/amber for disconnected
            }
            if (compactRoot.isBusy) {
                return Kirigami.Theme.positiveTextColor; // Green for actively working
            }
            return Kirigami.Theme.focusColor; // Blue for idle/waiting
        }

        border.color: Kirigami.Theme.backgroundColor
        border.width: 1

        // Pulse animation when agent is busy working
        SequentialAnimation on opacity {
            running: compactRoot.isBusy
            loops: Animation.Infinite
            NumberAnimation { to: 0.3; duration: 600; easing.type: Easing.InOutQuad }
            NumberAnimation { to: 1.0; duration: 600; easing.type: Easing.InOutQuad }
        }
    }

    // Count Badge (Top Right when > 1 active)
    Rectangle {
        visible: compactRoot.activeCount > 1
        width: Math.max(14, countText.contentWidth + 6)
        height: 14
        radius: 7
        anchors.top: parent.top
        anchors.right: parent.right
        color: Kirigami.Theme.highlightColor

        Text {
            id: countText
            anchors.centerIn: parent
            text: compactRoot.activeCount.toString()
            font.pixelSize: 9
            font.bold: true
            color: Kirigami.Theme.highlightedTextColor
        }
    }
}
