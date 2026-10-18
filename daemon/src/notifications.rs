use std::collections::HashMap;
use std::process::Command;
use tokio::process::Command as TokioCommand;

pub struct NotificationTracker {
    pub previous_agent_busy: HashMap<u32, bool>,
    pub last_notified_step: HashMap<u32, i64>,
    pub previous_5hr_pct: Option<f32>,
}

impl NotificationTracker {
    pub fn new() -> Self {
        Self {
            previous_agent_busy: HashMap::new(),
            last_notified_step: HashMap::new(),
            previous_5hr_pct: None,
        }
    }
}

pub fn send_desktop_notification(
    summary: &str,
    body: &str,
    conv_id: Option<&str>,
    workspace: Option<&str>,
    sound: bool,
) {
    let mut cmd = Command::new("notify-send");
    cmd.arg("-a")
        .arg("Antigravity")
        .arg("-i")
        .arg("antigravity")
        .arg(summary)
        .arg(body)
        .arg("-t")
        .arg("7000");

    if sound {
        cmd.arg("-h").arg("string:sound-name:message-new-instant");
    }

    if let (Some(cid), Some(ws)) = (conv_id, workspace) {
        cmd.arg("-A").arg("open=Otevřít terminál");
        let cid = cid.to_string();
        let ws = ws.to_string();

        let mut tokio_cmd = TokioCommand::from(cmd);
        tokio::spawn(async move {
            if let Ok(output) = tokio_cmd.output().await {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.trim() == "open" {
                    launch_terminal_session(&cid, &ws);
                }
            }
        });
    } else {
        let _ = cmd.spawn();
    }
}

pub fn sanitize_workspace_path(input: &str) -> Result<std::path::PathBuf, &'static str> {
    if input.is_empty() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        return Ok(std::path::PathBuf::from(home));
    }

    if input.chars().any(|c| c == '\0' || c.is_control()) {
        return Err("Path contains illegal null bytes or control characters");
    }

    let raw_path = std::path::Path::new(input);
    let canonical = std::fs::canonicalize(raw_path).map_err(|_| "Directory does not exist or cannot be resolved")?;

    if !canonical.is_dir() {
        return Err("Path is not a directory");
    }

    Ok(canonical)
}

pub fn sanitize_conversation_id(input: &str) -> Result<String, &'static str> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    if trimmed.len() > 128 {
        return Err("Invalid conversation ID length (> 128)");
    }

    if !trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.') {
        return Err("Conversation ID contains invalid characters or shell metacharacters");
    }

    Ok(trimmed.to_string())
}

pub fn launch_terminal_session(conversation_id: &str, workspace: &str) -> bool {
    let ws = match sanitize_workspace_path(workspace) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[agenticom-security] Invalid workspace path: {}", e);
            return false;
        }
    };

    let safe_cid = match sanitize_conversation_id(conversation_id) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[agenticom-security] Invalid conversation ID: {}", e);
            return false;
        }
    };

    let ws_str = ws.to_string_lossy().to_string();

    let spawn_terminal = |term: &str| -> bool {
        let mut cmd = Command::new(term);
        match term {
            "konsole" => {
                cmd.arg("--workdir").arg(&ws_str).arg("-e").arg("agy");
                if !safe_cid.is_empty() {
                    cmd.arg("--resume").arg(&safe_cid);
                }
            }
            "kitty" => {
                cmd.arg("--directory").arg(&ws_str).arg("agy");
                if !safe_cid.is_empty() {
                    cmd.arg("--resume").arg(&safe_cid);
                }
            }
            "alacritty" => {
                cmd.arg("--working-directory").arg(&ws_str).arg("-e").arg("agy");
                if !safe_cid.is_empty() {
                    cmd.arg("--resume").arg(&safe_cid);
                }
            }
            "wezterm" => {
                cmd.arg("start").arg("--cwd").arg(&ws_str).arg("--").arg("agy");
                if !safe_cid.is_empty() {
                    cmd.arg("--resume").arg(&safe_cid);
                }
            }
            "xterm" => {
                cmd.current_dir(&ws).arg("-e").arg("agy");
                if !safe_cid.is_empty() {
                    cmd.arg("--resume").arg(&safe_cid);
                }
            }
            _ => return false,
        }
        cmd.spawn().is_ok()
    };

    for term in &["konsole", "kitty", "alacritty", "wezterm", "xterm"] {
        if spawn_terminal(term) {
            return true;
        }
    }
    false
}

pub fn check_and_send_notifications(
    tracker: &mut NotificationTracker,
    state: &crate::state::AgyState,
) {
    if !state.settings.notifications_enabled {
        return;
    }

    // 1. Check agent state transition (Pracuje -> Čeká)
    let mut current_pids = std::collections::HashSet::new();
    for session in &state.active_sessions {
        current_pids.insert(session.pid);
        let was_busy = tracker.previous_agent_busy.get(&session.pid).copied().unwrap_or(false);
        let is_busy = session.not_fully_idle;

        let already_notified = tracker
            .last_notified_step
            .get(&session.pid)
            .map(|&s| s == session.step_count && s > 0)
            .unwrap_or(false);

        let proj = if session.project_name.is_empty() {
            "Antigravity".to_string()
        } else {
            session.project_name.clone()
        };

        if !already_notified {
            if session.is_asking_user {
                // Případ a): Agent se potřebuje poradit nebo čeká na odpověď uživatele (ask_question)
                tracker.last_notified_step.insert(session.pid, session.step_count);
                let summary = format!("Agent se potřebuje poradit [{}]", proj);
                let body = format!("Agent '{}' čeká na vaše rozhodnutí nebo odpověď na dotaz.", session.agent_name);
                send_desktop_notification(
                    &summary,
                    &body,
                    Some(&session.conversation_id),
                    Some(&session.workspace_path),
                    state.settings.notification_sound,
                );
            } else if was_busy && !is_busy {
                // Případ b): Agent dokončil CELOU svou práci (žádné další tool cally) a čeká na nový prompt
                tracker.last_notified_step.insert(session.pid, session.step_count);
                let summary = format!("Agent dokončil úlohu [{}]", proj);
                let body = format!("Agent '{}' dokončil celou práci a čeká na vaše pokyny.", session.agent_name);
                send_desktop_notification(
                    &summary,
                    &body,
                    Some(&session.conversation_id),
                    Some(&session.workspace_path),
                    state.settings.notification_sound,
                );
            }
        }

        tracker.previous_agent_busy.insert(session.pid, is_busy);
    }

    // Retain only currently active PIDs
    tracker.previous_agent_busy.retain(|pid, _| current_pids.contains(pid));
    tracker.last_notified_step.retain(|pid, _| current_pids.contains(pid));

    // 2. Check 5-hour quota reset
    // User condition: must have had < 95.0% remaining (<95% limit remaining) prior to reset
    let current_5hr = state.stats.real_quota.gemini_five_hour.percentage;
    if let Some(prev_pct) = tracker.previous_5hr_pct {
        if prev_pct < 95.0 && current_5hr >= 95.0 {
            let summary = "Obnova 5h AI kvóty".to_string();
            let body = format!(
                "5hodinové okno pro Gemini se obnovilo na {}%. Máte k dispozici čerstvou kapacitu.",
                current_5hr
            );
            send_desktop_notification(
                &summary,
                &body,
                None,
                None,
                state.settings.notification_sound,
            );
        }
    }
    tracker.previous_5hr_pct = Some(current_5hr);
}

