use std::fs;
use std::path::PathBuf;
use serde_json::Value;
use crate::proc_monitor::{has_child_processes, procfs_page_size_kb, read_proc_cwd, read_proc_memory_mb};
use crate::providers::AgentProvider;
use crate::state::{ActiveSession, ConversationItem, ProcessStats, TokenUsage};

pub struct CodexProvider {
    custom_binary: Option<String>,
    codex_home: PathBuf,
}

impl CodexProvider {
    pub fn new(custom_binary: Option<String>) -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let codex_home = PathBuf::from(home).join(".codex");
        Self {
            custom_binary,
            codex_home,
        }
    }

    pub fn is_codex_process(&self, cmdline: &str) -> bool {
        if cmdline.contains("agenticom-daemon")
            || cmdline.contains("agy-daemon")
            || cmdline.contains("grep")
            || cmdline.contains("plasmoid")
            || cmdline.contains("kate")
            || cmdline.contains("kwrite")
            || cmdline.contains("nvim")
            || cmdline.contains("/usage")
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
            || cmdline == "codex"
            || cmdline.contains("/bin/codex")
            || cmdline.contains("openai-codex")
            || (cmdline.contains("node") && cmdline.contains("codex"))
    }

    /// Read and aggregate token usage from Codex session transcripts
    pub fn check_codex_token_usage(&self, workspace_path: &str) -> TokenUsage {
        let mut usage = TokenUsage {
            context_limit: 128_000,
            ..Default::default()
        };

        let sessions_dir = self.codex_home.join("sessions");
        if let Ok(entries) = fs::read_dir(sessions_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if content.contains(workspace_path) {
                            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                                if let Some(m) = json.get("model").and_then(|v| v.as_str()) {
                                    if m.starts_with("o1") {
                                        usage.context_limit = 200_000;
                                    }
                                }
                                if let Some(in_t) = json.get("prompt_tokens").and_then(|v| v.as_u64()) {
                                    usage.input_tokens += in_t;
                                }
                                if let Some(out_t) = json.get("completion_tokens").and_then(|v| v.as_u64()) {
                                    usage.output_tokens += out_t;
                                }
                                if let Some(cache_t) = json.get("cached_tokens").and_then(|v| v.as_u64()) {
                                    usage.cache_read_tokens += cache_t;
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }

        usage.total_tokens = usage.input_tokens + usage.output_tokens;
        usage.current_context_tokens = usage.input_tokens + usage.cache_read_tokens;
        if usage.context_limit > 0 {
            let pct = (usage.current_context_tokens as f32 / usage.context_limit as f32) * 100.0;
            usage.session_percentage = ((pct * 10.0).round() / 10.0).clamp(0.0, 100.0);
        }

        usage
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
            let token_usage = self.check_codex_token_usage(&workspace_path);

            let stat_path = format!("/proc/{}/stat", pid);
            let mut is_asking_user = false;
            let mut not_fully_idle = false;
            let has_children = has_child_processes(pid);

            if let Ok(stat) = fs::read_to_string(&stat_path) {
                if stat.contains(") S ") || stat.contains(") I ") {
                    is_asking_user = !has_children;
                    not_fully_idle = has_children;
                } else if stat.contains(") R ") || stat.contains(") D ") {
                    not_fully_idle = true;
                }
            } else if has_children {
                not_fully_idle = true;
            }

            results.push(ActiveSession {
                pid,
                conversation_id: format!("codex-{}", pid),
                workspace_path,
                project_name: project_name.clone(),
                title: format!("Codex Session ({})", project_name),
                agent_name: "OpenAI Codex".to_string(),
                current_mode: "code".to_string(),
                current_effort: "o3-mini".to_string(),
                not_fully_idle,
                is_asking_user,
                step_count: 1,
                last_user_input_time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                stats: ProcessStats {
                    pid,
                    cpu_percent: 0.0,
                    memory_rss_mb,
                    runtime_seconds: 0,
                },
                token_usage,
                provider: "codex".to_string(),
                provider_badge_color: self.badge_color().to_string(),
                git_status: Default::default(),
                burn_rate: Default::default(),
            });
        }

        results
    }

    fn get_recent_conversations(&self, limit: usize) -> Vec<ConversationItem> {
        let mut items = Vec::new();
        let sessions_dir = self.codex_home.join("sessions");

        if let Ok(entries) = fs::read_dir(sessions_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(c) = fs::read_to_string(&path) {
                        if let Ok(json) = serde_json::from_str::<Value>(&c) {
                            let session_id = json.get("id").and_then(|v| v.as_str()).unwrap_or("codex-session").to_string();
                            let project_dir = json.get("cwd").and_then(|v| v.as_str()).unwrap_or(".").to_string();
                            let title = json.get("title").and_then(|v| v.as_str()).unwrap_or("Codex Development Task").to_string();
                            let last_mod = json.get("updated_at").and_then(|v| v.as_str()).unwrap_or("").to_string();

                            items.push(ConversationItem {
                                conversation_id: session_id,
                                title,
                                preview: "OpenAI Codex CLI Session".to_string(),
                                step_count: json.get("turns").and_then(|v| v.as_i64()).unwrap_or(1),
                                last_modified_time: last_mod,
                                workspace_uris: format!("[\"file://{}\"]", project_dir),
                                agent_name: "OpenAI Codex".to_string(),
                                token_usage: TokenUsage {
                                    context_limit: 128_000,
                                    ..Default::default()
                                },
                                provider: "codex".to_string(),
                            });

                            if items.len() >= limit {
                                break;
                            }
                        }
                    }
                }
            }
        }

        items
    }
}
