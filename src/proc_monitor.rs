use std::fs;
use std::path::Path;
use crate::state::ProcessStats;

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

    let uptime = read_system_uptime();

    let proc_dir = match fs::read_dir("/proc") {
        Ok(d) => d,
        Err(_) => return results,
    };

    let presence_str = presence_dir.to_string_lossy();

    for entry in proc_dir.flatten() {
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        let pid: u32 = match name_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        // Check cmdline
        let cmdline_path = format!("/proc/{}/cmdline", pid);
        let cmdline = match fs::read(&cmdline_path) {
            Ok(c) => String::from_utf8_lossy(&c).to_string(),
            Err(_) => continue,
        };

        if !cmdline.contains("agy") {
            continue;
        }

        // Look for open .lock file in /proc/{pid}/fd
        let fd_dir_path = format!("/proc/{}/fd", pid);
        let fd_entries = match fs::read_dir(&fd_dir_path) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let mut matched_conv_id: Option<String> = None;
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

        let conv_id = match matched_conv_id {
            Some(id) => id,
            None => continue,
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
                    memory_rss_mb = (resident_pages * page_size_kb) as f32 / 1024.0;
                }
            }
        }

        // Read stats from /proc/{pid}/stat
        let stat_path = format!("/proc/{}/stat", pid);
        let mut runtime_seconds = 0;
        let mut cpu_percent = 0.0;

        if let Ok(stat) = fs::read_to_string(&stat_path) {
            // Field 22 is starttime (in ticks after boot)
            let stat_parts: Vec<&str> = stat.split_whitespace().collect();
            if stat_parts.len() > 21 {
                if let Ok(start_ticks) = stat_parts[21].parse::<u64>() {
                    let ticks_per_sec = 100; // standard Linux x86_64 USER_HZ
                    let start_sec = start_ticks / ticks_per_sec;
                    if uptime > start_sec {
                        runtime_seconds = uptime - start_sec;
                    }
                }
                // user ticks = field 13, kernel ticks = field 14
                let utime: u64 = stat_parts.get(13).and_then(|v| v.parse().ok()).unwrap_or(0);
                let stime: u64 = stat_parts.get(14).and_then(|v| v.parse().ok()).unwrap_or(0);
                if runtime_seconds > 0 {
                    let total_cpu_sec = (utime + stime) as f32 / 100.0;
                    cpu_percent = (total_cpu_sec / (runtime_seconds as f32)) * 100.0;
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
                cpu_percent: (cpu_percent * 10.0).round() / 10.0,
                memory_rss_mb: (memory_rss_mb * 10.0).round() / 10.0,
                runtime_seconds,
            },
        });
    }

    results
}

fn read_system_uptime() -> u64 {
    if let Ok(uptime_str) = fs::read_to_string("/proc/uptime") {
        if let Some(first) = uptime_str.split_whitespace().next() {
            if let Ok(secs) = first.parse::<f64>() {
                return secs as u64;
            }
        }
    }
    0
}

fn procfs_page_size() -> Result<u64, ()> {
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
