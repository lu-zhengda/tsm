use crate::client::TransmissionClient;
use crate::error::Error;
use crate::rpc::methods;

pub fn execute(client: &TransmissionClient, id: i64, delete_data: bool) -> Result<(), Error> {
    methods::torrent_remove(client, id, delete_data)?;

    if delete_data {
        println!("Removed torrent {id} and deleted local data.");
    } else {
        println!("Removed torrent {id} (data kept).");
    }
    Ok(())
}
