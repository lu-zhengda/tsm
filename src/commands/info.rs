use crate::client::TransmissionClient;
use crate::error::Error;
use crate::output::{json, table};
use crate::rpc::methods;

pub fn execute_info(client: &TransmissionClient, id: i64, json_output: bool) -> Result<(), Error> {
    let detail = methods::torrent_get_detail(client, id)?;

    if json_output {
        json::print_json(&detail)
    } else {
        table::print_torrent_detail(&detail);
        Ok(())
    }
}

pub fn execute_files(client: &TransmissionClient, id: i64, json_output: bool) -> Result<(), Error> {
    let (name, files) = methods::torrent_get_files(client, id)?;

    if json_output {
        json::print_json(&files)
    } else {
        table::print_torrent_files(&name, &files);
        Ok(())
    }
}
