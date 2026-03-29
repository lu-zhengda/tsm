use crate::client::TransmissionClient;
use crate::error::Error;
use crate::output::json as json_output;
use crate::rpc::methods;

pub fn execute(
    client: &TransmissionClient,
    id: i64,
    _on: bool,
    off: bool,
    json_mode: bool,
) -> Result<(), Error> {
    let enable = !off; // default to on when neither flag is set
    methods::torrent_set_sequential(client, id, enable)?;

    let (name, sequential) = methods::torrent_get_sequential(client, id)?;

    match sequential {
        Some(state) => {
            if json_mode {
                json_output::print_json(&serde_json::json!({
                    "id": id,
                    "name": name,
                    "sequential_download": state,
                }))
            } else {
                let status = if state { "Enabled" } else { "Disabled" };
                println!("Sequential download for \"{name}\" (ID: {id}): {status}");
                Ok(())
            }
        }
        None => Err(Error::Rpc(
            "Sequential download not supported (requires Transmission 4.0+)".to_string(),
        )),
    }
}
