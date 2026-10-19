use std::fs;
use std::path::Path;
use nix::unistd::getuid;
use crate::state::{ActiveSession, ProcessStats};

pub struct DetectedProcess {
    pub pid: u32,
    pub conversation_id: String,
    pub workspace_path: String,
    pub project_name: String,
    pub stats: ProcessStats,
}

pub fn scan_agy_processes(presence_dir: &Path) -> Vec<DetectedProcess> {
    let mut results = Vec::new();
    let page_size_kb = match procfs_page_size() {
        Ok(sz) => sz / 1024,
        Err(_) => 4, // 4KB default
    };

    let proc_dir = match fs::read_dir("/proc") {
        Ok(d) => d,
        Err(_) => return results,
    };

    let presence_str = presence_dir.to_string_lossy();

    let my_pid = std::process::id();

    for entry in proc_dir.flatten() {
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        let pid: u32 = match name_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        if pid == my_pid {
            continue;
        }

        // Check cmdline
        let cmdline_path = format!("/proc/{}/cmdline", pid);
        let cmdline = match fs::read(&cmdline_path) {
            Ok(c) => String::from_utf8_lossy(&c).to_string(),
            Err(_) => continue,
        };

        if !cmdline.contains("agy")
            || cmdline.contains("agy-daemon")
            || cmdline.contains("agenticom-daemon")
            || cmdline.contains("/usage")
            || cmdline.contains("grep")
        {
            continue;
        }

        // Check exe to verify it is the agy CLI binary
        let exe_path = format!("/proc/{}/exe", pid);
        let target_str = fs::read_link(&exe_path)
            .map(|t| t.to_string_lossy().to_string())
            .unwrap_or_default();

        if target_str.contains("agenticom-daemon") || target_str.contains("agy-daemon") {
            continue;
        }

        let is_agy_binary = target_str.ends_with("/agy")
            || target_str == "agy"
            || target_str.contains("/agy.")
            || target_str.contains("/agy ")
            || target_str.contains("/agy (deleted)")
            || target_str.contains("/.local/bin/agy")
            || cmdline.starts_with("agy\0")
            || cmdline.starts_with("agy ")
            || cmdline == "agy"
            || cmdline.contains("/bin/agy");

        if !is_agy_binary {
            continue;
        }

        // Look for open .lock file in /proc/{pid}/fd
        let fd_dir_path = format!("/proc/{}/fd", pid);
        let mut matched_conv_id: Option<String> = None;
        if let Ok(fd_entries) = fs::read_dir(&fd_dir_path) {
            for fd_entry in fd_entries.flatten() {
                if let Ok(link_target) = fs::read_link(fd_entry.path()) {
                    let target_str = link_target.to_string_lossy();
                    if target_str.contains(&*presence_str) && target_str.ends_with(".lock") {
                        if let Some(file_stem) = link_target.file_stem() {
                            matched_conv_id = Some(file_stem.to_string_lossy().to_string());
                            break;
                        }
                    }
                }
            }
        }

        let conv_id = match matched_conv_id {
            Some(id) => id,
            None => {
                // If waiting for prompt or new session, parse conversation UUID if given in cmdline
                let args: Vec<&str> = cmdline.split('\0').collect();
                let mut found_conv: Option<String> = None;
                for i in 0..args.len() {
                    if args[i] == "--conversation" && i + 1 < args.len() {
                        found_conv = Some(args[i + 1].to_string());
                        break;
                    }
                }
                found_conv.unwrap_or_else(|| format!("cli-{}", pid))
            }
        };

        // Read cwd
        let cwd_path = format!("/proc/{}/cwd", pid);
        let workspace_path = match fs::read_link(&cwd_path) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => String::from("unknown"),
        };

        let project_name = Path::new(&workspace_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| workspace_path.clone());

        // Read memory from /proc/{pid}/statm
        let statm_path = format!("/proc/{}/statm", pid);
        let mut memory_rss_mb = 0.0;
        if let Ok(statm) = fs::read_to_string(&statm_path) {
            let parts: Vec<&str> = statm.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(resident_pages) = parts[1].parse::<u64>() {
                    let mb = (resident_pages * page_size_kb) as f32 / 1024.0;
                    memory_rss_mb = mb.round(); // Integer MB prevents write thrashing
                }
            }
        }

        results.push(DetectedProcess {
            pid,
            conversation_id: conv_id,
            workspace_path,
            project_name,
            stats: ProcessStats {
                pid,
                cpu_percent: 0.0,
                memory_rss_mb,
                runtime_seconds: 0,
            },
        });
    }

    results
}

pub fn procfs_page_size_kb() -> u64 {
    match procfs_page_size() {
        Ok(sz) => sz / 1024,
        Err(_) => 4,
    }
}

pub fn read_proc_memory_mb(pid: u32, page_size_kb: u64) -> f32 {
    let statm_path = format!("/proc/{}/statm", pid);
    if let Ok(statm) = fs::read_to_string(&statm_path) {
        let parts: Vec<&str> = statm.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(resident_pages) = parts[1].parse::<u64>() {
                let mb = (resident_pages * page_size_kb) as f32 / 1024.0;
                return mb.round();
            }
        }
    }
    0.0
}

pub fn read_proc_cwd(pid: u32) -> (String, String) {
    let cwd_path = format!("/proc/{}/cwd", pid);
    let workspace_path = match fs::read_link(&cwd_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => String::from("unknown"),
    };
    let project_name = Path::new(&workspace_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| workspace_path.clone());
    (workspace_path, project_name)
}

pub fn procfs_page_size() -> Result<u64, ()> {
    unsafe {
        let sz = libc::sysconf(libc::_SC_PAGESIZE);
        if sz > 0 {
            Ok(sz as u64)
        } else {
            Err(())
        }
    }
}

pub fn has_child_processes(pid: u32) -> bool {
    let path = format!("/proc/{}/task/{}/children", pid, pid);
    if let Ok(content) = fs::read_to_string(path) {
        return !content.trim().is_empty();
    }
    false
}

#[derive(Debug, PartialEq, Eq)]
pub enum KillSecurityError {
    InvalidPid,
    SelfTargetingProhibited,
    ProcessNotFound,
    UnauthorizedUid { expected: u32, found: u32 },
    UntrackedAiSession,
}

impl std::fmt::Display for KillSecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KillSecurityError::InvalidPid => write!(f, "Invalid PID (< 2)"),
            KillSecurityError::SelfTargetingProhibited => write!(f, "Cannot terminate the daemon process itself"),
            KillSecurityError::ProcessNotFound => write!(f, "Process not found or status inaccessible"),
            KillSecurityError::UnauthorizedUid { expected, found } => {
                write!(f, "UID mismatch: expected {}, found {}", expected, found)
            }
            KillSecurityError::UntrackedAiSession => write!(f, "PID is not a recognized active AI CLI session"),
        }
    }
}

impl std::error::Error for KillSecurityError {}

/// Verify that target PID is a valid, tracked AI CLI session owned by current user
pub fn verify_killable_ai_session(pid: u32, active_sessions: &[ActiveSession]) -> Result<(), KillSecurityError> {
    if pid <= 1 {
        return Err(KillSecurityError::InvalidPid);
    }
    if pid == std::process::id() {
        return Err(KillSecurityError::SelfTargetingProhibited);
    }

    let is_tracked = active_sessions.iter().any(|s| s.pid == pid);
    if !is_tracked {
        return Err(KillSecurityError::UntrackedAiSession);
    }

    let status_path = format!("/proc/{}/status", pid);
    let status_content = fs::read_to_string(&status_path).map_err(|_| KillSecurityError::ProcessNotFound)?;

    let current_uid = getuid().as_raw();
    let mut uid_verified = false;

    for line in status_content.lines() {
        if line.starts_with("Uid:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let ruid = parts[1].parse::<u32>().unwrap_or(u32::MAX);
                let euid = parts[2].parse::<u32>().unwrap_or(u32::MAX);
                if ruid == current_uid && euid == current_uid {
                    uid_verified = true;
                } else {
                    return Err(KillSecurityError::UnauthorizedUid {
                        expected: current_uid,
                        found: euid,
                    });
                }
            }
            break;
        }
    }

    if !uid_verified {
        return Err(KillSecurityError::ProcessNotFound);
    }

    Ok(())
}

