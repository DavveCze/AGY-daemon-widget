use std::fs;
use std::path::Path;
use serde_json::Value;
use crate::state::{AgentOption, ModelOption, SettingsInfo, SkillItem};

pub fn read_settings(settings_file: &Path) -> SettingsInfo {
    let mut info = SettingsInfo {
        model: "Gemini 3.8 Flash (Medium)".to_string(),
        default_model: "gemini-3.8-flash-medium".to_string(),
        agent_mode: "accept-edits".to_string(),
        effort: "medium".to_string(),
        sandbox: true,
        notifications_enabled: true,
        notification_sound: true,
    };

    if let Ok(content) = fs::read_to_string(settings_file) {
        if let Ok(json) = serde_json::from_str::<Value>(&content) {
            if let Some(m) = json.get("model").and_then(|v| v.as_str()) {
                info.model = m.to_string();
            }
            if let Some(dm) = json.get("defaultModel").and_then(|v| v.as_str()) {
                info.default_model = dm.to_string();
            }
            if let Some(am) = json.get("agentMode").and_then(|v| v.as_str()) {
                info.agent_mode = am.to_string();
            }
            if let Some(ef) = json.get("effort").and_then(|v| v.as_str()) {
                info.effort = ef.to_string();
            }
            if let Some(sb) = json.get("enableTerminalSandbox").and_then(|v| v.as_bool()) {
                info.sandbox = sb;
            }
            if let Some(ne) = json.get("notificationsEnabled").and_then(|v| v.as_bool()) {
                info.notifications_enabled = ne;
            }
            if let Some(ns) = json.get("notificationSoundEnabled").and_then(|v| v.as_bool()) {
                info.notification_sound = ns;
            }
        }
    }

    // Auto-migrate obsolete gemini-3.8-flash ID to gemini-3.8-flash-medium
    if info.default_model == "gemini-3.8-flash" {
        info.default_model = "gemini-3.8-flash-medium".to_string();
        if info.model == "Gemini 3.8 Flash" {
            info.model = "Gemini 3.8 Flash (Medium)".to_string();
        }
    }

    info
}

pub fn update_model_setting(
    settings_file: &Path,
    model_id: &str,
    display_name: &str,
) -> Result<(), String> {
    let content = fs::read_to_string(settings_file).map_err(|e| e.to_string())?;
    let mut json: Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    if let Some(obj) = json.as_object_mut() {
        obj.insert("defaultModel".to_string(), Value::String(model_id.to_string()));
        obj.insert("model".to_string(), Value::String(display_name.to_string()));
    }

    let serialized = serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?;
    fs::write(settings_file, serialized).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_agent_mode(settings_file: &Path, new_mode: &str) -> Result<(), String> {
    let content = fs::read_to_string(settings_file).map_err(|e| e.to_string())?;
    let mut json: Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    if let Some(obj) = json.as_object_mut() {
        obj.insert("agentMode".to_string(), Value::String(new_mode.to_string()));
    }

    let serialized = serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?;
    fs::write(settings_file, serialized).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_effort_setting(settings_file: &Path, effort: &str) -> Result<(), String> {
    let content = fs::read_to_string(settings_file).map_err(|e| e.to_string())?;
    let mut json: Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    if let Some(obj) = json.as_object_mut() {
        obj.insert("effort".to_string(), Value::String(effort.to_string()));
    }

    let serialized = serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?;
    fs::write(settings_file, serialized).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_notifications_enabled(settings_file: &Path, enabled: bool) -> Result<(), String> {
    let content = fs::read_to_string(settings_file).map_err(|e| e.to_string())?;
    let mut json: Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    if let Some(obj) = json.as_object_mut() {
        obj.insert("notificationsEnabled".to_string(), Value::Bool(enabled));
    }

    let serialized = serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?;
    fs::write(settings_file, serialized).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_notification_sound(settings_file: &Path, sound: bool) -> Result<(), String> {
    let content = fs::read_to_string(settings_file).map_err(|e| e.to_string())?;
    let mut json: Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    if let Some(obj) = json.as_object_mut() {
        obj.insert("notificationSoundEnabled".to_string(), Value::Bool(sound));
    }

    let serialized = serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?;
    fs::write(settings_file, serialized).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_all_available_models() -> Vec<ModelOption> {
    vec![
        ModelOption {
            id: "gemini-3.8-flash-high".to_string(),
            name: "Gemini 3.8 Flash (High)".to_string(),
            provider: "Gemini".to_string(),
            thinking: true,
            available: true,
        },
        ModelOption {
            id: "gemini-3.8-flash-medium".to_string(),
            name: "Gemini 3.8 Flash (Medium)".to_string(),
            provider: "Gemini".to_string(),
            thinking: true,
            available: true,
        },
        ModelOption {
            id: "gemini-3.8-flash-low".to_string(),
            name: "Gemini 3.8 Flash (Low)".to_string(),
            provider: "Gemini".to_string(),
            thinking: true,
            available: true,
        },
        ModelOption {
            id: "claude-sonnet-4-6".to_string(),
            name: "Claude Sonnet 4.6 (Thinking)".to_string(),
            provider: "Outsource".to_string(),
            thinking: true,
            available: true,
        },
        ModelOption {
            id: "claude-opus-4-6-thinking".to_string(),
            name: "Claude Opus 4.6 (Thinking)".to_string(),
            provider: "Outsource".to_string(),
            thinking: true,
            available: true,
        },
        ModelOption {
            id: "gpt-oss-120b-medium".to_string(),
            name: "GPT-OSS 120B (Medium)".to_string(),
            provider: "Outsource".to_string(),
            thinking: true,
            available: true,
        },
    ]
}

pub fn get_all_available_agents() -> Vec<AgentOption> {
    vec![
        AgentOption {
            id: "self".to_string(),
            name: "Hlavní Agent (Self)".to_string(),
            description: "Plný univerzální AI agent se všemi nástroji a modely".to_string(),
        },
        AgentOption {
            id: "cro-psychologist".to_string(),
            name: "CRO & Psycholog".to_string(),
            description: "Behaviorální audit webu, konverzní tření, psychologické spouštěče".to_string(),
        },
        AgentOption {
            id: "dba-guardian".to_string(),
            name: "DBA Guardian".to_string(),
            description: "Audit SQL dotazů, kontrola indexů, validace rollbacků a docker dumpy".to_string(),
        },
        AgentOption {
            id: "geo-search-bot".to_string(),
            name: "GEO Search Bot".to_string(),
            description: "Simulace procházení AI agenty a optimalizace pro GEO motory".to_string(),
        },
        AgentOption {
            id: "system-architect".to_string(),
            name: "System Architect".to_string(),
            description: "Modelování systémů C4/Mermaid, use/misuse cases, architektura".to_string(),
        },
        AgentOption {
            id: "project-estimator".to_string(),
            name: "Project Estimator".to_string(),
            description: "Kalkulace náročnosti, velikosti kódu a cenotvorby webových projektů".to_string(),
        },
        AgentOption {
            id: "module-integrator".to_string(),
            name: "Module Integrator".to_string(),
            description: "Znovupoužití a sklizeň modulů, správa dev-library".to_string(),
        },
    ]
}

fn parse_skill_md(skill_md_path: &Path, default_name: &str) -> (String, String) {
    if let Ok(content) = fs::read_to_string(skill_md_path) {
        let mut in_frontmatter = false;
        let mut name = String::new();
        let mut description = String::new();
        let mut reading_desc = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "---" {
                if !in_frontmatter {
                    in_frontmatter = true;
                    continue;
                } else {
                    break;
                }
            }

            if in_frontmatter {
                if trimmed.starts_with("name:") {
                    reading_desc = false;
                    name = trimmed
                        .trim_start_matches("name:")
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                } else if trimmed.starts_with("description:") {
                    let desc_val = trimmed.trim_start_matches("description:").trim();
                    if desc_val == ">-" || desc_val == ">" || desc_val == "|" || desc_val == "|-" {
                        reading_desc = true;
                    } else {
                        reading_desc = false;
                        description = desc_val.trim_matches('"').trim_matches('\'').to_string();
                    }
                } else if reading_desc {
                    if !line.starts_with(" ") && !line.starts_with("\t") {
                        reading_desc = false;
                    } else {
                        if !description.is_empty() {
                            description.push(' ');
                        }
                        description.push_str(trimmed);
                    }
                }
            }
        }

        let final_name = if name.is_empty() {
            default_name.to_string()
        } else {
            name
        };
        (final_name, description)
    } else {
        (default_name.to_string(), String::new())
    }
}

pub fn scan_skills(
    skills_dir: &Path,
    builtin_skills_dir: &Path,
    plugins_dir: &Path,
) -> Vec<SkillItem> {
    let mut list = Vec::new();

    // 1. User skills (~/.gemini/antigravity-cli/skills/<skill-name>/SKILL.md)
    if let Ok(entries) = fs::read_dir(skills_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let folder_name = entry.file_name().to_string_lossy().to_string();
                let skill_md = path.join("SKILL.md");
                let (name, description) = if skill_md.exists() {
                    parse_skill_md(&skill_md, &folder_name)
                } else {
                    (folder_name.clone(), String::new())
                };

                let effective_path = if skill_md.exists() {
                    skill_md.to_string_lossy().to_string()
                } else {
                    path.to_string_lossy().to_string()
                };

                let command = format!("/{}", name);
                list.push(SkillItem {
                    name,
                    skill_type: "user".to_string(),
                    plugin_name: "user".to_string(),
                    description,
                    command,
                    path: effective_path,
                });
            }
        }
    }

    // 2. Builtin skills (~/.gemini/antigravity-cli/builtin/skills/<skill-name>/SKILL.md)
    if let Ok(entries) = fs::read_dir(builtin_skills_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let folder_name = entry.file_name().to_string_lossy().to_string();
                let skill_md = path.join("SKILL.md");
                let (name, description) = if skill_md.exists() {
                    parse_skill_md(&skill_md, &folder_name)
                } else {
                    (folder_name.clone(), String::new())
                };

                let effective_path = if skill_md.exists() {
                    skill_md.to_string_lossy().to_string()
                } else {
                    path.to_string_lossy().to_string()
                };

                let command = format!("/{}", name);
                list.push(SkillItem {
                    name,
                    skill_type: "builtin".to_string(),
                    plugin_name: "builtin".to_string(),
                    description,
                    command,
                    path: effective_path,
                });
            }
        }
    }

    // 3. Plugin skills (~/.gemini/config/plugins/<plugin-name>/skills/<skill-name>/SKILL.md)
    if let Ok(entries) = fs::read_dir(plugins_dir) {
        for entry in entries.flatten() {
            let plugin_path = entry.path();
            if plugin_path.is_dir() {
                let plugin_name = entry.file_name().to_string_lossy().to_string();
                let plugin_skills_dir = plugin_path.join("skills");

                if plugin_skills_dir.exists() && plugin_skills_dir.is_dir() {
                    if let Ok(skill_entries) = fs::read_dir(&plugin_skills_dir) {
                        for s_entry in skill_entries.flatten() {
                            let s_path = s_entry.path();
                            if s_path.is_dir() {
                                let s_folder = s_entry.file_name().to_string_lossy().to_string();
                                let skill_md = s_path.join("SKILL.md");
                                let (name, description) = if skill_md.exists() {
                                    parse_skill_md(&skill_md, &s_folder)
                                } else {
                                    (s_folder.clone(), String::new())
                                };

                                let effective_path = if skill_md.exists() {
                                    skill_md.to_string_lossy().to_string()
                                } else {
                                    s_path.to_string_lossy().to_string()
                                };

                                let command = format!("/{}:{}", plugin_name, name);
                                list.push(SkillItem {
                                    name,
                                    skill_type: "plugin".to_string(),
                                    plugin_name: plugin_name.clone(),
                                    description,
                                    command,
                                    path: effective_path,
                                });
                            }
                        }
                    }
                } else {
                    let skill_md = plugin_path.join("SKILL.md");
                    let (name, description) = if skill_md.exists() {
                        parse_skill_md(&skill_md, &plugin_name)
                    } else {
                        (plugin_name.clone(), String::new())
                    };
                    let effective_path = if skill_md.exists() {
                        skill_md.to_string_lossy().to_string()
                    } else {
                        plugin_path.to_string_lossy().to_string()
                    };
                    let command = format!("/{}", name);
                    list.push(SkillItem {
                        name,
                        skill_type: "plugin".to_string(),
                        plugin_name: plugin_name.clone(),
                        description,
                        command,
                        path: effective_path,
                    });
                }
            }
        }
    }

    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    list
}
