use agy_daemon::db_reader;
use agy_daemon::dbus_service;
use agy_daemon::proc_monitor;
use agy_daemon::settings_manager;
use agy_daemon::state;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use clap::{Parser, Subcommand};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::time::sleep;
use zbus::connection::Builder;

use crate::db_reader::{
    get_conversation_metadata, get_recent_conversations, open_db_readonly,
    read_conversation_token_usage,
};
use crate::dbus_service::AntigravityDbus;
use crate::proc_monitor::scan_agy_processes;
use crate::settings_manager::{
    get_all_available_agents, get_all_available_models, read_settings, scan_skills,
    update_agent_mode, update_effort_setting, update_model_setting,
    update_notifications_enabled, update_notification_sound,
};
use crate::state::{ActiveSession, AgyState, OverallStats};

#[derive(Parser, Debug)]
#[command(author = "David", version = "1.0", about = "Antigravity KDE Plasma 6 Kernel & Procfs Monitor Daemon")]
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
    presence_dir: &Path,
    db_path: &Path,
    settings_path: &Path,
    brain_dir: &Path,
    cached_skills: &[agy_daemon::state::SkillItem],
    quota: &agy_daemon::quota::RealQuotaData,
) -> AgyState {
    let detected = scan_agy_processes(presence_dir);
    let settings = read_settings(settings_path);

    // Open DB read-only
    let db_conn = open_db_readonly(db_path).ok();

    let mut active_sessions = Vec::new();
    let mut busy_count = 0;
    let mut total_memory = 0.0;

    for p in detected {
        total_memory += p.stats.memory_rss_mb;
        let mut title = String::new();
        let mut agent_name = String::from("self");
        let mut not_fully_idle = false;
        let mut step_count = 0;
        let mut last_user_input_time = String::new();

        if let Some(conn) = &db_conn {
            if let Some(meta) = get_conversation_metadata(conn, &p.conversation_id) {
                title = meta.title;
                agent_name = if meta.agent_name.is_empty() {
                    "self".to_string()
                } else {
                    meta.agent_name
                };
                not_fully_idle = meta.not_fully_idle;
                step_count = meta.step_count;
                last_user_input_time = meta.last_user_input_time;
            }
        }

        // Live real-time check from kernel procfs & live transcript:
        let mut is_asking_user = false;
        let has_children = proc_monitor::has_child_processes(p.pid);
        if let Some(live) = db_reader::get_live_transcript_status(&p.conversation_id, brain_dir) {
            not_fully_idle = live.is_busy || has_children;
            is_asking_user = live.is_asking_user;
            if live.step_index > step_count {
                step_count = live.step_index;
            }
        } else if has_children {
            not_fully_idle = true;
        }

        if not_fully_idle {
            busy_count += 1;
        }

        if title.is_empty() {
            if p.conversation_id.starts_with("cli-") {
                title = "Interaktivní CLI (čeká na zadání dotazu)".to_string();
                last_user_input_time = "Čeká na prompt".to_string();
            } else {
                title = format!("AGY sezení {}", p.project_name);
            }
        }

        let token_usage = read_conversation_token_usage(&p.conversation_id, brain_dir);

        active_sessions.push(ActiveSession {
            pid: p.pid,
            conversation_id: p.conversation_id,
            workspace_path: p.workspace_path,
            project_name: p.project_name,
            title,
            agent_name,
            current_mode: settings.agent_mode.clone(),
            current_effort: settings.effort.clone(),
            not_fully_idle,
            is_asking_user,
            step_count,
            last_user_input_time,
            stats: p.stats,
            token_usage,
        });
    }

    let recent = if let Some(conn) = &db_conn {
        get_recent_conversations(conn, 15, brain_dir)
    } else {
        Vec::new()
    };

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
    }
}

fn write_shared_memory_state(state: &AgyState, last_written: &mut String) {
    if let Ok(json_str) = serde_json::to_string(state) {
        if *last_written == json_str {
            return;
        }
        let shm_path = Path::new("/dev/shm/agy-state.json");
        let tmp_path = Path::new("/dev/shm/agy-state.json.tmp");
        if fs::write(tmp_path, json_str.as_bytes()).is_ok() {
            let _ = fs::rename(tmp_path, shm_path);
            *last_written = json_str;
        }
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
            let quota = agy_daemon::quota::fetch_quota_sync(&log_dir);
            let skills = scan_skills(&skills_dir, &builtin_skills_dir, &plugins_dir);
            let st = collect_state(
                &presence_dir,
                &db_path,
                &settings_path,
                &brain_dir,
                &skills,
                &quota,
            );
            println!("{}", serde_json::to_string_pretty(&st)?);
            return Ok(());
        }
        Commands::Kill { pid } => {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
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

    println!("[agy-daemon] Initializing Antigravity KDE Plasma 6 Daemon...");

    let initial_quota = agy_daemon::quota::fetch_quota_sync(&log_dir);
    let quota_arc = Arc::new(Mutex::new(initial_quota.clone()));

    let mut cached_skills = scan_skills(&skills_dir, &builtin_skills_dir, &plugins_dir);
    let mut last_written_json = String::new();

    let initial_state = collect_state(
        &presence_dir,
        &db_path,
        &settings_path,
        &brain_dir,
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
            _ = sleep(Duration::from_millis(1500)) => {
                tick_counter += 1;
                let current_quota = quota_arc.lock().map(|q| q.clone()).unwrap_or_default();
                let current_state = collect_state(
                    &presence_dir,
                    &db_path,
                    &settings_path,
                    &brain_dir,
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
                    &presence_dir,
                    &db_path,
                    &settings_path,
                    &brain_dir,
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
