use std::collections::{HashSet, VecDeque};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use nix::sys::signal::{kill, Signal};
use nix::unistd::{getuid, Pid};
use serde_json::Value;

use crate::state::{SubagentItem, TeamHierarchy};

pub fn get_subagent_icon(type_name: &str) -> &'static str {
    match type_name {
        "cro-psychologist" => "🧠",
        "dba-guardian" => "🛡️",
        "system-architect" => "🏗️",
        "project-estimator" => "📊",
        "geo-search-bot" => "🌐",
        "module-integrator" => "📦",
        "research" => "🔬",
        "image-generator" => "🎨",
        "self" => "🔄",
        _ => "🤖",
    }
}

pub fn map_subagent_state(state_raw: &str) -> (&'static str, &'static str, bool) {
    let s = state_raw.to_uppercase();
    if s.contains("ALIVE") || s.contains("RUNNING") {
        ("Běží", "#10b981", true)
    } else if s.contains("WAITING") || s.contains("INPUT") {
        ("Čeká", "#f59e0b", true)
    } else if s.contains("DONE") || s.contains("IDLE") {
        ("Dokončeno", "#3b82f6", false)
    } else if s.contains("ERROR") || s.contains("FAILED") {
        ("Chyba", "#ef4444", false)
    } else {
        ("Neznámý", "#64748b", false)
    }
}

pub fn scan_subagents_for_conversation(
    brain_dir: &Path,
    conversation_id: &str,
) -> Vec<SubagentItem> {
    if conversation_id.trim().is_empty() || conversation_id.contains("..") {
        return Vec::new();
    }

    let subagents_dir = brain_dir
        .join(conversation_id)
        .join(".system_generated")
        .join("subagents");

    if !subagents_dir.exists() {
        return Vec::new();
    }

    let entries = match fs::read_dir(&subagents_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut items = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let val: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let id = val
            .get("conversationId")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
            })
            .to_string();

        let descriptor = val.get("subagentDescriptor");
        let type_name = descriptor
            .and_then(|d| d.get("typeName"))
            .and_then(|v| v.as_str())
            .unwrap_or("subagent")
            .to_string();

        let role = descriptor
            .and_then(|d| d.get("role"))
            .and_then(|v| v.as_str())
            .unwrap_or(&type_name)
            .to_string();

        let raw_state = val
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("SUBAGENT_STATE_ALIVE");

        // Inspect subagent's transcript to determine real-time state (DONE/IDLE vs RUNNING)
        let mut resolved_state = raw_state.to_string();
        let transcript_path = brain_dir
            .join(&id)
            .join(".system_generated")
            .join("logs")
            .join("transcript.jsonl");

        if transcript_path.exists() {
            if let Ok(transcript_content) = fs::read_to_string(&transcript_path) {
                if let Some(last_line) = transcript_content.lines().rev().find(|l| !l.trim().is_empty()) {
                    if let Ok(last_step) = serde_json::from_str::<Value>(last_line) {
                        let step_type = last_step.get("type").and_then(|v| v.as_str()).unwrap_or("");
                        let status = last_step.get("status").and_then(|v| v.as_str()).unwrap_or("");

                        if status == "DONE" && (step_type == "PLANNER_RESPONSE" || step_type == "GENERIC") {
                            resolved_state = "DONE".to_string();
                        } else if status == "ERROR" {
                            resolved_state = "ERROR".to_string();
                        } else if step_type == "USER_INPUT" || status == "RUNNING" {
                            resolved_state = "RUNNING".to_string();
                        }
                    }
                }
            }
        }

        let (state_label, state_color, is_active) = map_subagent_state(&resolved_state);
        let icon_name = get_subagent_icon(&type_name).to_string();

        items.push(SubagentItem {
            id,
            name: role.clone(),
            role,
            type_name,
            state: resolved_state,
            state_label: state_label.to_string(),
            state_color: state_color.to_string(),
            is_active,
            pid: None,
            memory_mb: None,
            icon_name,
        });
    }

    items.sort_by(|a, b| a.name.cmp(&b.name));
    items
}

pub fn find_descendant_pids(parent_pid: u32) -> Vec<(u32, f32, String)> {
    if parent_pid <= 1 {
        return Vec::new();
    }

    let proc_dir = match fs::read_dir("/proc") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    let mut parent_to_children: std::collections::HashMap<u32, Vec<(u32, f32, String)>> =
        std::collections::HashMap::new();

    let page_size_kb = crate::proc_monitor::procfs_page_size()
        .map(|sz| sz / 1024)
        .unwrap_or(4);

    for entry in proc_dir.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        let pid: u32 = match name_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let stat_path = format!("/proc/{}/stat", pid);
        let stat_content = match fs::read_to_string(&stat_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Parse stat: PID (COMM) STATE PPID ... RSS(field 24)
        if let Some(rparen_idx) = stat_content.rfind(')') {
            let comm = if let Some(lparen_idx) = stat_content.find('(') {
                stat_content[lparen_idx + 1..rparen_idx].to_string()
            } else {
                "process".to_string()
            };

            let rest = &stat_content[rparen_idx + 1..];
            let fields: Vec<&str> = rest.split_whitespace().collect();
            // fields[0] is state, fields[1] is ppid, fields[21] is rss
            if fields.len() > 21 {
                let ppid: u32 = match fields[1].parse() {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                let rss_pages: u64 = fields[21].parse().unwrap_or(0);
                let rss_mb = ((rss_pages * page_size_kb) as f32 / 1024.0 * 10.0).round() / 10.0;

                parent_to_children
                    .entry(ppid)
                    .or_default()
                    .push((pid, rss_mb, comm));
            }
        }
    }

    // BFS to gather all transitive descendants
    let mut descendants = Vec::new();
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();

    queue.push_back(parent_pid);
    visited.insert(parent_pid);

    while let Some(curr) = queue.pop_front() {
        if let Some(children) = parent_to_children.get(&curr) {
            for child in children {
                if visited.insert(child.0) {
                    descendants.push(child.clone());
                    queue.push_back(child.0);
                }
            }
        }
    }

    descendants
}

pub fn build_team_hierarchy(
    brain_dir: &Path,
    conversation_id: &str,
    parent_pid: u32,
    parent_memory_mb: f32,
) -> TeamHierarchy {
    let mut subagents = scan_subagents_for_conversation(brain_dir, conversation_id);
    let child_processes = find_descendant_pids(parent_pid);

    let child_ram: f32 = child_processes.iter().map(|(_, rss, _)| *rss).sum();
    let total_team_memory_mb = ((parent_memory_mb + child_ram) * 10.0).round() / 10.0;

    // If subagents file list is empty but child processes exist, create synthetic items
    if subagents.is_empty() && !child_processes.is_empty() {
        for (c_pid, c_ram, c_comm) in &child_processes {
            subagents.push(SubagentItem {
                id: format!("proc-{}", c_pid),
                name: c_comm.clone(),
                role: format!("Child Process ({})", c_comm),
                type_name: "worker".to_string(),
                state: "RUNNING".to_string(),
                state_label: "Běží".to_string(),
                state_color: "#10b981".to_string(),
                is_active: true,
                pid: Some(*c_pid),
                memory_mb: Some(*c_ram),
                icon_name: "⚙️".to_string(),
            });
        }
    }

    let active_members = subagents.iter().filter(|s| s.is_active).count();
    let total_members = subagents.len();
    let has_subagents = total_members > 0 || !child_processes.is_empty();

    TeamHierarchy {
        has_subagents,
        total_members,
        active_members,
        total_team_memory_mb,
        child_processes_count: child_processes.len(),
        subagents,
    }
}

pub fn kill_entire_team(parent_pid: u32) -> Result<String, String> {
    if parent_pid <= 1 {
        return Err(format!("Odmítnuto: PID {} je chráněný proces.", parent_pid));
    }

    let proc_path = format!("/proc/{}", parent_pid);
    let meta = match fs::metadata(&proc_path) {
        Ok(m) => m,
        Err(_) => return Err(format!("Proces PID {} neexistuje nebo již skončil.", parent_pid)),
    };

    let my_uid = getuid().as_raw();
    if meta.uid() != my_uid {
        return Err(format!(
            "Odmítnuto: Proces PID {} patří jinému uživateli (UID {}).",
            parent_pid,
            meta.uid()
        ));
    }

    let descendants = find_descendant_pids(parent_pid);
    let child_count = descendants.len();

    // 1. Terminate all child descendants first with SIGTERM+SIGCONT, then SIGKILL
    for (child_pid, _, _) in &descendants {
        let c_pid = Pid::from_raw(*child_pid as i32);
        let _ = kill(c_pid, Signal::SIGTERM);
        let _ = kill(c_pid, Signal::SIGCONT);
        let _ = kill(c_pid, Signal::SIGKILL);
    }

    // 2. Terminate the main root session process with SIGTERM+SIGCONT, escalating to SIGKILL
    let target_pid = Pid::from_raw(parent_pid as i32);
    let _ = kill(target_pid, Signal::SIGTERM);
    let _ = kill(target_pid, Signal::SIGCONT);
    std::thread::sleep(std::time::Duration::from_millis(100));
    if Path::new(&proc_path).exists() {
        let _ = kill(target_pid, Signal::SIGKILL);
        let _ = kill(target_pid, Signal::SIGCONT);
    }

    Ok(format!(
        "Tým úspěšně ukončen: hlavní proces PID {} a {} potomků.",
        parent_pid, child_count
    ))
}
