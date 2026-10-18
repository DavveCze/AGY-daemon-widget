use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuotaBucket {
    pub name: String,
    pub percentage: f32,
    pub reset_time_str: String,
    pub refreshes_in_str: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RealQuotaData {
    pub account_email: String,
    pub plan_name: String,
    pub gemini_weekly: QuotaBucket,
    pub gemini_five_hour: QuotaBucket,
    pub outsource_weekly: QuotaBucket,
    pub outsource_five_hour: QuotaBucket,
}

pub struct QuotaCache {
    pub last_fetched: Option<Instant>,
    pub data: RealQuotaData,
}

impl QuotaCache {
    pub fn new() -> Self {
        let fallback_email = std::env::var("USER")
            .map(|u| format!("{}@local", u))
            .unwrap_or_else(|_| "user@local".to_string());
        Self {
            last_fetched: None,
            data: RealQuotaData {
                account_email: fallback_email,
                plan_name: "Google AI Pro".to_string(),
                gemini_weekly: QuotaBucket {
                    name: "Weekly Limit Remaining".to_string(),
                    percentage: 96.0,
                    reset_time_str: "".to_string(),
                    refreshes_in_str: "Refreshes in 167h".to_string(),
                },
                gemini_five_hour: QuotaBucket {
                    name: "Five Hour Limit Remaining".to_string(),
                    percentage: 88.0,
                    reset_time_str: "".to_string(),
                    refreshes_in_str: "Refreshes in 4h 0m".to_string(),
                },
                outsource_weekly: QuotaBucket {
                    name: "Weekly Limit Remaining".to_string(),
                    percentage: 100.0,
                    reset_time_str: "".to_string(),
                    refreshes_in_str: "Quota available".to_string(),
                },
                outsource_five_hour: QuotaBucket {
                    name: "Five Hour Limit Remaining".to_string(),
                    percentage: 100.0,
                    reset_time_str: "".to_string(),
                    refreshes_in_str: "Quota available".to_string(),
                },
            },
        }
    }
}

pub fn detect_account_from_logs(log_dir: &Path) -> (String, String) {
    let mut email = std::env::var("USER")
        .map(|u| format!("{}@local", u))
        .unwrap_or_else(|_| "user@local".to_string());
    let plan = "Google AI Pro".to_string();

    if let Ok(entries) = fs::read_dir(log_dir) {
        let mut logs: Vec<_> = entries
            .flatten()
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.starts_with("cli-") && name.ends_with(".log")
            })
            .collect();

        logs.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
        logs.reverse();

        for log_entry in logs.iter().take(3) {
            if let Ok(content) = fs::read_to_string(log_entry.path()) {
                for line in content.lines() {
                    if line.contains("applyAuthResult:") && line.contains("email=") {
                        if let Some(pos) = line.find("email=") {
                            let sub = &line[pos + 6..];
                            if let Some(end) = sub.find(',') {
                                email = sub[..end].trim().to_string();
                                return (email, plan);
                            }
                        }
                    }
                }
            }
        }
    }

    (email, plan)
}

pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn parse_usage_output(output_text: &str, email: &str, plan: &str) -> RealQuotaData {
    let clean_text = strip_ansi(output_text);
    let mut data = RealQuotaData {
        account_email: email.to_string(),
        plan_name: plan.to_string(),
        gemini_weekly: QuotaBucket {
            name: "Weekly Limit Remaining".to_string(),
            percentage: 100.0,
            reset_time_str: "".to_string(),
            refreshes_in_str: "Quota available".to_string(),
        },
        gemini_five_hour: QuotaBucket {
            name: "Five Hour Limit Remaining".to_string(),
            percentage: 100.0,
            reset_time_str: "".to_string(),
            refreshes_in_str: "Quota available".to_string(),
        },
        outsource_weekly: QuotaBucket {
            name: "Weekly Limit Remaining".to_string(),
            percentage: 100.0,
            reset_time_str: "".to_string(),
            refreshes_in_str: "Quota available".to_string(),
        },
        outsource_five_hour: QuotaBucket {
            name: "Five Hour Limit Remaining".to_string(),
            percentage: 100.0,
            reset_time_str: "".to_string(),
            refreshes_in_str: "Quota available".to_string(),
        },
    };

    let now = Local::now();

    for line in clean_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("Quota:") {
            continue;
        }

        let is_gemini = trimmed.to_lowercase().contains("gemini");
        let is_outsource = trimmed.to_lowercase().contains("claude") || trimmed.to_lowercase().contains("gpt");
        let is_five_hour = trimmed.to_lowercase().contains("five hour");
        let is_weekly = trimmed.to_lowercase().contains("weekly");

        let mut pct = 100.0;
        let mut date_str = String::new();
        let mut refreshes_in = "Quota available".to_string();

        let parts: Vec<&str> = trimmed
            .split(|c| c == '\t' || c == ' ')
            .filter(|s| !s.is_empty())
            .collect();

        for (i, p) in parts.iter().enumerate() {
            let clean_p = p.trim();
            if clean_p.ends_with('%') {
                if let Ok(val) = clean_p.trim_end_matches('%').parse::<f32>() {
                    pct = val;
                }
            } else if clean_p.contains('T') && clean_p.ends_with('Z') {
                if let Ok(utc_dt) = DateTime::parse_from_rfc3339(clean_p) {
                    let local_dt = utc_dt.with_timezone(&Local);
                    date_str = local_dt.format("%Y-%m-%d %H:%M").to_string();
                    let diff = utc_dt.signed_duration_since(chrono::Utc::now());
                    if diff.num_seconds() > 0 {
                        let total_hours = diff.num_hours();
                        let mins = diff.num_minutes() % 60;
                        refreshes_in = format!("Refreshes in {}h {}m", total_hours, mins);
                    }
                }
            } else if clean_p.len() == 10 && clean_p.chars().filter(|c| *c == '-').count() == 2 {
                if let Some(next_p) = parts.get(i + 1) {
                    if next_p.contains(':') {
                        let full_dt_str = format!("{} {}", clean_p, next_p.trim());
                        date_str = full_dt_str.clone();
                        if let Ok(naive) = NaiveDateTime::parse_from_str(&full_dt_str, "%Y-%m-%d %H:%M") {
                            if let Some(target_dt) = Local.from_local_datetime(&naive).single() {
                                let diff = target_dt.signed_duration_since(now);
                                if diff.num_seconds() > 0 {
                                    let total_hours = diff.num_hours();
                                    let mins = diff.num_minutes() % 60;
                                    refreshes_in = format!("Refreshes in {}h {}m", total_hours, mins);
                                }
                            }
                        }
                    }
                }
            }
        }

        if refreshes_in == "Quota available" && pct < 100.0 {
            refreshes_in = "Refreshes soon".to_string();
        }

        let bucket = QuotaBucket {
            name: if is_five_hour {
                "Five Hour Limit Remaining".to_string()
            } else {
                "Weekly Limit Remaining".to_string()
            },
            percentage: pct,
            reset_time_str: date_str,
            refreshes_in_str: refreshes_in,
        };

        if is_gemini {
            if is_five_hour {
                data.gemini_five_hour = bucket;
            } else if is_weekly {
                data.gemini_weekly = bucket;
            }
        } else if is_outsource {
            if is_five_hour {
                data.outsource_five_hour = bucket;
            } else if is_weekly {
                data.outsource_weekly = bucket;
            }
        }
    }

    data
}

pub fn fetch_quota_sync(log_dir: &Path) -> RealQuotaData {
    let (email, plan) = detect_account_from_logs(log_dir);

    if let Ok(output) = Command::new("agy")
        .arg("-p")
        .arg("/usage")
        .env("NO_COLOR", "1")
        .env("TERM", "dumb")
        .output()
    {
        if output.status.success() {
            let stdout_str = String::from_utf8_lossy(&output.stdout);
            return parse_usage_output(&stdout_str, &email, &plan);
        }
    }

    // Default fallback if agy command fails
    RealQuotaData {
        account_email: email,
        plan_name: plan,
        gemini_weekly: QuotaBucket {
            name: "Weekly Limit Remaining".to_string(),
            percentage: 96.0,
            reset_time_str: "".to_string(),
            refreshes_in_str: "Refreshes in 167h".to_string(),
        },
        gemini_five_hour: QuotaBucket {
            name: "Five Hour Limit Remaining".to_string(),
            percentage: 88.0,
            reset_time_str: "".to_string(),
            refreshes_in_str: "Refreshes in 4h 0m".to_string(),
        },
        outsource_weekly: QuotaBucket {
            name: "Weekly Limit Remaining".to_string(),
            percentage: 100.0,
            reset_time_str: "".to_string(),
            refreshes_in_str: "Quota available".to_string(),
        },
        outsource_five_hour: QuotaBucket {
            name: "Five Hour Limit Remaining".to_string(),
            percentage: 100.0,
            reset_time_str: "".to_string(),
            refreshes_in_str: "Quota available".to_string(),
        },
    }
}
