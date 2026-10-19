# AgentiCom User Guide (Client Manual)

Welcome to the user guide for **AgentiCom: AI CLI Monitoring System for KDE Plasma 6**. This guide provides an easy-to-follow walkthrough for daily operation without requiring deep technical knowledge.

---

## 1. Overview & Capabilities

**AgentiCom** is an elegant desktop panel widget and background monitoring system designed for KDE Plasma 6. It gives you real-time visibility and control over your autonomous AI coding agents:
* 🔵 **Google Antigravity (`agy`):** Gemini 3.8 Flash models, rapid code editing, and task planning.
* 🟠 **Anthropic Claude Code (`claude`):** Claude Sonnet and Opus with Extended Thinking mode.
* 🟢 **OpenAI Codex (`codex`):** o3-mini and o1 models tailored for specialized coding tasks.

### Key Benefits:
1. **Zero System Overhead:** When no agents are actively computing, the monitor uses 0.00% CPU and conserves your laptop battery.
2. **Instant Visual Signaling:** A pulsating indicator dot in your panel immediately informs you whether an agent is generating code (pulsing green), awaiting approval (blue/orange), or completely idle.
3. **One-Click Model & Effort Switching:** Change default models and reasoning effort directly in the UI without running command-line flags.

---

## 2. Step-by-Step Daily Usage

### How to Launch a New Agent Session
1. Click the **AgentiCom** icon in your KDE panel (located next to the system tray or clock).
2. The main popup representation opens.
3. Click the primary action button at the bottom: **"Launch New Session"**.
4. Your preferred terminal emulator (e.g. Konsole) automatically opens with the agent initialized in your project workspace.

### Monitoring Active Agent Work
Under the **"Active Sessions"** tab, each running task is displayed with a clean card:
* **Project Name & Folder:** Identifies the active repository and working directory.
* **Status Badge:**
  * `Working` (Green badge): The agent is analyzing files, generating edits, or running test suites.
  * `Waiting` (Muted/Blue badge): The agent has paused and is waiting for your confirmation or prompt.
* **Context Consumption (Token Usage):** A progress bar indicates how much of the context window (e.g. 1,000,000 tokens) is currently used.
* **RAM Memory Usage:** Displays real-time RSS memory allocated by the agent process in megabytes.

### Inspecting Changes & 1-Click Rollback
AgentiCom continuously monitors the git status of your active workspaces:
* **Git Status Badge:** Shows the active branch (e.g. `feat/predictive-token-shield-burn-rate`) and the number of uncommitted modifications (`3 changes (+42/-10)`).
* **View Diff (`Diff` button):** Click to expand an inline, monospace preview of all modifications made by the agent, or view it in a terminal.
* **1-Click Rollback (`Rollback` button):** If the agent makes a mistake, click **"Rollback"** to bring up the safety confirmation bar. You can choose whether to keep or remove new untracked files, then confirm to instantly restore your workspace to its clean git commit state.

### Predictive Token Shield & Burn Rate Radar
AgentiCom continuously computes the velocity at which your AI agents consume tokens:
* **Token Radar (Header):** Displays real-time burn velocity across all active agents (e.g. `⚡ 14.7k tok/min (↗ Accelerating)`). Hover to view estimated time remaining before the 5-hour quota reset.
* **Session Burn Rate Badge:** Each active card displays its specific consumption rate (e.g. `🔥 14.7k tok/min` or `⚡ Stable`) and estimated minutes remaining before the 1M context window fills up.
* **Proactive Token Shield Alerts:** If an agent encounters a runaway loop or excessively consumes tokens (> 75,000 tokens/min), AgentiCom sends an urgent desktop notification so you can intervene before exhausting your quota.

### Quick-Prompt Launching
Launch tasks without having to open a terminal first:
1. In the bottom footer, click the **"Rychlý prompt"** (Quick Prompt) toggle button.
2. A multi-line prompt drawer will expand. Select your desired provider (`antigravity`, `claude`, `codex`) and type your task description.
3. Click **"🚀 Spustit sezení s promptem"** – AgentiCom spawns an isolated terminal and immediately begins working on your task.

### Inline Steering & Decision Helper
* **Interactive Confirmations:** When an agent waits for user approval (`is_asking_user`), an inline decision banner appears with `[ ✅ Ano (y) ]` and `[ ❌ Ne (n) ]` buttons. You can answer directly from the widget without switching to the terminal.
* **Inline Steering Input:** Each active card has an inline prompt bar allowing you to type instructions and send them directly into the session with a single click or by pressing Enter.

### Agent Swarm & Subagent Tree View
When a primary agent delegates work to specialized subagents (e.g. *CRO Psychologist*, *DBA Guardian*, *System Architect*), AgentiCom provides real-time hierarchical visibility:
* **Team Summary Badge:** Displays total subagent count, active worker count, and aggregated team RAM consumption (e.g. `👥 Tým: 3 subagenti (3 aktivní) • Celkem RAM: 751 MB`).
* **Expandable Tree View:** Click the badge to expand an ASCII-styled tree hierarchy showing each subagent's visual persona icon, role, type, and live state (`Běží`, `Dokončeno`, `Čeká`).
* **Stop Entire Team Action:** The **"Zastavit tým"** button cleanly halts both the root session and all child processes recursively with an ergonomic safety confirmation.

### Resuming an Existing Session
If you accidentally close a terminal window, your agent continues running safely in the background.
1. Click the terminal icon (`utilities-terminal`) on the session card.
2. The terminal immediately reopens and attaches to the active task.

### Changing the Default AI Model & Reasoning Effort
1. Switch to the **"Settings"** tab at the top of the widget.
2. Select your desired model:
   * **Gemini 3.8 Flash (High / Medium / Low):** Ideal for fast daily programming and large context windows.
   * **Claude Sonnet 4.6 (Thinking):** Best suited for deep reasoning and multi-step refactoring.
   * **OpenAI o3-mini / o1:** Excels at algorithmic and mathematical problems.
3. Configuration is immediately persisted—all subsequent sessions will use your selected model.

---

## 3. Frequently Asked Questions (FAQ)

### "The widget shows 'All tasks finished', but I have an open terminal."
* **Cause:** If your terminal contains a standard shell session (like bash or zsh) or an unrelated program, AgentiCom intentionally ignores it to avoid clutter and save system resources.
* **Resolution:** Run your AI CLI agent (e.g. `agy`, `claude`, or `codex`). Within one second, the session card will automatically appear in the active list.

### "An agent is stuck or unresponsive. How do I safely terminate it?"
* Click the red stop button (`process-stop`) on the session card.
* The daemon safely sends a `SIGTERM` signal to stop the process gracefully.
* **Kernel Security Invariant:** AgentiCom has kernel UID verification and process tracking. It is physically impossible to terminate system processes (like PID 1) or software belonging to other users.

### "Can other users on this computer see my prompts or code?"
* **No.** AgentiCom exchanges data through shared memory (`/dev/shm/agenticom-state.json`) with strict POSIX mode `0600` (readable and writable only by your user account).

---

## 4. Maintenance & Support

* **Automatic Startup:** The background service `agenticom-daemon` runs automatically via `systemd --user`. No manual start is required.
* **Updating:** To update to a newer release, download the package and run `./install.sh`. Your settings and session history are fully preserved.
* **Diagnostics & Health Check:** If you need to verify system status or submit a support ticket, run:
  ```bash
  agenticom-daemon dump
  ```
  This command outputs a complete JSON snapshot of daemon health, active providers, and memory states.
