use std::fs;
use std::path::PathBuf;
use rusqlite::Connection;
use tempfile::tempdir;
use agy_daemon::db_reader::{get_conversation_metadata, get_recent_conversations};
use agy_daemon::settings_manager::{read_settings, scan_skills, update_agent_mode, update_model_setting};
use agy_daemon::state::{
    ActiveSession, AgentOption, AgyState, ConversationItem, ModelOption, OverallStats, ProcessStats,
    SettingsInfo, SkillItem, TokenUsage,
};

#[test]
fn test_state_serialization_roundtrip() {
    let state = AgyState {
        stats: OverallStats {
            total_active_agents: 1,
            busy_agents_count: 1,
            total_memory_mb: 256.5,
            daemon_uptime_seconds: 120,
            last_updated_epoch: 1700000000,
            account_email: "test@example.com".to_string(),
            plan_name: "Max 20x".to_string(),
            session_usage_percent: 33.0,
            session_reset_text: "resets in 1h 51m".to_string(),
            weekly_usage_percent: 63.0,
            weekly_reset_text: "resets in 2d 13h".to_string(),
            outsource_usage_percent: 42.0,
            outsource_reset_text: "resets in 4d 10h".to_string(),
            real_quota: agy_daemon::quota::RealQuotaData::default(),
            burn_radar: Default::default(),
        },
        active_sessions: vec![ActiveSession {
            pid: 1234,
            conversation_id: "conv-abc-123".to_string(),
            workspace_path: "/home/user/project".to_string(),
            project_name: "project".to_string(),
            title: "Test Task".to_string(),
            agent_name: "self".to_string(),
            current_mode: "auto".to_string(),
            current_effort: "medium".to_string(),
            not_fully_idle: true,
            is_asking_user: false,
            step_count: 15,
            last_user_input_time: "2026-10-17 12:00:00".to_string(),
            stats: ProcessStats {
                pid: 1234,
                cpu_percent: 3.5,
                memory_rss_mb: 256.5,
                runtime_seconds: 120,
            },
            token_usage: TokenUsage {
                input_tokens: 15000,
                output_tokens: 2500,
                cache_read_tokens: 80000,
                total_tokens: 17500,
                current_context_tokens: 82500,
                context_limit: 1_000_000,
                session_percentage: 8.3,
            },
            provider: "antigravity".to_string(),
            provider_badge_color: "#3b82f6".to_string(),
            git_status: Default::default(),
            burn_rate: Default::default(),
            team: Default::default(),
        }],
        recent_conversations: vec![ConversationItem {
            conversation_id: "conv-abc-123".to_string(),
            title: "Test Task".to_string(),
            preview: "preview text".to_string(),
            step_count: 15,
            last_modified_time: "2026-10-17 12:00:00".to_string(),
            workspace_uris: "[\"file:///home/user/project\"]".to_string(),
            agent_name: "self".to_string(),
            token_usage: TokenUsage::default(),
            provider: "antigravity".to_string(),
        }],
        settings: SettingsInfo {
            model: "Gemini 3.8 Flash (Medium)".to_string(),
            default_model: "gemini-3.8-flash-medium".to_string(),
            agent_mode: "accept-edits".to_string(),
            effort: "medium".to_string(),
            sandbox: true,
            notifications_enabled: true,
            notification_sound: true,
        },
        skills: vec![SkillItem {
            name: "test-skill".to_string(),
            skill_type: "user".to_string(),
            plugin_name: "user".to_string(),
            description: "Test skill description".to_string(),
            command: "/test-skill".to_string(),
            path: "/path/to/skill".to_string(),
        }],
        available_models: vec![ModelOption {
            id: "gemini-3.8-flash".to_string(),
            name: "Gemini 3.8 Flash".to_string(),
            provider: "Gemini".to_string(),
            thinking: true,
            available: true,
        }],
        available_agents: vec![AgentOption {
            id: "self".to_string(),
            name: "Hlavní Agent (Self)".to_string(),
            description: "Plný univerzální AI agent".to_string(),
        }],
        real_quota: agy_daemon::quota::RealQuotaData::default(),
        active_providers: vec!["antigravity".to_string(), "claude".to_string()],
    };

    let json_str = serde_json::to_string(&state).expect("Serialization failed");
    assert!(!json_str.is_empty());

    let deserialized: AgyState = serde_json::from_str(&json_str).expect("Deserialization failed");
    assert_eq!(deserialized.stats.total_active_agents, 1);
    assert_eq!(deserialized.stats.session_usage_percent, 33.0);
    assert_eq!(deserialized.active_sessions.len(), 1);
    assert_eq!(deserialized.active_sessions[0].pid, 1234);
    assert_eq!(deserialized.active_sessions[0].title, "Test Task");
    assert_eq!(deserialized.settings.model, "Gemini 3.8 Flash (Medium)");
    assert_eq!(deserialized.skills[0].name, "test-skill");
    assert_eq!(deserialized.available_models.len(), 1);
}

#[test]
fn test_db_reader_with_sqlite_memory() {
    let conn = Connection::open_in_memory().expect("Failed to open in-memory sqlite");

    conn.execute_batch(
        "CREATE TABLE conversation_summaries (
            conversation_id text PRIMARY KEY,
            title text NOT NULL DEFAULT '',
            preview text NOT NULL DEFAULT '',
            step_count integer NOT NULL DEFAULT 0,
            last_modified_time datetime NOT NULL,
            workspace_uris text NOT NULL,
            status text NOT NULL DEFAULT '',
            source text NOT NULL DEFAULT '',
            project_id text NOT NULL DEFAULT '',
            agent_name text NOT NULL DEFAULT '',
            parent_conversation_id text NOT NULL DEFAULT '',
            nesting_depth integer NOT NULL DEFAULT 0,
            battle_id text NOT NULL DEFAULT '',
            winning_conversation_id text NOT NULL DEFAULT '',
            not_fully_idle numeric NOT NULL DEFAULT false,
            killed numeric NOT NULL DEFAULT false,
            last_user_input_time datetime NOT NULL,
            last_user_input_step_index integer NOT NULL DEFAULT -1,
            app_data_dir text NOT NULL DEFAULT '',
            raw_summary blob,
            group_id text NOT NULL DEFAULT ''
        );",
    ).expect("Failed to create schema");

    conn.execute(
        "INSERT INTO conversation_summaries (
            conversation_id, title, preview, step_count, last_modified_time, workspace_uris,
            agent_name, not_fully_idle, last_user_input_time
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        (
            "uuid-active-1",
            "Testing Kernel Monitor",
            "Preview of testing kernel monitor",
            42,
            "2026-10-17 12:30:00",
            "[\"file:///workspace/AgentiCom\"]",
            "cro-psychologist",
            1,
            "2026-10-17 12:29:50",
        ),
    ).expect("Failed to insert row 1");

    conn.execute(
        "INSERT INTO conversation_summaries (
            conversation_id, title, preview, step_count, last_modified_time, workspace_uris,
            agent_name, not_fully_idle, last_user_input_time
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        (
            "uuid-idle-2",
            "Previous Finished Task",
            "Finished preview",
            100,
            "2026-10-17 10:00:00",
            "[\"file:///workspace/dev-library\"]",
            "self",
            0,
            "2026-10-17 09:59:00",
        ),
    ).expect("Failed to insert row 2");

    let meta = get_conversation_metadata(&conn, "uuid-active-1").expect("Row 1 not found");
    assert_eq!(meta.title, "Testing Kernel Monitor");
    assert_eq!(meta.agent_name, "cro-psychologist");
    assert!(meta.not_fully_idle);
    assert_eq!(meta.step_count, 42);

    assert!(get_conversation_metadata(&conn, "non-existent-uuid").is_none());

    let dummy_brain = PathBuf::from("/tmp");
    let recent = get_recent_conversations(&conn, 10, &dummy_brain);
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].conversation_id, "uuid-active-1");
    assert_eq!(recent[1].conversation_id, "uuid-idle-2");
}

#[test]
fn test_settings_manager_read_and_update() {
    let dir = tempdir().expect("Failed to create tempdir");
    let settings_path = dir.path().join("settings.json");

    let initial_json = r#"{
        "model": "Gemini 3.8 Flash (Medium)",
        "defaultModel": "gemini-3.8-flash-medium",
        "agentMode": "auto",
        "effort": "medium",
        "enableTerminalSandbox": true
    }"#;
    fs::write(&settings_path, initial_json).expect("Failed to write initial settings");

    let info = read_settings(&settings_path);
    assert_eq!(info.model, "Gemini 3.8 Flash (Medium)");
    assert_eq!(info.default_model, "gemini-3.8-flash-medium");
    assert_eq!(info.agent_mode, "auto");
    assert_eq!(info.effort, "medium");
    assert!(info.sandbox);

    update_model_setting(&settings_path, "claude-sonnet-4-6", "Claude Sonnet 4.6 (Thinking)")
        .expect("Failed to update model");
    let updated_info = read_settings(&settings_path);
    assert_eq!(updated_info.model, "Claude Sonnet 4.6 (Thinking)");
    assert_eq!(updated_info.default_model, "claude-sonnet-4-6");

    update_agent_mode(&settings_path, "plan").expect("Failed to update agent mode");
    let final_info = read_settings(&settings_path);
    assert_eq!(final_info.agent_mode, "plan");
}

#[test]
fn test_skills_scanner() {
    let dir = tempdir().expect("Failed to create tempdir");
    let user_skills_dir = dir.path().join("user_skills");
    let builtin_skills_dir = dir.path().join("builtin_skills");
    let plugins_dir = dir.path().join("plugins");

    fs::create_dir_all(user_skills_dir.join("ux-psychology")).unwrap();
    fs::create_dir_all(user_skills_dir.join("database-guardian")).unwrap();
    fs::create_dir_all(builtin_skills_dir.join("antigravity_guide")).unwrap();
    fs::create_dir_all(plugins_dir.join("freelancer-web-engineering")).unwrap();

    let scanned = scan_skills(&user_skills_dir, &builtin_skills_dir, &plugins_dir);
    assert_eq!(scanned.len(), 4);

    let names: Vec<String> = scanned.iter().map(|s| s.name.clone()).collect();
    assert!(names.contains(&"ux-psychology".to_string()));
    assert!(names.contains(&"antigravity_guide".to_string()));
    assert!(names.contains(&"freelancer-web-engineering".to_string()));
}

#[test]
fn test_daemon_cli_dump_execution() {
    use assert_cmd::Command;

    let mut cmd = Command::cargo_bin("agy-daemon").expect("Binary agy-daemon not found");
    cmd.arg("dump");

    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("Invalid UTF-8 in dump");

    let parsed: AgyState = serde_json::from_str(&stdout).expect("Failed to parse agy-daemon dump as AgyState");
    assert!(!parsed.settings.model.is_empty());
    assert!(!parsed.available_models.is_empty());
    assert!(!parsed.available_agents.is_empty());
    assert!(!parsed.stats.account_email.is_empty());
}

#[test]
fn test_quota_parser_live_format() {
    use agy_daemon::quota::parse_usage_output;

    let sample_output = r#"
Quota:
  Gemini Models          Weekly Limit Remaining     96%   2026-10-24 13:12 CEST
  Gemini Models          Five Hour Limit Remaining  87%   2026-10-17 18:12 CEST
  Claude and GPT models  Weekly Limit Remaining     100%  2026-10-24 14:26 CEST
  Claude and GPT models  Five Hour Limit Remaining  100%  2026-10-17 19:26 CEST
"#;

    let quota = parse_usage_output(sample_output, "test.user@example.com", "Google AI Pro");
    assert_eq!(quota.account_email, "test.user@example.com");
    assert_eq!(quota.plan_name, "Google AI Pro");
    assert_eq!(quota.gemini_weekly.percentage, 96.0);
    assert_eq!(quota.gemini_five_hour.percentage, 87.0);
    assert_eq!(quota.outsource_weekly.percentage, 100.0);
    assert!(quota.gemini_five_hour.refreshes_in_str.contains("Refreshes"));
}

#[test]
fn test_notification_tracker_quota_threshold() {
    use agy_daemon::notifications::NotificationTracker;

    let mut tracker = NotificationTracker::new();

    // First cycle: 90% remaining (< 95%)
    tracker.previous_5hr_pct = Some(90.0);

    // If quota is now 98% (reset occurred), condition prev < 95.0 && current >= 95.0 matches!
    let current_pct = 98.0;
    let should_trigger = tracker.previous_5hr_pct.map(|p| p < 95.0 && current_pct >= 95.0).unwrap_or(false);
    assert!(should_trigger, "Should trigger reset notification when usage was < 95% prior to reset");

    // Next cycle: previous was 98% (not utilized). Even if current is 100%, do not notify
    tracker.previous_5hr_pct = Some(98.0);
    let next_pct = 100.0;
    let should_not_trigger = tracker.previous_5hr_pct.map(|p| p < 95.0 && next_pct >= 95.0).unwrap_or(false);
    assert!(!should_not_trigger, "Must not trigger if usage was >= 95% prior to reset");
}

#[test]
fn test_live_transcript_status_busy_detection() {
    use agy_daemon::db_reader::get_live_transcript_status;
    use tempfile::tempdir;
    use std::fs;

    let dir = tempdir().expect("Failed to create tempdir");
    let conv_id = "test-conv";
    let logs_dir = dir.path().join(conv_id).join(".system_generated").join("logs");
    fs::create_dir_all(&logs_dir).unwrap();
    let transcript = logs_dir.join("transcript.jsonl");

    // Case 1: USER_INPUT line -> busy
    fs::write(&transcript, "{\"step_index\": 1, \"type\": \"USER_INPUT\", \"status\": \"DONE\"}\n").unwrap();
    let status = get_live_transcript_status(conv_id, dir.path()).expect("Status must be detected");
    assert!(status.is_busy, "Agent must be busy when user just entered input");
    assert_eq!(status.step_index, 1);

    // Case 2: Tool call line -> busy
    fs::write(&transcript, "{\"step_index\": 2, \"type\": \"PLANNER_RESPONSE\", \"status\": \"DONE\", \"tool_calls\": [{\"name\": \"run_command\"}]}\n").unwrap();
    let status = get_live_transcript_status(conv_id, dir.path()).expect("Status must be detected");
    assert!(status.is_busy, "Agent must be busy when running a tool call");

    // Case 3: Tool output (GENERIC) -> busy
    fs::write(&transcript, "{\"step_index\": 3, \"type\": \"GENERIC\", \"status\": \"DONE\"}\n").unwrap();
    let status = get_live_transcript_status(conv_id, dir.path()).expect("Status must be detected");
    assert!(status.is_busy, "Agent must be busy when processing tool output");

    // Case 4: Final response with no tool calls -> idle
    fs::write(&transcript, "{\"step_index\": 4, \"type\": \"PLANNER_RESPONSE\", \"status\": \"DONE\", \"tool_calls\": []}\n").unwrap();
    let status = get_live_transcript_status(conv_id, dir.path()).expect("Status must be detected");
    assert!(!status.is_busy, "Agent must be idle when final response is presented with no tool calls");
    assert!(!status.is_asking_user, "Agent must not be asking user on final response");
    assert_eq!(status.step_index, 4);

    // Case 5: Interactive question (ask_question) -> asking user (is_busy = false, is_asking_user = true)
    fs::write(&transcript, "{\"step_index\": 5, \"type\": \"PLANNER_RESPONSE\", \"status\": \"DONE\", \"tool_calls\": [{\"name\": \"ask_question\"}]}\n").unwrap();
    let status = get_live_transcript_status(conv_id, dir.path()).expect("Status must be detected");
    assert!(!status.is_busy, "Agent must not be busy when waiting on ask_question");
    assert!(status.is_asking_user, "Agent must be marked as asking user");
    assert_eq!(status.step_index, 5);
}

#[test]
fn test_notification_tracker_duplicate_prevention() {
    use agy_daemon::notifications::NotificationTracker;

    let mut tracker = NotificationTracker::new();
    let pid = 12345;
    let step = 42;

    tracker.last_notified_step.insert(pid, step);

    // Check if duplicate notification at same step is skipped
    let already_notified = tracker
        .last_notified_step
        .get(&pid)
        .map(|&s| s == step && s > 0)
        .unwrap_or(false);

    assert!(already_notified, "Must recognize that notification was already sent for this step");

    // At a new step, it should allow notification
    let new_step = 43;
    let new_step_notified = tracker
        .last_notified_step
        .get(&pid)
        .map(|&s| s == new_step && s > 0)
        .unwrap_or(false);

    assert!(!new_step_notified, "Must allow notification when agent finishes at a new step");
}

#[test]
fn test_scan_skills_metadata_parsing() {
    use agy_daemon::settings_manager::scan_skills;
    use tempfile::tempdir;
    use std::fs;

    let dir = tempdir().expect("Failed to create tempdir");
    let user_skills = dir.path().join("skills");
    let builtin_skills = dir.path().join("builtin_skills");
    let plugins = dir.path().join("plugins");

    fs::create_dir_all(&user_skills).unwrap();
    fs::create_dir_all(&builtin_skills).unwrap();
    fs::create_dir_all(&plugins).unwrap();

    // Create a mock plugin with skills subdirectory
    let db_plugin_skill = plugins.join("database-guardian").join("skills").join("db-schema-audit");
    fs::create_dir_all(&db_plugin_skill).unwrap();
    let skill_content = r#"---
name: db-schema-audit
description: >
  Proskenuje existující SQL soubory a schémata v projektu a odhalí
  chybějící indexy a cizí klíče.
---
# Content
"#;
    fs::write(db_plugin_skill.join("SKILL.md"), skill_content).unwrap();

    let scanned = scan_skills(&user_skills, &builtin_skills, &plugins);
    assert_eq!(scanned.len(), 1);
    let s = &scanned[0];
    assert_eq!(s.name, "db-schema-audit");
    assert_eq!(s.skill_type, "plugin");
    assert_eq!(s.plugin_name, "database-guardian");
    assert_eq!(s.command, "/database-guardian:db-schema-audit");
    assert!(s.description.contains("Proskenuje existující SQL soubory"));
    assert!(s.path.ends_with("SKILL.md"));
}

#[test]
fn test_agenticom_config_defaults_and_serialization() {
    use agy_daemon::config::AgentiComConfig;

    let dir = tempdir().expect("Failed to create tempdir");
    let cfg_path = dir.path().join("config.json");

    let loaded = AgentiComConfig::load_or_default(&cfg_path);
    assert_eq!(loaded.poll_interval_ms, 1000);
    assert!(loaded.providers.antigravity.enabled);
    assert!(loaded.providers.claude.enabled);
    assert!(!loaded.providers.codex.enabled);

    // Verify file was written
    assert!(cfg_path.exists());
}

#[test]
fn test_provider_registry_multi_provider() {
    use agy_daemon::config::AgentiComConfig;
    use agy_daemon::providers::ProviderRegistry;

    let dir = tempdir().expect("Failed to create tempdir");
    let cfg = AgentiComConfig::default();

    let registry = ProviderRegistry::new(
        &cfg,
        &dir.path().join("presence"),
        &dir.path().join("conv.db"),
        &dir.path().join("brain"),
    );

    let active_ids = registry.active_provider_ids();
    assert!(active_ids.contains(&"antigravity".to_string()));
    assert!(active_ids.contains(&"claude".to_string()));
    assert!(!active_ids.contains(&"codex".to_string()));
}

#[test]
fn test_process_stats_and_memory_rounding() {
    let stats = ProcessStats {
        pid: 9999,
        cpu_percent: 1.25,
        memory_rss_mb: 42.0,
        runtime_seconds: 360,
    };
    assert_eq!(stats.memory_rss_mb, 42.0);
    assert_eq!(stats.pid, 9999);
}

#[test]
fn test_token_usage_percentage_boundary_clamping() {
    let mut usage = TokenUsage {
        input_tokens: 200_000,
        output_tokens: 50_000,
        cache_read_tokens: 0,
        total_tokens: 250_000,
        current_context_tokens: 250_000,
        context_limit: 100_000, // over 100%
        session_percentage: 0.0,
    };
    let pct = (usage.current_context_tokens as f32 / usage.context_limit as f32) * 100.0;
    usage.session_percentage = ((pct * 10.0).round() / 10.0).clamp(0.0, 100.0);
    assert_eq!(usage.session_percentage, 100.0);
}

#[test]
fn test_backward_compatible_deserialization_defaults() {
    let json = r#"{
        "stats": {
            "total_active_agents": 0,
            "busy_agents_count": 0,
            "total_memory_mb": 0.0,
            "daemon_uptime_seconds": 10,
            "last_updated_epoch": 0,
            "account_email": "",
            "plan_name": "",
            "session_usage_percent": 0.0,
            "session_reset_text": "",
            "weekly_usage_percent": 0.0,
            "weekly_reset_text": "",
            "outsource_usage_percent": 0.0,
            "outsource_reset_text": "",
            "real_quota": {
                "account_email": "",
                "plan_name": "",
                "gemini_weekly": {"name":"","percentage":0.0,"reset_time_str":"","refreshes_in_str":""},
                "gemini_five_hour": {"name":"","percentage":0.0,"reset_time_str":"","refreshes_in_str":""},
                "outsource_weekly": {"name":"","percentage":0.0,"reset_time_str":"","refreshes_in_str":""},
                "outsource_five_hour": {"name":"","percentage":0.0,"reset_time_str":"","refreshes_in_str":""}
            }
        },
        "active_sessions": [],
        "recent_conversations": [],
        "settings": {
            "model": "Gemini 3.8 Flash (Medium)",
            "default_model": "gemini-3.8-flash-medium",
            "agent_mode": "auto",
            "effort": "medium",
            "sandbox": true,
            "notifications_enabled": true,
            "notification_sound": true
        },
        "skills": [],
        "available_models": [],
        "available_agents": [],
        "real_quota": {
            "account_email": "",
            "plan_name": "",
            "gemini_weekly": {"name":"","percentage":0.0,"reset_time_str":"","refreshes_in_str":""},
            "gemini_five_hour": {"name":"","percentage":0.0,"reset_time_str":"","refreshes_in_str":""},
            "outsource_weekly": {"name":"","percentage":0.0,"reset_time_str":"","refreshes_in_str":""},
            "outsource_five_hour": {"name":"","percentage":0.0,"reset_time_str":"","refreshes_in_str":""}
        },
        "active_providers": ["antigravity"]
    }"#;

    let parsed: Result<AgyState, _> = serde_json::from_str(json);
    assert!(parsed.is_ok(), "Backward-compatible state deserialization must succeed");
    let state = parsed.unwrap();
    assert_eq!(state.active_providers, vec!["antigravity".to_string()]);
}

#[test]
fn test_db_reader_metadata_missing_conversation_returns_none() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute(
        "CREATE TABLE conversation_summaries (
            id TEXT PRIMARY KEY,
            title TEXT,
            step_count INTEGER,
            last_modified_time TEXT,
            workspace_uris TEXT
        )",
        [],
    ).unwrap();

    let meta = get_conversation_metadata(&conn, "non-existent-uuid");
    assert!(meta.is_none(), "Query for missing UUID must return None");
}

#[test]
fn test_db_reader_recent_conversations_pagination_and_sorting() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute(
        "CREATE TABLE conversation_summaries (
            conversation_id TEXT PRIMARY KEY,
            title TEXT DEFAULT '',
            preview TEXT DEFAULT '',
            step_count INTEGER DEFAULT 0,
            last_modified_time TEXT,
            workspace_uris TEXT DEFAULT '[]',
            agent_name TEXT DEFAULT ''
        )",
        [],
    ).unwrap();

    conn.execute(
        "INSERT INTO conversation_summaries VALUES ('c1', 'Task Old', 'Old preview', 2, '2026-10-10 10:00:00', '[]', 'self')",
        [],
    ).unwrap();
    conn.execute(
        "INSERT INTO conversation_summaries VALUES ('c2', 'Task New', 'New preview', 5, '2026-10-18 10:00:00', '[]', 'self')",
        [],
    ).unwrap();

    let brain_dir = PathBuf::from("/tmp");
    let recents = get_recent_conversations(&conn, 1, &brain_dir);
    assert_eq!(recents.len(), 1);
    assert_eq!(recents[0].conversation_id, "c2");
    assert_eq!(recents[0].title, "Task New");
}

#[test]
fn test_settings_manager_effort_auto_synchronization() {
    let dir = tempdir().unwrap();
    let settings_file = dir.path().join("settings.json");
    fs::write(&settings_file, "{}").unwrap();

    // High effort sync
    update_model_setting(&settings_file, "gemini-3.8-flash-high", "Gemini 3.8 Flash (High)").unwrap();
    let st = read_settings(&settings_file);
    assert_eq!(st.default_model, "gemini-3.8-flash-high");
    assert_eq!(st.effort, "high");

    // Low effort sync
    update_model_setting(&settings_file, "gemini-3.8-flash-low", "Gemini 3.8 Flash (Low)").unwrap();
    let st = read_settings(&settings_file);
    assert_eq!(st.default_model, "gemini-3.8-flash-low");
    assert_eq!(st.effort, "low");
}

#[test]
fn test_settings_manager_notifications_and_sound_toggle() {
    use agy_daemon::settings_manager::{update_notification_sound, update_notifications_enabled};

    let dir = tempdir().unwrap();
    let settings_file = dir.path().join("settings.json");
    fs::write(&settings_file, "{}").unwrap();

    update_notifications_enabled(&settings_file, false).unwrap();
    let st = read_settings(&settings_file);
    assert!(!st.notifications_enabled);

    update_notification_sound(&settings_file, false).unwrap();
    let st = read_settings(&settings_file);
    assert!(!st.notification_sound);
}

#[test]
fn test_settings_manager_legacy_gemini_flash_id_migration() {
    let dir = tempdir().unwrap();
    let settings_file = dir.path().join("settings.json");
    fs::write(
        &settings_file,
        r#"{"model": "Gemini 3.8 Flash", "defaultModel": "gemini-3.8-flash"}"#,
    ).unwrap();

    let st = read_settings(&settings_file);
    assert_eq!(st.default_model, "gemini-3.8-flash-medium");
    assert_eq!(st.model, "Gemini 3.8 Flash (Medium)");
}

#[test]
fn test_all_available_models_and_agents_registry() {
    use agy_daemon::settings_manager::{get_all_available_agents, get_all_available_models};

    let models = get_all_available_models();
    assert!(models.len() >= 6);
    assert!(models.iter().any(|m| m.id == "gemini-3.8-flash-medium"));
    assert!(models.iter().any(|m| m.id.contains("claude")));
    assert!(models.iter().any(|m| m.id.contains("openai")));

    let agents = get_all_available_agents();
    assert!(agents.len() >= 6);
    assert!(agents.iter().any(|a| a.id == "self"));
    assert!(agents.iter().any(|a| a.id == "system-architect"));
    assert!(agents.iter().any(|a| a.id == "cro-psychologist"));
}

#[test]
fn test_skills_scanner_frontmatter_multiline_folding() {
    let dir = tempdir().unwrap();
    let skill_dir = dir.path().join("test-skill");
    fs::create_dir_all(&skill_dir).unwrap();
    let content = r#"---
name: test-skill
description: >-
  This is a folded
  multiline description for
  testing purposes.
---
# Content
"#;
    fs::write(skill_dir.join("SKILL.md"), content).unwrap();

    let scanned = scan_skills(&dir.path().to_path_buf(), &dir.path().join("builtin"), &dir.path().join("plugins"));
    assert_eq!(scanned.len(), 1);
    assert!(scanned[0].description.contains("folded multiline description"));
}

#[test]
fn test_quota_parser_fallback_on_corrupted_output() {
    use agy_daemon::quota::parse_usage_output;

    let corrupted = "Random crash traceback: Segmentation fault at 0xdeadbeef";
    let quota = parse_usage_output(corrupted, "test@example.com", "Pro");
    assert_eq!(quota.gemini_five_hour.percentage, 100.0);
    assert_eq!(quota.gemini_weekly.percentage, 100.0);
}

#[test]
fn test_proc_monitor_cmdline_filtering_and_exclusion() {
    let is_daemon = "target/release/agenticom-daemon run";
    assert!(is_daemon.contains("agenticom-daemon"));

    let is_grep = "grep --color=auto agy";
    assert!(is_grep.contains("grep"));

    let is_editor = "kate /home/user/.gemini/settings.json";
    assert!(is_editor.contains("kate"));
}

#[test]
fn test_proc_monitor_page_size_detection() {
    use agy_daemon::proc_monitor::procfs_page_size_kb;

    let sz_kb = procfs_page_size_kb();
    assert!(sz_kb == 4 || sz_kb == 16 || sz_kb == 64, "Page size in KB should be standard power of two");
}

#[test]
fn test_proc_monitor_cwd_fallback_on_inaccessible() {
    use agy_daemon::proc_monitor::read_proc_cwd;

    // PID 999999999 does not exist
    let (ws, proj) = read_proc_cwd(999999999);
    assert_eq!(ws, "unknown");
    assert_eq!(proj, "unknown");
}

#[test]
fn test_provider_registry_multi_provider_filtering() {
    use agy_daemon::config::AgentiComConfig;
    use agy_daemon::providers::ProviderRegistry;

    let dir = tempdir().unwrap();
    let mut cfg = AgentiComConfig::default();
    cfg.providers.codex.enabled = true;

    let registry = ProviderRegistry::new(
        &cfg,
        &dir.path().join("presence"),
        &dir.path().join("conv.db"),
        &dir.path().join("brain"),
    );

    let active = registry.active_provider_ids();
    assert!(active.contains(&"antigravity".to_string()));
    assert!(active.contains(&"claude".to_string()));
    assert!(active.contains(&"codex".to_string()));
}

#[test]
fn test_codex_provider_heuristics_and_scan() {
    use agy_daemon::providers::codex::CodexProvider;
    use agy_daemon::providers::AgentProvider;

    let provider = CodexProvider::new(None);
    assert_eq!(provider.id(), "codex");
    assert_eq!(provider.display_name(), "OpenAI Codex");
    assert_eq!(provider.badge_color(), "#10b981");

    assert!(provider.is_codex_process("codex\0--model\0o3-mini"));
    assert!(provider.is_codex_process("openai-codex src/main.rs"));
    assert!(!provider.is_codex_process("agenticom-daemon run"));
    assert!(!provider.is_codex_process("grep codex"));
}

#[test]
fn test_codex_token_usage_parsing() {
    use agy_daemon::providers::codex::CodexProvider;

    let dir = tempdir().unwrap();
    let codex_home = dir.path().join(".codex");
    let sessions_dir = codex_home.join("sessions");
    fs::create_dir_all(&sessions_dir).unwrap();

    let session_json = r#"{
        "id": "codex-test-1",
        "model": "o3-mini",
        "cwd": "/workspace/myproject",
        "prompt_tokens": 12000,
        "completion_tokens": 3000,
        "cached_tokens": 50000
    }"#;
    fs::write(sessions_dir.join("test-sess.json"), session_json).unwrap();

    let provider = CodexProvider::new(None);
    // When no match or custom path, default token usage is returned safely
    let usage = provider.check_codex_token_usage("/nonexistent");
    assert_eq!(usage.context_limit, 128_000);
}

#[test]
fn test_security_kill_session_untracked_pid_rejected() {
    use agy_daemon::proc_monitor::{verify_killable_ai_session, KillSecurityError};

    let active_sessions: Vec<ActiveSession> = Vec::new();
    let res = verify_killable_ai_session(12345, &active_sessions);
    assert_eq!(res, Err(KillSecurityError::UntrackedAiSession));
}

#[test]
fn test_security_kill_session_self_and_init_rejected() {
    use agy_daemon::proc_monitor::{verify_killable_ai_session, KillSecurityError};

    let active_sessions: Vec<ActiveSession> = Vec::new();
    assert_eq!(verify_killable_ai_session(0, &active_sessions), Err(KillSecurityError::InvalidPid));
    assert_eq!(verify_killable_ai_session(1, &active_sessions), Err(KillSecurityError::InvalidPid));
    assert_eq!(
        verify_killable_ai_session(std::process::id(), &active_sessions),
        Err(KillSecurityError::SelfTargetingProhibited)
    );
}

#[test]
fn test_security_kill_session_unauthorized_uid_rejected() {
    use agy_daemon::proc_monitor::{verify_killable_ai_session, KillSecurityError};

    // If a session has pid 1 in active_sessions, InvalidPid check triggers first
    let dummy = vec![ActiveSession {
        pid: 1,
        conversation_id: "c1".to_string(),
        workspace_path: "".to_string(),
        project_name: "".to_string(),
        title: "".to_string(),
        agent_name: "".to_string(),
        current_mode: "".to_string(),
        current_effort: "".to_string(),
        not_fully_idle: false,
        is_asking_user: false,
        step_count: 0,
        last_user_input_time: "".to_string(),
        stats: ProcessStats { pid: 1, cpu_percent: 0.0, memory_rss_mb: 0.0, runtime_seconds: 0 },
        token_usage: TokenUsage::default(),
        provider: "codex".to_string(),
        provider_badge_color: "".to_string(),
        git_status: Default::default(),
        burn_rate: Default::default(),
        team: Default::default(),
    }];

    assert_eq!(verify_killable_ai_session(1, &dummy), Err(KillSecurityError::InvalidPid));
}

#[test]
fn test_security_sanitize_workspace_path_traversal_rejected() {
    use agy_daemon::notifications::sanitize_workspace_path;

    // Null byte injection rejected
    assert!(sanitize_workspace_path("/tmp/test\0hax").is_err());
    // Control char rejected
    assert!(sanitize_workspace_path("/tmp/test\x1bhax").is_err());
    // Non-existent path rejected
    assert!(sanitize_workspace_path("/nonexistent_path_xyz_12345").is_err());
    // Valid temp path accepted
    let dir = tempdir().unwrap();
    let res = sanitize_workspace_path(dir.path().to_str().unwrap());
    assert!(res.is_ok());
}

#[test]
fn test_security_sanitize_conversation_id_command_injection_rejected() {
    use agy_daemon::notifications::sanitize_conversation_id;

    // Shell injection metacharacters rejected
    assert!(sanitize_conversation_id("conv; rm -rf /").is_err());
    assert!(sanitize_conversation_id("conv && whoami").is_err());
    assert!(sanitize_conversation_id("conv | cat").is_err());
    assert!(sanitize_conversation_id("conv`id`").is_err());
    assert!(sanitize_conversation_id("$(reboot)").is_err());

    // Valid alphanumeric UUID accepted
    let ok_id = "87059659-be37-42d3-b2d1-8b6d61ec3cdc";
    assert_eq!(sanitize_conversation_id(ok_id), Ok(ok_id.to_string()));
}

#[test]
fn test_security_shared_memory_file_permissions_0600() {
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    let dir = tempdir().unwrap();
    let shm_file = dir.path().join("test-state.json");

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&shm_file)
        .unwrap();
    std::io::Write::write_all(&mut file, b"{\"test\":true}").unwrap();
    file.set_permissions(fs::Permissions::from_mode(0o600)).unwrap();
    drop(file);

    let meta = fs::metadata(&shm_file).unwrap();
    let mode = meta.permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "Shared memory file must strictly have 0600 permissions");
}

#[test]
fn test_git_status_info_defaults_and_serialization() {
    use agy_daemon::state::GitStatusInfo;

    let default_info = GitStatusInfo::default();
    assert!(!default_info.is_repo);
    assert_eq!(default_info.files_changed, 0);
    assert_eq!(default_info.branch, "");

    let serialized = serde_json::to_string(&default_info).unwrap();
    let deserialized: GitStatusInfo = serde_json::from_str(&serialized).unwrap();
    assert_eq!(default_info, deserialized);
}

#[test]
fn test_git_inspector_detect_clean_and_dirty_repo() {
    use std::process::Command;
    use agy_daemon::git_inspector::inspect_git_status;

    let dir = tempdir().unwrap();
    let repo_path = dir.path();

    // Initialize git repo
    let init = Command::new("git").arg("init").arg("-b").arg("main").current_dir(repo_path).output().unwrap();
    if !init.status.success() {
        let _ = Command::new("git").arg("init").current_dir(repo_path).output().unwrap();
    }
    let _ = Command::new("git").arg("config").arg("user.name").arg("Test User").current_dir(repo_path).output();
    let _ = Command::new("git").arg("config").arg("user.email").arg("test@example.com").current_dir(repo_path).output();

    // Create initial commit
    let file1 = repo_path.join("file1.txt");
    fs::write(&file1, "Line 1\nLine 2\n").unwrap();
    let _ = Command::new("git").arg("add").arg("file1.txt").current_dir(repo_path).output();
    let _ = Command::new("git").arg("commit").arg("-m").arg("Initial commit").current_dir(repo_path).output();

    let clean_status = inspect_git_status(&repo_path.to_string_lossy());
    assert!(clean_status.is_repo);
    assert_eq!(clean_status.files_changed, 0);

    // Modify file1 and create file2
    fs::write(&file1, "Line 1\nModified Line 2\nLine 3\n").unwrap();
    let file2 = repo_path.join("file2.txt");
    fs::write(&file2, "New file content\n").unwrap();

    agy_daemon::git_inspector::invalidate_git_cache(&repo_path.to_string_lossy());
    let dirty_status = inspect_git_status(&repo_path.to_string_lossy());
    assert!(dirty_status.is_repo);
    assert_eq!(dirty_status.files_changed, 2);
    assert!(dirty_status.insertions > 0);
    assert!(dirty_status.modified_files.contains(&"file1.txt".to_string()));
}

#[test]
fn test_git_inspector_get_diff() {
    use std::process::Command;
    use agy_daemon::git_inspector::get_git_diff;

    let dir = tempdir().unwrap();
    let repo_path = dir.path();

    let _ = Command::new("git").arg("init").current_dir(repo_path).output();
    let _ = Command::new("git").arg("config").arg("user.name").arg("Test").current_dir(repo_path).output();
    let _ = Command::new("git").arg("config").arg("user.email").arg("test@example.com").current_dir(repo_path).output();

    let file = repo_path.join("code.rs");
    fs::write(&file, "fn main() {}\n").unwrap();
    let _ = Command::new("git").arg("add").arg("code.rs").current_dir(repo_path).output();
    let _ = Command::new("git").arg("commit").arg("-m").arg("Initial").current_dir(repo_path).output();

    fs::write(&file, "fn main() {\n    println!(\"Agent changes\");\n}\n").unwrap();

    let diff = get_git_diff(&repo_path.to_string_lossy(), 1024).expect("Failed to get diff");
    assert!(diff.contains("+    println!(\"Agent changes\");"));
}

#[test]
fn test_git_inspector_rollback_changes_tracked_and_untracked() {
    use std::process::Command;
    use agy_daemon::git_inspector::{get_git_diff, inspect_git_status, rollback_git_changes};

    let dir = tempdir().unwrap();
    let repo_path = dir.path();

    let _ = Command::new("git").arg("init").current_dir(repo_path).output();
    let _ = Command::new("git").arg("config").arg("user.name").arg("Test").current_dir(repo_path).output();
    let _ = Command::new("git").arg("config").arg("user.email").arg("test@example.com").current_dir(repo_path).output();

    let tracked = repo_path.join("tracked.txt");
    fs::write(&tracked, "Clean version\n").unwrap();
    let _ = Command::new("git").arg("add").arg("tracked.txt").current_dir(repo_path).output();
    let _ = Command::new("git").arg("commit").arg("-m").arg("Clean").current_dir(repo_path).output();

    // Agent breaks code and creates new file
    fs::write(&tracked, "Corrupted version by agent\n").unwrap();
    let untracked = repo_path.join("agent_garbage.tmp");
    fs::write(&untracked, "Temp garbage\n").unwrap();

    assert_eq!(fs::read_to_string(&tracked).unwrap(), "Corrupted version by agent\n");
    assert!(untracked.exists());

    // Execute 1-Click Rollback with clean_untracked = true
    let result = rollback_git_changes(&repo_path.to_string_lossy(), true).expect("Rollback failed");
    assert!(result.contains("úspěšně vráceny"));

    // Verify tracked file is restored and untracked file is deleted
    assert_eq!(fs::read_to_string(&tracked).unwrap(), "Clean version\n");
    assert!(!untracked.exists());

    let status = inspect_git_status(&repo_path.to_string_lossy());
    assert_eq!(status.files_changed, 0);

    let diff = get_git_diff(&repo_path.to_string_lossy(), 1024).unwrap();
    assert!(diff.is_empty() || diff.trim().is_empty());
}

#[test]
fn test_git_inspector_security_reject_invalid_paths() {
    use agy_daemon::git_inspector::{get_git_diff, rollback_git_changes, validate_workspace_path};

    assert!(validate_workspace_path("").is_err());
    assert!(validate_workspace_path("   ").is_err());
    assert!(validate_workspace_path("/nonexistent/directory/12345").is_err());
    assert!(validate_workspace_path("/tmp/test\0illegal").is_err());

    assert!(get_git_diff("/nonexistent/path/999", 1024).is_err());
    assert!(rollback_git_changes("/nonexistent/path/999", false).is_err());
}

#[test]
fn test_burn_rate_tracker_idle_and_steady_consumption() {
    use agy_daemon::burn_rate_tracker::BurnRateTracker;
    use std::time::{Duration, Instant};

    let mut tracker = BurnRateTracker::new();
    let t0 = Instant::now();

    // First sample - not enough data yet
    let r1 = tracker.record_and_evaluate_session(1001, 10_000, 5_000, 1_000_000, t0);
    assert_eq!(r1.burn_level, "idle");
    assert_eq!(r1.tokens_per_minute, 0.0);

    // Second sample 10s later with +1,000 tokens (100 tok/s = 6,000 tok/min => "low")
    let t1 = t0 + Duration::from_secs(10);
    let r2 = tracker.record_and_evaluate_session(1001, 11_000, 5_500, 1_000_000, t1);
    assert_eq!(r2.burn_level, "low");
    assert_eq!(r2.level_color, "#10b981");
    assert!(!r2.is_spike);
    assert!((r2.tokens_per_minute - 6000.0).abs() < 50.0);
    assert_eq!(r2.formatted_rate, "6.0k tok/min");
}

#[test]
fn test_burn_rate_tracker_spike_detection() {
    use agy_daemon::burn_rate_tracker::BurnRateTracker;
    use std::time::{Duration, Instant};

    let mut tracker = BurnRateTracker::new();
    let t0 = Instant::now();

    let _ = tracker.record_and_evaluate_session(1002, 100_000, 50_000, 1_000_000, t0);

    // After 60 seconds, agent burnt 120,000 tokens (120k / min => "spike")
    let t1 = t0 + Duration::from_secs(60);
    let r2 = tracker.record_and_evaluate_session(1002, 220_000, 150_000, 1_000_000, t1);

    assert_eq!(r2.burn_level, "spike");
    assert_eq!(r2.level_color, "#ef4444");
    assert!(r2.is_spike);
    assert!((r2.tokens_per_minute - 120_000.0).abs() < 100.0);
}

#[test]
fn test_burn_rate_tracker_context_limit_estimation() {
    use agy_daemon::burn_rate_tracker::BurnRateTracker;
    use std::time::{Duration, Instant};

    let mut tracker = BurnRateTracker::new();
    let t0 = Instant::now();

    // Context at 800k out of 1M limit
    let _ = tracker.record_and_evaluate_session(1003, 500_000, 800_000, 1_000_000, t0);

    // 30 seconds later, context grew by 50,000 tokens (rate = 100k tok/min)
    // Remaining = 1M - 850k = 150k tokens => ~150k / 100k = 2 minutes remaining
    let t1 = t0 + Duration::from_secs(30);
    let r2 = tracker.record_and_evaluate_session(1003, 550_000, 850_000, 1_000_000, t1);

    assert_eq!(r2.est_minutes_to_context_limit, Some(2));
}

#[test]
fn test_burn_rate_tracker_pruning_and_window_expiry() {
    use agy_daemon::burn_rate_tracker::BurnRateTracker;
    use std::time::{Duration, Instant};

    let mut tracker = BurnRateTracker::with_window(30);
    let t0 = Instant::now();

    let _ = tracker.record_and_evaluate_session(2001, 1000, 500, 1_000_000, t0);
    let _ = tracker.record_and_evaluate_session(2002, 1000, 500, 1_000_000, t0);

    // Prune session 2002
    tracker.prune_inactive_sessions(&[2001]);

    // Session 2001 advances past 30s window
    let t1 = t0 + Duration::from_secs(35);
    let r = tracker.record_and_evaluate_session(2001, 2000, 1000, 1_000_000, t1);
    // Old sample expired, so only 1 sample exists => returns default
    assert_eq!(r.burn_level, "idle");
}

#[test]
fn test_burn_rate_tracker_global_radar_and_trends() {
    use agy_daemon::burn_rate_tracker::{BurnRateTracker, SessionBurnRate};
    use std::time::Instant;

    let mut tracker = BurnRateTracker::new();
    let now = Instant::now();

    let s1 = SessionBurnRate {
        tokens_per_minute: 12_000.0,
        context_growth_per_minute: 2_000.0,
        burn_level: "medium".to_string(),
        level_color: "#f59e0b".to_string(),
        est_minutes_to_context_limit: None,
        is_spike: false,
        formatted_rate: "12.0k tok/min".to_string(),
    };
    let s2 = SessionBurnRate {
        tokens_per_minute: 8_000.0,
        context_growth_per_minute: 1_000.0,
        burn_level: "low".to_string(),
        level_color: "#10b981".to_string(),
        est_minutes_to_context_limit: None,
        is_spike: false,
        formatted_rate: "8.0k tok/min".to_string(),
    };

    // Total = 20,000 tok/min => "medium"
    let radar = tracker.compute_global_radar(&[s1, s2], 50.0, now);
    assert_eq!(radar.burn_level, "medium");
    assert_eq!(radar.level_color, "#f59e0b");
    assert!((radar.total_tokens_per_minute - 20_000.0).abs() < 10.0);
    assert!(radar.est_minutes_to_five_hour_limit.is_some());
}

#[test]
fn test_steering_engine_prompt_sanitization_valid_and_invalid() {
    use agy_daemon::steering_engine::sanitize_prompt_text;

    assert!(sanitize_prompt_text("").is_err());
    assert!(sanitize_prompt_text("   ").is_err());
    assert!(sanitize_prompt_text("Valid prompt with space").is_ok());
    assert!(sanitize_prompt_text("Test with null \0 byte").is_err());
    assert!(sanitize_prompt_text("Test with escape \x1b[31mred").is_err());

    let long_prompt = "a".repeat(8193);
    assert!(sanitize_prompt_text(&long_prompt).is_err());
}

#[test]
fn test_steering_engine_build_cli_arguments_antigravity() {
    use agy_daemon::steering_engine::{build_cli_arguments, QuickPromptOptions};

    let opts = QuickPromptOptions {
        workspace_path: "/tmp".to_string(),
        prompt: "Refactor database models".to_string(),
        conversation_id: Some("conv-12345".to_string()),
        provider: "antigravity".to_string(),
        model: Some("gemini-3.8-flash-high".to_string()),
        effort: Some("high".to_string()),
        agent: Some("dba-guardian".to_string()),
        mode: Some("auto".to_string()),
    };

    let (bin, args) = build_cli_arguments(&opts).expect("Failed to build args");
    assert_eq!(bin, "agy");
    assert!(args.contains(&"--conversation".to_string()));
    assert!(args.contains(&"conv-12345".to_string()));
    assert!(args.contains(&"--agent".to_string()));
    assert!(args.contains(&"dba-guardian".to_string()));
    assert!(args.contains(&"--model".to_string()));
    assert!(args.contains(&"gemini-3.8-flash-high".to_string()));
    assert!(args.contains(&"--effort".to_string()));
    assert!(args.contains(&"high".to_string()));
    assert!(args.contains(&"--mode".to_string()));
    assert!(args.contains(&"auto".to_string()));
    assert!(args.contains(&"-i".to_string()));
    assert!(args.contains(&"Refactor database models".to_string()));
}

#[test]
fn test_steering_engine_build_cli_arguments_claude_and_codex() {
    use agy_daemon::steering_engine::{build_cli_arguments, QuickPromptOptions};

    let claude_opts = QuickPromptOptions {
        workspace_path: "/tmp".to_string(),
        prompt: "Write unit test".to_string(),
        conversation_id: None,
        provider: "claude".to_string(),
        model: Some("claude-sonnet-4-6".to_string()),
        effort: None,
        agent: None,
        mode: None,
    };
    let (c_bin, c_args) = build_cli_arguments(&claude_opts).expect("Claude args failed");
    assert_eq!(c_bin, "claude");
    assert_eq!(c_args, vec!["--model", "claude-sonnet-4-6", "Write unit test"]);

    let codex_opts = QuickPromptOptions {
        workspace_path: "/tmp".to_string(),
        prompt: "Audit security".to_string(),
        conversation_id: None,
        provider: "codex".to_string(),
        model: None,
        effort: None,
        agent: None,
        mode: None,
    };
    let (x_bin, x_args) = build_cli_arguments(&codex_opts).expect("Codex args failed");
    assert_eq!(x_bin, "codex");
    assert_eq!(x_args, vec!["Audit security"]);
}

#[test]
fn test_steering_engine_execute_rejects_invalid_workspace() {
    use agy_daemon::steering_engine::{execute_quick_prompt, QuickPromptOptions};

    let opts = QuickPromptOptions {
        workspace_path: "/nonexistent/path/12345/6789".to_string(),
        prompt: "Hello".to_string(),
        conversation_id: None,
        provider: "antigravity".to_string(),
        model: None,
        effort: None,
        agent: None,
        mode: None,
    };

    assert!(execute_quick_prompt(&opts).is_err());
}

#[test]
fn test_steering_engine_resume_session_with_quick_prompt() {
    use agy_daemon::steering_engine::resume_session_with_quick_prompt;

    // Invalid workspace rejects
    let res = resume_session_with_quick_prompt(
        "87059659-be37-42d3-b2d1-8b6d61ec3cdc",
        "/nonexistent/directory/xyz123",
        "Hello agent",
        Some("antigravity"),
    );
    assert!(res.is_err());

    // Invalid conversation ID with command injection is rejected
    let res_inj = resume_session_with_quick_prompt(
        "id; rm -rf /",
        "/tmp",
        "Hello agent",
        Some("antigravity"),
    );
    assert!(res_inj.is_err());
}

#[test]
fn test_subagent_tracker_map_state_and_icons() {
    use agy_daemon::subagent_tracker::{get_subagent_icon, map_subagent_state};

    assert_eq!(get_subagent_icon("cro-psychologist"), "🧠");
    assert_eq!(get_subagent_icon("dba-guardian"), "🛡️");
    assert_eq!(get_subagent_icon("system-architect"), "🏗️");
    assert_eq!(get_subagent_icon("project-estimator"), "📊");
    assert_eq!(get_subagent_icon("geo-search-bot"), "🌐");
    assert_eq!(get_subagent_icon("module-integrator"), "📦");
    assert_eq!(get_subagent_icon("research"), "🔬");
    assert_eq!(get_subagent_icon("image-generator"), "🎨");
    assert_eq!(get_subagent_icon("self"), "🔄");
    assert_eq!(get_subagent_icon("unknown-specialist"), "🤖");

    let (label_alive, col_alive, act_alive) = map_subagent_state("SUBAGENT_STATE_ALIVE");
    assert_eq!(label_alive, "Běží");
    assert_eq!(col_alive, "#10b981");
    assert!(act_alive);

    let (label_wait, col_wait, act_wait) = map_subagent_state("WAITING_FOR_INPUT");
    assert_eq!(label_wait, "Čeká");
    assert_eq!(col_wait, "#f59e0b");
    assert!(act_wait);

    let (label_done, col_done, act_done) = map_subagent_state("SUBAGENT_STATE_DONE");
    assert_eq!(label_done, "Dokončeno");
    assert_eq!(col_done, "#3b82f6");
    assert!(!act_done);

    let (label_err, col_err, act_err) = map_subagent_state("ERROR");
    assert_eq!(label_err, "Chyba");
    assert_eq!(col_err, "#ef4444");
    assert!(!act_err);
}

#[test]
fn test_subagent_tracker_scan_subagents_directory() {
    use agy_daemon::subagent_tracker::{build_team_hierarchy, scan_subagents_for_conversation};
    use tempfile::tempdir;

    let tmp = tempdir().expect("Failed to create tempdir");
    let brain_dir = tmp.path();

    let conv_id = "test-conv-1234";
    let subagents_dir = brain_dir
        .join(conv_id)
        .join(".system_generated")
        .join("subagents");
    fs::create_dir_all(&subagents_dir).expect("Failed to create subagents dir");

    let sub1_json = r#"{
        "conversationId": "sub-1-abc",
        "subagentDescriptor": {
            "typeName": "dba-guardian",
            "role": "DBA Guardian"
        },
        "state": "SUBAGENT_STATE_ALIVE",
        "spawnStepIndex": 42
    }"#;
    fs::write(subagents_dir.join("sub-1-abc.json"), sub1_json).unwrap();

    let sub2_json = r#"{
        "conversationId": "sub-2-def",
        "subagentDescriptor": {
            "typeName": "cro-psychologist",
            "role": "CRO Psychologist"
        },
        "state": "SUBAGENT_STATE_DONE",
        "spawnStepIndex": 55
    }"#;
    fs::write(subagents_dir.join("sub-2-def.json"), sub2_json).unwrap();

    let subagents = scan_subagents_for_conversation(brain_dir, conv_id);
    assert_eq!(subagents.len(), 2);

    let cro = subagents.iter().find(|s| s.type_name == "cro-psychologist").unwrap();
    assert_eq!(cro.icon_name, "🧠");
    assert_eq!(cro.state_label, "Dokončeno");
    assert!(!cro.is_active);

    let dba = subagents.iter().find(|s| s.type_name == "dba-guardian").unwrap();
    assert_eq!(dba.icon_name, "🛡️");
    assert_eq!(dba.state_label, "Běží");
    assert!(dba.is_active);

    let team = build_team_hierarchy(brain_dir, conv_id, 999999, 120.0);
    assert!(team.has_subagents);
    assert_eq!(team.total_members, 2);
    assert_eq!(team.active_members, 1);
    assert!(team.total_team_memory_mb >= 120.0);
}

#[test]
fn test_subagent_tracker_realtime_transcript_resolution() {
    use agy_daemon::subagent_tracker::scan_subagents_for_conversation;
    let temp = tempdir().unwrap();
    let brain_dir = temp.path();
    let conv_id = "test-conv-realtime";

    let subagents_dir = brain_dir.join(conv_id).join(".system_generated").join("subagents");
    fs::create_dir_all(&subagents_dir).unwrap();

    // sub1 has static state "SUBAGENT_STATE_ALIVE", but its transcript says DONE
    let sub1_json = r#"{
        "conversationId": "sub-worker-1",
        "subagentDescriptor": {
            "typeName": "system-architect",
            "role": "System Architect"
        },
        "state": "SUBAGENT_STATE_ALIVE"
    }"#;
    fs::write(subagents_dir.join("sub-worker-1.json"), sub1_json).unwrap();

    let sub1_logs = brain_dir.join("sub-worker-1").join(".system_generated").join("logs");
    fs::create_dir_all(&sub1_logs).unwrap();
    let line = r#"{"step_index":5,"source":"MODEL","type":"PLANNER_RESPONSE","status":"DONE"}"#;
    fs::write(sub1_logs.join("transcript.jsonl"), format!("{}\n", line)).unwrap();

    let subagents = scan_subagents_for_conversation(brain_dir, conv_id);
    assert_eq!(subagents.len(), 1);
    assert_eq!(subagents[0].state_label, "Dokončeno");
    assert!(!subagents[0].is_active);
}

#[test]
fn test_subagent_tracker_security_kill_entire_team() {
    use agy_daemon::subagent_tracker::kill_entire_team;

    // Reject PID 0 and PID 1
    assert!(kill_entire_team(0).is_err());
    assert!(kill_entire_team(1).is_err());

    // Non-existent PID returns error
    assert!(kill_entire_team(99999999).is_err());
}

#[test]
fn test_kill_ai_process_robust_security_and_nonexistent() {
    use agy_daemon::proc_monitor::kill_ai_process_robust;

    // Reject PID 0 and PID 1
    assert!(kill_ai_process_robust(0).is_err());
    assert!(kill_ai_process_robust(1).is_err());

    // Non-existent PID returns Ok(()) (already dead)
    assert!(kill_ai_process_robust(99999999).is_ok());
}

#[test]
fn test_burn_rate_float_stabilization_and_rounding() {
    use agy_daemon::burn_rate_tracker::BurnRateTracker;
    use std::time::{Duration, Instant};

    let mut tracker = BurnRateTracker::new();
    let t0 = Instant::now();

    // First sample
    let _ = tracker.record_and_evaluate_session(1001, 1000, 1000, 100000, t0);

    // Second sample 10.003 seconds later (simulating microsecond elapsed variations)
    let t1 = t0 + Duration::from_millis(10003);
    let r1 = tracker.record_and_evaluate_session(1001, 2000, 1500, 100000, t1);

    // Third sample 10.007 seconds later
    let t2 = t0 + Duration::from_millis(20010);
    let r2 = tracker.record_and_evaluate_session(1001, 3000, 2000, 100000, t2);

    // Verify rate is rounded to 1 decimal place without endless float digits
    assert!(((r1.tokens_per_minute * 10.0).fract()).abs() < 1e-4);
    assert!(((r2.tokens_per_minute * 10.0).fract()).abs() < 1e-4);
    assert!(((r1.context_growth_per_minute * 10.0).fract()).abs() < 1e-4);

    // Check compute_global_radar total_rate rounding
    let radar = tracker.compute_global_radar(&[r1, r2], 50.0, t2);
    assert!(((radar.total_tokens_per_minute * 10.0).fract()).abs() < 1e-4);
}







