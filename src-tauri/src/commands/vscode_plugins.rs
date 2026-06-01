#![allow(non_snake_case)]

use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;

use crate::app_config::AppType;
use crate::store::AppState;

const MANAGED_BY_KEY: &str = "managedBy";
const MANAGED_BY_VALUE: &str = "ak-switch";
const CLAUDE_ENV_SETTING: &str = "claudeCode.environmentVariables";
const CLAUDE_DISABLE_LOGIN_PROMPT_SETTING: &str = "claudeCode.disableLoginPrompt";
const CLAUDE_DISABLE_LOGIN_PROMPT_MARKER: &str = "akSwitch.managedClaudeCodeDisableLoginPrompt";
const MANAGED_OPENCODE_PROVIDER_ID: &str = "ak-switch-current";

#[derive(Clone, Copy)]
struct PluginTarget {
    id: &'static str,
    label: &'static str,
    extension_id: &'static str,
    extension_prefixes: &'static [&'static str],
}

#[derive(Serialize)]
pub struct VscodePluginStatus {
    id: String,
    label: String,
    extensionId: String,
    installed: bool,
    version: Option<String>,
    configPaths: Vec<String>,
    status: String,
    message: Option<String>,
}

#[derive(Serialize)]
pub struct VscodePluginChangePreview {
    targetId: String,
    paths: Vec<String>,
    summary: String,
    destructive: bool,
}

const TARGETS: &[PluginTarget] = &[
    PluginTarget {
        id: "claude",
        label: "Claude Code",
        extension_id: "Anthropic.claude-code",
        extension_prefixes: &["anthropic.claude-code-"],
    },
    PluginTarget {
        id: "codex",
        label: "Codex",
        extension_id: "openai.chatgpt",
        extension_prefixes: &["openai.chatgpt-"],
    },
    PluginTarget {
        id: "kilo",
        label: "Kilo",
        extension_id: "kilocode.kilo-code",
        extension_prefixes: &["kilocode.kilo-code-", "kilo-code-", "kilo."],
    },
    PluginTarget {
        id: "opencode",
        label: "OpenCode",
        extension_id: "sst-dev.opencode",
        extension_prefixes: &[
            "sst-dev.opencode-",
            "opencode.opencode-",
            "opencode-",
            "open-code-",
        ],
    },
];

fn home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())
}

fn target_by_id(id: &str) -> Result<PluginTarget, String> {
    TARGETS
        .iter()
        .copied()
        .find(|target| target.id == id)
        .ok_or_else(|| format!("未知的 VS Code 插件目标: {id}"))
}

fn extension_dirs() -> Vec<PathBuf> {
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };

    let mut dirs = vec![
        home.join(".vscode/extensions"),
        home.join(".vscode-insiders/extensions"),
        home.join(".cursor/extensions"),
        home.join(".windsurf/extensions"),
    ];

    if cfg!(target_os = "windows") {
        if let Some(data_dir) = dirs::data_dir() {
            dirs.push(data_dir.join("Code/User/extensions"));
        }
    }

    dirs
}

fn extension_version_from_dir_name(name: &str, target: PluginTarget) -> Option<String> {
    let version = target
        .extension_prefixes
        .iter()
        .find_map(|prefix| name.strip_prefix(prefix))?;

    let candidate: String = version
        .chars()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect();
    let candidate = candidate.trim_end_matches('.');

    if candidate.chars().any(|ch| ch.is_ascii_digit()) {
        Some(candidate.to_string())
    } else {
        None
    }
}

fn detect_extension_in_names<I, S>(target: PluginTarget, names: I) -> (bool, Option<String>)
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut names: Vec<String> = names
        .into_iter()
        .map(|name| name.as_ref().to_lowercase())
        .collect();
    names.sort();

    for prefix in target.extension_prefixes {
        if let Some(name) = names.iter().find(|name| name.starts_with(prefix)) {
            let version = extension_version_from_dir_name(name, target);
            return (true, version);
        }
    }

    (false, None)
}

fn detect_extension_in_dirs(target: PluginTarget, dirs: &[PathBuf]) -> (bool, Option<String>) {
    let mut names = Vec::new();

    for dir in dirs {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    detect_extension_in_names(target, names)
}

fn detect_extension(target: PluginTarget) -> (bool, Option<String>) {
    detect_extension_in_dirs(target, &extension_dirs())
}

fn code_user_settings_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    if cfg!(target_os = "macos") {
        Some(home.join("Library/Application Support/Code/User/settings.json"))
    } else if cfg!(target_os = "windows") {
        dirs::data_dir().map(|dir| dir.join("Code/User/settings.json"))
    } else {
        dirs::config_dir().map(|dir| dir.join("Code/User/settings.json"))
    }
}

fn read_json_object_at(path: &Path) -> Result<Map<String, Value>, String> {
    if !path.exists() {
        return Ok(Map::new());
    }

    let content = fs::read_to_string(path).map_err(|e| format!("读取配置失败: {e}"))?;
    if content.trim().is_empty() {
        return Ok(Map::new());
    }

    match serde_json::from_str::<Value>(&content).map_err(|e| format!("解析 JSON 配置失败: {e}"))?
    {
        Value::Object(map) => Ok(map),
        _ => Err("VS Code settings.json 必须是 JSON 对象".to_string()),
    }
}

fn backup_existing_file(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
    let backup_path = path.with_extension(format!("json.ak-switch.{timestamp}.bak"));
    fs::copy(path, &backup_path).map_err(|e| format!("备份配置失败: {e}"))?;
    Ok(())
}

fn write_json_object_if_changed(path: &Path, obj: Map<String, Value>) -> Result<bool, String> {
    let serialized = serde_json::to_string_pretty(&Value::Object(obj))
        .map_err(|e| format!("序列化 JSON 配置失败: {e}"))?;
    let next = format!("{serialized}\n");
    let current = if path.exists() {
        fs::read_to_string(path).map_err(|e| format!("读取配置失败: {e}"))?
    } else {
        String::new()
    };

    if current == next {
        return Ok(false);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {e}"))?;
    }

    backup_existing_file(path)?;
    fs::write(path, next).map_err(|e| format!("写入配置失败: {e}"))?;
    Ok(true)
}

fn read_jsonc_object_at(path: &Path) -> Result<Map<String, Value>, String> {
    if !path.exists() {
        return Ok(Map::new());
    }

    let content = fs::read_to_string(path).map_err(|e| format!("读取配置失败: {e}"))?;
    if content.trim().is_empty() {
        return Ok(Map::new());
    }

    match json5::from_str::<Value>(&content).map_err(|e| format!("解析 JSONC 配置失败: {e}"))?
    {
        Value::Object(map) => Ok(map),
        _ => Err("插件配置必须是 JSON/JSONC 对象".to_string()),
    }
}

fn provider_to_opencode_style_value(provider: &crate::provider::Provider) -> Result<Value, String> {
    let mut value = provider.settings_config.clone();
    let obj = value
        .as_object_mut()
        .ok_or_else(|| "当前 provider 配置不是对象".to_string())?;

    obj.entry("name".to_string())
        .or_insert_with(|| Value::String(provider.name.clone()));

    let metadata = obj
        .entry("metadata".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    let metadata_obj = metadata
        .as_object_mut()
        .ok_or_else(|| "provider.metadata 必须是对象".to_string())?;
    metadata_obj.insert(
        MANAGED_BY_KEY.to_string(),
        Value::String(MANAGED_BY_VALUE.to_string()),
    );
    metadata_obj.insert(
        "sourceProviderId".to_string(),
        Value::String(provider.id.clone()),
    );

    Ok(value)
}

fn sync_opencode_style_provider_at(
    path: &Path,
    provider: &crate::provider::Provider,
) -> Result<bool, String> {
    let mut root = read_jsonc_object_at(path)?;
    let provider_value = provider_to_opencode_style_value(provider)?;

    let providers = root
        .entry("provider".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    let providers_obj = providers
        .as_object_mut()
        .ok_or_else(|| "provider 字段必须是对象".to_string())?;
    providers_obj.insert(MANAGED_OPENCODE_PROVIDER_ID.to_string(), provider_value);

    write_json_object_if_changed(path, root)
}

fn clear_opencode_style_provider_at(path: &Path) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }

    let mut root = read_jsonc_object_at(path)?;
    let Some(providers) = root
        .get_mut("provider")
        .and_then(|value| value.as_object_mut())
    else {
        return Ok(false);
    };

    let managed = providers
        .get(MANAGED_OPENCODE_PROVIDER_ID)
        .and_then(|value| value.get("metadata"))
        .and_then(|value| value.get(MANAGED_BY_KEY))
        .and_then(|value| value.as_str())
        == Some(MANAGED_BY_VALUE);

    if !managed {
        return Ok(false);
    }

    providers.remove(MANAGED_OPENCODE_PROVIDER_ID);
    if providers.is_empty() {
        root.remove("provider");
    }

    write_json_object_if_changed(path, root)
}

fn opencode_style_config_has_managed_provider(path: &Path) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }

    let root = read_jsonc_object_at(path)?;
    Ok(root
        .get("provider")
        .and_then(|value| value.get(MANAGED_OPENCODE_PROVIDER_ID))
        .and_then(|value| value.get("metadata"))
        .and_then(|value| value.get(MANAGED_BY_KEY))
        .and_then(|value| value.as_str())
        == Some(MANAGED_BY_VALUE))
}

fn sync_claude_code_settings_at(
    path: &Path,
    env: &BTreeMap<String, String>,
) -> Result<bool, String> {
    let mut settings = read_json_object_at(path)?;
    let mut env_vars = settings
        .remove(CLAUDE_ENV_SETTING)
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default();

    env_vars.retain(|item| {
        item.get(MANAGED_BY_KEY).and_then(|value| value.as_str()) != Some(MANAGED_BY_VALUE)
    });

    for (name, value) in env {
        if value.is_empty() {
            continue;
        }

        let mut item = Map::new();
        item.insert("name".to_string(), Value::String(name.clone()));
        item.insert("value".to_string(), Value::String(value.clone()));
        item.insert(
            MANAGED_BY_KEY.to_string(),
            Value::String(MANAGED_BY_VALUE.to_string()),
        );
        env_vars.push(Value::Object(item));
    }

    settings.insert(CLAUDE_ENV_SETTING.to_string(), Value::Array(env_vars));
    settings.insert(
        CLAUDE_DISABLE_LOGIN_PROMPT_SETTING.to_string(),
        Value::Bool(true),
    );
    settings.insert(
        CLAUDE_DISABLE_LOGIN_PROMPT_MARKER.to_string(),
        Value::Bool(true),
    );

    write_json_object_if_changed(path, settings)
}

fn clear_claude_code_settings_at(path: &Path) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }

    let mut settings = read_json_object_at(path)?;
    let mut changed = false;

    if let Some(Value::Array(existing)) = settings.remove(CLAUDE_ENV_SETTING) {
        let retained: Vec<Value> = existing
            .into_iter()
            .filter(|item| {
                item.get(MANAGED_BY_KEY).and_then(|value| value.as_str()) != Some(MANAGED_BY_VALUE)
            })
            .collect();
        changed = true;
        if !retained.is_empty() {
            settings.insert(CLAUDE_ENV_SETTING.to_string(), Value::Array(retained));
        }
    }

    let managed_disable_login_prompt = settings
        .get(CLAUDE_DISABLE_LOGIN_PROMPT_MARKER)
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    if managed_disable_login_prompt {
        settings.remove(CLAUDE_DISABLE_LOGIN_PROMPT_SETTING);
        settings.remove(CLAUDE_DISABLE_LOGIN_PROMPT_MARKER);
        changed = true;
    }

    if !changed {
        return Ok(false);
    }

    write_json_object_if_changed(path, settings)
}

fn current_claude_env(state: &AppState) -> Result<BTreeMap<String, String>, String> {
    let current_id = crate::settings::get_effective_current_provider(&state.db, &AppType::Claude)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "尚未选择 Claude provider".to_string())?;
    let provider = state
        .db
        .get_provider_by_id(&current_id, AppType::Claude.as_str())
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("当前 Claude provider 不存在: {current_id}"))?;
    let env = provider
        .settings_config
        .get("env")
        .and_then(|value| value.as_object())
        .ok_or_else(|| "当前 Claude provider 没有 env 配置".to_string())?;

    Ok(env
        .iter()
        .filter_map(|(key, value)| {
            value
                .as_str()
                .filter(|value| !value.is_empty())
                .map(|value| (key.clone(), value.to_string()))
        })
        .collect())
}

fn current_opencode_provider(state: &AppState) -> Result<crate::provider::Provider, String> {
    let current_id = state
        .db
        .get_current_provider(AppType::OpenCode.as_str())
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "尚未选择 OpenCode provider".to_string())?;
    state
        .db
        .get_provider_by_id(&current_id, AppType::OpenCode.as_str())
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("当前 OpenCode provider 不存在: {current_id}"))
}

fn kilo_config_path() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".config/kilo/kilo.jsonc"))
}

fn config_paths(target_id: &str) -> Result<Vec<PathBuf>, String> {
    let home = home_dir()?;
    let mut paths = Vec::new();

    match target_id {
        "claude" => {
            if let Some(path) = code_user_settings_path() {
                paths.push(path);
            }
            paths.push(crate::claude_plugin::claude_config_path().map_err(|e| e.to_string())?);
            paths.push(home.join(".claude/settings.json"));
        }
        "codex" => {
            paths.push(crate::get_codex_config_path());
            paths.push(crate::get_codex_auth_path());
        }
        "kilo" => {
            paths.push(home.join(".config/kilo/kilo.jsonc"));
        }
        "opencode" => {
            paths.push(home.join(".config/opencode/opencode.json"));
            paths.push(home.join(".local/share/opencode/auth.json"));
        }
        _ => return Err(format!("未知的 VS Code 插件目标: {target_id}")),
    }

    Ok(paths)
}

fn is_managed(target_id: &str, paths: &[PathBuf]) -> Result<bool, String> {
    match target_id {
        "claude" => crate::claude_plugin::is_claude_config_applied().map_err(|e| e.to_string()),
        "codex" => Ok(false),
        "opencode" | "kilo" => paths
            .first()
            .map(|path| opencode_style_config_has_managed_provider(path))
            .unwrap_or(Ok(false)),
        _ => Err(format!("未知的 VS Code 插件目标: {target_id}")),
    }
}

fn read_status(target_id: &str) -> Result<VscodePluginStatus, String> {
    let target = target_by_id(target_id)?;
    let (installed, version) = detect_extension(target);
    let paths = config_paths(target.id)?;
    let managed = is_managed(target.id, &paths)?;
    let status = if !installed {
        "not_installed"
    } else if managed {
        "managed"
    } else {
        "detected"
    };

    let message = match target.id {
        "codex" if installed => Some("Codex VS Code 插件复用 ~/.codex 配置与认证文件".to_string()),
        "opencode" if installed => Some(
            "OpenCode VS Code 插件可同步 ToolRouter 管理的 provider 到 opencode.json".to_string(),
        ),
        "kilo" if installed => {
            Some("Kilo VS Code 插件可同步 ToolRouter 管理的 provider 到 kilo.jsonc".to_string())
        }
        _ => None,
    };

    Ok(VscodePluginStatus {
        id: target.id.to_string(),
        label: target.label.to_string(),
        extensionId: target.extension_id.to_string(),
        installed,
        version,
        configPaths: paths
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect(),
        status: status.to_string(),
        message,
    })
}

#[tauri::command]
pub fn list_vscode_plugin_targets() -> Result<Vec<VscodePluginStatus>, String> {
    TARGETS
        .iter()
        .map(|target| read_status(target.id))
        .collect()
}

#[tauri::command]
pub fn read_vscode_plugin_status(targetId: String) -> Result<VscodePluginStatus, String> {
    read_status(&targetId)
}

#[tauri::command]
pub fn sync_vscode_plugin_provider(
    state: State<'_, AppState>,
    targetId: String,
) -> Result<bool, String> {
    match targetId.as_str() {
        "claude" => {
            let mut changed =
                crate::claude_plugin::write_claude_config().map_err(|e| e.to_string())?;
            if let Some(path) = code_user_settings_path() {
                let env = current_claude_env(&state)?;
                changed |= sync_claude_code_settings_at(&path, &env)?;
            }
            Ok(changed)
        }
        "codex" => Ok(false),
        "opencode" => {
            let provider = current_opencode_provider(&state)?;
            sync_opencode_style_provider_at(
                &crate::opencode_config::get_opencode_config_path(),
                &provider,
            )
        }
        "kilo" => {
            let provider = current_opencode_provider(&state)?;
            sync_opencode_style_provider_at(&kilo_config_path()?, &provider)
        }
        _ => Err(format!("未知的 VS Code 插件目标: {targetId}")),
    }
}

#[tauri::command]
pub fn clear_vscode_plugin_provider(targetId: String) -> Result<bool, String> {
    match targetId.as_str() {
        "claude" => {
            let mut changed =
                crate::claude_plugin::clear_claude_config().map_err(|e| e.to_string())?;
            if let Some(path) = code_user_settings_path() {
                changed |= clear_claude_code_settings_at(&path)?;
            }
            Ok(changed)
        }
        "codex" => Ok(false),
        "opencode" => {
            clear_opencode_style_provider_at(&crate::opencode_config::get_opencode_config_path())
        }
        "kilo" => clear_opencode_style_provider_at(&kilo_config_path()?),
        _ => Err(format!("未知的 VS Code 插件目标: {targetId}")),
    }
}

#[tauri::command]
pub fn preview_vscode_plugin_changes(
    targetId: String,
) -> Result<VscodePluginChangePreview, String> {
    let paths = config_paths(&targetId)?;
    let summary = match targetId.as_str() {
        "claude" => {
            "写入或清除 ~/.claude/config.json，并同步 VS Code settings.json 中 ToolRouter 管理的 claudeCode 环境变量和登录提示设置".to_string()
        }
        "codex" => "Codex 插件复用 ~/.codex 配置；当前无需额外写入".to_string(),
        "opencode" => {
            "写入或清除 opencode.json 中 ToolRouter 管理的 provider.ak-switch-current".to_string()
        }
        "kilo" => {
            "写入或清除 kilo.jsonc 中 ToolRouter 管理的 provider.ak-switch-current".to_string()
        }
        _ => return Err(format!("未知的 VS Code 插件目标: {targetId}")),
    };

    Ok(VscodePluginChangePreview {
        targetId,
        paths: paths
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect(),
        summary,
        destructive: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeMap;

    #[test]
    fn detect_extension_in_dirs_extracts_version_before_platform_suffix() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(
            dir.path()
                .join("anthropic.claude-code-2.1.138-darwin-arm64"),
        )
        .expect("create claude extension");
        fs::create_dir_all(dir.path().join("openai.chatgpt-26.5506.31421-darwin-arm64"))
            .expect("create codex extension");
        fs::create_dir_all(dir.path().join("kilocode.kilo-code-4.98.0"))
            .expect("create kilo extension");
        fs::create_dir_all(dir.path().join("sst-dev.opencode-0.5.3-darwin-arm64"))
            .expect("create official opencode extension");
        fs::create_dir_all(dir.path().join("opencode.opencode-0.4.7-linux-x64"))
            .expect("create legacy opencode extension");

        let dirs = [dir.path().to_path_buf()];

        assert_eq!(
            detect_extension_in_dirs(target_by_id("claude").expect("target"), &dirs),
            (true, Some("2.1.138".to_string()))
        );
        assert_eq!(
            detect_extension_in_dirs(target_by_id("codex").expect("target"), &dirs),
            (true, Some("26.5506.31421".to_string()))
        );
        assert_eq!(
            detect_extension_in_dirs(target_by_id("kilo").expect("target"), &dirs),
            (true, Some("4.98.0".to_string()))
        );
        assert_eq!(
            detect_extension_in_dirs(target_by_id("opencode").expect("target"), &dirs),
            (true, Some("0.5.3".to_string()))
        );
    }

    #[test]
    fn detect_opencode_prefers_official_extension_over_legacy_matches() {
        assert_eq!(
            detect_extension_in_names(
                target_by_id("opencode").expect("target"),
                [
                    "opencode.opencode-0.4.7-linux-x64",
                    "sst-dev.opencode-0.5.3-darwin-arm64",
                ],
            ),
            (true, Some("0.5.3".to_string()))
        );
    }

    #[test]
    fn sync_claude_code_settings_preserves_unrelated_settings_and_marks_managed_env() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("settings.json");
        fs::write(
            &path,
            serde_json::to_string_pretty(&json!({
                "editor.fontSize": 15,
                "claudeCode.environmentVariables": [
                    { "name": "USER_FLAG", "value": "keep" },
                    { "name": "ANTHROPIC_BASE_URL", "value": "user-owned" }
                ],
                "claudeCode.disableLoginPrompt": false
            }))
            .expect("serialize"),
        )
        .expect("write settings");

        let env = BTreeMap::from([
            (
                "ANTHROPIC_BASE_URL".to_string(),
                "https://gateway.example.com".to_string(),
            ),
            ("ANTHROPIC_AUTH_TOKEN".to_string(), "test-token".to_string()),
        ]);

        let changed = sync_claude_code_settings_at(&path, &env).expect("sync settings");
        assert!(changed);

        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read settings"))
                .expect("parse settings");
        assert_eq!(value["editor.fontSize"], 15);
        assert_eq!(value["claudeCode.disableLoginPrompt"], true);
        assert_eq!(value["akSwitch.managedClaudeCodeDisableLoginPrompt"], true);

        let env_vars = value["claudeCode.environmentVariables"]
            .as_array()
            .expect("env array");
        assert!(env_vars
            .iter()
            .any(|item| item["name"] == "USER_FLAG" && item["value"] == "keep"));
        assert!(env_vars
            .iter()
            .any(|item| item["name"] == "ANTHROPIC_BASE_URL"
                && item["value"] == "user-owned"
                && item.get("managedBy").is_none()));
        assert!(env_vars
            .iter()
            .any(|item| item["name"] == "ANTHROPIC_BASE_URL"
                && item["value"] == "https://gateway.example.com"
                && item["managedBy"] == "ak-switch"));
        assert!(env_vars
            .iter()
            .any(|item| item["name"] == "ANTHROPIC_AUTH_TOKEN"
                && item["value"] == "test-token"
                && item["managedBy"] == "ak-switch"));
    }

    #[test]
    fn clear_claude_code_settings_removes_only_ak_switch_managed_values() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("settings.json");
        fs::write(
            &path,
            serde_json::to_string_pretty(&json!({
                "editor.fontSize": 15,
                "claudeCode.disableLoginPrompt": true,
                "akSwitch.managedClaudeCodeDisableLoginPrompt": true,
                "claudeCode.environmentVariables": [
                    { "name": "USER_FLAG", "value": "keep" },
                    { "name": "ANTHROPIC_BASE_URL", "value": "user-owned" },
                    {
                        "name": "ANTHROPIC_BASE_URL",
                        "value": "https://gateway.example.com",
                        "managedBy": "ak-switch"
                    },
                    {
                        "name": "ANTHROPIC_AUTH_TOKEN",
                        "value": "test-token",
                        "managedBy": "ak-switch"
                    }
                ]
            }))
            .expect("serialize"),
        )
        .expect("write settings");

        let changed = clear_claude_code_settings_at(&path).expect("clear settings");
        assert!(changed);

        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read settings"))
                .expect("parse settings");
        assert_eq!(value["editor.fontSize"], 15);
        assert!(value.get("claudeCode.disableLoginPrompt").is_none());
        assert!(value
            .get("akSwitch.managedClaudeCodeDisableLoginPrompt")
            .is_none());

        let env_vars = value["claudeCode.environmentVariables"]
            .as_array()
            .expect("env array");
        assert_eq!(env_vars.len(), 2);
        assert!(env_vars
            .iter()
            .any(|item| item["name"] == "USER_FLAG" && item["value"] == "keep"));
        assert!(env_vars
            .iter()
            .any(|item| item["name"] == "ANTHROPIC_BASE_URL"
                && item["value"] == "user-owned"
                && item.get("managedBy").is_none()));
    }

    #[test]
    fn sync_opencode_style_config_writes_managed_provider_without_losing_existing_config() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("opencode.json");
        fs::write(
            &path,
            serde_json::to_string_pretty(&json!({
                "$schema": "https://opencode.ai/config.json",
                "theme": "system",
                "provider": {
                    "user-provider": {
                        "npm": "@ai-sdk/openai-compatible",
                        "name": "User Provider",
                        "options": { "baseURL": "https://user.example/v1" },
                        "models": {}
                    }
                }
            }))
            .expect("serialize"),
        )
        .expect("write config");

        let provider = crate::provider::Provider::with_id(
            "ak-test".to_string(),
            "AK Test".to_string(),
            json!({
                "npm": "@ai-sdk/openai-compatible",
                "options": {
                    "baseURL": "https://gateway.example.com/v1",
                    "apiKey": "{env:AK_SWITCH_API_KEY}"
                },
                "models": {
                    "gpt-5": { "name": "GPT-5" }
                }
            }),
            None,
        );

        let changed = sync_opencode_style_provider_at(&path, &provider).expect("sync provider");
        assert!(changed);

        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read config"))
                .expect("parse config");
        assert_eq!(value["theme"], "system");
        assert!(value["provider"].get("user-provider").is_some());
        assert_eq!(
            value["provider"]["ak-switch-current"]["options"]["baseURL"],
            "https://gateway.example.com/v1"
        );
        assert_eq!(
            value["provider"]["ak-switch-current"]["models"]["gpt-5"]["name"],
            "GPT-5"
        );
        assert_eq!(
            value["provider"]["ak-switch-current"]["metadata"]["managedBy"],
            "ak-switch"
        );
        assert_eq!(
            value["provider"]["ak-switch-current"]["metadata"]["sourceProviderId"],
            "ak-test"
        );
    }

    #[test]
    fn clear_opencode_style_config_removes_only_managed_provider() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("kilo.jsonc");
        fs::write(
            &path,
            r#"{
  // keep user comments in JSONC input readable by the parser
  "provider": {
    "user-provider": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "User Provider",
      "models": {}
    },
    "ak-switch-current": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "ToolRouter Current",
      "metadata": {
        "managedBy": "ak-switch"
      },
      "models": {}
    }
  }
}
"#,
        )
        .expect("write config");

        let changed = clear_opencode_style_provider_at(&path).expect("clear provider");
        assert!(changed);

        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read config"))
                .expect("parse config");
        assert!(value["provider"].get("user-provider").is_some());
        assert!(value["provider"].get("ak-switch-current").is_none());
    }

    #[test]
    fn opencode_style_status_is_managed_only_when_ak_switch_provider_exists() {
        let dir = tempfile::tempdir().expect("tempdir");
        let user_only_path = dir.path().join("user-only-opencode.json");
        fs::write(
            &user_only_path,
            serde_json::to_string_pretty(&json!({
                "provider": {
                    "user-provider": {
                        "npm": "@ai-sdk/openai-compatible",
                        "name": "User Provider",
                        "models": {}
                    }
                }
            }))
            .expect("serialize"),
        )
        .expect("write user config");

        assert!(!opencode_style_config_has_managed_provider(&user_only_path)
            .expect("check user config"));

        let managed_path = dir.path().join("managed-opencode.json");
        fs::write(
            &managed_path,
            serde_json::to_string_pretty(&json!({
                "provider": {
                    "ak-switch-current": {
                        "npm": "@ai-sdk/openai-compatible",
                        "name": "ToolRouter Current",
                        "metadata": {
                            "managedBy": "ak-switch"
                        },
                        "models": {}
                    }
                }
            }))
            .expect("serialize"),
        )
        .expect("write managed config");

        assert!(opencode_style_config_has_managed_provider(&managed_path)
            .expect("check managed config"));
    }

    #[test]
    fn codex_status_is_not_managed_just_because_shared_config_exists() {
        let dir = tempfile::tempdir().expect("tempdir");
        let config_path = dir.path().join("config.toml");
        let auth_path = dir.path().join("auth.json");
        fs::write(&config_path, "model_provider = \"openai\"\n").expect("write config");
        fs::write(&auth_path, "{}\n").expect("write auth");

        assert!(!is_managed("codex", &[config_path, auth_path]).expect("check codex managed"));
    }

    #[test]
    fn claude_preview_mentions_vscode_settings_and_claude_config() {
        let preview = preview_vscode_plugin_changes("claude".to_string()).expect("preview");

        assert_eq!(preview.targetId, "claude");
        assert!(preview.summary.contains("VS Code settings.json"));
        assert!(preview.summary.contains("~/.claude/config.json"));
        assert!(!preview.destructive);
    }
}
