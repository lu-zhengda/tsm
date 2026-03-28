use crate::client::TransmissionClient;
use crate::error::Error;
use crate::rpc::methods;

pub fn execute(client: &TransmissionClient, id: i64, path: &str) -> Result<(), Error> {
    methods::torrent_set_location(client, id, path)?;
    println!("Torrent {id} moved to {path}");
    Ok(())
}
