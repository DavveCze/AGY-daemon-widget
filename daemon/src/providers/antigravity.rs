use std::path::PathBuf;
use crate::db_reader::{
    get_conversation_metadata, get_live_transcript_status, get_recent_conversations,
    open_db_readonly, read_conversation_token_usage,
};
use crate::proc_monitor::scan_agy_processes;
use crate::providers::AgentProvider;
use crate::state::{ActiveSession, ConversationItem};

pub struct AntigravityProvider {
    presence_dir: PathBuf,
    db_path: PathBuf,
    brain_dir: PathBuf,
}

impl AntigravityProvider {
    pub fn new(presence_dir: PathBuf, db_path: PathBuf, brain_dir: PathBuf) -> Self {
        Self {
            presence_dir,
            db_path,
            brain_dir,
        }
    }
}

impl AgentProvider for AntigravityProvider {
    fn id(&self) -> &'static str {
        "antigravity"
    }

    fn display_name(&self) -> &'static str {
        "Google Antigravity"
    }

    fn badge_color(&self) -> &'static str {
        "#3b82f6"
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn scan_sessions(&self) -> Vec<ActiveSession> {
        let detected = scan_agy_processes(&self.presence_dir);
        let db_conn = open_db_readonly(&self.db_path).ok();

        let mut sessions = Vec::new();

        for p in detected {
            let meta = db_conn
                .as_ref()
                .and_then(|conn| get_conversation_metadata(conn, &p.conversation_id));

            let title = meta
                .as_ref()
                .map(|m| m.title.clone())
                .unwrap_or_else(|| {
                    if p.conversation_id.starts_with("cli-") {
                        "Interaktivní CLI (čeká na zadání dotazu)".to_string()
                    } else {
                        format!("AGY sezení {}", p.project_name)
                    }
                });

            let mut step_count = meta.as_ref().map(|m| m.step_count).unwrap_or(0);
            let agent_name = meta
                .as_ref()
                .map(|m| {
                    if m.agent_name.is_empty() {
                        "self".to_string()
                    } else {
                        m.agent_name.clone()
                    }
                })
                .unwrap_or_else(|| "self".to_string());

            let mut not_fully_idle = meta.as_ref().map(|m| m.not_fully_idle).unwrap_or(false);
            let last_user_input_time = meta
                .as_ref()
                .map(|m| m.last_user_input_time.clone())
                .unwrap_or_default();

            let token_usage = read_conversation_token_usage(&p.conversation_id, &self.brain_dir);

            // Live check from transcript
            let mut is_asking_user = false;
            let has_children = crate::proc_monitor::has_child_processes(p.pid);
            if let Some(live) = get_live_transcript_status(&p.conversation_id, &self.brain_dir) {
                not_fully_idle = live.is_busy || has_children;
                is_asking_user = live.is_asking_user;
                if live.step_index > step_count {
                    step_count = live.step_index;
                }
            } else if has_children {
                not_fully_idle = true;
            }

            sessions.push(ActiveSession {
                pid: p.pid,
                conversation_id: p.conversation_id,
                workspace_path: p.workspace_path,
                project_name: p.project_name,
                title,
                agent_name,
                current_mode: "auto".to_string(),
                current_effort: "medium".to_string(),
                not_fully_idle,
                is_asking_user,
                step_count,
                last_user_input_time,
                stats: p.stats,
                token_usage,
                provider: "antigravity".to_string(),
                provider_badge_color: "#3b82f6".to_string(),
            });
        }

        sessions
    }

    fn get_recent_conversations(&self, limit: usize) -> Vec<ConversationItem> {
        let db_conn = match open_db_readonly(&self.db_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let mut items = get_recent_conversations(&db_conn, limit, &self.brain_dir);
        for item in &mut items {
            item.provider = "antigravity".to_string();
        }
        items
    }
}
