use crate::config::{self, Config};
use crate::error::Error;
use crate::output::json;

pub fn execute(config: &Config, json_output: bool) -> Result<(), Error> {
    let config_path = config::default_config_path();

    if json_output {
        let value = serde_json::json!({
            "host": config.host,
            "port": config.port,
            "username": config.username,
            "password": config.password.as_ref().map(|_| "[REDACTED]"),
            "json": config.json,
            "no_color": config.no_color,
            "config_path": config_path.display().to_string(),
        });
        json::print_json(&value)
    } else {
        println!("Host:        {}", config.host);
        println!("Port:        {}", config.port);
        println!("Username:    {}", config.username.as_deref().unwrap_or("-"));
        println!(
            "Password:    {}",
            if config.password.is_some() {
                "[REDACTED]"
            } else {
                "-"
            }
        );
        println!("JSON:        {}", config.json);
        println!("No Color:    {}", config.no_color);
        println!("Config File: {}", config_path.display());
        Ok(())
    }
}
