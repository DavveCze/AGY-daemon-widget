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
        let ws = if workspace.is_empty() {
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
        } else {
            workspace
        };

        // Try spawning a terminal with agy
        let terminals = ["konsole", "kitty", "alacritty", "wezterm", "xterm"];
        for term in terminals {
            if std::process::Command::new(term)
                .arg("--workdir")
                .arg(&ws)
                .arg("-e")
                .arg("agy")
                .spawn()
                .is_ok()
            {
                return true;
            }
        }
        false
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

        crate::notifications::send_desktop_notification(
            "Antigravity Test",
            "Interaktivní notifikace a signalizace v KDE Plasma 6 fungují správně!",
            Some("test-session"),
            Some("/home/david"),
            sound,
        );
        true
    }

    async fn resume_session(&self, conversation_id: String, workspace: String) -> bool {
        crate::notifications::launch_terminal_session(&conversation_id, &workspace);
        true
    }
}
