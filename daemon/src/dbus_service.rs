use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use zbus::interface;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use crate::state::AgyState;

pub struct AntigravityDbus {
    pub state: Arc<Mutex<AgyState>>,
    pub settings_path: PathBuf,
}

#[interface(name = "org.kde.plasma.antigravity")]
impl AntigravityDbus {
    async fn get_state_json(&self) -> String {
        let st = match self.state.lock() {
            Ok(s) => s,
            Err(p) => p.into_inner(),
        };
        serde_json::to_string(&*st).unwrap_or_else(|_| "{}".to_string())
    }

    async fn get_active_count(&self) -> u32 {
        let st = match self.state.lock() {
            Ok(s) => s,
            Err(p) => p.into_inner(),
        };
        st.stats.total_active_agents as u32
    }

    async fn kill_session(&self, pid: u32) -> bool {
        let sessions = {
            let st = match self.state.lock() {
                Ok(s) => s,
                Err(p) => p.into_inner(),
            };
            st.active_sessions.clone()
        };

        if let Err(e) = crate::proc_monitor::verify_killable_ai_session(pid, &sessions) {
            eprintln!("[agenticom-security] Blocked illegal kill_session for PID {}: {}", pid, e);
            return false;
        }

        let target_pid = Pid::from_raw(pid as i32);
        // First try graceful SIGTERM
        if kill(target_pid, Signal::SIGTERM).is_ok() {
            true
        } else {
            // Fallback to SIGKILL
            kill(target_pid, Signal::SIGKILL).is_ok()
        }
    }

    async fn set_model(&self, model: String) -> bool {
        let display_name = crate::settings_manager::get_all_available_models()
            .into_iter()
            .find(|m| m.id == model || m.name == model)
            .map(|m| m.name)
            .unwrap_or_else(|| model.clone());
        crate::settings_manager::update_model_setting(&self.settings_path, &model, &display_name).is_ok()
    }

    async fn set_effort(&self, effort: String) -> bool {
        crate::settings_manager::update_effort_setting(&self.settings_path, &effort).is_ok()
    }

    async fn set_agent_mode(&self, mode: String) -> bool {
        crate::settings_manager::update_agent_mode(&self.settings_path, &mode).is_ok()
    }

    async fn launch_session(&self, workspace: String) -> bool {
        crate::notifications::launch_terminal_session("", &workspace)
    }

    async fn set_notifications_enabled(&self, enabled: bool) -> bool {
        crate::settings_manager::update_notifications_enabled(&self.settings_path, enabled).is_ok()
    }

    async fn set_notification_sound(&self, sound: bool) -> bool {
        crate::settings_manager::update_notification_sound(&self.settings_path, sound).is_ok()
    }

    async fn test_notification(&self) -> bool {
        let sound = {
            let st = match self.state.lock() {
                Ok(s) => s,
                Err(p) => p.into_inner(),
            };
            st.settings.notification_sound
        };

        let home_env = std::env::var("HOME").ok();
        crate::notifications::send_desktop_notification(
            "Antigravity Test",
            "Interaktivní notifikace a signalizace v KDE Plasma 6 fungují správně!",
            Some("test-session"),
            home_env.as_deref(),
            sound,
        );
        true
    }

    async fn resume_session(&self, conversation_id: String, workspace: String) -> bool {
        crate::notifications::launch_terminal_session(&conversation_id, &workspace);
        true
    }

    async fn get_git_diff(&self, workspace: String) -> String {
        crate::git_inspector::get_git_diff(&workspace, 64 * 1024)
            .unwrap_or_else(|e| format!("Chyba při načítání diffu: {}", e))
    }

    async fn rollback_git_changes(&self, workspace: String, clean_untracked: bool) -> (bool, String) {
        match crate::git_inspector::rollback_git_changes(&workspace, clean_untracked) {
            Ok(msg) => (true, msg),
            Err(e) => (false, e),
        }
    }

    async fn launch_with_prompt(
        &self,
        workspace: String,
        prompt: String,
        provider: String,
        model: String,
        effort: String,
        agent: String,
        mode: String,
    ) -> bool {
        let options = crate::steering_engine::QuickPromptOptions {
            workspace_path: workspace,
            prompt,
            conversation_id: None,
            provider,
            model: if model.is_empty() { None } else { Some(model) },
            effort: if effort.is_empty() { None } else { Some(effort) },
            agent: if agent.is_empty() { None } else { Some(agent) },
            mode: if mode.is_empty() { None } else { Some(mode) },
        };
        crate::steering_engine::execute_quick_prompt(&options).unwrap_or(false)
    }

    async fn resume_with_prompt(
        &self,
        conversation_id: String,
        workspace: String,
        prompt: String,
    ) -> bool {
        crate::steering_engine::resume_session_with_quick_prompt(&conversation_id, &workspace, &prompt, None)
            .unwrap_or(false)
    }
}
