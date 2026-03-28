use serde_json::json;

use crate::client::TransmissionClient;
use crate::error::Error;
use crate::rpc::methods;

pub fn execute_start(client: &TransmissionClient, target: &str) -> Result<(), Error> {
    let ids = parse_target(target)?;
    methods::torrent_start(client, ids)?;

    if target == "all" {
        println!("Started all torrents.");
    } else {
        println!("Started torrent {target}.");
    }
    Ok(())
}

pub fn execute_stop(client: &TransmissionClient, target: &str) -> Result<(), Error> {
    let ids = parse_target(target)?;
    methods::torrent_stop(client, ids)?;

    if target == "all" {
        println!("Stopped all torrents.");
    } else {
        println!("Stopped torrent {target}.");
    }
    Ok(())
}

pub fn execute_verify(client: &TransmissionClient, id: i64) -> Result<(), Error> {
    methods::torrent_verify(client, id)?;
    println!("Verifying torrent {id}.");
    Ok(())
}

fn parse_target(target: &str) -> Result<serde_json::Value, Error> {
    if target == "all" {
        // Empty ids array means "all" in Transmission RPC
        // Actually, we need to get all torrent IDs
        Ok(json!([]))
    } else {
        let id: i64 = target
            .parse()
            .map_err(|_| Error::Config(format!("Invalid torrent ID: {target}")))?;
        Ok(json!([id]))
    }
}
