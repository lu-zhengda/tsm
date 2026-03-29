use serde_json::json;

use crate::cli::BandwidthPriority;
use crate::client::TransmissionClient;
use crate::error::Error;
use crate::output::{json as json_output, table};
use crate::rpc::methods;

#[allow(clippy::too_many_arguments)]
pub fn execute(
    client: &TransmissionClient,
    id: Option<i64>,
    set_down: Option<i64>,
    set_up: Option<i64>,
    alt_on: bool,
    alt_off: bool,
    no_limit: bool,
    priority: Option<&BandwidthPriority>,
    no_honor_global: bool,
    json_mode: bool,
) -> Result<(), Error> {
    match id {
        Some(torrent_id) => execute_per_torrent(
            client,
            torrent_id,
            set_down,
            set_up,
            no_limit,
            priority,
            no_honor_global,
            json_mode,
        ),
        None => execute_session(
            client, set_down, set_up, alt_on, alt_off, no_limit, json_mode,
        ),
    }
}

fn execute_session(
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
        apply_session_changes(client, set_down, set_up, alt_on, alt_off, no_limit)?;
    }

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

#[allow(clippy::too_many_arguments)]
fn execute_per_torrent(
    client: &TransmissionClient,
    id: i64,
    set_down: Option<i64>,
    set_up: Option<i64>,
    no_limit: bool,
    priority: Option<&BandwidthPriority>,
    no_honor_global: bool,
    json_mode: bool,
) -> Result<(), Error> {
    let has_mutation =
        set_down.is_some() || set_up.is_some() || no_limit || priority.is_some() || no_honor_global;

    if has_mutation {
        let honors_session = if no_honor_global { Some(false) } else { None };
        methods::torrent_set_bandwidth(
            client,
            id,
            set_down,
            set_up,
            priority.map(|p| p.to_rpc_value()),
            honors_session,
            no_limit,
        )?;
    }

    // Show current torrent bandwidth info
    let info = methods::torrent_get_bandwidth(client, id)?;

    if json_mode {
        json_output::print_json(&info)
    } else {
        print_torrent_bandwidth(&info);
        Ok(())
    }
}

fn print_torrent_bandwidth(info: &serde_json::Value) {
    let name = info
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");
    let id = info.get("id").and_then(|v| v.as_i64()).unwrap_or(0);

    println!("Torrent: {name} (ID: {id})");

    let dl_limited = info
        .get("downloadLimited")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let ul_limited = info
        .get("uploadLimited")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if dl_limited {
        let limit = info
            .get("downloadLimit")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        println!("Download Limit:    {limit} KB/s");
    } else {
        println!("Download Limit:    Unlimited");
    }

    if ul_limited {
        let limit = info
            .get("uploadLimit")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        println!("Upload Limit:      {limit} KB/s");
    } else {
        println!("Upload Limit:      Unlimited");
    }

    let priority = info
        .get("bandwidthPriority")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let priority_str = match priority {
        -1 => "Low",
        0 => "Normal",
        1 => "High",
        _ => "Unknown",
    };
    println!("Priority:          {priority_str}");

    let honors = info
        .get("honorsSessionLimits")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    println!("Honors Session:    {}", if honors { "Yes" } else { "No" });
}

fn apply_session_changes(
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
