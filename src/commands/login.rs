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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ProfileEntry {
    host: String,
    port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
}

pub fn execute(profile_name: &str) -> Result<(), Error> {
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
    // Try to disable echo for password input
    print!("{label}: ");
    io::stdout().flush().ok();

    let password = read_password_from_tty()?;
    println!(); // newline after hidden input
    Ok(password)
}

fn read_password_from_tty() -> Result<String, Error> {
    // Use stty to disable echo on Unix
    let stty_result = std::process::Command::new("stty")
        .arg("-echo")
        .stdin(std::process::Stdio::inherit())
        .status();

    let mut input = String::new();
    let result = io::stdin().read_line(&mut input).map_err(Error::Io);

    // Always re-enable echo
    let _ = std::process::Command::new("stty")
        .arg("echo")
        .stdin(std::process::Stdio::inherit())
        .status();

    if stty_result.is_err() {
        // Fallback: stty not available, password was visible
    }

    result?;
    Ok(input.trim().to_string())
}

fn save_profile(
    config_path: &std::path::Path,
    profile_name: &str,
    host: &str,
    port: u16,
    username: &Option<String>,
    password: &Option<String>,
) -> Result<(), Error> {
    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            Error::Config(format!(
                "Cannot create config directory {}: {e}",
                parent.display()
            ))
        })?;
    }

    // Load existing config or create new
    let mut config_file = match std::fs::read_to_string(config_path) {
        Ok(content) => toml::from_str::<ConfigFile>(&content).unwrap_or_default(),
        Err(_) => ConfigFile::default(),
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

    std::fs::write(config_path, toml_string)
        .map_err(|e| Error::Config(format!("Failed to write config: {e}")))?;

    // Set file permissions to 600 (owner read/write only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(config_path, perms)
            .map_err(|e| Error::Config(format!("Failed to set config file permissions: {e}")))?;
    }

    Ok(())
}
