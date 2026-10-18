use std::fs;
use crate::proc_monitor::{procfs_page_size_kb, read_proc_cwd, read_proc_memory_mb};
use crate::providers::AgentProvider;
use crate::state::{ActiveSession, ConversationItem, ProcessStats, TokenUsage};

pub struct CodexProvider {
    custom_binary: Option<String>,
}

impl CodexProvider {
    pub fn new(custom_binary: Option<String>) -> Self {
        Self { custom_binary }
    }

    fn is_codex_process(&self, cmdline: &str) -> bool {
        if cmdline.contains("agenticom-daemon")
            || cmdline.contains("agy-daemon")
            || cmdline.contains("grep")
            || cmdline.contains("plasmoid")
            || cmdline.contains("kate")
            || cmdline.contains("kwrite")
        {
            return false;
        }

        if let Some(ref bin) = self.custom_binary {
            if cmdline.contains(bin) {
                return true;
            }
        }

        cmdline.starts_with("codex\0")
            || cmdline.starts_with("codex ")
            || cmdline.contains("/bin/codex")
            || cmdline.contains("openai-codex")
    }
}

impl AgentProvider for CodexProvider {
    fn id(&self) -> &'static str {
        "codex"
    }

    fn display_name(&self) -> &'static str {
        "OpenAI Codex"
    }

    fn badge_color(&self) -> &'static str {
        "#10b981"
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn scan_sessions(&self) -> Vec<ActiveSession> {
        let mut results = Vec::new();
        let page_size_kb = procfs_page_size_kb();
        let my_pid = std::process::id();

        let proc_dir = match fs::read_dir("/proc") {
            Ok(d) => d,
            Err(_) => return results,
        };

        for entry in proc_dir.flatten() {
            let name_str = entry.file_name().to_string_lossy().to_string();
            let pid: u32 = match name_str.parse() {
                Ok(p) => p,
                Err(_) => continue,
            };

            if pid == my_pid {
                continue;
            }

            let cmdline_path = format!("/proc/{}/cmdline", pid);
            let cmdline = match fs::read(&cmdline_path) {
                Ok(c) => String::from_utf8_lossy(&c).to_string(),
                Err(_) => continue,
            };

            if !self.is_codex_process(&cmdline) {
                continue;
            }

            let (workspace_path, project_name) = read_proc_cwd(pid);
            let memory_rss_mb = read_proc_memory_mb(pid, page_size_kb);

            results.push(ActiveSession {
                pid,
                conversation_id: format!("codex-{}", pid),
                workspace_path,
                project_name: project_name.clone(),
                title: format!("Codex Session ({})", project_name),
                agent_name: "OpenAI Codex".to_string(),
                current_mode: "code".to_string(),
                current_effort: "o3-mini".to_string(),
                not_fully_idle: false,
                is_asking_user: false,
                step_count: 1,
                last_user_input_time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                stats: ProcessStats {
                    pid,
                    cpu_percent: 0.0,
                    memory_rss_mb,
                    runtime_seconds: 0,
                },
                token_usage: TokenUsage {
                    context_limit: 128_000,
                    ..Default::default()
                },
                provider: "codex".to_string(),
                provider_badge_color: "#10b981".to_string(),
            });
        }

        results
    }

    fn get_recent_conversations(&self, _limit: usize) -> Vec<ConversationItem> {
        Vec::new()
    }
}
