use crate::cli::SortField;
use crate::client::TransmissionClient;
use crate::error::Error;
use crate::output::{json, table};
use crate::rpc::methods;

use super::list::sort_torrents;

pub fn execute(
    client: &TransmissionClient,
    query: &str,
    sort: &Option<SortField>,
    json_output: bool,
    no_color: bool,
) -> Result<(), Error> {
    let mut torrents = methods::torrent_get_list(client)?;

    let query_lower = query.to_lowercase();
    torrents.retain(|t| t.name.to_lowercase().contains(&query_lower));

    if let Some(sort) = sort {
        sort_torrents(&mut torrents, sort);
    }

    if json_output {
        json::print_json(&torrents)
    } else {
        if torrents.is_empty() {
            println!("No torrents found.");
        } else {
            table::print_torrent_list(&torrents, no_color);
        }
        Ok(())
    }
}
