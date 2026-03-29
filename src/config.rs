use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

use crate::cli::Cli;
use crate::error::Error;

#[derive(Debug, Deserialize, Clone)]
pub struct SeedPolicy {
    pub name: String,
    pub match_label: String,
    pub seed_ratio: Option<f64>,
    pub seed_idle_minutes: Option<i64>,
}

#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub json: bool,
    pub no_color: bool,
    pub on_complete_script: Option<String>,
    pub on_complete_webhook: Option<String>,
    pub policies: Vec<SeedPolicy>,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("username", &self.username)
            .field("password", &self.password.as_ref().map(|_| "[REDACTED]"))
            .field("json", &self.json)
            .field("no_color", &self.no_color)
            .field("on_complete_script", &self.on_complete_script)
            .field("on_complete_webhook", &self.on_complete_webhook)
            .field("policies", &self.policies)
            .finish()
    }
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    default: Option<ProfileConfig>,
    profiles: Option<HashMap<String, ProfileConfig>>,
    notifications: Option<NotificationConfig>,
    #[serde(default)]
    policies: Vec<SeedPolicy>,
}

#[derive(Debug, Deserialize, Default, Clone)]
struct NotificationConfig {
    on_complete: Option<String>,
    webhook: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ProfileConfig {
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
}

pub fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("tsm")
        .join("config.toml")
}

fn load_config_file(path: &std::path::Path) -> Result<Option<ConfigFile>, Error> {
    match std::fs::read_to_string(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::Config(format!("Cannot read config file: {e}"))),
        Ok(content) => toml::from_str(&content)
            .map(Some)
            .map_err(|e| Error::Config(format!("Invalid config file syntax: {e}"))),
    }
}

fn resolve_profile(file: &ConfigFile, profile_name: &str) -> ProfileConfig {
    if profile_name == "default"
        && let Some(default) = &file.default
    {
        return ProfileConfig {
            host: default.host.clone(),
            port: default.port,
            username: default.username.clone(),
            password: default.password.clone(),
        };
    }

    if let Some(profiles) = &file.profiles
        && let Some(profile) = profiles.get(profile_name)
    {
        return ProfileConfig {
            host: profile.host.clone(),
            port: profile.port,
            username: profile.username.clone(),
            password: profile.password.clone(),
        };
    }

    ProfileConfig::default()
}

pub fn resolve(cli: &Cli) -> Result<Config, Error> {
    // Load .env file (best-effort)
    let _ = dotenvy::dotenv();
    resolve_from_env(cli)
}

fn resolve_from_env(cli: &Cli) -> Result<Config, Error> {
    // Layer 1: Config file
    let config_path = cli.config.clone().unwrap_or_else(default_config_path);
    let config_file = load_config_file(&config_path)?;
    let notifications = config_file
        .as_ref()
        .and_then(|f| f.notifications.clone())
        .unwrap_or_default();
    let policies = config_file
        .as_ref()
        .map(|f| f.policies.clone())
        .unwrap_or_default();
    let file_profile = match config_file {
        Some(f) => {
            let env_profile = std::env::var("TSM_PROFILE").ok();
            let profile_name = cli
                .profile
                .as_deref()
                .or(env_profile.as_deref())
                .unwrap_or("default");
            resolve_profile(&f, profile_name)
        }
        None => ProfileConfig::default(),
    };

    // Layer 2: Env vars (ignore empty values)
    let env_host = std::env::var("TSM_HOST").ok().filter(|s| !s.is_empty());
    let env_port = std::env::var("TSM_PORT")
        .ok()
        .filter(|s| !s.is_empty())
        .and_then(|p| p.parse().ok());
    let env_username = std::env::var("TSM_USERNAME").ok().filter(|s| !s.is_empty());
    let env_password = std::env::var("TSM_PASSWORD").ok().filter(|s| !s.is_empty());

    // Layer 3: CLI flags (highest priority)
    let host = cli
        .host
        .clone()
        .or(env_host)
        .or(file_profile.host)
        .unwrap_or_else(|| "localhost".to_string());

    let port = cli.port.or(env_port).or(file_profile.port).unwrap_or(9091);

    let username = cli
        .username
        .clone()
        .or(env_username)
        .or(file_profile.username);

    let password = cli
        .password
        .clone()
        .or(env_password)
        .or(file_profile.password);

    let no_color = cli.no_color || std::env::var("NO_COLOR").is_ok();

    Ok(Config {
        host,
        port,
        username,
        password,
        json: cli.json,
        no_color,
        on_complete_script: notifications.on_complete,
        on_complete_webhook: notifications.webhook,
        policies,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Env-var tests must run sequentially to avoid races.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn make_cli(
        host: Option<&str>,
        port: Option<u16>,
        username: Option<&str>,
        password: Option<&str>,
        json: bool,
    ) -> Cli {
        Cli {
            host: host.map(String::from),
            port,
            username: username.map(String::from),
            password: password.map(String::from),
            json,
            config: Some(PathBuf::from("/nonexistent/config.toml")),
            profile: None,
            no_color: false,
            command: crate::cli::Command::Session,
        }
    }

    #[test]
    fn test_defaults() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::remove_var("TSM_HOST");
            std::env::remove_var("TSM_PORT");
            std::env::remove_var("TSM_USERNAME");
            std::env::remove_var("TSM_PASSWORD");
        }

        let cli = make_cli(None, None, None, None, false);
        let config = resolve_from_env(&cli).unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 9091);
        assert!(config.username.is_none());
        assert!(config.password.is_none());
        assert!(!config.json);
    }

    #[test]
    fn test_cli_flags_override_env() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("TSM_HOST", "envhost");
            std::env::set_var("TSM_PORT", "1234");
        }

        let cli = make_cli(Some("clihost"), Some(5678), None, None, true);
        let config = resolve_from_env(&cli).unwrap();
        assert_eq!(config.host, "clihost");
        assert_eq!(config.port, 5678);
        assert!(config.json);

        unsafe {
            std::env::remove_var("TSM_HOST");
            std::env::remove_var("TSM_PORT");
        }
    }

    #[test]
    fn test_env_vars_override_defaults() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("TSM_HOST", "envhost");
            std::env::set_var("TSM_PORT", "4321");
            std::env::set_var("TSM_USERNAME", "envuser");
            std::env::set_var("TSM_PASSWORD", "envpass");
        }

        let cli = make_cli(None, None, None, None, false);
        let config = resolve_from_env(&cli).unwrap();
        assert_eq!(config.host, "envhost");
        assert_eq!(config.port, 4321);
        assert_eq!(config.username.as_deref(), Some("envuser"));
        assert_eq!(config.password.as_deref(), Some("envpass"));

        unsafe {
            std::env::remove_var("TSM_HOST");
            std::env::remove_var("TSM_PORT");
            std::env::remove_var("TSM_USERNAME");
            std::env::remove_var("TSM_PASSWORD");
        }
    }
}
