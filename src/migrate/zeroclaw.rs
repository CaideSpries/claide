//! ZeroClaw → Claide migration module.
//!
//! ZeroClaw uses TOML config at `~/.zeroclaw/config.toml` and stores workspace
//! data, scripts, and skills under `~/.zeroclaw/workspace/`. This module handles
//! detection, config conversion (TOML→JSON), and data copying.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::{Config, DiscordConfig, ProviderConfig};

// ---------------------------------------------------------------------------
// Detection
// ---------------------------------------------------------------------------

/// Well-known ZeroClaw config file name.
const ZEROCLAW_CONFIG: &str = "config.toml";

/// Detect a ZeroClaw installation directory.
///
/// Checks:
/// 1. `$ZEROCLAW_STATE_DIR` environment variable
/// 2. `~/.zeroclaw`
pub fn detect_zeroclaw_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("ZEROCLAW_STATE_DIR") {
        let p = PathBuf::from(&dir);
        if p.join(ZEROCLAW_CONFIG).is_file() {
            return Some(p);
        }
    }

    let home = dirs::home_dir()?;
    let candidate = home.join(".zeroclaw");
    if candidate.join(ZEROCLAW_CONFIG).is_file() {
        return Some(candidate);
    }

    None
}

// ---------------------------------------------------------------------------
// TOML config loading
// ---------------------------------------------------------------------------

/// Load and parse a ZeroClaw TOML config file into a generic TOML `Value`.
pub fn load_zeroclaw_config(zeroclaw_dir: &Path) -> Result<toml::Value> {
    let path = zeroclaw_dir.join(ZEROCLAW_CONFIG);
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let value: toml::Value =
        content.parse().with_context(|| format!("Failed to parse {} as TOML", path.display()))?;
    Ok(value)
}

// ---------------------------------------------------------------------------
// Config conversion
// ---------------------------------------------------------------------------

/// Result of ZeroClaw config conversion.
pub struct ZeroClawConfigResult {
    pub migrated: Vec<String>,
    pub skipped: Vec<(String, String)>,
    pub not_portable: Vec<String>,
    pub warnings: Vec<String>,
}

/// Convert a ZeroClaw TOML config into a Claide `Config`, merging into `existing`.
pub fn convert_zeroclaw_config(
    zc: &toml::Value,
    existing: &mut Config,
) -> ZeroClawConfigResult {
    let mut migrated = Vec::new();
    let mut skipped = Vec::new();
    let mut not_portable = Vec::new();
    let mut warnings = Vec::new();

    // ── Provider (top-level api_key, default_provider, default_model) ──
    if let Some(provider) = toml_str(zc, &["default_provider"]) {
        if let Some(api_key) = toml_str(zc, &["api_key"]) {
            if let Some(pc) = get_provider_mut(&mut existing.providers, provider) {
                pc.api_key = Some(api_key.to_string());
                migrated.push(format!("default_provider ({})", provider));
            } else {
                skipped.push((
                    format!("default_provider: {}", provider),
                    "Unknown provider".into(),
                ));
            }
        }

        if let Some(model) = toml_str(zc, &["default_model"]) {
            existing.agents.defaults.model = model.to_string();
            migrated.push("default_model".into());
        }

        if let Some(temp) = zc.get("default_temperature").and_then(|v| v.as_float()) {
            existing.agents.defaults.temperature = temp as f32;
            migrated.push("default_temperature".into());
        }
    }

    // ── Workspace ────────────────────────────────────────────────────
    if let Some(workspace) = toml_str(zc, &["workspace"]) {
        // Rewrite ~/.zeroclaw/ paths to ~/.claide/
        let ws = workspace.replace(".zeroclaw", ".claide");
        existing.agents.defaults.workspace = ws;
        migrated.push("workspace".into());
    }

    // ── Discord channel (channels_config.discord) ─────────────────────
    if let Some(token) = toml_str(zc, &["channels_config", "discord", "bot_token"]) {
        let dc = existing
            .channels
            .discord
            .get_or_insert_with(DiscordConfig::default);
        dc.token = token.to_string();
        dc.enabled = true;
        migrated.push("channels_config.discord.bot_token".into());

        // Allowed users
        if let Some(users) = toml_array_strings(zc, &["channels_config", "discord", "allowed_users"]) {
            dc.allow_from = users;
            migrated.push("channels_config.discord.allowed_users".into());
        }
    }

    // ── Agent settings (agent.compact_context, agent.max_history_messages) ──
    if let Some(compact) = toml_bool(zc, &["agent", "compact_context"]) {
        existing.compaction.enabled = compact;
        migrated.push("agent.compact_context".into());
    }

    if let Some(max_hist) = toml_u64(zc, &["agent", "max_history_messages"]) {
        // Map to compaction settings — ZeroClaw uses this to limit conversation history
        warnings.push(format!(
            "max_history_messages={} — Claide uses compaction tiers instead. \
             Set compaction.context_limit for similar behavior.",
            max_hist
        ));
        skipped.push((
            "max_history_messages".into(),
            "Claide uses compaction tiers".into(),
        ));
    }

    // ── Transcription ────────────────────────────────────────────────
    if let Some(enabled) = toml_bool(zc, &["transcription", "enabled"]) {
        existing.transcription.enabled = enabled;
        migrated.push("transcription.enabled".into());
    }
    if let Some(model) = toml_str(zc, &["transcription", "model"]) {
        existing.transcription.model = model.to_string();
        migrated.push("transcription.model".into());
    }
    if let Some(api_url) = toml_str(zc, &["transcription", "api_url"]) {
        warnings.push(format!(
            "transcription.api_url={} — Claide routes transcription through configured providers. \
             Ensure the provider with transcription support is configured.",
            api_url
        ));
    }

    // ── Cost tracking ────────────────────────────────────────────────
    if let Some(enabled) = toml_bool(zc, &["cost", "enabled"]) {
        existing.cost.enabled = enabled;
        migrated.push("cost.enabled".into());
    }
    if zc.get("cost").and_then(|c| c.get("daily_limit_usd")).is_some() {
        warnings.push(
            "cost.daily_limit_usd — Claide does not have per-day cost limits. \
             Use provider-level rate limits instead."
                .into(),
        );
    }

    // ── Autonomy / shell settings (autonomy.*) ─────────────────────
    if let Some(commands) = toml_array_strings(zc, &["autonomy", "allowed_commands"]) {
        if !commands.is_empty() {
            migrated.push(format!(
                "autonomy.allowed_commands ({} commands)",
                commands.len()
            ));
            warnings.push(
                "autonomy.allowed_commands migrated — review Claide's approval gate config \
                 for equivalent security policy."
                    .into(),
            );
        }
    }

    // ZeroClaw workspace_only = false is critical — map to Claide's shell security
    if let Some(ws_only) = toml_bool(zc, &["autonomy", "workspace_only"]) {
        if !ws_only {
            warnings.push(
                "shell.workspace_only=false was set in ZeroClaw. \
                 Claide defaults to workspace-only shell access — \
                 review security.shell config if scripts need access outside workspace."
                    .into(),
            );
        }
        skipped.push((
            "shell.workspace_only".into(),
            "Claide uses different shell security model".into(),
        ));
    }

    // ── Auto-approve (autonomy.auto_approve) ─────────────────────────
    if toml_array_strings(zc, &["autonomy", "auto_approve"]).is_some() {
        not_portable.push(
            "auto_approve — Claide uses approval gate config instead. \
             For async channels, tools are auto-approved by default."
                .into(),
        );
    }

    // ── Heartbeat ────────────────────────────────────────────────────
    if toml_bool(zc, &["heartbeat", "enabled"]).is_some() {
        not_portable.push(
            "heartbeat — Claide uses health endpoint + Prometheus metrics instead.".into(),
        );
    }

    // ── Cron ─────────────────────────────────────────────────────────
    if zc.get("cron").is_some() {
        warnings.push(
            "ZeroClaw cron config found. Claide has its own cron system — \
             review and migrate cron jobs manually."
                .into(),
        );
    }

    // ── Runtime trace ────────────────────────────────────────────────
    if toml_str(zc, &["runtime_trace_mode"]).is_some() {
        skipped.push((
            "runtime_trace_mode".into(),
            "Claide uses audit trail instead".into(),
        ));
    }

    ZeroClawConfigResult {
        migrated,
        skipped,
        not_portable,
        warnings,
    }
}

// ---------------------------------------------------------------------------
// Workspace data + scripts copying
// ---------------------------------------------------------------------------

/// Result of copying workspace directories.
pub struct WorkspaceCopyResult {
    pub copied_files: Vec<String>,
    pub skipped_files: Vec<(String, String)>,
}

/// Copy ZeroClaw workspace data and scripts to Claide workspace.
///
/// Copies:
/// - `workspace/data/*` → `~/.claide/workspace/data/`
/// - `workspace/scripts/*` → `~/.claide/workspace/scripts/`
/// - `workspace/SOUL.md`, `USER.md`, `IDENTITY.md`, `AGENTS.md`, etc.
///
/// Does NOT overwrite existing files.
pub fn copy_workspace(zeroclaw_dir: &Path, claide_workspace: &Path) -> Result<WorkspaceCopyResult> {
    let zc_workspace = zeroclaw_dir.join("workspace");
    let mut copied = Vec::new();
    let mut skipped = Vec::new();

    if !zc_workspace.is_dir() {
        return Ok(WorkspaceCopyResult {
            copied_files: copied,
            skipped_files: skipped,
        });
    }

    std::fs::create_dir_all(claide_workspace)
        .with_context(|| format!("Failed to create {}", claide_workspace.display()))?;

    // Copy top-level persona files (SOUL.md, USER.md, etc.)
    let persona_files = ["SOUL.md", "USER.md", "IDENTITY.md", "AGENTS.md", "TOOLS.md", "MEMORY.md"];
    for name in &persona_files {
        let src = zc_workspace.join(name);
        let dst = claide_workspace.join(name);
        if src.is_file() {
            if dst.exists() {
                skipped.push((name.to_string(), "already exists".into()));
            } else {
                std::fs::copy(&src, &dst)
                    .with_context(|| format!("Failed to copy {}", name))?;
                copied.push(name.to_string());
            }
        }
    }

    // Copy data/ directory
    copy_dir_no_overwrite(
        &zc_workspace.join("data"),
        &claide_workspace.join("data"),
        &mut copied,
        &mut skipped,
        "data/",
    )?;

    // Copy scripts/ directory (preserve executable permissions)
    copy_dir_no_overwrite(
        &zc_workspace.join("scripts"),
        &claide_workspace.join("scripts"),
        &mut copied,
        &mut skipped,
        "scripts/",
    )?;

    // Copy skills/ directory
    copy_dir_no_overwrite(
        &zc_workspace.join("skills"),
        &claide_workspace.join("skills"),
        &mut copied,
        &mut skipped,
        "skills/",
    )?;

    Ok(WorkspaceCopyResult {
        copied_files: copied,
        skipped_files: skipped,
    })
}

/// Copy a directory recursively, skipping files that already exist at destination.
fn copy_dir_no_overwrite(
    src: &Path,
    dst: &Path,
    copied: &mut Vec<String>,
    skipped: &mut Vec<(String, String)>,
    prefix: &str,
) -> Result<()> {
    if !src.is_dir() {
        return Ok(());
    }

    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);
        let display_name = format!("{}{}", prefix, file_name.to_string_lossy());

        if src_path.is_dir() {
            let sub_prefix = format!("{}/", display_name);
            copy_dir_no_overwrite(&src_path, &dst_path, copied, skipped, &sub_prefix)?;
        } else if dst_path.exists() {
            skipped.push((display_name, "already exists".into()));
        } else {
            // Copy file preserving permissions
            std::fs::copy(&src_path, &dst_path)
                .with_context(|| format!("Failed to copy {}", display_name))?;

            // Preserve executable bit on scripts
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(src_meta) = std::fs::metadata(&src_path) {
                    let mode = src_meta.permissions().mode();
                    if mode & 0o111 != 0 {
                        let _ = std::fs::set_permissions(
                            &dst_path,
                            std::fs::Permissions::from_mode(mode),
                        );
                    }
                }
            }

            copied.push(display_name);
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// TOML helpers
// ---------------------------------------------------------------------------

/// Navigate nested TOML by a sequence of keys.
fn toml_pointer<'a>(value: &'a toml::Value, keys: &[&str]) -> Option<&'a toml::Value> {
    let mut current = value;
    for key in keys {
        current = current.get(*key)?;
    }
    Some(current)
}

/// Get a string value at a nested TOML path.
fn toml_str<'a>(value: &'a toml::Value, keys: &[&str]) -> Option<&'a str> {
    toml_pointer(value, keys).and_then(|v| v.as_str())
}

/// Get a boolean value at a nested TOML path.
fn toml_bool(value: &toml::Value, keys: &[&str]) -> Option<bool> {
    toml_pointer(value, keys).and_then(|v| v.as_bool())
}

/// Get a u64 value at a nested TOML path.
fn toml_u64(value: &toml::Value, keys: &[&str]) -> Option<u64> {
    toml_pointer(value, keys).and_then(|v| v.as_integer()).map(|i| i as u64)
}

/// Get a string array at a nested TOML path.
fn toml_array_strings(value: &toml::Value, keys: &[&str]) -> Option<Vec<String>> {
    toml_pointer(value, keys)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
}

/// Get a mutable reference to a provider entry by name.
fn get_provider_mut<'a>(
    providers: &'a mut crate::config::ProvidersConfig,
    name: &str,
) -> Option<&'a mut ProviderConfig> {
    match name {
        "openai" => Some(providers.openai.get_or_insert_with(ProviderConfig::default)),
        "groq" => Some(providers.groq.get_or_insert_with(ProviderConfig::default)),
        "gemini" | "google" => Some(providers.gemini.get_or_insert_with(ProviderConfig::default)),
        "ollama" => Some(providers.ollama.get_or_insert_with(ProviderConfig::default)),
        "anthropic" => Some(
            providers
                .anthropic
                .get_or_insert_with(ProviderConfig::default),
        ),
        "openrouter" => Some(
            providers
                .openrouter
                .get_or_insert_with(ProviderConfig::default),
        ),
        _ => None,
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn sample_zeroclaw_toml() -> &'static str {
        r#"
workspace = "~/.zeroclaw/workspace"
compact_context = true
max_history_messages = 6

[default_chat_model]
provider = "gemini"
model = "gemini-2.5-flash"
api_key = "AIza-test-key"

[discord]
bot_token = "MTIz-discord-token"
allowed_users = ["691964645736448032"]

[transcription]
enabled = true
model = "whisper-large-v3-turbo"

[shell]
workspace_only = false
allowed_commands = ["kb-save", "kb-ask", "birthday-check"]

[heartbeat]
enabled = true
interval_seconds = 300
"#
    }

    #[test]
    fn test_detect_zeroclaw_dir_none() {
        // Should not panic on a clean system.
        let _ = detect_zeroclaw_dir();
    }

    #[test]
    fn test_detect_zeroclaw_dir_env_var() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("config.toml"), "# empty").unwrap();

        std::env::set_var("ZEROCLAW_STATE_DIR", tmp.path().to_str().unwrap());
        let result = detect_zeroclaw_dir();
        std::env::remove_var("ZEROCLAW_STATE_DIR");

        assert_eq!(result, Some(tmp.path().to_path_buf()));
    }

    #[test]
    fn test_load_zeroclaw_config() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("config.toml"), sample_zeroclaw_toml()).unwrap();

        let config = load_zeroclaw_config(tmp.path()).unwrap();
        assert_eq!(
            config["default_chat_model"]["provider"].as_str(),
            Some("gemini")
        );
        assert_eq!(
            config["default_chat_model"]["model"].as_str(),
            Some("gemini-2.5-flash")
        );
    }

    #[test]
    fn test_convert_zeroclaw_provider() {
        let toml_val: toml::Value = sample_zeroclaw_toml().parse().unwrap();
        let mut config = Config::default();
        let result = convert_zeroclaw_config(&toml_val, &mut config);

        let gemini = config.providers.gemini.as_ref().unwrap();
        assert_eq!(gemini.api_key.as_deref(), Some("AIza-test-key"));
        assert_eq!(config.agents.defaults.model, "gemini-2.5-flash");
        assert!(result.migrated.iter().any(|s| s.contains("gemini")));
    }

    #[test]
    fn test_convert_zeroclaw_discord() {
        let toml_val: toml::Value = sample_zeroclaw_toml().parse().unwrap();
        let mut config = Config::default();
        convert_zeroclaw_config(&toml_val, &mut config);

        let dc = config.channels.discord.as_ref().unwrap();
        assert!(dc.enabled);
        assert_eq!(dc.token, "MTIz-discord-token");
        assert_eq!(dc.allow_from, vec!["691964645736448032"]);
    }

    #[test]
    fn test_convert_zeroclaw_workspace_path_rewrite() {
        let toml_val: toml::Value = sample_zeroclaw_toml().parse().unwrap();
        let mut config = Config::default();
        convert_zeroclaw_config(&toml_val, &mut config);

        assert_eq!(config.agents.defaults.workspace, "~/.claide/workspace");
    }

    #[test]
    fn test_convert_zeroclaw_transcription() {
        let toml_val: toml::Value = sample_zeroclaw_toml().parse().unwrap();
        let mut config = Config::default();
        convert_zeroclaw_config(&toml_val, &mut config);

        assert!(config.transcription.enabled);
        assert_eq!(config.transcription.model, "whisper-large-v3-turbo");
    }

    #[test]
    fn test_convert_zeroclaw_not_portable() {
        let toml_val: toml::Value = sample_zeroclaw_toml().parse().unwrap();
        let mut config = Config::default();
        let result = convert_zeroclaw_config(&toml_val, &mut config);

        // heartbeat should be not-portable
        assert!(result.not_portable.iter().any(|s| s.contains("heartbeat")));
    }

    #[test]
    fn test_convert_zeroclaw_empty() {
        let toml_val: toml::Value = "".parse().unwrap();
        let mut config = Config::default();
        let result = convert_zeroclaw_config(&toml_val, &mut config);

        assert!(result.migrated.is_empty());
        assert!(result.not_portable.is_empty());
    }

    #[test]
    fn test_copy_workspace_persona_files() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();

        let ws = src.path().join("workspace");
        fs::create_dir_all(&ws).unwrap();
        fs::write(ws.join("SOUL.md"), "# Boris").unwrap();
        fs::write(ws.join("USER.md"), "# Caide").unwrap();

        let dst_ws = dst.path().join("workspace");
        let result = copy_workspace(src.path(), &dst_ws).unwrap();

        assert!(result.copied_files.contains(&"SOUL.md".to_string()));
        assert!(result.copied_files.contains(&"USER.md".to_string()));
        assert!(dst_ws.join("SOUL.md").is_file());
        assert!(dst_ws.join("USER.md").is_file());
    }

    #[test]
    fn test_copy_workspace_no_overwrite() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();

        let ws = src.path().join("workspace");
        fs::create_dir_all(&ws).unwrap();
        fs::write(ws.join("SOUL.md"), "# New Boris").unwrap();

        let dst_ws = dst.path().join("workspace");
        fs::create_dir_all(&dst_ws).unwrap();
        fs::write(dst_ws.join("SOUL.md"), "# Existing Boris").unwrap();

        let result = copy_workspace(src.path(), &dst_ws).unwrap();

        assert!(result.copied_files.is_empty());
        assert!(result.skipped_files.iter().any(|(n, _)| n == "SOUL.md"));
        // Existing file should NOT be overwritten
        let content = fs::read_to_string(dst_ws.join("SOUL.md")).unwrap();
        assert_eq!(content, "# Existing Boris");
    }

    #[test]
    fn test_copy_workspace_scripts_and_data() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();

        let ws = src.path().join("workspace");
        fs::create_dir_all(ws.join("scripts")).unwrap();
        fs::create_dir_all(ws.join("data")).unwrap();
        fs::write(ws.join("scripts/morning-briefing"), "#!/bin/bash\necho hi").unwrap();
        fs::write(ws.join("data/birthdays.txt"), "2000-01-15 | Alice").unwrap();

        let dst_ws = dst.path().join("workspace");
        let result = copy_workspace(src.path(), &dst_ws).unwrap();

        assert!(result
            .copied_files
            .iter()
            .any(|s| s.contains("morning-briefing")));
        assert!(result
            .copied_files
            .iter()
            .any(|s| s.contains("birthdays.txt")));
        assert!(dst_ws.join("scripts/morning-briefing").is_file());
        assert!(dst_ws.join("data/birthdays.txt").is_file());
    }

    #[test]
    fn test_copy_workspace_missing_dir() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        // No workspace/ directory in src

        let dst_ws = dst.path().join("workspace");
        let result = copy_workspace(src.path(), &dst_ws).unwrap();

        assert!(result.copied_files.is_empty());
        assert!(result.skipped_files.is_empty());
    }
}
