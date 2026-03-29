use crate::client::TransmissionClient;
use crate::error::Error;
use crate::rpc::methods;

pub fn execute(client: &TransmissionClient, id: i64) -> Result<(), Error> {
    methods::torrent_reannounce(client, id)?;
    println!("Reannouncing torrent {id}.");
    Ok(())
}
