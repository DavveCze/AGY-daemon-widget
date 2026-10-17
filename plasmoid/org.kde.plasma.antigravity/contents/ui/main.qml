import QtQuick
import org.kde.plasma.plasmoid
import org.kde.plasma.core as PlasmaCore
import org.kde.plasma.plasma5support as Plasma5Support

PlasmoidItem {
    id: root

    preferredRepresentation: Plasmoid.formFactor === PlasmaCore.Types.Planar ? fullRepresentation : compactRepresentation
    Plasmoid.backgroundHints: PlasmaCore.Types.DefaultBackground | PlasmaCore.Types.ConfigurableBackground

    property var agyState: ({
        stats: {
            total_active_agents: 0,
            busy_agents_count: 0,
            total_memory_mb: 0.0,
            daemon_uptime_seconds: 0,
            last_updated_epoch: 0,
            account_email: "david.faul@seznam.cz",
            plan_name: "PRO",
            session_usage_percent: 33.0,
            session_reset_text: "resets in 1h 51m",
            weekly_usage_percent: 63.0,
            weekly_reset_text: "resets in 2d 13h",
            outsource_usage_percent: 42.0,
            outsource_reset_text: "resets in 4d 10h"
        },
        active_sessions: [],
        recent_conversations: [],
        settings: {
            model: "Gemini 3.8 Flash (Medium)",
            default_model: "gemini-3.8-flash-medium",
            agent_mode: "accept-edits",
            effort: "medium",
            sandbox: true
        },
        skills: [],
        available_models: [],
        available_agents: []
    })

    property string daemonBinPath: "/home/david/.local/bin/agy-daemon"
    property bool isDaemonActive: false

    Plasma5Support.DataSource {
        id: stateReader
        engine: "executable"
        connectedSources: []

        onNewData: (sourceName, data) => {
            disconnectSource(sourceName);
            var stdout = data["stdout"];
            if (data["exit code"] === 0 && stdout && stdout.length > 0) {
                try {
                    root.agyState = JSON.parse(stdout);
                    root.isDaemonActive = true;
                    return;
                } catch (e) {
                    console.warn("[AGY Plasmoid] Parse error:", e);
                }
            }

            // Fallback to agy-daemon dump if cat fails
            if (sourceName !== root.daemonBinPath + " dump") {
                connectSource(root.daemonBinPath + " dump");
            } else {
                root.isDaemonActive = false;
            }
        }

        function read() {
            connectSource("cat /dev/shm/agy-state.json");
        }
    }

    Plasma5Support.DataSource {
        id: commandRunner
        engine: "executable"
        connectedSources: []

        onNewData: (sourceName, data) => {
            disconnectSource(sourceName);
            stateTimer.restart();
            stateReader.read();
        }

        function run(cmd) {
            connectSource(cmd);
        }
    }

    function reloadState() {
        stateReader.read();
    }

    function killSession(pid) {
        commandRunner.run(root.daemonBinPath + " kill " + pid);
    }

    function setDefaultModel(modelId, modelName) {
        commandRunner.run(root.daemonBinPath + " set-model \"" + modelId + "\" \"" + modelName + "\"");
    }

    function setDefaultMode(modeName) {
        commandRunner.run(root.daemonBinPath + " set-mode " + modeName);
    }

    function setDefaultEffort(effortLevel) {
        commandRunner.run(root.daemonBinPath + " set-effort " + effortLevel);
    }

    function setNotificationsEnabled(enabled) {
        commandRunner.run(root.daemonBinPath + " set-notifications " + (enabled ? "true" : "false"));
    }

    function setNotificationSound(sound) {
        commandRunner.run(root.daemonBinPath + " set-notification-sound " + (sound ? "true" : "false"));
    }

    function testNotification() {
        commandRunner.run(root.daemonBinPath + " test-notification");
    }

    function openFile(path) {
        commandRunner.run(root.daemonBinPath + " open-file \"" + path + "\"");
    }

    function copyToClipboard(text) {
        commandRunner.run("wl-copy \"" + text.replace(/"/g, "\\\"") + "\"");
    }

    function launchSkillInTerminal(command, workspacePath) {
        var ws = workspacePath ? workspacePath : "/home/david";
        var safeCmd = command ? command.replace(/"/g, "\\\"") : "";
        commandRunner.run("konsole --workdir \"" + ws + "\" -e bash -c 'agy --prompt \"" + safeCmd + "\"; exec bash' &");
    }

    function launchTerminal(workspacePath) {
        var ws = workspacePath ? workspacePath : "/home/david";
        commandRunner.run("konsole --workdir \"" + ws + "\" -e agy &");
    }

    function resumeSession(conversationId, workspacePath) {
        var ws = workspacePath ? workspacePath : "/home/david";
        commandRunner.run("konsole --workdir \"" + ws + "\" -e agy --conversation " + conversationId + " &");
    }

    function resumeWithCustomSettings(conversationId, workspacePath, agent, mode, effort) {
        var ws = workspacePath ? workspacePath : "/home/david";
        var cmd = "konsole --workdir \"" + ws + "\" -e agy";
        if (agent && agent.length > 0 && agent !== "self") {
            cmd += " --agent " + agent;
        }
        if (mode && mode.length > 0) {
            cmd += " --mode " + mode;
        }
        if (effort && effort.length > 0) {
            cmd += " --effort " + effort;
        }
        cmd += " --conversation " + conversationId + " &";
        commandRunner.run(cmd);
    }

    Timer {
        id: stateTimer
        interval: 1500
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: {
            stateReader.read();
        }
    }

    compactRepresentation: CompactRepresentation {}
    fullRepresentation: FullRepresentation {}
}
