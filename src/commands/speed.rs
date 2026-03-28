use serde_json::json;

use crate::client::TransmissionClient;
use crate::error::Error;
use crate::output::{json as json_output, table};
use crate::rpc::methods;

pub fn execute(
    client: &TransmissionClient,
    set_down: Option<i64>,
    set_up: Option<i64>,
    alt_on: bool,
    alt_off: bool,
    no_limit: bool,
    json_mode: bool,
) -> Result<(), Error> {
    let has_mutation = set_down.is_some() || set_up.is_some() || alt_on || alt_off || no_limit;

    if has_mutation {
        apply_speed_changes(client, set_down, set_up, alt_on, alt_off, no_limit)?;
    }

    // Always show current state after changes
    let session = methods::session_get(client)?;

    if json_mode {
        let speed_info = json!({
            "speed-limit-down": session.get("speed-limit-down"),
            "speed-limit-down-enabled": session.get("speed-limit-down-enabled"),
            "speed-limit-up": session.get("speed-limit-up"),
            "speed-limit-up-enabled": session.get("speed-limit-up-enabled"),
            "alt-speed-enabled": session.get("alt-speed-enabled"),
            "alt-speed-down": session.get("alt-speed-down"),
            "alt-speed-up": session.get("alt-speed-up"),
        });
        json_output::print_json(&speed_info)
    } else {
        table::print_speed_info(&session);
        Ok(())
    }
}

fn apply_speed_changes(
    client: &TransmissionClient,
    set_down: Option<i64>,
    set_up: Option<i64>,
    alt_on: bool,
    alt_off: bool,
    no_limit: bool,
) -> Result<(), Error> {
    let mut params = serde_json::Map::new();

    if no_limit {
        params.insert("speed-limit-down-enabled".to_string(), json!(false));
        params.insert("speed-limit-up-enabled".to_string(), json!(false));
        params.insert("alt-speed-enabled".to_string(), json!(false));
    }

    if let Some(limit) = set_down {
        params.insert("speed-limit-down".to_string(), json!(limit));
        params.insert("speed-limit-down-enabled".to_string(), json!(true));
    }

    if let Some(limit) = set_up {
        params.insert("speed-limit-up".to_string(), json!(limit));
        params.insert("speed-limit-up-enabled".to_string(), json!(true));
    }

    if alt_on {
        params.insert("alt-speed-enabled".to_string(), json!(true));
    }

    if alt_off {
        params.insert("alt-speed-enabled".to_string(), json!(false));
    }

    if !params.is_empty() {
        methods::session_set(client, serde_json::Value::Object(params))?;
    }

    Ok(())
}
