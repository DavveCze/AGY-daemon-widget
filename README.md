# AgentiCom: AI CLI Monitoring System for KDE Plasma 6

[![KDE Plasma 6](https://img.shields.io/badge/KDE-Plasma%206-31a8e0?style=for-the-badge&logo=kde&logoColor=white)](https://kde.org/plasma-desktop/)
[![Qt 6](https://img.shields.io/badge/Qt-6.8%2B-41cd52?style=for-the-badge&logo=qt&logoColor=white)](https://www.qt.io/)
[![Rust](https://img.shields.io/badge/Rust-2024%20Edition-black?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Linux Native](https://img.shields.io/badge/Linux-Kernel%20inotify%20%26%20procfs-fbc02d?style=for-the-badge&logo=linux&logoColor=black)](https://www.kernel.org/)
[![Tests](https://img.shields.io/badge/Tests-62%20passed-success?style=for-the-badge)](tests/)
[![Security](https://img.shields.io/badge/Security-POSIX%200600%20%7C%20Kernel%20UID-blueviolet?style=for-the-badge)](docs/WALKTHROUGH.md)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)](LICENSE)

An ultra-responsive, modular, zero-overhead KDE Plasma 6 plasmoid and Linux procfs daemon for monitoring and controlling AI CLI coding agents:
* 🔵 **Google Antigravity CLI (`agy`)**
* 🟠 **Anthropic Claude Code (`claude`)**
* 🟢 **OpenAI Codex CLI (`codex`)**

---

## 📚 Documentation
* 📖 **[Comprehensive Technical Walkthrough](docs/WALKTHROUGH.md):** In-depth architecture, kernel security, IPC protocol, and testing strategy.
* 👤 **[Client & User Manual (English)](docs/CLIENT_MANUAL.md):** User-friendly guide for daily usage, FAQ, and troubleshooting.
* 🇨🇿 **[Uživatelská příručka (Čeština)](docs/CLIENT_MANUAL_CS.md):** Přehledný klientský manuál pro každodenní používání.

---

## 📸 Key Features & Architecture Highlights

- **⚡ Zero Idle Overhead:** Event-driven architecture powered by Linux kernel `inotify` and `/proc` polling. Consumes **0.00% CPU** when idle.
- **🛡️ Linux Kernel Security & Sandboxing:**
  - **Process Protection:** D-Bus and CLI `kill` commands verify target PIDs against active AI session whitelists and kernel UID (`/proc/{pid}/status`), preventing unauthorized signaling of system or user processes.
  - **Robust Termination Escalation:** Process halting sends `SIGTERM + SIGCONT`, allowing suspended (`T`) processes to wake and terminate cleanly, escalating to `SIGKILL + SIGCONT` if necessary.
  - **Shared Memory Isolation:** Writes state to `/dev/shm/agenticom-state.json` strictly with POSIX mode `0600` and atomic `rename`, blocking local eavesdropping and symlink attacks.
  - **Shell Injection Immunity:** Direct vector execution for terminals (`konsole`, `kitty`, `alacritty`, `wezterm`, `xterm`) without `bash -c`, with path sanitization and conversation UUID whitelist validation.
- **🔌 Multi-Provider Modular Architecture:** Pluggable provider system (`AgentProvider` trait) configurable in `~/.config/agenticom/config.json`:
  - 🔵 **Google Antigravity Provider:** Native `agy` process tracking, Gemini 3.8 Flash, 5-hour quota reset tracking, and 45+ skill indexing.
  - 🟠 **Anthropic Claude Code Provider:** Native `claude` CLI process detection, Sonnet 3.7 / Opus 4.6 token tracking, project transcripts, and user prompt alerts.
  - 🟢 **OpenAI Codex Provider:** Native detection for Codex CLI sessions (`o3-mini`, `o1`, `gpt-4o`) with subprocess tracking.

### 🌟 4 Transformative Power-User Features
1. **⚡ Phase A: Interactive Quick-Prompt Steering:**
   - Launch new tasks directly from the widget drawer with custom prompts, provider selection, and model overrides.
   - Inline decision helper allows responding to agent confirmation prompts (`y/n`) directly inside the widget card without alt-tabbing.
   - Inline prompt dispatching directly into active sessions.
2. **🛡️ Phase B: Predictive Token Shield & Burn-Rate Radar:**
   - Real-time token consumption tracking (tokens/min and context growth/min) with stable float rounding.
   - Global Burn-Rate Radar in header with color-coded velocity indicators and time-to-limit estimation (ETA).
   - Context window limit estimation and proactive desktop alerts for runaway loops (> 75k tok/min).
3. **🔍 Phase C: Smart In-Widget Git Diff & 1-Click Rollback:**
   - Real-time git status tracking (branch name, uncommitted changes, +added/-deleted line counts).
   - Inline monospace diff viewer with syntax highlighting and hunk inspection.
   - 1-Click Rollback safely restores workspace to the last clean git commit with optional deletion of untracked files.
4. **👥 Phase D: Agent Swarm & Subagent Tree View:**
   - Real-time detection of spawned subagents (e.g. CRO Psychologist, DBA Guardian, System Architect) and child worker processes.
   - ASCII-styled tree view hierarchy displaying subagent role icons, types, execution status, and aggregated RAM consumption.
   - "Zastavit tým" recursive termination stops root session and all child processes with a safety confirmation.
   - Intelligent subagent filtering hides finished/killed swarms when no workers remain active.

### 🎯 Flicker-Free State Reconciliation Engine
- **Raw State Caching:** Intercepts identical state pulses in `main.qml` to prevent unnecessary QML property re-evaluations.
- **Scroll Preservation:** Uses native `contentX`/`contentY` bindings on `Flickable` to keep reading position steady in the diff viewer and session list across 1-second background refresh ticks.
- **Input Guard:** Active typing in prompt drawers and inline search bars locks focus, eliminating cursor jumps and lost keystrokes.

---

## 🏛️ System Architecture

```mermaid
flowchart TD
    subgraph UI["KDE Plasma 6 Desktop"]
        Tray["System Tray / Panel Icon\n(Pulsing state dot & active badge)"]
        Dashboard["Full Dashboard Window\n(QML, Kirigami, Provider Pills)"]
        QuickPrompt["Quick-Prompt & Inline Steering"]
        DiffViewer["Inline Git Diff & Rollback"]
        SwarmTree["Agent Swarm Tree View"]
    end

    subgraph Memory["RAM (Linux tmpfs)"]
        ShmState["/dev/shm/agenticom-state.json\n(Atomic multi-provider state, Mode: 0600)"]
    end

    subgraph Daemon["AgentiCom Daemon (Rust 2024)"]
        Scanner["Procfs & Inotify Scanner\n(proc_monitor.rs)"]
        BurnRate["Predictive Burn-Rate Tracker\n(burn_rate_tracker.rs)"]
        GitInsp["Git Inspector\n(git_inspector.rs)"]
        SubagentTrk["Subagent Tracker\n(subagent_tracker.rs)"]
        Reg["Provider Registry\n(providers/registry.rs)"]
        DbusService["D-Bus Session Service\n(org.kde.plasma.antigravity)"]
    end

    subgraph CLI["CLI Agent Runtimes"]
        A_AGY["Google Antigravity (agy)"]
        A_Claude["Anthropic Claude Code (claude)"]
        A_Codex["OpenAI Codex (codex)"]
        Subagents["Specialized Subagents & Workers"]
    end

    Tray <--> Dashboard
    Dashboard --> QuickPrompt
    Dashboard --> DiffViewer
    Dashboard --> SwarmTree
    Dashboard <-->|Read state every 1.0s| ShmState
    Dashboard <-->|D-Bus Methods: Kill, Launch, Steering, Git, Swarm| DbusService

    Reg --> Scanner
    Reg --> BurnRate
    Reg --> GitInsp
    Reg --> SubagentTrk

    A_AGY --> Subagents
    SubagentTrk --> Subagents
    Daemon -->|Atomic secure write| ShmState
```

---

## 🚀 Installation & Quick Start

### Option 1: Quick Install (Prebuilt Binaries)

No Rust toolchain required! Simply download the release package:

```bash
# 1. Download and extract the latest release package
tar -xzf antigravity-plasma6-widget-v1.0.0-linux-x86_64.tar.gz
cd antigravity-plasma6-widget-v1.0.0-linux-x86_64

# 2. Run the installer
./install.sh
```

### Option 2: Build from Source

Requires `cargo` (Rust) and KDE Plasma 6 development tools:

```bash
# Clone the repository
git clone https://github.com/DavveCze/AgentiCom.git
cd AgentiCom

# Run the installer (compiles optimized binary)
./install.sh
```

### Selective Provider Installation:
You can choose which providers to enable during installation:
```bash
./install.sh --modules=antigravity,claude
```

---

## 🖥️ Usage & Desktop Integration

### Adding to KDE Plasma Panel
1. Right-click any empty space on your Plasma panel and select **Add Widgets...** (or press `Meta + T`).
2. Search for **AgentiCom: AI CLI Monitor**.
3. Drag the widget onto your panel or into the System Tray.

### Standalone Window Preview (For Testing)
```bash
plasmawindowed org.kde.plasma.antigravity
```

### Managing the Background Daemon
```bash
# Check service status
systemctl --user status agenticom-daemon.service

# View live daemon logs
journalctl --user -u agenticom-daemon.service -f

# Restart daemon
systemctl --user restart agenticom-daemon.service

# Dump current JSON state
agenticom-daemon dump
```

---

## 🧪 Comprehensive Verification

AgentiCom includes an automated test runner validating all system layers:

```bash
./test.sh
```

The test runner validates:
1. **Rust Unit & Integration Tests (57 Tests):** Multi-provider detection, process monitoring, SQLite readers, quota parsers, burn-rate estimation, git diff/rollback, subagent swarm hierarchy, robust termination escalation, and kernel security invariants.
2. **Plasmoid Structure & QML Linting:** Syntax validation of `main.qml`, `CompactRepresentation.qml`, and `FullRepresentation.qml` against Qt 6 and KDE Plasma 6 standards.
3. **Causal UI Scroll & State Reconciliation Tests (5 Causal Tests):** Validates raw state identity caching, scroll preservation during refresh pulses, diff viewer line retention, draft input locks, and float stabilization.
4. **D-Bus Interface Verification:** Tests live IPC responsiveness on session bus `org.kde.plasma.antigravity`.
5. **Systemd User Unit:** Verifies healthy daemon execution and background persistence.

---

## ⚙️ D-Bus API Reference

The daemon exposes an IPC interface on the session bus under `org.kde.plasma.antigravity` at `/org/kde/plasma/antigravity`:

| Method | Signature | Description |
| :--- | :--- | :--- |
| `get_state_json` | `() -> String` | Returns the complete serialized system state. |
| `get_active_count` | `() -> u32` | Returns the number of active AI sessions. |
| `kill_session` | `(u32 pid) -> bool` | Robustly terminates an AI session with SIGTERM/SIGCONT/SIGKILL escalation. |
| `kill_team` | `(u32 pid) -> bool` | Recursively terminates the root session and all its child subagents/workers. |
| `launch_session` | `(String workspace) -> bool` | Launches a new terminal session in the given workspace. |
| `resume_session` | `(String conv_id, String workspace) -> bool` | Attaches terminal to an existing conversation. |
| `resume_session_with_quick_prompt` | `(conv_id, workspace, prompt, model, agent, effort) -> bool` | Resumes or launches a session with inline steering/prompt instructions. |
| `get_git_diff` | `(String workspace) -> String` | Returns git diff of uncommitted changes in the workspace. |
| `rollback_git_changes` | `(String workspace, bool discard_untracked) -> bool` | Safely rolls back uncommitted changes to clean git state. |
| `set_model` | `(String model) -> bool` | Updates the default AI model. |
| `set_effort` | `(String effort) -> bool` | Updates reasoning effort (`low`, `medium`, `high`, `max`). |
| `set_agent_mode` | `(String mode) -> bool` | Updates agent mode (`auto`, `accept-edits`, `plan`). |

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
Contributions, feedback, and pull requests are welcome!
