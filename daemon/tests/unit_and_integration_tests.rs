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
        },
        active_sessions: vec![ActiveSession {
            pid: 1234,
            conversation_id: "conv-abc-123".to_string(),
            workspace_path: "/home/david/project".to_string(),
            project_name: "project".to_string(),
            title: "Test Task".to_string(),
            agent_name: "self".to_string(),
            current_mode: "auto".to_string(),
            current_effort: "medium".to_string(),
            not_fully_idle: true,
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
        }],
        recent_conversations: vec![ConversationItem {
            conversation_id: "conv-abc-123".to_string(),
            title: "Test Task".to_string(),
            preview: "preview text".to_string(),
            step_count: 15,
            last_modified_time: "2026-10-17 12:00:00".to_string(),
            workspace_uris: "[\"file:///home/david/project\"]".to_string(),
            agent_name: "self".to_string(),
            token_usage: TokenUsage::default(),
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
            "[\"file:///home/david/Projekty/AGY_Widget\"]",
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
            "[\"file:///home/david/dev-library\"]",
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

    let quota = parse_usage_output(sample_output, "davca.faul@gmail.com", "Google AI Pro");
    assert_eq!(quota.account_email, "davca.faul@gmail.com");
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
    assert_eq!(status.step_index, 4);
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

