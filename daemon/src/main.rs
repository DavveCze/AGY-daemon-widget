
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use clap::{Parser, Subcommand};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::time::sleep;
use zbus::connection::Builder;

use agy_daemon::config::AgentiComConfig;
use agy_daemon::dbus_service::AntigravityDbus;
use agy_daemon::providers::ProviderRegistry;
use agy_daemon::settings_manager::{
    get_all_available_agents, get_all_available_models, read_settings, scan_skills,
    update_agent_mode, update_effort_setting, update_model_setting,
    update_notifications_enabled, update_notification_sound,
};
use agy_daemon::state::{AgyState, OverallStats};

#[derive(Parser, Debug)]
#[command(author = "David", version = "1.0", about = "AgentiCom: AI CLI Monitoring System Daemon")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run daemon in background (default)
    Run,
    /// Dump current state as JSON and exit
    Dump,
    /// Terminate an active agent session by PID
    Kill { pid: u32 },
    /// Update active model in settings (sets both defaultModel and display name)
    SetModel {
        id: String,
        name: Option<String>,
    },
    /// Update agent mode (auto, accept-edits, plan)
    SetMode { mode: String },
    /// Update thinking effort level (low, medium, high, xhigh, max)
    SetEffort { effort: String },
    /// Enable or disable desktop notifications
    SetNotifications { enabled: bool },
    /// Enable or disable notification sound
    SetNotificationSound { sound: bool },
    /// Send a test desktop notification
    TestNotification,
    /// Open a file in the user's default desktop editor
    OpenFile { path: String },
}

fn get_base_paths() -> (
    PathBuf,
    PathBuf,
    PathBuf,
    PathBuf,
    PathBuf,
    PathBuf,
    PathBuf,
    PathBuf,
) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/david".to_string());
    let base = PathBuf::from(&home).join(".gemini");
    let agy_cli = base.join("antigravity-cli");

    let presence_dir = agy_cli.join("presence");
    let db_path = agy_cli.join("conversation_summaries.db");
    let settings_path = agy_cli.join("settings.json");
    let skills_dir = agy_cli.join("skills");
    let builtin_skills_dir = agy_cli.join("builtin").join("skills");
    let plugins_dir = base.join("config").join("plugins");
    let brain_dir = agy_cli.join("brain");
    let log_dir = agy_cli.join("log");

    (
        presence_dir,
        db_path,
        settings_path,
        skills_dir,
        builtin_skills_dir,
        plugins_dir,
        brain_dir,
        log_dir,
    )
}



fn collect_state(
    registry: &ProviderRegistry,
    settings_path: &Path,
    cached_skills: &[agy_daemon::state::SkillItem],
    quota: &agy_daemon::quota::RealQuotaData,
) -> AgyState {
    let settings = read_settings(settings_path);
    let active_sessions = registry.scan_all_sessions();
    let recent = registry.get_all_recent_conversations(15);
    let active_providers = registry.active_provider_ids();

    let mut busy_count = 0;
    let mut total_memory = 0.0;

    for s in &active_sessions {
        total_memory += s.stats.memory_rss_mb;
        if s.not_fully_idle {
            busy_count += 1;
        }
    }

    let available_models = get_all_available_models();
    let available_agents = get_all_available_agents();

    let stats = OverallStats {
        total_active_agents: active_sessions.len(),
        busy_agents_count: busy_count,
        total_memory_mb: total_memory.round(),
        daemon_uptime_seconds: 0,
        last_updated_epoch: 0,
        account_email: quota.account_email.clone(),
        plan_name: quota.plan_name.clone(),
        session_usage_percent: quota.gemini_five_hour.percentage,
        session_reset_text: quota.gemini_five_hour.refreshes_in_str.clone(),
        weekly_usage_percent: quota.gemini_weekly.percentage,
        weekly_reset_text: quota.gemini_weekly.refreshes_in_str.clone(),
        outsource_usage_percent: quota.outsource_weekly.percentage,
        outsource_reset_text: quota.outsource_weekly.refreshes_in_str.clone(),
        real_quota: quota.clone(),
    };

    AgyState {
        stats,
        active_sessions,
        recent_conversations: recent,
        settings,
        skills: cached_skills.to_vec(),
        available_models,
        available_agents,
        real_quota: quota.clone(),
        active_providers,
    }
}

use std::fs::{OpenOptions, Permissions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

fn write_secure_shm_file(target_path: &Path, content: &str) -> Result<(), std::io::Error> {
    let my_pid = std::process::id();
    let tmp_path = target_path.with_file_name(format!(
        ".{}.tmp.{}",
        target_path.file_name().unwrap().to_string_lossy(),
        my_pid
    ));

    // If pre-existing file or symlink exists at tmp_path, safely remove it
    if let Ok(meta) = fs::symlink_metadata(&tmp_path) {
        if meta.file_type().is_symlink() || (meta.permissions().mode() & 0o777 != 0o600) {
            let _ = fs::remove_file(&tmp_path);
        }
    }

    // Open file with mode 0600 (owner read/write only)
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&tmp_path)?;

    file.set_permissions(Permissions::from_mode(0o600))?;
    file.write_all(content.as_bytes())?;
    file.flush()?;
    drop(file);

    // Atomic rename
    fs::rename(&tmp_path, target_path)?;

    // Ensure target path has 0600
    let _ = fs::set_permissions(target_path, Permissions::from_mode(0o600));

    Ok(())
}

fn write_shared_memory_state(state: &AgyState, last_written: &mut String) {
    if let Ok(json_str) = serde_json::to_string(state) {
        if *last_written == json_str {
            return;
        }
        let shm_path_new = Path::new("/dev/shm/agenticom-state.json");
        let shm_path_legacy = Path::new("/dev/shm/agy-state.json");
        let _ = write_secure_shm_file(shm_path_new, &json_str);
        let _ = write_secure_shm_file(shm_path_legacy, &json_str);
        *last_written = json_str;
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let (
        presence_dir,
        db_path,
        settings_path,
        skills_dir,
        builtin_skills_dir,
        plugins_dir,
        brain_dir,
        log_dir,
    ) = get_base_paths();

    match cli.command.unwrap_or(Commands::Run) {
        Commands::Dump => {
            let config_path = AgentiComConfig::get_default_config_path();
            let config = AgentiComConfig::load_or_default(&config_path);
            let registry = ProviderRegistry::new(&config, &presence_dir, &db_path, &brain_dir);
            let quota = agy_daemon::quota::fetch_quota_sync(&log_dir);
            let skills = scan_skills(&skills_dir, &builtin_skills_dir, &plugins_dir);
            let st = collect_state(
                &registry,
                &settings_path,
                &skills,
                &quota,
            );
            println!("{}", serde_json::to_string_pretty(&st)?);
            return Ok(());
        }
        Commands::Kill { pid } => {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            let config_path = AgentiComConfig::get_default_config_path();
            let config = AgentiComConfig::load_or_default(&config_path);
            let registry = ProviderRegistry::new(&config, &presence_dir, &db_path, &brain_dir);
            let active_sessions = registry.scan_all_sessions();

            if let Err(e) = agy_daemon::proc_monitor::verify_killable_ai_session(pid, &active_sessions) {
                eprintln!("[agenticom-security] Blocked illegal kill command for PID {}: {}", pid, e);
                return Ok(());
            }

            let target_pid = Pid::from_raw(pid as i32);
            if kill(target_pid, Signal::SIGTERM).is_ok() {
                println!("Successfully signaled PID {}", pid);
            } else {
                eprintln!("Failed to signal PID {}", pid);
            }
            return Ok(());
        }
        Commands::SetModel { id, name } => {
            let display_name = name.unwrap_or_else(|| {
                // Find matching display name or default to id
                get_all_available_models()
                    .into_iter()
                    .find(|m| m.id == id)
                    .map(|m| m.name)
                    .unwrap_or_else(|| id.clone())
            });

            if let Err(e) = update_model_setting(&settings_path, &id, &display_name) {
                eprintln!("Error updating model: {}", e);
            } else {
                println!("Model successfully set to '{}' (defaultModel: {})", display_name, id);
            }
            return Ok(());
        }
        Commands::SetMode { mode } => {
            if let Err(e) = update_agent_mode(&settings_path, &mode) {
                eprintln!("Error updating mode: {}", e);
            } else {
                println!("Agent mode updated to '{}'", mode);
            }
            return Ok(());
        }
        Commands::SetEffort { effort } => {
            if let Err(e) = update_effort_setting(&settings_path, &effort) {
                eprintln!("Error updating effort: {}", e);
            } else {
                println!("Reasoning effort updated to '{}'", effort);
            }
            return Ok(());
        }
        Commands::SetNotifications { enabled } => {
            if let Err(e) = update_notifications_enabled(&settings_path, enabled) {
                eprintln!("Error updating notifications setting: {}", e);
            } else {
                println!("Desktop notifications set to {}", enabled);
            }
            return Ok(());
        }
        Commands::SetNotificationSound { sound } => {
            if let Err(e) = update_notification_sound(&settings_path, sound) {
                eprintln!("Error updating notification sound setting: {}", e);
            } else {
                println!("Notification sound set to {}", sound);
            }
            return Ok(());
        }
        Commands::TestNotification => {
            let sound = read_settings(&settings_path).notification_sound;
            agy_daemon::notifications::send_desktop_notification(
                "Antigravity Test",
                "Interaktivní notifikace a signalizace v KDE Plasma 6 fungují správně!",
                Some("test-session"),
                Some("/home/david"),
                sound,
            );
            println!("Test desktop notification triggered.");
            return Ok(());
        }
        Commands::OpenFile { path } => {
            let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
            return Ok(());
        }
        Commands::Run => {}
    }

    println!("[agenticom-daemon] Initializing AgentiCom: AI CLI Monitoring System Daemon...");

    let config_path = AgentiComConfig::get_default_config_path();
    let config = AgentiComConfig::load_or_default(&config_path);
    let registry = ProviderRegistry::new(&config, &presence_dir, &db_path, &brain_dir);

    let initial_quota = agy_daemon::quota::fetch_quota_sync(&log_dir);
    let quota_arc = Arc::new(Mutex::new(initial_quota.clone()));

    let mut cached_skills = scan_skills(&skills_dir, &builtin_skills_dir, &plugins_dir);
    let mut last_written_json = String::new();

    let initial_state = collect_state(
        &registry,
        &settings_path,
        &cached_skills,
        &initial_quota,
    );
    write_shared_memory_state(&initial_state, &mut last_written_json);
    unsafe { libc::malloc_trim(0); }

    let state_arc = Arc::new(Mutex::new(initial_state));
    let dbus_service = AntigravityDbus {
        state: Arc::clone(&state_arc),
        settings_path: settings_path.clone(),
    };

    // Export D-Bus service org.kde.plasma.antigravity
    let _connection = match Builder::session()?
        .name("org.kde.plasma.antigravity")?
        .serve_at("/org/kde/plasma/antigravity", dbus_service)?
        .build()
        .await
    {
        Ok(c) => {
            println!("[agy-daemon] D-Bus service registered on 'org.kde.plasma.antigravity'");
            Some(c)
        }
        Err(e) => {
            eprintln!("[agy-daemon] Warning: Could not register D-Bus: {}. Continuing with /dev/shm fallback.", e);
            None
        }
    };

    // Spawn 300s background task for fetching quota from agy CLI without blocking
    let quota_arc_clone = Arc::clone(&quota_arc);
    let log_dir_clone = log_dir.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(300)).await;
            let log_dir_inner = log_dir_clone.clone();
            let new_quota = tokio::task::spawn_blocking(move || {
                agy_daemon::quota::fetch_quota_sync(&log_dir_inner)
            })
            .await
            .unwrap_or_default();
            if let Ok(mut lock) = quota_arc_clone.lock() {
                *lock = new_quota;
            }
        }
    });

    // Setup inotify watcher
    let (tx, mut rx) = tokio::sync::mpsc::channel(50);
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        },
        Config::default(),
    )?;

    if presence_dir.exists() {
        let _ = watcher.watch(&presence_dir, RecursiveMode::NonRecursive);
    }
    if db_path.exists() {
        let _ = watcher.watch(&db_path, RecursiveMode::NonRecursive);
    }
    if skills_dir.exists() {
        let _ = watcher.watch(&skills_dir, RecursiveMode::Recursive);
    }
    if plugins_dir.exists() {
        let _ = watcher.watch(&plugins_dir, RecursiveMode::Recursive);
    }

    println!("[agy-daemon] Monitoring started. Press Ctrl+C to exit.");

    let mut notification_tracker = agy_daemon::notifications::NotificationTracker::new();
    let mut tick_counter: u64 = 0;

    // Main loop
    loop {
        tokio::select! {
            _ = sleep(Duration::from_millis(config.poll_interval_ms)) => {
                tick_counter += 1;
                let current_quota = quota_arc.lock().map(|q| q.clone()).unwrap_or_default();
                let current_state = collect_state(
                    &registry,
                    &settings_path,
                    &cached_skills,
                    &current_quota,
                );
                write_shared_memory_state(&current_state, &mut last_written_json);
                agy_daemon::notifications::check_and_send_notifications(
                    &mut notification_tracker,
                    &current_state,
                );
                if let Ok(mut lock) = state_arc.lock() {
                    *lock = current_state;
                }
                if tick_counter % 20 == 0 {
                    unsafe { libc::malloc_trim(0); }
                }
            }
            Some(event) = rx.recv() => {
                // If skills/plugins changed, refresh cache
                let paths = event.paths;
                if paths.iter().any(|p| p.starts_with(&skills_dir) || p.starts_with(&plugins_dir)) {
                    cached_skills = scan_skills(&skills_dir, &builtin_skills_dir, &plugins_dir);
                }

                let current_quota = quota_arc.lock().map(|q| q.clone()).unwrap_or_default();
                let current_state = collect_state(
                    &registry,
                    &settings_path,
                    &cached_skills,
                    &current_quota,
                );
                write_shared_memory_state(&current_state, &mut last_written_json);
                agy_daemon::notifications::check_and_send_notifications(
                    &mut notification_tracker,
                    &current_state,
                );
                if let Ok(mut lock) = state_arc.lock() {
                    *lock = current_state;
                }
            }
        }
    }
}
