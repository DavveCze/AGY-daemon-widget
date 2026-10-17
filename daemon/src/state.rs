use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcessStats {
    pub pid: u32,
    pub cpu_percent: f32,
    pub memory_rss_mb: f32,
    pub runtime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub total_tokens: u64,
    pub current_context_tokens: u64,
    pub context_limit: u64,
    pub session_percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSession {
    pub pid: u32,
    pub conversation_id: String,
    pub workspace_path: String,
    pub project_name: String,
    pub title: String,
    pub agent_name: String,
    pub current_mode: String,
    pub current_effort: String,
    pub not_fully_idle: bool,
    pub step_count: i64,
    pub last_user_input_time: String,
    pub stats: ProcessStats,
    pub token_usage: TokenUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationItem {
    pub conversation_id: String,
    pub title: String,
    pub preview: String,
    pub step_count: i64,
    pub last_modified_time: String,
    pub workspace_uris: String,
    pub agent_name: String,
    pub token_usage: TokenUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsInfo {
    pub model: String,
    pub default_model: String,
    pub agent_mode: String,
    pub effort: String,
    pub sandbox: bool,
    pub notifications_enabled: bool,
    pub notification_sound: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillItem {
    pub name: String,
    pub skill_type: String, // "user", "builtin", "plugin"
    #[serde(default)]
    pub plugin_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub command: String,
    pub path: String,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelOption {
    pub id: String,
    pub name: String,
    pub provider: String, // "Gemini" | "Outsource"
    pub thinking: bool,
    #[serde(default = "default_true")]
    pub available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOption {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallStats {
    pub total_active_agents: usize,
    pub busy_agents_count: usize,
    pub total_memory_mb: f32,
    pub daemon_uptime_seconds: u64,
    pub last_updated_epoch: u64,
    pub account_email: String,
    pub plan_name: String,
    pub session_usage_percent: f32,
    pub session_reset_text: String,
    pub weekly_usage_percent: f32,
    pub weekly_reset_text: String,
    pub outsource_usage_percent: f32,
    pub outsource_reset_text: String,
    pub real_quota: crate::quota::RealQuotaData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgyState {
    pub stats: OverallStats,
    pub active_sessions: Vec<ActiveSession>,
    pub recent_conversations: Vec<ConversationItem>,
    pub settings: SettingsInfo,
    pub skills: Vec<SkillItem>,
    pub available_models: Vec<ModelOption>,
    pub available_agents: Vec<AgentOption>,
    pub real_quota: crate::quota::RealQuotaData,
}
