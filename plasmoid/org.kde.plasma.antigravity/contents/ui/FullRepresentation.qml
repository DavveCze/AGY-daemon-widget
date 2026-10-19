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
    readonly property bool isClaudeSelected: {
        var defM = (settingsData && settingsData.default_model) ? ("" + settingsData.default_model).toLowerCase() : "";
        var curM = (settingsData && settingsData.model) ? ("" + settingsData.model).toLowerCase() : "";
        return defM.indexOf("claude") !== -1 || curM.indexOf("claude") !== -1;
    }

    property var expandedPids: ({})
    property var expandedSkills: ({})
    property string selectedSkillCategory: "all"
    property string selectedProviderFilter: "all"

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

    property var activeRollbackPid: 0
    property var diffOpenedPid: 0

    function toggleDiffView(pid, workspace) {
        if (diffOpenedPid === pid) {
            diffOpenedPid = 0;
        } else {
            diffOpenedPid = pid;
            root.viewGitDiff(workspace);
        }
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
                            text: "AgentiCom: AI CLI Monitor"
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
                        text: fullRoot.realQuota?.account_email || stateData?.stats?.account_email || i18n("Uživatel")
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
                icon.name: "utilities-terminal"
                text: "Aktivní (" + activeSessions.length + ")"
            }
            PlasmaComponents3.TabButton {
                icon.name: "view-history"
                text: "Historie"
            }
            PlasmaComponents3.TabButton {
                icon.name: "configure"
                text: "Nastavení"
            }
            PlasmaComponents3.TabButton {
                icon.name: "plugins"
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

                // Provider Filter Pills (All / AGY / Claude / Codex)
                RowLayout {
                    id: filterPillsRow
                    anchors.top: parent.top
                    anchors.left: parent.left
                    anchors.right: parent.right
                    spacing: 6
                    visible: activeSessions.length > 0

                    Repeater {
                        model: [
                            { id: "all", label: "Vše (" + activeSessions.length + ")", color: Kirigami.Theme.highlightColor },
                            { id: "antigravity", label: "AGY (" + activeSessions.filter(s => (s.provider || "antigravity") === "antigravity").length + ")", color: "#3b82f6" },
                            { id: "claude", label: "Claude (" + activeSessions.filter(s => s.provider === "claude").length + ")", color: "#ea580c" },
                            { id: "codex", label: "Codex (" + activeSessions.filter(s => s.provider === "codex").length + ")", color: "#10b981" },
                        ]
                        delegate: Rectangle {
                            id: pill
                            required property var modelData
                            readonly property bool isSelected: fullRoot.selectedProviderFilter === modelData.id

                            Layout.preferredHeight: 30
                            Layout.preferredWidth: pillText.contentWidth + 18
                            radius: 15
                            color: isSelected ? modelData.color : Kirigami.Theme.alternateBackgroundColor
                            border.color: isSelected ? modelData.color : fullRoot.borderColor
                            border.width: 1

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: fullRoot.selectedProviderFilter = pill.modelData.id
                            }

                            Text {
                                id: pillText
                                anchors.centerIn: parent
                                text: pill.modelData.label
                                font.pixelSize: 11
                                font.bold: pill.isSelected
                                color: pill.isSelected ? "#ffffff" : Kirigami.Theme.textColor
                            }
                        }
                    }

                    Item { Layout.fillWidth: true }
                }

                // Empty state - centered in parent
                ColumnLayout {
                    anchors.centerIn: parent
                    visible: activeSessions.filter(s => fullRoot.selectedProviderFilter === "all" || (s.provider || "antigravity") === fullRoot.selectedProviderFilter).length === 0
                    spacing: Kirigami.Units.largeSpacing

                    Kirigami.Icon {
                        source: "dialog-information"
                        Layout.alignment: Qt.AlignHCenter
                        Layout.preferredWidth: 48
                        Layout.preferredHeight: 48
                        opacity: 0.5
                    }

                    Text {
                        text: fullRoot.selectedProviderFilter === "all"
                            ? "Všechny úkoly jsou hotové. Žádné sezení neběží."
                            : ("Žádné aktivní sezení pro " + (fullRoot.selectedProviderFilter === "codex" ? "OpenAI Codex" : (fullRoot.selectedProviderFilter === "claude" ? "Anthropic Claude" : "Google Antigravity")))
                        font.pixelSize: 13
                        color: Kirigami.Theme.disabledTextColor
                        Layout.alignment: Qt.AlignHCenter
                    }

                    PlasmaComponents3.Button {
                        text: fullRoot.selectedProviderFilter === "codex" ? "Spustit nové Codex sezení" : "Spustit nové sezení"
                        icon.name: "utilities-terminal"
                        highlighted: true
                        Layout.alignment: Qt.AlignHCenter
                        Layout.preferredHeight: 38
                        onClicked: root.launchTerminal("")
                    }
                }

                // Active sessions list
                ListView {
                    anchors.top: filterPillsRow.visible ? filterPillsRow.bottom : parent.top
                    anchors.topMargin: filterPillsRow.visible ? Kirigami.Units.smallSpacing : 0
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    visible: activeSessions.filter(s => fullRoot.selectedProviderFilter === "all" || (s.provider || "antigravity") === fullRoot.selectedProviderFilter).length > 0
                    clip: true
                    spacing: Kirigami.Units.smallSpacing
                    model: activeSessions.filter(s => fullRoot.selectedProviderFilter === "all" || (s.provider || "antigravity") === fullRoot.selectedProviderFilter)

                    delegate: Rectangle {
                        id: sessionDelegate
                        required property var modelData
                        property bool isExpanded: fullRoot.isSessionExpanded(modelData.pid)

                        width: ListView.view.width
                        height: {
                            var h = (modelData.git_status && modelData.git_status.is_repo) ? 156 : 108;
                            if (sessionDelegate.isExpanded) h += 240;
                            if (fullRoot.diffOpenedPid === modelData.pid) h += 215;
                            if (fullRoot.activeRollbackPid === modelData.pid) h += 75;
                            return h;
                        }
                        Behavior on height { NumberAnimation { duration: 200; easing.type: Easing.InOutQuad } }

                        radius: Kirigami.Units.smallSpacing
                        color: Kirigami.Theme.alternateBackgroundColor
                        border.color: modelData.not_fully_idle ? Kirigami.Theme.positiveTextColor : fullRoot.borderColor
                        border.width: modelData.not_fully_idle ? 2 : 1

                        // Selected session-specific overrides
                        property string chosenAgent: modelData.agent_name || "self"
                        property string chosenMode: modelData.current_mode || "auto"
                        property string chosenEffort: modelData.current_effort || "medium"

                        // Left vertical provider identity stripe
                        Rectangle {
                            id: providerStripe
                            anchors.left: parent.left
                            anchors.top: parent.top
                            anchors.bottom: parent.bottom
                            width: 3
                            radius: 1.5
                            color: modelData.provider_badge_color || (modelData.provider === "claude" ? "#ea580c" : (modelData.provider === "codex" ? "#10b981" : "#3b82f6"))
                        }

                        ColumnLayout {
                            anchors.fill: parent
                            anchors.leftMargin: Kirigami.Units.smallSpacing + 3
                            anchors.topMargin: Kirigami.Units.smallSpacing
                            anchors.rightMargin: Kirigami.Units.smallSpacing
                            anchors.bottomMargin: Kirigami.Units.smallSpacing
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
                                    color: modelData.provider_badge_color || (modelData.provider === "claude" ? "#ea580c" : (modelData.provider === "codex" ? "#10b981" : "#3b82f6"))
                                    Layout.preferredWidth: providerBadgeLabel.contentWidth + 8
                                    Layout.preferredHeight: 18

                                    Text {
                                        id: providerBadgeLabel
                                        anchors.centerIn: parent
                                        text: modelData.provider === "claude" ? "Claude" : (modelData.provider === "codex" ? "Codex" : "AGY")
                                        font.pixelSize: 9
                                        font.bold: true
                                        color: "#ffffff"
                                    }
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
                                    Layout.preferredWidth: 34
                                    Layout.preferredHeight: 34
                                    PlasmaComponents3.ToolTip.text: "Otevřít terminál a pokračovat v sezení"
                                    onClicked: root.resumeSession(modelData.conversation_id, modelData.workspace_path)
                                }

                                Item { Layout.preferredWidth: 8 }

                                PlasmaComponents3.Button {
                                    icon.name: "process-stop"
                                    display: PlasmaComponents3.AbstractButton.IconOnly
                                    Layout.preferredWidth: 34
                                    Layout.preferredHeight: 34
                                    PlasmaComponents3.ToolTip.text: "Zastavit sezení (Kill PID " + modelData.pid + ")"
                                    onClicked: root.killSession(modelData.pid)
                                }
                            }

                            // ==========================================
                            // GIT STATUS & 1-CLICK ROLLBACK BAR
                            // ==========================================
                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 6
                                visible: modelData.git_status && modelData.git_status.is_repo

                                Rectangle {
                                    radius: 3
                                    color: Qt.rgba(Kirigami.Theme.highlightColor.r, Kirigami.Theme.highlightColor.g, Kirigami.Theme.highlightColor.b, 0.15)
                                    border.color: Kirigami.Theme.highlightColor
                                    border.width: 1
                                    Layout.preferredHeight: 22
                                    Layout.preferredWidth: gitBranchRow.implicitWidth + 14

                                    RowLayout {
                                        id: gitBranchRow
                                        anchors.centerIn: parent
                                        spacing: 4
                                        Kirigami.Icon {
                                            source: "vcs-branch"
                                            Layout.preferredWidth: 12
                                            Layout.preferredHeight: 12
                                        }
                                        Text {
                                            text: (modelData.git_status?.branch && modelData.git_status.branch !== "HEAD") ? modelData.git_status.branch : (modelData.git_status?.branch || "main")
                                            font.pixelSize: 10
                                            font.bold: true
                                            color: Kirigami.Theme.highlightColor
                                        }
                                    }
                                }

                                Rectangle {
                                    radius: 3
                                    color: (modelData.git_status?.files_changed > 0)
                                        ? Qt.rgba(0.96, 0.62, 0.07, 0.15)
                                        : Qt.rgba(Kirigami.Theme.positiveTextColor.r, Kirigami.Theme.positiveTextColor.g, Kirigami.Theme.positiveTextColor.b, 0.12)
                                    border.color: (modelData.git_status?.files_changed > 0) ? "#f59e0b" : Kirigami.Theme.positiveTextColor
                                    border.width: 1
                                    Layout.preferredHeight: 22
                                    Layout.preferredWidth: gitChangesLabel.contentWidth + 14

                                    Text {
                                        id: gitChangesLabel
                                        anchors.centerIn: parent
                                        text: (modelData.git_status?.files_changed > 0)
                                            ? (modelData.git_status.files_changed + " změn (+" + modelData.git_status.insertions + "/-" + modelData.git_status.deletions + ")")
                                            : "Čistý strom (Clean)"
                                        font.pixelSize: 10
                                        font.bold: modelData.git_status?.files_changed > 0
                                        color: (modelData.git_status?.files_changed > 0) ? "#d97706" : Kirigami.Theme.positiveTextColor
                                    }
                                }

                                Item { Layout.fillWidth: true }

                                PlasmaComponents3.Button {
                                    icon.name: "document-preview"
                                    text: "Diff"
                                    display: PlasmaComponents3.AbstractButton.TextBesideIcon
                                    Layout.preferredHeight: 24
                                    PlasmaComponents3.ToolTip.text: "Zobrazit přehled změn (Git Diff)"
                                    onClicked: fullRoot.toggleDiffView(modelData.pid, modelData.workspace_path)
                                }

                                PlasmaComponents3.Button {
                                    icon.name: "edit-undo"
                                    text: "Vrátit"
                                    display: PlasmaComponents3.AbstractButton.TextBesideIcon
                                    Layout.preferredHeight: 24
                                    enabled: (modelData.git_status?.files_changed > 0)
                                    opacity: (modelData.git_status?.files_changed > 0) ? 1.0 : 0.45
                                    PlasmaComponents3.ToolTip.text: (modelData.git_status?.files_changed > 0)
                                        ? "1-Click Rollback: Vrátit neuložené změny agenta"
                                        : "Pracovní strom je čistý (žádné změny k vrácení)"
                                    onClicked: {
                                        if (fullRoot.activeRollbackPid === modelData.pid) {
                                            fullRoot.activeRollbackPid = 0;
                                        } else {
                                            fullRoot.activeRollbackPid = modelData.pid;
                                        }
                                    }
                                }
                            }

                            // Rollback Inline Confirmation
                            Rectangle {
                                Layout.fillWidth: true
                                Layout.preferredHeight: 68
                                visible: fullRoot.activeRollbackPid === modelData.pid
                                radius: 4
                                color: Qt.rgba(Kirigami.Theme.negativeTextColor.r, Kirigami.Theme.negativeTextColor.g, Kirigami.Theme.negativeTextColor.b, 0.12)
                                border.color: Kirigami.Theme.negativeTextColor
                                border.width: 1

                                ColumnLayout {
                                    anchors.fill: parent
                                    anchors.margins: 6
                                    spacing: 4

                                    RowLayout {
                                        Layout.fillWidth: true
                                        Kirigami.Icon {
                                            source: "dialog-warning"
                                            Layout.preferredWidth: 16
                                            Layout.preferredHeight: 16
                                        }
                                        Text {
                                            text: "Vrátit všechny neuložené změny v projektu?"
                                            font.bold: true
                                            font.pixelSize: 11
                                            color: Kirigami.Theme.negativeTextColor
                                        }
                                        Item { Layout.fillWidth: true }
                                        QQC2.CheckBox {
                                            id: cleanUntrackedCheck
                                            text: "Smazat i nové soubory"
                                            checked: true
                                        }
                                    }

                                    RowLayout {
                                        Layout.fillWidth: true
                                        spacing: 8
                                        Item { Layout.fillWidth: true }
                                        PlasmaComponents3.Button {
                                            text: "Zrušit"
                                            Layout.preferredHeight: 26
                                            onClicked: fullRoot.activeRollbackPid = 0
                                        }
                                        PlasmaComponents3.Button {
                                            text: "Potvrdit Rollback"
                                            icon.name: "edit-undo"
                                            highlighted: true
                                            Layout.preferredHeight: 26
                                            onClicked: {
                                                root.rollbackGitChanges(modelData.workspace_path, cleanUntrackedCheck.checked);
                                                fullRoot.activeRollbackPid = 0;
                                                fullRoot.diffOpenedPid = 0;
                                            }
                                        }
                                    }
                                }
                            }

                            // Inline Diff Drawer
                            Rectangle {
                                Layout.fillWidth: true
                                Layout.preferredHeight: 200
                                visible: fullRoot.diffOpenedPid === modelData.pid
                                radius: 4
                                color: Kirigami.Theme.backgroundColor
                                border.color: fullRoot.borderColor
                                border.width: 1

                                ColumnLayout {
                                    anchors.fill: parent
                                    anchors.margins: 6
                                    spacing: 4

                                    RowLayout {
                                        Layout.fillWidth: true
                                        Text {
                                            text: "Git Diff: " + (modelData.project_name || "")
                                            font.bold: true
                                            font.pixelSize: 11
                                            color: Kirigami.Theme.textColor
                                        }
                                        Item { Layout.fillWidth: true }
                                        PlasmaComponents3.Button {
                                            icon.name: "utilities-terminal"
                                            text: "V terminálu"
                                            Layout.preferredHeight: 22
                                            onClicked: root.openGitDiffInTool(modelData.workspace_path)
                                        }
                                        PlasmaComponents3.Button {
                                            icon.name: "edit-copy"
                                            text: "Kopírovat"
                                            Layout.preferredHeight: 22
                                            onClicked: root.copyToClipboard(root.activeDiffContent)
                                        }
                                        PlasmaComponents3.Button {
                                            icon.name: "window-close"
                                            display: PlasmaComponents3.AbstractButton.IconOnly
                                            Layout.preferredHeight: 22
                                            Layout.preferredWidth: 22
                                            onClicked: fullRoot.diffOpenedPid = 0
                                        }
                                    }

                                    QQC2.ScrollView {
                                        Layout.fillWidth: true
                                        Layout.fillHeight: true
                                        clip: true

                                        QQC2.TextArea {
                                            readOnly: true
                                            text: root.activeDiffContent
                                            font.family: "monospace"
                                            font.pixelSize: 10
                                            color: Kirigami.Theme.textColor
                                            background: null
                                            wrapMode: TextEdit.NoWrap
                                        }
                                    }
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

                                    Rectangle {
                                        radius: 2
                                        color: modelData.provider === "claude" ? "#ea580c" : (modelData.provider === "codex" ? "#10b981" : "#3b82f6")
                                        Layout.preferredWidth: histBadgeLabel.contentWidth + 6
                                        Layout.preferredHeight: 14

                                        Text {
                                            id: histBadgeLabel
                                            anchors.centerIn: parent
                                            text: modelData.provider === "claude" ? "Claude" : (modelData.provider === "codex" ? "Codex" : "AGY")
                                            font.pixelSize: 8
                                            font.bold: true
                                            color: "#ffffff"
                                        }
                                    }

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

                        // Thinking Level (Effort) - only shown for Claude models!
                        ColumnLayout {
                            id: claudeThinkingSection
                            visible: fullRoot.isClaudeSelected
                            Layout.fillWidth: true
                            spacing: 8

                            Rectangle {
                                Layout.fillWidth: true
                                height: 1
                                color: fullRoot.borderColor
                            }

                            Text {
                                text: "Výchozí Thinking Level pro Claude (Extended Thinking):"
                                font.bold: true
                                font.pixelSize: 13
                                color: Kirigami.Theme.textColor
                            }

                            Text {
                                text: "Určuje rozpočet uvažování tokenů pro modely Claude Sonnet a Opus (od minima po maximum)."
                                font.pixelSize: 11
                                color: Kirigami.Theme.disabledTextColor
                                Layout.fillWidth: true
                                wrapMode: Text.WordWrap
                            }

                            RowLayout {
                                Layout.fillWidth: true
                                spacing: Kirigami.Units.smallSpacing

                                Repeater {
                                    model: ["low", "medium", "high", "xhigh", "max"]
                                    delegate: Rectangle {
                                        id: effortItem
                                        required property var modelData
                                        readonly property bool isSelected: ("" + (settingsData?.effort ?? "medium")).toLowerCase() === modelData

                                        Layout.fillWidth: true
                                        Layout.preferredHeight: 36
                                        radius: Kirigami.Units.smallSpacing

                                        color: isSelected ? "#ea580c" : Kirigami.Theme.alternateBackgroundColor
                                        border.color: isSelected ? "#f97316" : fullRoot.borderColor
                                        border.width: isSelected ? 2 : 1

                                        MouseArea {
                                            anchors.fill: parent
                                            cursorShape: Qt.PointingHandCursor
                                            onClicked: root.setDefaultEffort(effortItem.modelData)
                                        }

                                        RowLayout {
                                            anchors.centerIn: parent
                                            spacing: 4

                                            Kirigami.Icon {
                                                visible: effortItem.isSelected
                                                source: "checkmark"
                                                Layout.preferredWidth: 14
                                                Layout.preferredHeight: 14
                                                color: "#ffffff"
                                            }

                                            Text {
                                                text: effortItem.modelData
                                                font.bold: effortItem.isSelected
                                                font.pixelSize: 11
                                                color: effortItem.isSelected ? "#ffffff" : Kirigami.Theme.textColor
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

        // --- BOTTOM ACTIONS FOOTER (Single Primary CTA & Error Prevention) ---
        RowLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.mediumSpacing

            PlasmaComponents3.Button {
                text: "Spustit nové sezení"
                icon.name: "utilities-terminal"
                highlighted: true
                Layout.fillWidth: true
                Layout.preferredHeight: 38
                onClicked: root.launchTerminal("")
            }

            PlasmaComponents3.Button {
                text: "Zastavit vše"
                icon.name: "process-stop"
                Layout.preferredHeight: 38
                enabled: activeSessions.length > 0
                visible: activeSessions.length > 0
                PlasmaComponents3.ToolTip.text: "Nouzově ukončit všechna běžící AI CLI sezení"
                onClicked: {
                    for (var i = 0; i < fullRoot.activeSessions.length; i++) {
                        root.killSession(fullRoot.activeSessions[i].pid);
                    }
                }
            }
        }
    }
}
