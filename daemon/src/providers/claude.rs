use std::fs;
use std::path::PathBuf;
use serde_json::Value;
use crate::proc_monitor::{procfs_page_size_kb, read_proc_cwd, read_proc_memory_mb};
use crate::providers::AgentProvider;
use crate::state::{ActiveSession, ConversationItem, ProcessStats, TokenUsage};

pub struct ClaudeProvider {
    custom_binary: Option<String>,
    claude_home: PathBuf,
}

impl ClaudeProvider {
    pub fn new(custom_binary: Option<String>) -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let claude_home = PathBuf::from(home).join(".claude");
        Self {
            custom_binary,
            claude_home,
        }
    }

    fn is_claude_process(&self, cmdline: &str) -> bool {
        if cmdline.contains("agenticom-daemon")
            || cmdline.contains("agy-daemon")
            || cmdline.contains("grep")
            || cmdline.contains("/usage")
            || cmdline.contains("plasmoid")
            || cmdline.contains("kwrite")
            || cmdline.contains("kate")
        {
            return false;
        }

        if let Some(ref bin) = self.custom_binary {
            if cmdline.contains(bin) {
                return true;
            }
        }

        cmdline.starts_with("claude\0")
            || cmdline.starts_with("claude ")
            || cmdline.contains("/bin/claude")
            || cmdline.contains("/.claude/")
            || (cmdline.contains("node") && cmdline.contains("claude"))
    }

    fn check_claude_token_usage(&self, workspace_path: &str) -> TokenUsage {
        let mut usage = TokenUsage {
            context_limit: 200_000,
            ..Default::default()
        };

        // Try reading latest project transcript in ~/.claude/projects/<slug>
        let projects_dir = self.claude_home.join("projects");
        if let Ok(entries) = fs::read_dir(projects_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Check if project metadata matches workspace_path
                    let meta_file = path.join("project.json");
                    let mut matched = false;
                    if let Ok(content) = fs::read_to_string(&meta_file) {
                        if content.contains(workspace_path) {
                            matched = true;
                        }
                    }
                    if matched {
                        // Read turns / tokens from session JSON files
                        if let Ok(files) = fs::read_dir(&path) {
                            for f in files.flatten() {
                                if f.file_name().to_string_lossy().ends_with(".json") {
                                    if let Ok(c) = fs::read_to_string(f.path()) {
                                        if let Ok(json) = serde_json::from_str::<Value>(&c) {
                                            if let Some(in_t) = json.get("input_tokens").and_then(|v| v.as_u64()) {
                                                usage.input_tokens += in_t;
                                            }
                                            if let Some(out_t) = json.get("output_tokens").and_then(|v| v.as_u64()) {
                                                usage.output_tokens += out_t;
                                            }
                                            if let Some(cache_t) = json.get("cache_read_tokens").and_then(|v| v.as_u64()) {
                                                usage.cache_read_tokens += cache_t;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        break;
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

impl AgentProvider for ClaudeProvider {
    fn id(&self) -> &'static str {
        "claude"
    }

    fn display_name(&self) -> &'static str {
        "Claude Code"
    }

    fn badge_color(&self) -> &'static str {
        "#ea580c"
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

            if !self.is_claude_process(&cmdline) {
                continue;
            }

            let (workspace_path, project_name) = read_proc_cwd(pid);
            let memory_rss_mb = read_proc_memory_mb(pid, page_size_kb);
            let token_usage = self.check_claude_token_usage(&workspace_path);

            // Determine if asking user by checking stat state (sleeping in terminal)
            let stat_path = format!("/proc/{}/stat", pid);
            let mut is_asking_user = false;
            let mut not_fully_idle = false;

            if let Ok(stat) = fs::read_to_string(&stat_path) {
                // If process is in 'S' state and CPU is low, waiting for input
                if stat.contains(") S ") || stat.contains(") I ") {
                    is_asking_user = true;
                } else if stat.contains(") R ") || stat.contains(") D ") {
                    not_fully_idle = true;
                }
            }

            results.push(ActiveSession {
                pid,
                conversation_id: format!("claude-{}", pid),
                workspace_path,
                project_name: project_name.clone(),
                title: format!("Claude Session ({})", project_name),
                agent_name: "Claude Code".to_string(),
                current_mode: "agentic".to_string(),
                current_effort: "extended-thinking".to_string(),
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
                provider: "claude".to_string(),
                provider_badge_color: "#ea580c".to_string(),
            });
        }

        results
    }

    fn get_recent_conversations(&self, limit: usize) -> Vec<ConversationItem> {
        let mut items = Vec::new();
        let projects_dir = self.claude_home.join("projects");

        if let Ok(entries) = fs::read_dir(projects_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let project_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "project".to_string());

                    let modified_time = if let Ok(metadata) = fs::metadata(&path) {
                        if let Ok(mtime) = metadata.modified() {
                            let dt: chrono::DateTime<chrono::Local> = mtime.into();
                            dt.format("%Y-%m-%d %H:%M:%S").to_string()
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };

                    items.push(ConversationItem {
                        conversation_id: format!("claude-proj-{}", project_name),
                        title: format!("Claude Project ({})", project_name),
                        preview: "Anthropic Claude Code Workspace".to_string(),
                        step_count: 1,
                        last_modified_time: modified_time,
                        workspace_uris: format!("[\"file://{}\"]", path.to_string_lossy()),
                        agent_name: "Claude Code".to_string(),
                        token_usage: TokenUsage {
                            context_limit: 200_000,
                            ..Default::default()
                        },
                        provider: "claude".to_string(),
                    });

                    if items.len() >= limit {
                        break;
                    }
                }
            }
        }

        items
    }
}
