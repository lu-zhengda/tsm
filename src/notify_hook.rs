use crate::config::Config;

pub fn fire_completion(name: &str, id: i64, config: &Config) {
    if let Some(script) = &config.on_complete_script {
        match std::process::Command::new(script)
            .arg(name)
            .arg(id.to_string())
            .spawn()
        {
            Ok(_) => {}
            Err(e) => eprintln!("Warning: notification script failed: {e}"),
        }
    }

    if let Some(url) = &config.on_complete_webhook {
        let payload = serde_json::json!({
            "event": "torrent_complete",
            "name": name,
            "id": id,
        });
        let body = serde_json::to_vec(&payload).unwrap_or_default();
        let agent: ureq::Agent = ureq::Agent::new_with_defaults();
        match agent
            .post(url)
            .header("Content-Type", "application/json")
            .send(&body[..])
        {
            Ok(_) => {}
            Err(e) => eprintln!("Warning: notification webhook failed: {e}"),
        }
    }
}
