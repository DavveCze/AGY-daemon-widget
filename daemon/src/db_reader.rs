use rusqlite::{Connection, OpenFlags};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;
use serde_json::Value;
use crate::state::{ConversationItem, TokenUsage};

pub struct ConversationMetadata {
    pub title: String,
    pub step_count: i64,
    pub agent_name: String,
    pub not_fully_idle: bool,
    pub last_user_input_time: String,
}

pub fn open_db_readonly(db_path: &Path) -> Result<Connection, rusqlite::Error> {
    Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX | OpenFlags::SQLITE_OPEN_URI,
    )
}

pub fn read_conversation_token_usage(conv_id: &str, brain_dir: &Path) -> TokenUsage {
    let mut usage = TokenUsage::default();
    let transcript_path = brain_dir
        .join(conv_id)
        .join(".system_generated")
        .join("logs")
        .join("transcript.jsonl");

    if let Ok(file) = File::open(&transcript_path) {
        let reader = BufReader::new(file);
        let mut last_context: u64 = 0;
        for line in reader.lines().flatten() {
            if let Ok(json) = serde_json::from_str::<Value>(&line) {
                let in_t = json.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                let out_t = json.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                let cache_t = json.get("cache_read_tokens").and_then(|v| v.as_u64()).unwrap_or(0);

                usage.input_tokens += in_t;
                usage.output_tokens += out_t;
                usage.cache_read_tokens += cache_t;

                if in_t > 0 || cache_t > 0 {
                    last_context = in_t + cache_t;
                }
            }
        }
        // Total net processed tokens (input + output)
        usage.total_tokens = usage.input_tokens + usage.output_tokens;
        // Current active context window size in tokens
        usage.current_context_tokens = last_context;
        // Standard 1M model context window limit
        usage.context_limit = 1_000_000;

        let pct = if usage.context_limit > 0 {
            (usage.current_context_tokens as f32 / usage.context_limit as f32) * 100.0
        } else {
            0.0
        };
        usage.session_percentage = ((pct * 10.0).round() / 10.0).clamp(0.0, 100.0);
    }

    usage
}

pub fn get_conversation_metadata(
    conn: &Connection,
    conv_id: &str,
) -> Option<ConversationMetadata> {
    let mut stmt = match conn.prepare(
        "SELECT title, step_count, agent_name, not_fully_idle, last_user_input_time 
         FROM conversation_summaries 
         WHERE conversation_id = ?1",
    ) {
        Ok(s) => s,
        Err(_) => return None,
    };

    let mut rows = match stmt.query([conv_id]) {
        Ok(r) => r,
        Err(_) => return None,
    };

    if let Ok(Some(row)) = rows.next() {
        let title: String = row.get(0).unwrap_or_default();
        let step_count: i64 = row.get(1).unwrap_or(0);
        let agent_name: String = row.get(2).unwrap_or_default();
        let not_fully_idle_raw: i64 = row.get(3).unwrap_or(0);
        let last_user_input_time: String = row.get(4).unwrap_or_default();

        Some(ConversationMetadata {
            title,
            step_count,
            agent_name,
            not_fully_idle: not_fully_idle_raw != 0,
            last_user_input_time,
        })
    } else {
        None
    }
}

pub fn get_recent_conversations(
    conn: &Connection,
    limit: usize,
    brain_dir: &Path,
) -> Vec<ConversationItem> {
    let mut results = Vec::new();
    let mut stmt = match conn.prepare(
        "SELECT conversation_id, title, preview, step_count, last_modified_time, workspace_uris, agent_name 
         FROM conversation_summaries 
         ORDER BY last_modified_time DESC 
         LIMIT ?1",
    ) {
        Ok(s) => s,
        Err(_) => return results,
    };

    let rows = match stmt.query_map([limit as i64], |row| {
        let conv_id: String = row.get(0).unwrap_or_default();
        let title: String = row.get(1).unwrap_or_default();
        let preview: String = row.get(2).unwrap_or_default();
        let step_count: i64 = row.get(3).unwrap_or(0);
        let last_modified_time: String = row.get(4).unwrap_or_default();
        let workspace_uris: String = row.get(5).unwrap_or_default();
        let agent_name: String = row.get(6).unwrap_or_default();

        Ok((
            conv_id,
            title,
            preview,
            step_count,
            last_modified_time,
            workspace_uris,
            agent_name,
        ))
    }) {
        Ok(r) => r,
        Err(_) => return results,
    };

    for item in rows.flatten() {
        let (conv_id, title, preview, step_count, last_modified_time, workspace_uris, agent_name) =
            item;
        let token_usage = read_conversation_token_usage(&conv_id, brain_dir);

        results.push(ConversationItem {
            conversation_id: conv_id,
            title,
            preview,
            step_count,
            last_modified_time,
            workspace_uris,
            agent_name,
            token_usage,
        });
    }

    results
}

#[derive(Debug, Clone, Copy)]
pub struct LiveStatus {
    pub is_busy: bool,
    pub step_index: i64,
}

pub fn get_live_transcript_status(conv_id: &str, brain_dir: &Path) -> Option<LiveStatus> {
    let transcript_path = brain_dir
        .join(conv_id)
        .join(".system_generated")
        .join("logs")
        .join("transcript.jsonl");

    let file = File::open(&transcript_path).ok()?;
    let metadata = file.metadata().ok()?;
    let len = metadata.len();

    let mut reader = BufReader::new(file);
    // Seek to the last 16KB of the file to inspect the latest turn/step instantaneously
    let seek_pos = len.saturating_sub(16384);
    if reader.seek(SeekFrom::Start(seek_pos)).is_err() {
        return None;
    }

    let mut last_line = String::new();
    let lines = reader.lines();
    for line_res in lines {
        if let Ok(line) = line_res {
            if !line.trim().is_empty() {
                last_line = line;
            }
        }
    }

    if last_line.is_empty() {
        return None;
    }

    let json: Value = serde_json::from_str(&last_line).ok()?;
    let step_index = json.get("step_index").and_then(|v| v.as_i64()).unwrap_or(0);
    let step_type = json.get("type").and_then(|v| v.as_str()).unwrap_or("");

    let is_busy = match step_type {
        "USER_INPUT" => true,
        "GENERIC" => true,
        "PLANNER_RESPONSE" => {
            if let Some(tool_calls) = json.get("tool_calls").and_then(|v| v.as_array()) {
                if tool_calls.is_empty() {
                    false
                } else {
                    // Check if waiting on user interactive question
                    let is_asking_user = tool_calls.iter().any(|tc| {
                        tc.get("name").and_then(|n| n.as_str()) == Some("ask_question")
                    });
                    !is_asking_user
                }
            } else {
                false
            }
        }
        _ => false,
    };

    Some(LiveStatus {
        is_busy,
        step_index,
    })
}
