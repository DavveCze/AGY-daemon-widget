use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use nix::unistd::getuid;
use crate::state::GitStatusInfo;

static GIT_CACHE: Mutex<Option<HashMap<PathBuf, (Instant, GitStatusInfo)>>> = Mutex::new(None);

/// Validates that a workspace path is a canonical, existing directory owned by the current user
pub fn validate_workspace_path(input: &str) -> Result<PathBuf, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Workspace path is empty".to_string());
    }

    if trimmed.chars().any(|c| c == '\0' || c.is_control()) {
        return Err("Workspace path contains illegal characters".to_string());
    }

    let raw_path = Path::new(trimmed);
    let canonical = fs::canonicalize(raw_path).map_err(|e| format!("Failed to resolve path: {}", e))?;

    if !canonical.is_dir() {
        return Err("Workspace path is not a directory".to_string());
    }

    // Kernel Security Check: Ensure directory is owned by the current user UID
    let metadata = fs::metadata(&canonical).map_err(|e| format!("Failed to read metadata: {}", e))?;
    let current_uid = getuid().as_raw();
    if metadata.uid() != current_uid {
        return Err(format!(
            "Workspace security violation: path owned by UID {}, expected current UID {}",
            metadata.uid(),
            current_uid
        ));
    }

    Ok(canonical)
}

/// Inspect git status for a workspace directory with time-based caching (2 seconds TTL)
pub fn inspect_git_status(workspace: &str) -> GitStatusInfo {
    let canonical = match validate_workspace_path(workspace) {
        Ok(p) => p,
        Err(_) => return GitStatusInfo::default(),
    };

    // Check cache
    let now = Instant::now();
    {
        let mut lock = GIT_CACHE.lock().unwrap_or_else(|p| p.into_inner());
        let map = lock.get_or_insert_with(HashMap::new);
        if let Some((timestamp, info)) = map.get(&canonical) {
            if now.duration_since(*timestamp) < Duration::from_millis(2000) {
                return info.clone();
            }
        }
    }

    let status = run_git_status_check(&canonical);

    // Update cache
    {
        let mut lock = GIT_CACHE.lock().unwrap_or_else(|p| p.into_inner());
        let map = lock.get_or_insert_with(HashMap::new);
        map.insert(canonical, (now, status.clone()));
    }

    status
}

/// Clears cached git status for a workspace
pub fn invalidate_git_cache(workspace: &str) {
    if let Ok(canonical) = validate_workspace_path(workspace) {
        let mut lock = GIT_CACHE.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(map) = lock.as_mut() {
            map.remove(&canonical);
        }
    }
}

fn run_git_status_check(dir: &Path) -> GitStatusInfo {
    // 1. Verify if git work tree
    let is_inside_worktree = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .current_dir(dir)
        .output();

    match is_inside_worktree {
        Ok(out) if out.status.success() && String::from_utf8_lossy(&out.stdout).trim() == "true" => {}
        _ => return GitStatusInfo::default(),
    }

    // 2. Get current branch name
    let branch = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .current_dir(dir)
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .unwrap_or_else(|_| "HEAD".to_string());

    // 3. Get changed files from porcelain status
    let status_out = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(dir)
        .output();

    let mut modified_files = Vec::new();
    let mut files_changed = 0;

    if let Ok(out) = status_out {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            if line.len() > 3 {
                files_changed += 1;
                let file_path = line[3..].trim();
                // Strip git rename target if present: "orig -> new"
                let target_path = file_path.split(" -> ").last().unwrap_or(file_path);
                if modified_files.len() < 12 {
                    modified_files.push(target_path.to_string());
                }
            }
        }
    }

    // 4. Get insertions & deletions from diff --numstat (working tree vs HEAD)
    let diff_numstat = Command::new("git")
        .arg("diff")
        .arg("HEAD")
        .arg("--numstat")
        .current_dir(dir)
        .output();

    let mut insertions = 0;
    let mut deletions = 0;

    if let Ok(out) = diff_numstat {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(ins) = parts[0].parse::<usize>() {
                    insertions += ins;
                }
                if let Ok(del) = parts[1].parse::<usize>() {
                    deletions += del;
                }
            }
        }
    }

    GitStatusInfo {
        is_repo: true,
        branch,
        files_changed,
        insertions,
        deletions,
        modified_files,
    }
}

/// Retrieve working tree and staged diff up to max_bytes
pub fn get_git_diff(workspace: &str, max_bytes: usize) -> Result<String, String> {
    let canonical = validate_workspace_path(workspace)?;

    let output = Command::new("git")
        .arg("diff")
        .arg("HEAD")
        .current_dir(&canonical)
        .output()
        .map_err(|e| format!("Failed to execute git diff: {}", e))?;

    if !output.status.success() {
        // Fallback to git diff without HEAD in case repository has no commits yet
        let fallback_output = Command::new("git")
            .arg("diff")
            .current_dir(&canonical)
            .output()
            .map_err(|e| format!("Failed to execute fallback git diff: {}", e))?;

        if !fallback_output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        let diff_str = String::from_utf8_lossy(&fallback_output.stdout);
        return Ok(truncate_diff(&diff_str, max_bytes));
    }

    let diff_str = String::from_utf8_lossy(&output.stdout);
    Ok(truncate_diff(&diff_str, max_bytes))
}

fn truncate_diff(diff: &str, max_bytes: usize) -> String {
    if diff.len() <= max_bytes {
        diff.to_string()
    } else {
        let truncated = &diff[..max_bytes];
        format!("{}\n\n... [Výpis zkrácen na {} KB] ...", truncated, max_bytes / 1024)
    }
}

/// Safely roll back uncommitted changes in the repository.
/// Reverts staged changes, restores modified files, and optionally removes untracked files.
pub fn rollback_git_changes(workspace: &str, clean_untracked: bool) -> Result<String, String> {
    let canonical = validate_workspace_path(workspace)?;

    // 1. Unstage any staged files
    let _ = Command::new("git")
        .arg("restore")
        .arg("--staged")
        .arg(".")
        .current_dir(&canonical)
        .output();

    // 2. Restore modified and deleted tracked files
    let restore_output = Command::new("git")
        .arg("restore")
        .arg(".")
        .current_dir(&canonical)
        .output()
        .map_err(|e| format!("Chyba při volání git restore: {}", e))?;

    if !restore_output.status.success() {
        // Fallback for older git versions
        let checkout_output = Command::new("git")
            .arg("checkout")
            .arg("HEAD")
            .arg("--")
            .arg(".")
            .current_dir(&canonical)
            .output()
            .map_err(|e| format!("Chyba při volání git checkout: {}", e))?;

        if !checkout_output.status.success() {
            let err = String::from_utf8_lossy(&checkout_output.stderr);
            return Err(format!("Obnova souborů selhala: {}", err));
        }
    }

    // 3. Clean untracked files and directories if requested
    if clean_untracked {
        let clean_output = Command::new("git")
            .arg("clean")
            .arg("-f")
            .arg("-d")
            .current_dir(&canonical)
            .output()
            .map_err(|e| format!("Chyba při volání git clean: {}", e))?;

        if !clean_output.status.success() {
            let err = String::from_utf8_lossy(&clean_output.stderr);
            return Err(format!("Smazání nových souborů selhalo: {}", err));
        }
    }

    // Invalidate cache immediately so UI reflects updated clean status
    invalidate_git_cache(workspace);

    let msg = if clean_untracked {
        "Všechny upravené i nově vytvořené soubory byly úspěšně vráceny zpět do výchozího stavu commitu."
    } else {
        "Upravené sledované soubory byly úspěšně vráceny zpět do výchozího stavu."
    };

    Ok(msg.to_string())
}
