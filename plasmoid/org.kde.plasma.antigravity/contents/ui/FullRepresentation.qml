import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import org.kde.plasma.core as PlasmaCore
import org.kde.plasma.components as PlasmaComponents3
import "components" as Components

Item {
    id: fullRoot

    Layout.minimumWidth: 500
    Layout.minimumHeight: 640
    Layout.preferredWidth: 530
    Layout.preferredHeight: 690

    readonly property var stateData: root.agyState
    readonly property var activeSessions: stateData?.active_sessions ?? []
    readonly property var recentConversations: stateData?.recent_conversations ?? []
    readonly property var settingsData: stateData?.settings ?? ({})
    readonly property var skillsList: stateData?.skills ?? []
    readonly property var modelsList: stateData?.available_models ?? []
    readonly property var agentsList: stateData?.available_agents ?? []
    readonly property var realQuota: stateData?.real_quota ?? stateData?.stats?.real_quota ?? ({})
    readonly property color borderColor: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.15)

    property var expandedPids: ({})
    property var expandedSkills: ({})
    property string selectedSkillCategory: "all"

    function isSessionExpanded(pid) {
        return !!expandedPids[pid];
    }

    function toggleSessionExpanded(pid) {
        var map = Object.assign({}, expandedPids);
        if (map[pid]) {
            delete map[pid];
        } else {
            map[pid] = true;
        }
        expandedPids = map;
    }

    function isSkillExpanded(name) {
        return !!expandedSkills[name];
    }

    function toggleSkillExpanded(name) {
        var map = Object.assign({}, expandedSkills);
        if (map[name]) {
            delete map[name];
        } else {
            map[name] = true;
        }
        expandedSkills = map;
    }

    function getPluginBadgeColor(pName, sType) {
        if (sType === "user") return "#64748b";
        if (sType === "builtin") return Kirigami.Theme.highlightColor;
        var p = (pName || "").toLowerCase();
        if (p.indexOf("database") !== -1) return "#2563eb";
        if (p.indexOf("ux") !== -1) return "#9333ea";
        if (p.indexOf("seo") !== -1) return "#ea580c";
        if (p.indexOf("architect") !== -1) return "#059669";
        if (p.indexOf("reuse") !== -1) return "#d97706";
        if (p.indexOf("freelancer") !== -1) return "#0891b2";
        return Kirigami.Theme.highlightColor;
    }

    function getSkillIcon(pName, sType) {
        if (sType === "user") return "user-bookmarks";
        if (sType === "builtin") return "system-run";
        var p = (pName || "").toLowerCase();
        if (p.indexOf("database") !== -1) return "server-database";
        if (p.indexOf("ux") !== -1) return "preferences-desktop-theme";
        if (p.indexOf("seo") !== -1) return "globe";
        if (p.indexOf("architect") !== -1) return "draw-polygon";
        if (p.indexOf("reuse") !== -1) return "view-refresh";
        return "plugins";
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Kirigami.Units.smallSpacing
        spacing: Kirigami.Units.smallSpacing

        // --- TOP HEADER ---
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 64
            radius: Kirigami.Units.smallSpacing
            color: Kirigami.Theme.backgroundColor
            border.color: fullRoot.borderColor
            border.width: 1

            RowLayout {
                anchors.fill: parent
                anchors.margins: Kirigami.Units.mediumSpacing
                spacing: Kirigami.Units.mediumSpacing

                Kirigami.Icon {
                    source: Qt.resolvedUrl("../assets/antigravity.svg")
                    fallback: "utilities-system-monitor"
                    Layout.preferredWidth: 36
                    Layout.preferredHeight: 36
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 2

                    RowLayout {
                        spacing: Kirigami.Units.smallSpacing
                        Text {
                            text: "Antigravity CLI Monitor"
                            font.bold: true
                            font.pixelSize: 14
                            color: Kirigami.Theme.textColor
                        }
                        Rectangle {
                            width: 8
                            height: 8
                            radius: 4
                            color: root.isDaemonActive ? Kirigami.Theme.positiveTextColor : Kirigami.Theme.neutralTextColor
                        }
                    }

                    Text {
                        text: root.isDaemonActive
                            ? (activeSessions.length + " sezení | " + (stateData?.stats?.total_memory_mb ?? 0) + " MB RAM | Výchozí: " + (settingsData.model ?? "N/A"))
                            : "Démon se připojuje k /dev/shm..."
                        font.pixelSize: 11
                        color: Kirigami.Theme.disabledTextColor
                        elide: Text.ElideRight
                        Layout.fillWidth: true
                    }
                }

                PlasmaComponents3.Button {
                    icon.name: "view-refresh"
                    display: PlasmaComponents3.AbstractButton.IconOnly
                    PlasmaComponents3.ToolTip.text: "Obnovit stav"
                    onClicked: root.reloadState()
                }
            }
        }

        // --- ACCOUNT & REAL AGY QUOTA DASHBOARD (GEMINI & CLAUDE/GPT) ---
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 168
            radius: Kirigami.Units.smallSpacing
            color: Kirigami.Theme.alternateBackgroundColor
            border.color: fullRoot.borderColor
            border.width: 1

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: Kirigami.Units.smallSpacing
                spacing: 6

                // Account & Plan Header
                RowLayout {
                    Layout.fillWidth: true

                    Kirigami.Icon {
                        source: "user-identity"
                        Layout.preferredWidth: 16
                        Layout.preferredHeight: 16
                    }

                    Text {
                        text: fullRoot.realQuota?.account_email || stateData?.stats?.account_email || "davca.faul@gmail.com"
                        font.pixelSize: 11
                        font.bold: true
                        color: Kirigami.Theme.textColor
                        elide: Text.ElideMiddle
                        Layout.fillWidth: true
                    }

                    Rectangle {
                        radius: 3
                        color: Kirigami.Theme.highlightColor
                        Layout.preferredWidth: planText.contentWidth + 12
                        Layout.preferredHeight: 20

                        Text {
                            id: planText
                            anchors.centerIn: parent
                            text: fullRoot.realQuota?.plan_name || stateData?.stats?.plan_name || "Google AI Pro"
                            font.pixelSize: 10
                            font.bold: true
                            color: Kirigami.Theme.highlightedTextColor
                        }
                    }
                }

                // Two Distinct Cards: Gemini Core & Claude/GPT Outsource
                RowLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    spacing: Kirigami.Units.smallSpacing

                    // Card 1: Gemini Models (Core)
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        radius: Kirigami.Units.smallSpacing
                        color: Kirigami.Theme.backgroundColor
                        border.color: fullRoot.borderColor
                        border.width: 1

                        ColumnLayout {
                            anchors.fill: parent
                            anchors.margins: 8
                            spacing: 6

                            RowLayout {
                                Layout.fillWidth: true
                                Text {
                                    text: "GEMINI CORE"
                                    font.bold: true
                                    font.pixelSize: 10
                                    color: Kirigami.Theme.highlightColor
                                    Layout.fillWidth: true
                                }
                            }

                            // 5-Hour Limit
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 2

                                RowLayout {
                                    Layout.fillWidth: true
                                    Text {
                                        text: "Five Hour Limit"
                                        font.pixelSize: 10
                                        color: Kirigami.Theme.textColor
                                    }
                                    Item { Layout.fillWidth: true }
                                    Text {
                                        text: Math.round(fullRoot.realQuota?.gemini_five_hour?.percentage ?? stateData?.stats?.session_usage_percent ?? 83) + "%"
                                        font.bold: true
                                        font.pixelSize: 10
                                        color: (fullRoot.realQuota?.gemini_five_hour?.percentage ?? 83) < 30 ? Kirigami.Theme.negativeTextColor : Kirigami.Theme.positiveTextColor
                                    }
                                }

                                Rectangle {
                                    Layout.fillWidth: true
                                    height: 5
                                    radius: 2.5
                                    color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.12)
                                    Rectangle {
                                        width: Math.max(2, parent.width * ((fullRoot.realQuota?.gemini_five_hour?.percentage ?? 83) / 100))
                                        height: parent.height
                                        radius: 2.5
                                        color: (fullRoot.realQuota?.gemini_five_hour?.percentage ?? 83) < 30 ? Kirigami.Theme.negativeTextColor : Kirigami.Theme.positiveTextColor
                                    }
                                }

                                Text {
                                    text: fullRoot.realQuota?.gemini_five_hour?.refreshes_in_str || stateData?.stats?.session_reset_text || "Refreshes soon"
                                    font.pixelSize: 9
                                    color: Kirigami.Theme.disabledTextColor
                                    elide: Text.ElideRight
                                    Layout.fillWidth: true
                                }
                            }

                            // Weekly Limit
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 2

                                RowLayout {
                                    Layout.fillWidth: true
                                    Text {
                                        text: "Weekly Limit"
                                        font.pixelSize: 10
                                        color: Kirigami.Theme.textColor
                                    }
                                    Item { Layout.fillWidth: true }
                                    Text {
                                        text: Math.round(fullRoot.realQuota?.gemini_weekly?.percentage ?? stateData?.stats?.weekly_usage_percent ?? 95) + "%"
                                        font.bold: true
                                        font.pixelSize: 10
                                        color: Kirigami.Theme.highlightColor
                                    }
                                }

                                Rectangle {
                                    Layout.fillWidth: true
                                    height: 5
                                    radius: 2.5
                                    color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.12)
                                    Rectangle {
                                        width: Math.max(2, parent.width * ((fullRoot.realQuota?.gemini_weekly?.percentage ?? 95) / 100))
                                        height: parent.height
                                        radius: 2.5
                                        color: Kirigami.Theme.highlightColor
                                    }
                                }

                                Text {
                                    text: fullRoot.realQuota?.gemini_weekly?.refreshes_in_str || stateData?.stats?.weekly_reset_text || "Refreshes in 166h"
                                    font.pixelSize: 9
                                    color: Kirigami.Theme.disabledTextColor
                                    elide: Text.ElideRight
                                    Layout.fillWidth: true
                                }
                            }
                        }
                    }

                    // Card 2: Claude & GPT Models (Outsource)
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        radius: Kirigami.Units.smallSpacing
                        color: Kirigami.Theme.backgroundColor
                        border.color: fullRoot.borderColor
                        border.width: 1

                        ColumnLayout {
                            anchors.fill: parent
                            anchors.margins: 8
                            spacing: 6

                            RowLayout {
                                Layout.fillWidth: true
                                Text {
                                    text: "CLAUDE & GPT"
                                    font.bold: true
                                    font.pixelSize: 10
                                    color: "#f97316"
                                    Layout.fillWidth: true
                                }
                            }

                            // 5-Hour Limit
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 2

                                RowLayout {
                                    Layout.fillWidth: true
                                    Text {
                                        text: "Five Hour Limit"
                                        font.pixelSize: 10
                                        color: Kirigami.Theme.textColor
                                    }
                                    Item { Layout.fillWidth: true }
                                    Text {
                                        text: Math.round(fullRoot.realQuota?.outsource_five_hour?.percentage ?? 100) + "%"
                                        font.bold: true
                                        font.pixelSize: 10
                                        color: Kirigami.Theme.positiveTextColor
                                    }
                                }

                                Rectangle {
                                    Layout.fillWidth: true
                                    height: 5
                                    radius: 2.5
                                    color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.12)
                                    Rectangle {
                                        width: Math.max(2, parent.width * ((fullRoot.realQuota?.outsource_five_hour?.percentage ?? 100) / 100))
                                        height: parent.height
                                        radius: 2.5
                                        color: Kirigami.Theme.positiveTextColor
                                    }
                                }

                                Text {
                                    text: fullRoot.realQuota?.outsource_five_hour?.refreshes_in_str || "Quota available"
                                    font.pixelSize: 9
                                    color: Kirigami.Theme.disabledTextColor
                                    elide: Text.ElideRight
                                    Layout.fillWidth: true
                                }
                            }

                            // Weekly Limit
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 2

                                RowLayout {
                                    Layout.fillWidth: true
                                    Text {
                                        text: "Weekly Limit"
                                        font.pixelSize: 10
                                        color: Kirigami.Theme.textColor
                                    }
                                    Item { Layout.fillWidth: true }
                                    Text {
                                        text: Math.round(fullRoot.realQuota?.outsource_weekly?.percentage ?? stateData?.stats?.outsource_usage_percent ?? 100) + "%"
                                        font.bold: true
                                        font.pixelSize: 10
                                        color: "#f97316"
                                    }
                                }

                                Rectangle {
                                    Layout.fillWidth: true
                                    height: 5
                                    radius: 2.5
                                    color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.12)
                                    Rectangle {
                                        width: Math.max(2, parent.width * ((fullRoot.realQuota?.outsource_weekly?.percentage ?? 100) / 100))
                                        height: parent.height
                                        radius: 2.5
                                        color: "#f97316"
                                    }
                                }

                                Text {
                                    text: fullRoot.realQuota?.outsource_weekly?.refreshes_in_str || stateData?.stats?.outsource_reset_text || "Quota available"
                                    font.pixelSize: 9
                                    color: Kirigami.Theme.disabledTextColor
                                    elide: Text.ElideRight
                                    Layout.fillWidth: true
                                }
                            }
                        }
                    }
                }
            }
        }

        // --- NAVIGATION TABS ---
        PlasmaComponents3.TabBar {
            id: tabBar
            Layout.fillWidth: true

            PlasmaComponents3.TabButton {
                text: "Aktivní (" + activeSessions.length + ")"
            }
            PlasmaComponents3.TabButton {
                text: "Historie"
            }
            PlasmaComponents3.TabButton {
                text: "Nastavení"
            }
            PlasmaComponents3.TabButton {
                text: "Skilly (" + skillsList.length + ")"
            }
        }

        // --- TAB CONTENT STACK ---
        StackLayout {
            id: contentStack
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: tabBar.currentIndex

            // ==========================================
            // TAB 0: AKTIVNÍ SEZENÍ (EXPANDABLE DETAIL)
            // ==========================================
            Item {
                Layout.fillWidth: true
                Layout.fillHeight: true

                // Empty state
                ColumnLayout {
                    anchors.centerIn: parent
                    visible: activeSessions.length === 0
                    spacing: Kirigami.Units.largeSpacing

                    Kirigami.Icon {
                        source: "dialog-information"
                        Layout.alignment: Qt.AlignHCenter
                        Layout.preferredWidth: 48
                        Layout.preferredHeight: 48
                        opacity: 0.5
                    }

                    Text {
                        text: "Žádný agent ani CLI sezení právě neběží"
                        font.pixelSize: 13
                        color: Kirigami.Theme.disabledTextColor
                        Layout.alignment: Qt.AlignHCenter
                    }

                    PlasmaComponents3.Button {
                        text: "Spustit nové AGY sezení"
                        icon.name: "utilities-terminal"
                        Layout.alignment: Qt.AlignHCenter
                        onClicked: root.launchTerminal("")
                    }
                }

                // Active sessions list
                ListView {
                    anchors.fill: parent
                    visible: activeSessions.length > 0
                    clip: true
                    spacing: Kirigami.Units.smallSpacing
                    model: activeSessions

                    delegate: Rectangle {
                        id: sessionDelegate
                        required property var modelData
                        property bool isExpanded: fullRoot.isSessionExpanded(modelData.pid)

                        width: ListView.view.width
                        height: isExpanded ? 340 : 108
                        Behavior on height { NumberAnimation { duration: 200; easing.type: Easing.InOutQuad } }

                        radius: Kirigami.Units.smallSpacing
                        color: Kirigami.Theme.alternateBackgroundColor
                        border.color: modelData.not_fully_idle ? Kirigami.Theme.positiveTextColor : fullRoot.borderColor
                        border.width: modelData.not_fully_idle ? 2 : 1

                        // Selected session-specific overrides
                        property string chosenAgent: modelData.agent_name || "self"
                        property string chosenMode: modelData.current_mode || "auto"
                        property string chosenEffort: modelData.current_effort || "medium"

                        ColumnLayout {
                            anchors.fill: parent
                            anchors.margins: Kirigami.Units.smallSpacing
                            spacing: 4

                            // Top row: Project name, PID, Status, Expand toggle
                            RowLayout {
                                Layout.fillWidth: true

                                Kirigami.Icon {
                                    source: "folder"
                                    Layout.preferredWidth: 16
                                    Layout.preferredHeight: 16
                                }

                                Text {
                                    text: modelData.project_name
                                    font.bold: true
                                    font.pixelSize: 13
                                    color: Kirigami.Theme.textColor
                                    elide: Text.ElideRight
                                    Layout.fillWidth: true
                                }

                                Rectangle {
                                    radius: 3
                                    color: modelData.not_fully_idle ? Kirigami.Theme.positiveTextColor : Kirigami.Theme.neutralTextColor
                                    Layout.preferredWidth: statusLabel.contentWidth + 8
                                    Layout.preferredHeight: 18

                                    Text {
                                        id: statusLabel
                                        anchors.centerIn: parent
                                        text: modelData.not_fully_idle ? "Pracuje" : "Čeká"
                                        font.pixelSize: 10
                                        font.bold: true
                                        color: Kirigami.Theme.backgroundColor
                                    }
                                }

                                Rectangle {
                                    radius: 3
                                    color: fullRoot.borderColor
                                    Layout.preferredWidth: pidText.contentWidth + 6
                                    Layout.preferredHeight: 18

                                    Text {
                                        id: pidText
                                        anchors.centerIn: parent
                                        text: "PID " + modelData.pid
                                        font.pixelSize: 10
                                        color: Kirigami.Theme.textColor
                                    }
                                }

                                PlasmaComponents3.Button {
                                    icon.name: sessionDelegate.isExpanded ? "arrow-up" : "arrow-down"
                                    display: PlasmaComponents3.AbstractButton.IconOnly
                                    PlasmaComponents3.ToolTip.text: sessionDelegate.isExpanded ? "Sbalit detaily" : "Rozbalit spotřebu a přepnutí agenta"
                                    onClicked: fullRoot.toggleSessionExpanded(modelData.pid)
                                }
                            }

                            // Middle row: Conversation Title
                            Text {
                                text: modelData.title ? modelData.title : "(Nová konverzace)"
                                font.pixelSize: 11
                                color: Kirigami.Theme.disabledTextColor
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                            }

                            // Summary row: Agent, Tokens, Memory, CPU
                            RowLayout {
                                Layout.fillWidth: true
                                spacing: Kirigami.Units.mediumSpacing

                                Text {
                                    text: "Agent: " + (modelData.agent_name || "self")
                                    font.pixelSize: 10
                                    color: Kirigami.Theme.textColor
                                }
                                Text {
                                    text: "Kontext: " + (modelData.token_usage?.session_percentage ?? 0) + "% (" + Math.round((modelData.token_usage?.current_context_tokens ?? 0) / 1000) + "k / 1M)"
                                    font.pixelSize: 10
                                    font.bold: true
                                    color: (modelData.token_usage?.session_percentage > 80) ? Kirigami.Theme.negativeTextColor : ((modelData.token_usage?.session_percentage > 60) ? "#f97316" : Kirigami.Theme.positiveTextColor)
                                }
                                Text {
                                    text: "RAM: " + (modelData.stats?.memory_rss_mb ?? 0) + " MB"
                                    font.pixelSize: 10
                                    color: Kirigami.Theme.disabledTextColor
                                }

                                Item { Layout.fillWidth: true }

                                PlasmaComponents3.Button {
                                    icon.name: "utilities-terminal"
                                    display: PlasmaComponents3.AbstractButton.IconOnly
                                    PlasmaComponents3.ToolTip.text: "Otevřít terminál a pokračovat v sezení"
                                    onClicked: root.resumeSession(modelData.conversation_id, modelData.workspace_path)
                                }

                                PlasmaComponents3.Button {
                                    icon.name: "process-stop"
                                    display: PlasmaComponents3.AbstractButton.IconOnly
                                    PlasmaComponents3.ToolTip.text: "Zastavit sezení (Kill PID)"
                                    onClicked: root.killSession(modelData.pid)
                                }
                            }

                            // ==========================================
                            // EXPANDED SECTION (TOKENS + AGENT + EFFORT)
                            // ==========================================
                            ColumnLayout {
                                Layout.fillWidth: true
                                visible: sessionDelegate.isExpanded
                                spacing: 6

                                Rectangle {
                                    Layout.fillWidth: true
                                    height: 1
                                    color: fullRoot.borderColor
                                }

                                // 1. Model context fullness & Token consumption
                                RowLayout {
                                    Layout.fillWidth: true
                                    Text {
                                        text: "Zaplnění kontextu (1M):"
                                        font.bold: true
                                        font.pixelSize: 11
                                        color: Kirigami.Theme.textColor
                                    }
                                    Text {
                                        text: (modelData.token_usage?.session_percentage ?? 0) + "%"
                                        font.pixelSize: 11
                                        font.bold: true
                                        color: (modelData.token_usage?.session_percentage > 80) ? Kirigami.Theme.negativeTextColor : ((modelData.token_usage?.session_percentage > 60) ? "#f97316" : Kirigami.Theme.positiveTextColor)
                                    }
                                    Item { Layout.fillWidth: true }
                                    Text {
                                        text: Math.round((modelData.token_usage?.current_context_tokens ?? 0) / 1000) + "k kontext • " + Math.round((modelData.token_usage?.input_tokens ?? 0) / 1000) + "k in / " + Math.round((modelData.token_usage?.output_tokens ?? 0) / 1000) + "k out"
                                        font.pixelSize: 10
                                        color: Kirigami.Theme.disabledTextColor
                                    }
                                }

                                // Token Context Progress Bar
                                Rectangle {
                                    Layout.fillWidth: true
                                    height: 6
                                    radius: 3
                                    color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.12)

                                    Rectangle {
                                        width: Math.min(parent.width * ((modelData.token_usage?.session_percentage ?? 0) / 100), parent.width)
                                        height: parent.height
                                        radius: 3
                                        color: ((modelData.token_usage?.session_percentage ?? 0) > 80)
                                            ? Kirigami.Theme.negativeTextColor
                                            : (((modelData.token_usage?.session_percentage ?? 0) > 60) ? "#f97316" : Kirigami.Theme.positiveTextColor)
                                    }
                                }

                                // 2. Quick Switch Agent
                                Text {
                                    text: "Změnit Agenta pro toto sezení:"
                                    font.bold: true
                                    font.pixelSize: 11
                                    color: Kirigami.Theme.textColor
                                }

                                Flow {
                                    Layout.fillWidth: true
                                    spacing: 4

                                    Repeater {
                                        model: agentsList
                                        delegate: PlasmaComponents3.Button {
                                            required property var modelData
                                            text: modelData.name
                                            checkable: true
                                            checked: sessionDelegate.chosenAgent === modelData.id
                                            onClicked: {
                                                sessionDelegate.chosenAgent = modelData.id;
                                            }
                                        }
                                    }
                                }

                                // 3. Režim & Thinking Level (Effort)
                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: Kirigami.Units.smallSpacing

                                    ColumnLayout {
                                        spacing: 2
                                        Text {
                                            text: "Režim (Mode):"
                                            font.pixelSize: 10
                                            font.bold: true
                                            color: Kirigami.Theme.disabledTextColor
                                        }
                                        RowLayout {
                                            spacing: 2
                                            PlasmaComponents3.Button {
                                                text: "accept-edits"
                                                checkable: true
                                                checked: sessionDelegate.chosenMode === "accept-edits"
                                                onClicked: sessionDelegate.chosenMode = "accept-edits"
                                            }
                                            PlasmaComponents3.Button {
                                                text: "auto"
                                                checkable: true
                                                checked: sessionDelegate.chosenMode === "auto"
                                                onClicked: sessionDelegate.chosenMode = "auto"
                                            }
                                            PlasmaComponents3.Button {
                                                text: "plan"
                                                checkable: true
                                                checked: sessionDelegate.chosenMode === "plan"
                                                onClicked: sessionDelegate.chosenMode = "plan"
                                            }
                                        }
                                    }

                                    ColumnLayout {
                                        spacing: 2
                                        Text {
                                            text: "Thinking Level (Effort):"
                                            font.pixelSize: 10
                                            font.bold: true
                                            color: Kirigami.Theme.disabledTextColor
                                        }
                                        RowLayout {
                                            spacing: 2
                                            PlasmaComponents3.Button {
                                                text: "low"
                                                checkable: true
                                                checked: sessionDelegate.chosenEffort === "low"
                                                onClicked: sessionDelegate.chosenEffort = "low"
                                            }
                                            PlasmaComponents3.Button {
                                                text: "medium"
                                                checkable: true
                                                checked: sessionDelegate.chosenEffort === "medium"
                                                onClicked: sessionDelegate.chosenEffort = "medium"
                                            }
                                            PlasmaComponents3.Button {
                                                text: "high"
                                                checkable: true
                                                checked: sessionDelegate.chosenEffort === "high"
                                                onClicked: sessionDelegate.chosenEffort = "high"
                                            }
                                            PlasmaComponents3.Button {
                                                text: "max"
                                                checkable: true
                                                checked: sessionDelegate.chosenEffort === "max"
                                                onClicked: sessionDelegate.chosenEffort = "max"
                                            }
                                        }
                                    }
                                }

                                // Apply & Resume Button
                                PlasmaComponents3.Button {
                                    text: "Aplikovat a pokračovat v Terminálu (s agentem '" + sessionDelegate.chosenAgent + "')"
                                    icon.name: "media-playback-start"
                                    Layout.fillWidth: true
                                    highlighted: true
                                    onClicked: {
                                        root.resumeWithCustomSettings(
                                            modelData.conversation_id,
                                            modelData.workspace_path,
                                            sessionDelegate.chosenAgent,
                                            sessionDelegate.chosenMode,
                                            sessionDelegate.chosenEffort
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ==========================================
            // TAB 1: HISTORIE KONVERZACÍ
            // ==========================================
            Item {
                Layout.fillWidth: true
                Layout.fillHeight: true

                ListView {
                    anchors.fill: parent
                    clip: true
                    spacing: 4
                    model: recentConversations

                    delegate: Rectangle {
                        required property var modelData
                        width: ListView.view.width
                        height: 64
                        radius: Kirigami.Units.smallSpacing
                        color: Kirigami.Theme.alternateBackgroundColor
                        border.color: fullRoot.borderColor
                        border.width: 1

                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: Kirigami.Units.smallSpacing
                            spacing: Kirigami.Units.mediumSpacing

                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 2

                                Text {
                                    text: modelData.title || "(Bez názvu)"
                                    font.bold: true
                                    font.pixelSize: 12
                                    color: Kirigami.Theme.textColor
                                    elide: Text.ElideRight
                                    Layout.fillWidth: true
                                }

                                RowLayout {
                                    spacing: Kirigami.Units.mediumSpacing
                                    Text {
                                        text: "Kroky: " + modelData.step_count
                                        font.pixelSize: 10
                                        color: Kirigami.Theme.disabledTextColor
                                    }
                                    Text {
                                        text: "Spotřeba: " + Math.round((modelData.token_usage?.total_tokens ?? 0) / 1000) + "k tok"
                                        font.pixelSize: 10
                                        color: Kirigami.Theme.highlightColor
                                    }
                                    Text {
                                        text: modelData.last_modified_time
                                        font.pixelSize: 10
                                        color: Kirigami.Theme.disabledTextColor
                                    }
                                }
                            }

                            PlasmaComponents3.Button {
                                text: "Pokračovat"
                                icon.name: "media-playback-start"
                                onClicked: root.resumeSession(modelData.conversation_id, "")
                            }
                        }
                    }
                }
            }

            // ==============================================================
            // TAB 2: NASTAVENÍ A VÝCHOZÍ MODELY (ALL 6 MODELS + PERMANENT)
            // ==============================================================
            Item {
                Layout.fillWidth: true
                Layout.fillHeight: true

                PlasmaComponents3.ScrollView {
                    anchors.fill: parent
                    clip: true
                    contentWidth: availableWidth
                    QQC2.ScrollBar.horizontal.policy: QQC2.ScrollBar.AlwaysOff

                    ColumnLayout {
                        width: parent.width
                        spacing: Kirigami.Units.mediumSpacing

                        // Header / Note
                        Text {
                            text: "Výchozí AI Model (Default pro nová sezení):"
                            font.bold: true
                            font.pixelSize: 13
                            color: Kirigami.Theme.textColor
                        }

                        Text {
                            text: "Zvolený model se nastaví trvale do ~/.gemini/antigravity-cli/settings.json a je ihned aktivní."
                            font.pixelSize: 11
                            color: Kirigami.Theme.disabledTextColor
                            Layout.fillWidth: true
                            wrapMode: Text.WordWrap
                        }

                        // Gemini Models Section
                        RowLayout {
                            Layout.fillWidth: true
                            Text {
                                text: "GEMINI CORE MODELY"
                                font.bold: true
                                font.pixelSize: 11
                                color: Kirigami.Theme.highlightColor
                                Layout.fillWidth: true
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 6

                            Repeater {
                                model: modelsList.filter(m => m.provider === "Gemini")
                                delegate: Rectangle {
                                    id: geminiCard
                                    required property var modelData
                                    readonly property bool isSelected: (settingsData.default_model === modelData.id || settingsData.model === modelData.name)
                                    readonly property bool isAvailable: modelData.available !== false

                                    Layout.fillWidth: true
                                    Layout.preferredHeight: 44
                                    radius: Kirigami.Units.smallSpacing
                                    opacity: isAvailable ? 1.0 : 0.6
                                    color: isSelected
                                        ? Qt.rgba(Kirigami.Theme.highlightColor.r, Kirigami.Theme.highlightColor.g, Kirigami.Theme.highlightColor.b, 0.15)
                                        : Kirigami.Theme.alternateBackgroundColor
                                    border.color: isSelected
                                        ? Kirigami.Theme.highlightColor
                                        : fullRoot.borderColor
                                    border.width: isSelected ? 2 : 1

                                    MouseArea {
                                        anchors.fill: parent
                                        enabled: geminiCard.isAvailable
                                        cursorShape: geminiCard.isAvailable ? Qt.PointingHandCursor : Qt.ArrowCursor
                                        onClicked: {
                                            if (geminiCard.isAvailable) {
                                                root.setDefaultModel(modelData.id, modelData.name);
                                            }
                                        }
                                    }

                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.margins: Kirigami.Units.smallSpacing
                                        spacing: Kirigami.Units.smallSpacing

                                        PlasmaComponents3.RadioButton {
                                            checked: geminiCard.isSelected
                                            enabled: geminiCard.isAvailable
                                            onClicked: {
                                                if (geminiCard.isAvailable) {
                                                    root.setDefaultModel(modelData.id, modelData.name);
                                                }
                                            }
                                        }

                                        ColumnLayout {
                                            Layout.fillWidth: true
                                            spacing: 1

                                            Text {
                                                text: modelData.name
                                                font.pixelSize: 12
                                                font.bold: geminiCard.isSelected
                                                color: geminiCard.isAvailable ? Kirigami.Theme.textColor : Kirigami.Theme.disabledTextColor
                                                elide: Text.ElideRight
                                                Layout.fillWidth: true
                                            }

                                            Text {
                                                text: modelData.id + (geminiCard.isAvailable ? (modelData.thinking ? " • Thinking" : "") : " • Zatím nedostupný v API")
                                                font.pixelSize: 10
                                                color: Kirigami.Theme.disabledTextColor
                                                elide: Text.ElideRight
                                                Layout.fillWidth: true
                                            }
                                        }

                                        Rectangle {
                                            visible: geminiCard.isSelected
                                            radius: 3
                                            color: Kirigami.Theme.highlightColor
                                            Layout.preferredWidth: defaultLabel.contentWidth + 8
                                            Layout.preferredHeight: 18

                                            Text {
                                                id: defaultLabel
                                                anchors.centerIn: parent
                                                text: "VÝCHOZÍ"
                                                font.pixelSize: 9
                                                font.bold: true
                                                color: Kirigami.Theme.highlightedTextColor
                                            }
                                        }

                                        Rectangle {
                                            visible: !geminiCard.isAvailable
                                            radius: 3
                                            color: fullRoot.borderColor
                                            Layout.preferredWidth: unavailLabel.contentWidth + 8
                                            Layout.preferredHeight: 18

                                            Text {
                                                id: unavailLabel
                                                anchors.centerIn: parent
                                                text: "NEDOSTUPNÝ"
                                                font.pixelSize: 9
                                                font.bold: true
                                                color: Kirigami.Theme.disabledTextColor
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Outsource Models Section (Claude / GPT)
                        RowLayout {
                            Layout.fillWidth: true
                            Text {
                                text: "OUTSOURCE MODELY (Claude & GPT-OSS)"
                                font.bold: true
                                font.pixelSize: 11
                                color: "#f97316"
                                Layout.fillWidth: true
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 6

                            Repeater {
                                model: modelsList.filter(m => m.provider === "Outsource")
                                delegate: Rectangle {
                                    id: outsourceCard
                                    required property var modelData
                                    readonly property bool isSelected: (settingsData.default_model === modelData.id || settingsData.model === modelData.name)

                                    Layout.fillWidth: true
                                    Layout.preferredHeight: 44
                                    radius: Kirigami.Units.smallSpacing
                                    color: isSelected
                                        ? Qt.rgba(1, 0.5, 0, 0.15)
                                        : Kirigami.Theme.alternateBackgroundColor
                                    border.color: isSelected
                                        ? "#f97316"
                                        : fullRoot.borderColor
                                    border.width: isSelected ? 2 : 1

                                    MouseArea {
                                        anchors.fill: parent
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: root.setDefaultModel(modelData.id, modelData.name)
                                    }

                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.margins: Kirigami.Units.smallSpacing
                                        spacing: Kirigami.Units.smallSpacing

                                        PlasmaComponents3.RadioButton {
                                            checked: outsourceCard.isSelected
                                            onClicked: root.setDefaultModel(modelData.id, modelData.name)
                                        }

                                        ColumnLayout {
                                            Layout.fillWidth: true
                                            spacing: 1

                                            Text {
                                                text: modelData.name
                                                font.pixelSize: 12
                                                font.bold: outsourceCard.isSelected
                                                color: Kirigami.Theme.textColor
                                                elide: Text.ElideRight
                                                Layout.fillWidth: true
                                            }

                                            Text {
                                                text: modelData.id + (modelData.thinking ? " • Thinking" : "")
                                                font.pixelSize: 10
                                                color: Kirigami.Theme.disabledTextColor
                                                elide: Text.ElideRight
                                                Layout.fillWidth: true
                                            }
                                        }

                                        Rectangle {
                                            visible: outsourceCard.isSelected
                                            radius: 3
                                            color: "#f97316"
                                            Layout.preferredWidth: defaultOutLabel.contentWidth + 8
                                            Layout.preferredHeight: 18

                                            Text {
                                                id: defaultOutLabel
                                                anchors.centerIn: parent
                                                text: "VÝCHOZÍ"
                                                font.pixelSize: 9
                                                font.bold: true
                                                color: "#ffffff"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        Rectangle {
                            Layout.fillWidth: true
                            height: 1
                            color: fullRoot.borderColor
                        }

                        // Thinking Level (Effort)
                        Text {
                            text: "Výchozí Thinking Level (Effort):"
                            font.bold: true
                            font.pixelSize: 13
                            color: Kirigami.Theme.textColor
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            spacing: Kirigami.Units.smallSpacing

                            Repeater {
                                model: ["low", "medium", "high", "xhigh", "max"]
                                delegate: PlasmaComponents3.Button {
                                    required property var modelData
                                    text: modelData
                                    Layout.fillWidth: true
                                    highlighted: (settingsData.effort ?? "medium") === modelData
                                    onClicked: root.setDefaultEffort(modelData)
                                }
                            }
                        }

                        Rectangle {
                            Layout.fillWidth: true
                            height: 1
                            color: fullRoot.borderColor
                        }

                        // Default Agent Mode
                        Text {
                            text: "Výchozí Režim Provádění (Agent Mode):"
                            font.bold: true
                            font.pixelSize: 13
                            color: Kirigami.Theme.textColor
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            spacing: Kirigami.Units.smallSpacing

                            PlasmaComponents3.Button {
                                text: "auto"
                                Layout.fillWidth: true
                                highlighted: (settingsData.agent_mode ?? "auto") === "auto"
                                onClicked: root.setDefaultMode("auto")
                            }
                            PlasmaComponents3.Button {
                                text: "accept-edits"
                                Layout.fillWidth: true
                                highlighted: settingsData.agent_mode === "accept-edits"
                                onClicked: root.setDefaultMode("accept-edits")
                            }
                            PlasmaComponents3.Button {
                                text: "plan"
                                Layout.fillWidth: true
                                highlighted: settingsData.agent_mode === "plan"
                                onClicked: root.setDefaultMode("plan")
                            }
                        }

                        Rectangle {
                            Layout.fillWidth: true
                            height: 1
                            color: fullRoot.borderColor
                        }

                        // Sandbox Indicator
                        RowLayout {
                            Text {
                                text: "Terminálový sandbox:"
                                font.pixelSize: 12
                                color: Kirigami.Theme.textColor
                            }
                            Rectangle {
                                radius: 3
                                color: settingsData.sandbox ? Kirigami.Theme.positiveTextColor : Kirigami.Theme.negativeTextColor
                                Layout.preferredWidth: sandboxText.contentWidth + 8
                                Layout.preferredHeight: 18

                                Text {
                                    id: sandboxText
                                    anchors.centerIn: parent
                                    text: settingsData.sandbox ? "Aktivní (Standard)" : "Vypnutý (Bypass)"
                                    font.pixelSize: 10
                                    font.bold: true
                                    color: Kirigami.Theme.backgroundColor
                                }
                            }
                        }

                        Rectangle {
                            Layout.fillWidth: true
                            height: 1
                            color: fullRoot.borderColor
                        }

                        // Desktop Notifications (KDE Plasma 6)
                        Text {
                            text: "Desktopové Notifikace a Signalizace (KDE Plasma 6):"
                            font.bold: true
                            font.pixelSize: 13
                            color: Kirigami.Theme.textColor
                        }

                        Text {
                            text: "Upozornění při dokončení práce agenta a resetu 5h kvóty (s tlačítkem pro návrat do terminálu)."
                            font.pixelSize: 11
                            color: Kirigami.Theme.disabledTextColor
                            Layout.fillWidth: true
                            wrapMode: Text.WordWrap
                        }

                        PlasmaComponents3.Switch {
                            id: notifSwitch
                            text: "Povolit desktopové notifikace"
                            checked: settingsData.notifications_enabled ?? true
                            onToggled: root.setNotificationsEnabled(checked)
                            Layout.fillWidth: true
                        }

                        PlasmaComponents3.Switch {
                            id: soundSwitch
                            text: "Zvukový signál (KDE KNotify / bell)"
                            checked: settingsData.notification_sound ?? true
                            enabled: notifSwitch.checked
                            onToggled: root.setNotificationSound(checked)
                            Layout.fillWidth: true
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            spacing: Kirigami.Units.smallSpacing

                            PlasmaComponents3.Button {
                                text: "Vyzkoušet notifikaci"
                                icon.name: "dialog-information"
                                onClicked: root.testNotification()
                            }

                            Item { Layout.fillWidth: true }
                        }
                    }
                }
            }

            // ==========================================
            // TAB 3: SKILLY A PLUGINY
            // ==========================================
            Item {
                Layout.fillWidth: true
                Layout.fillHeight: true

                ColumnLayout {
                    anchors.fill: parent
                    spacing: Kirigami.Units.smallSpacing

                    // 1. Search Bar
                    PlasmaComponents3.TextField {
                        id: skillSearch
                        placeholderText: "Hledat skill, plugin, účel (např. docker, seo, audit)..."
                        Layout.fillWidth: true
                    }

                    // 2. Category Filter Pills (Clean Flow without clipped scrollbars)
                    Flow {
                        Layout.fillWidth: true
                        spacing: 4

                        Repeater {
                            model: [
                                { cat: "all", label: "Vše (" + skillsList.length + ")" },
                                { cat: "plugins", label: "Pluginy" },
                                { cat: "database", label: "Databáze" },
                                { cat: "ux", label: "UX & CRO" },
                                { cat: "seo", label: "SEO / GEO" },
                                { cat: "architect", label: "Architekt" },
                                { cat: "user", label: "Uživatelské" }
                            ]

                            delegate: Rectangle {
                                id: pillBtn
                                required property var modelData
                                readonly property bool isSelected: fullRoot.selectedSkillCategory === modelData.cat

                                width: pillLabel.contentWidth + 16
                                height: 26
                                radius: 13
                                color: isSelected
                                    ? Kirigami.Theme.highlightColor
                                    : Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.08)
                                border.color: isSelected ? Kirigami.Theme.highlightColor : fullRoot.borderColor
                                border.width: 1

                                Text {
                                    id: pillLabel
                                    anchors.centerIn: parent
                                    text: pillBtn.modelData.label
                                    font.pixelSize: 10
                                    font.bold: pillBtn.isSelected
                                    color: pillBtn.isSelected ? Kirigami.Theme.highlightedTextColor : Kirigami.Theme.textColor
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: fullRoot.selectedSkillCategory = pillBtn.modelData.cat
                                }
                            }
                        }
                    }

                    // 3. Skills Rich List
                    ListView {
                        id: skillsListView
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        clip: true
                        spacing: 6
                        model: skillsList.filter(s => {
                            var cat = fullRoot.selectedSkillCategory;
                            if (cat === "plugins" && s.skill_type !== "plugin") return false;
                            if (cat === "database" && (s.plugin_name || "").indexOf("database") === -1) return false;
                            if (cat === "ux" && (s.plugin_name || "").indexOf("ux") === -1) return false;
                            if (cat === "seo" && (s.plugin_name || "").indexOf("seo") === -1) return false;
                            if (cat === "architect" && (s.plugin_name || "").indexOf("architect") === -1) return false;
                            if (cat === "user" && s.skill_type !== "user" && s.skill_type !== "builtin") return false;
                            if (skillSearch.text === "") return true;
                            var q = skillSearch.text.toLowerCase().trim();
                            return (s.name && s.name.toLowerCase().indexOf(q) !== -1)
                                || (s.description && s.description.toLowerCase().indexOf(q) !== -1)
                                || (s.plugin_name && s.plugin_name.toLowerCase().indexOf(q) !== -1)
                                || (s.command && s.command.toLowerCase().indexOf(q) !== -1);
                        })

                        delegate: Rectangle {
                            id: skillCard
                            required property var modelData
                            property bool isExpanded: fullRoot.isSkillExpanded(modelData.name)

                            width: ListView.view.width
                            height: isExpanded ? 172 : 74
                            Behavior on height { NumberAnimation { duration: 180; easing.type: Easing.InOutQuad } }

                            radius: Kirigami.Units.smallSpacing
                            color: Kirigami.Theme.alternateBackgroundColor
                            border.color: isExpanded ? Kirigami.Theme.highlightColor : fullRoot.borderColor
                            border.width: isExpanded ? 2 : 1
                            clip: true

                            ColumnLayout {
                                anchors.fill: parent
                                anchors.margins: Kirigami.Units.smallSpacing
                                spacing: 4

                                // Top row: Icon, Name + Plugin Badge, Command, Expand Button
                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: Kirigami.Units.smallSpacing

                                    Kirigami.Icon {
                                        source: fullRoot.getSkillIcon(modelData.plugin_name, modelData.skill_type)
                                        Layout.preferredWidth: 20
                                        Layout.preferredHeight: 20
                                    }

                                    ColumnLayout {
                                        Layout.fillWidth: true
                                        spacing: 1

                                        RowLayout {
                                            spacing: Kirigami.Units.smallSpacing
                                            Text {
                                                text: modelData.name
                                                font.pixelSize: 12
                                                font.bold: true
                                                color: Kirigami.Theme.textColor
                                                elide: Text.ElideRight
                                            }

                                            Rectangle {
                                                radius: 3
                                                color: fullRoot.getPluginBadgeColor(modelData.plugin_name, modelData.skill_type)
                                                Layout.preferredWidth: badgeLabel.contentWidth + 8
                                                Layout.preferredHeight: 16

                                                Text {
                                                    id: badgeLabel
                                                    anchors.centerIn: parent
                                                    text: modelData.plugin_name || modelData.skill_type
                                                    font.pixelSize: 9
                                                    font.bold: true
                                                    color: "#ffffff"
                                                }
                                            }
                                        }

                                        Text {
                                            text: modelData.command || ("/" + modelData.name)
                                            font.pixelSize: 10
                                            font.bold: true
                                            font.family: "monospace"
                                            color: Kirigami.Theme.highlightColor
                                            elide: Text.ElideRight
                                            Layout.fillWidth: true
                                        }
                                    }

                                    PlasmaComponents3.Button {
                                        icon.name: skillCard.isExpanded ? "arrow-up" : "arrow-down"
                                        display: PlasmaComponents3.AbstractButton.IconOnly
                                        flat: true
                                        Layout.preferredWidth: 26
                                        Layout.preferredHeight: 26
                                        PlasmaComponents3.ToolTip.text: skillCard.isExpanded ? "Sbalit detaily" : "Zobrazit popis a akce"
                                        onClicked: fullRoot.toggleSkillExpanded(modelData.name)
                                    }
                                }

                                // Middle row: Description with high contrast and readable typography
                                Text {
                                    text: modelData.description || "(Bez popisu - systémový konfigurační skill)"
                                    font.pixelSize: 11
                                    color: skillCard.isExpanded ? Kirigami.Theme.textColor : Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.85)
                                    elide: skillCard.isExpanded ? Text.ElideNone : Text.ElideRight
                                    wrapMode: skillCard.isExpanded ? Text.WordWrap : Text.NoWrap
                                    maximumLineCount: skillCard.isExpanded ? 3 : 1
                                    Layout.fillWidth: true
                                }

                                // Expanded Section: Action Buttons
                                RowLayout {
                                    Layout.fillWidth: true
                                    visible: skillCard.isExpanded
                                    spacing: Kirigami.Units.smallSpacing

                                    PlasmaComponents3.Button {
                                        text: "Kopírovat"
                                        icon.name: "edit-copy"
                                        PlasmaComponents3.ToolTip.text: "Zkopírovat slash command do schránky"
                                        onClicked: root.copyToClipboard(modelData.command || ("/" + modelData.name))
                                    }

                                    PlasmaComponents3.Button {
                                        text: "Spustit v terminálu"
                                        icon.name: "utilities-terminal"
                                        highlighted: true
                                        PlasmaComponents3.ToolTip.text: "Spustit nové sezení agy s tímto skillem"
                                        onClicked: root.launchSkillInTerminal(modelData.command || ("/" + modelData.name), "")
                                    }

                                    PlasmaComponents3.Button {
                                        text: "SKILL.md"
                                        icon.name: "document-edit"
                                        PlasmaComponents3.ToolTip.text: "Otevřít definici v systémovém editoru"
                                        onClicked: root.openFile(modelData.path)
                                    }

                                    Item { Layout.fillWidth: true }
                                }

                                // Expanded Section: File Path
                                Text {
                                    visible: skillCard.isExpanded
                                    text: modelData.path
                                    font.pixelSize: 9
                                    color: Kirigami.Theme.disabledTextColor
                                    elide: Text.ElideMiddle
                                    Layout.fillWidth: true
                                }
                            }
                        }
                    }
                }
            }
        }

        // --- BOTTOM ACTIONS FOOTER ---
        RowLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing

            PlasmaComponents3.Button {
                text: "Nové sezení"
                icon.name: "list-add"
                Layout.fillWidth: true
                onClicked: root.launchTerminal("")
            }

            PlasmaComponents3.Button {
                text: "Nouzově zastavit vše"
                icon.name: "process-stop"
                Layout.fillWidth: true
                enabled: activeSessions.length > 0
                onClicked: {
                    for (var i = 0; i < fullRoot.activeSessions.length; i++) {
                        root.killSession(fullRoot.activeSessions[i].pid);
                    }
                }
            }
        }
    }
}
