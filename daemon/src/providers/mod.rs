pub mod antigravity;
pub mod claude;
pub mod codex;

use crate::config::AgentiComConfig;
use crate::state::{ActiveSession, ConversationItem};
use std::path::Path;

pub trait AgentProvider: Send + Sync {
    /// Identifier: e.g. "antigravity", "claude", "codex"
    fn id(&self) -> &'static str;

    /// Display name: e.g. "Google Antigravity", "Claude Code", "OpenAI Codex"
    fn display_name(&self) -> &'static str;

    /// Badge color for widget UI
    fn badge_color(&self) -> &'static str;

    /// Whether this provider is enabled
    fn is_enabled(&self) -> bool;

    /// Scan and return active running sessions for this provider
    fn scan_sessions(&self) -> Vec<ActiveSession>;

    /// Get recent conversation history for this provider
    fn get_recent_conversations(&self, limit: usize) -> Vec<ConversationItem>;
}

pub struct ProviderRegistry {
    providers: Vec<Box<dyn AgentProvider>>,
}

impl ProviderRegistry {
    pub fn new(
        config: &AgentiComConfig,
        presence_dir: &Path,
        db_path: &Path,
        brain_dir: &Path,
    ) -> Self {
        let mut providers: Vec<Box<dyn AgentProvider>> = Vec::new();

        if config.providers.antigravity.enabled {
            providers.push(Box::new(antigravity::AntigravityProvider::new(
                presence_dir.to_path_buf(),
                db_path.to_path_buf(),
                brain_dir.to_path_buf(),
            )));
        }

        if config.providers.claude.enabled {
            providers.push(Box::new(claude::ClaudeProvider::new(
                config.providers.claude.custom_binary.clone(),
            )));
        }

        if config.providers.codex.enabled {
            providers.push(Box::new(codex::CodexProvider::new(
                config.providers.codex.custom_binary.clone(),
            )));
        }

        Self { providers }
    }

    pub fn scan_all_sessions(&self) -> Vec<ActiveSession> {
        let mut all = Vec::new();
        for p in &self.providers {
            if p.is_enabled() {
                all.extend(p.scan_sessions());
            }
        }
        all
    }

    pub fn get_all_recent_conversations(&self, limit: usize) -> Vec<ConversationItem> {
        let mut all = Vec::new();
        for p in &self.providers {
            if p.is_enabled() {
                all.extend(p.get_recent_conversations(limit));
            }
        }
        // Sort newest first by last_modified_time
        all.sort_by(|a, b| b.last_modified_time.cmp(&a.last_modified_time));
        if all.len() > limit {
            all.truncate(limit);
        }
        all
    }

    pub fn active_provider_ids(&self) -> Vec<String> {
        self.providers
            .iter()
            .filter(|p| p.is_enabled())
            .map(|p| p.id().to_string())
            .collect()
    }
}
