use crate::client::TransmissionClient;
use crate::error::Error;
use crate::output::{json, table};
use crate::rpc::methods;

pub fn execute_session(client: &TransmissionClient, json_output: bool) -> Result<(), Error> {
    let session = methods::session_get(client)?;

    if json_output {
        json::print_json(&session)
    } else {
        table::print_session_info(&session);
        Ok(())
    }
}

pub fn execute_stats(client: &TransmissionClient, json_output: bool) -> Result<(), Error> {
    let stats = methods::session_stats(client)?;

    if json_output {
        json::print_json(&stats)
    } else {
        table::print_session_stats(&stats);
        Ok(())
    }
}

pub fn execute_free(
    client: &TransmissionClient,
    path: Option<&str>,
    json_output: bool,
) -> Result<(), Error> {
    // If no path provided, get download dir from session
    let target_path = match path {
        Some(p) => p.to_string(),
        None => {
            let session = methods::session_get(client)?;
            session
                .get("download-dir")
                .and_then(|v| v.as_str())
                .unwrap_or("/")
                .to_string()
        }
    };

    let space = methods::free_space(client, &target_path)?;

    if json_output {
        json::print_json(&space)
    } else {
        table::print_free_space(&space);
        Ok(())
    }
}
