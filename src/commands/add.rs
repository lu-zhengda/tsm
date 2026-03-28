use crate::client::TransmissionClient;
use crate::error::Error;
use crate::output::json;
use crate::rpc::methods;

pub fn execute(
    client: &TransmissionClient,
    source: &str,
    paused: bool,
    download_dir: Option<&str>,
    json_output: bool,
) -> Result<(), Error> {
    let result = methods::torrent_add(client, source, paused, download_dir)?;

    if json_output {
        json::print_json(&result)?;
    } else if let Some(added) = result.get("torrent-added") {
        let name = added
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("Unknown");
        let id = added.get("id").and_then(|i| i.as_i64()).unwrap_or(0);
        println!("Added torrent: {name} (ID: {id})");
    } else if let Some(duplicate) = result.get("torrent-duplicate") {
        let name = duplicate
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("Unknown");
        println!("Torrent already exists: {name}");
    } else {
        println!("Torrent added successfully.");
    }

    Ok(())
}
