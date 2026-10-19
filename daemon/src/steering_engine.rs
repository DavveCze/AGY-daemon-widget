use std::path::Path;
use std::process::Command;
use crate::notifications::{sanitize_conversation_id, sanitize_workspace_path};

#[derive(Debug, Clone)]
pub struct QuickPromptOptions {
    pub workspace_path: String,
    pub prompt: String,
    pub conversation_id: Option<String>,
    pub provider: String,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub agent: Option<String>,
    pub mode: Option<String>,
}

pub fn sanitize_prompt_text(prompt: &str) -> Result<String, &'static str> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        return Err("Prompt nesmí být prázdný");
    }
    if trimmed.len() > 8192 {
        return Err("Délka promptu překročila maximální povolený limit (8192 znaků)");
    }
    // Reject dangerous null bytes or terminal escape sequences
    if trimmed.chars().any(|c| c == '\0' || c == '\x1b') {
        return Err("Prompt obsahuje nepovolené řídicí nebo escape znaky");
    }
    Ok(trimmed.to_string())
}

pub fn build_cli_arguments(options: &QuickPromptOptions) -> Result<(String, Vec<String>), &'static str> {
    let prompt = sanitize_prompt_text(&options.prompt)?;
    let provider = options.provider.trim().to_lowercase();

    match provider.as_str() {
        "claude" => {
            let mut args = Vec::new();
            if let Some(ref m) = options.model {
                let m_clean = m.trim();
                if !m_clean.is_empty() {
                    args.push("--model".to_string());
                    args.push(m_clean.to_string());
                }
            }
            args.push(prompt);
            Ok(("claude".to_string(), args))
        }
        "codex" => {
            let mut args = Vec::new();
            if let Some(ref m) = options.model {
                let m_clean = m.trim();
                if !m_clean.is_empty() {
                    args.push("--model".to_string());
                    args.push(m_clean.to_string());
                }
            }
            args.push(prompt);
            Ok(("codex".to_string(), args))
        }
        _ => {
            // Default to Google Antigravity (agy)
            let mut args = Vec::new();

            if let Some(ref cid) = options.conversation_id {
                let safe_cid = sanitize_conversation_id(cid)?;
                if !safe_cid.is_empty() {
                    args.push("--conversation".to_string());
                    args.push(safe_cid);
                }
            }

            if let Some(ref a) = options.agent {
                let a_clean = a.trim();
                if !a_clean.is_empty() && a_clean != "self" {
                    args.push("--agent".to_string());
                    args.push(a_clean.to_string());
                }
            }

            if let Some(ref m) = options.model {
                let m_clean = m.trim();
                if !m_clean.is_empty() {
                    args.push("--model".to_string());
                    args.push(m_clean.to_string());
                }
            }

            if let Some(ref eff) = options.effort {
                let eff_clean = eff.trim();
                if !eff_clean.is_empty() {
                    args.push("--effort".to_string());
                    args.push(eff_clean.to_string());
                }
            }

            if let Some(ref mode) = options.mode {
                let mode_clean = mode.trim();
                if !mode_clean.is_empty() {
                    args.push("--mode".to_string());
                    args.push(mode_clean.to_string());
                }
            }

            args.push("-i".to_string());
            args.push(prompt);

            Ok(("agy".to_string(), args))
        }
    }
}

pub fn spawn_terminal_with_cli_command(
    workspace: &Path,
    binary: &str,
    args: &[String],
) -> bool {
    let ws_str = workspace.to_string_lossy().to_string();

    let spawn_terminal = |term: &str| -> bool {
        let mut cmd = Command::new(term);
        match term {
            "konsole" => {
                cmd.arg("--workdir").arg(&ws_str).arg("-e").arg(binary).args(args);
            }
            "kitty" => {
                cmd.arg("--directory").arg(&ws_str).arg(binary).args(args);
            }
            "alacritty" => {
                cmd.arg("--working-directory").arg(&ws_str).arg("-e").arg(binary).args(args);
            }
            "wezterm" => {
                cmd.arg("start").arg("--cwd").arg(&ws_str).arg("--").arg(binary).args(args);
            }
            "xterm" => {
                cmd.current_dir(workspace).arg("-e").arg(binary).args(args);
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

pub fn execute_quick_prompt(options: &QuickPromptOptions) -> Result<bool, &'static str> {
    let workspace = sanitize_workspace_path(&options.workspace_path)?;
    let (binary, args) = build_cli_arguments(options)?;

    if spawn_terminal_with_cli_command(&workspace, &binary, &args) {
        Ok(true)
    } else {
        Err("Nepodařilo se spustit žádný z podporovaných terminálů (konsole, kitty, alacritty, wezterm, xterm)")
    }
}

pub fn resume_session_with_quick_prompt(
    conversation_id: &str,
    workspace: &str,
    prompt: &str,
) -> Result<bool, &'static str> {
    let options = QuickPromptOptions {
        workspace_path: workspace.to_string(),
        prompt: prompt.to_string(),
        conversation_id: Some(conversation_id.to_string()),
        provider: "antigravity".to_string(),
        model: None,
        effort: None,
        agent: None,
        mode: None,
    };
    execute_quick_prompt(&options)
}
