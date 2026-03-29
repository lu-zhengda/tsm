use std::collections::HashMap;
use std::io::{self, Write};

use serde::{Deserialize, Serialize};

use crate::client::TransmissionClient;
use crate::config::Config;
use crate::error::Error;

#[derive(Debug, Serialize, Deserialize, Default)]
struct ConfigFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<ProfileEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    profiles: Option<HashMap<String, ProfileEntry>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ProfileEntry {
    host: String,
    port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
}

impl std::fmt::Debug for ProfileEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProfileEntry")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("username", &self.username)
            .field("password", &self.password.as_ref().map(|_| "[REDACTED]"))
            .finish()
    }
}

fn validate_profile_name(name: &str) -> Result<(), Error> {
    if name.is_empty() || name.len() > 64 {
        return Err(Error::Config(
            "Profile name must be 1-64 characters".to_string(),
        ));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(Error::Config(
            "Profile name may only contain [a-zA-Z0-9_-]".to_string(),
        ));
    }
    Ok(())
}

pub fn execute(profile_name: &str) -> Result<(), Error> {
    validate_profile_name(profile_name)?;

    println!("Configuring profile: {profile_name}");
    println!();

    let host = prompt("Host", "localhost")?;
    let port_str = prompt("Port", "9091")?;
    let port: u16 = port_str
        .parse()
        .map_err(|_| Error::Config(format!("Invalid port: {port_str}")))?;
    let username = prompt_optional("Username")?;
    let password = if username.is_some() {
        Some(prompt_password("Password")?)
    } else {
        None
    };

    // Test the connection
    print!("Testing connection... ");
    io::stdout().flush().ok();

    let config = Config {
        host: host.clone(),
        port,
        username: username.clone(),
        password: password.clone(),
        json: false,
        no_color: false,
        on_complete_script: None,
        on_complete_webhook: None,
        policies: vec![],
    };

    let client = TransmissionClient::new(&config)?;
    let session = crate::rpc::methods::session_get(&client)?;

    let version = session
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    println!("OK (Transmission {version})");

    // Save to config file
    let config_path = crate::config::default_config_path();
    save_profile(
        &config_path,
        profile_name,
        &host,
        port,
        &username,
        &password,
    )?;

    println!("Config saved to {}", config_path.display());
    Ok(())
}

fn prompt(label: &str, default: &str) -> Result<String, Error> {
    print!("{label} [{default}]: ");
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(Error::Io)?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

fn prompt_optional(label: &str) -> Result<Option<String>, Error> {
    print!("{label} (leave empty for none): ");
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(Error::Io)?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

fn prompt_password(label: &str) -> Result<String, Error> {
    let password = rpassword::prompt_password(format!("{label}: "))
        .map_err(|e| Error::Config(format!("Failed to read password: {e}")))?;
    Ok(password)
}

fn save_profile(
    config_path: &std::path::Path,
    profile_name: &str,
    host: &str,
    port: u16,
    username: &Option<String>,
    password: &Option<String>,
) -> Result<(), Error> {
    // Ensure parent directory exists with 700 permissions
    if let Some(parent) = config_path.parent() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            std::fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(parent)
                .map_err(|e| {
                    Error::Config(format!(
                        "Cannot create config directory {}: {e}",
                        parent.display()
                    ))
                })?;
        }
        #[cfg(not(unix))]
        {
            std::fs::create_dir_all(parent).map_err(|e| {
                Error::Config(format!(
                    "Cannot create config directory {}: {e}",
                    parent.display()
                ))
            })?;
        }
    }

    // Load existing config or create new
    let mut config_file = match std::fs::read_to_string(config_path) {
        Ok(content) => toml::from_str::<ConfigFile>(&content).map_err(|e| {
            Error::Config(format!(
                "Existing config file is invalid: {e}\nFix or delete {} before running login.",
                config_path.display()
            ))
        })?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => ConfigFile::default(),
        Err(e) => return Err(Error::Config(format!("Cannot read config file: {e}"))),
    };

    let entry = ProfileEntry {
        host: host.to_string(),
        port,
        username: username.clone(),
        password: password.clone(),
    };

    if profile_name == "default" {
        config_file.default = Some(entry);
    } else {
        let profiles = config_file.profiles.get_or_insert_with(HashMap::new);
        profiles.insert(profile_name.to_string(), entry);
    }

    let toml_string = toml::to_string_pretty(&config_file)
        .map_err(|e| Error::Config(format!("Failed to serialize config: {e}")))?;

    // Write atomically: create temp file with 600 perms, then rename
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let tmp_path = config_path.with_extension("tmp");
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp_path)
            .map_err(|e| Error::Config(format!("Failed to write config: {e}")))?;
        file.write_all(toml_string.as_bytes())
            .map_err(|e| Error::Config(format!("Failed to write config: {e}")))?;
        file.flush()
            .map_err(|e| Error::Config(format!("Failed to flush config: {e}")))?;
        std::fs::rename(&tmp_path, config_path)
            .map_err(|e| Error::Config(format!("Failed to save config: {e}")))?;
    }

    #[cfg(not(unix))]
    {
        std::fs::write(config_path, toml_string)
            .map_err(|e| Error::Config(format!("Failed to write config: {e}")))?;
    }

    Ok(())
}
