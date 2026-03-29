use crate::cli::FilePriority;
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

#[allow(clippy::too_many_arguments)]
pub fn execute_files(
    client: &TransmissionClient,
    id: i64,
    priority: Option<&FilePriority>,
    priority_indices: Option<&[usize]>,
    skip: Option<&[usize]>,
    unskip: Option<&[usize]>,
    json_output: bool,
) -> Result<(), Error> {
    let has_mutation = priority.is_some() || skip.is_some() || unskip.is_some();

    if has_mutation {
        // Pre-fetch to validate indices
        let (_, files, _) = methods::torrent_get_files(client, id)?;
        let file_count = files.len();

        // Validate all indices
        let all_indices: Vec<usize> = priority_indices
            .into_iter()
            .flatten()
            .chain(skip.into_iter().flatten())
            .chain(unskip.into_iter().flatten())
            .copied()
            .collect();

        for &idx in &all_indices {
            if idx >= file_count {
                return Err(Error::Config(format!(
                    "File index {idx} out of range (torrent has {file_count} files, indices 0-{})",
                    file_count.saturating_sub(1)
                )));
            }
        }

        // Build priority arrays
        let (p_high, p_normal, p_low) = match priority {
            Some(FilePriority::High) => (priority_indices, None, None),
            Some(FilePriority::Normal) => (None, priority_indices, None),
            Some(FilePriority::Low) => (None, None, priority_indices),
            None => (None, None, None),
        };

        methods::torrent_set_file_properties(client, id, p_high, p_normal, p_low, unskip, skip)?;
    }

    // Fetch and display
    let (name, files, stats) = methods::torrent_get_files(client, id)?;

    if json_output {
        let json_files: Vec<serde_json::Value> = files
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let stat = stats.get(i);
                serde_json::json!({
                    "index": i,
                    "name": f.name,
                    "length": f.length,
                    "bytesCompleted": f.bytes_completed,
                    "priority": table::priority_string(stat.map(|s| s.priority).unwrap_or(0)),
                    "wanted": stat.map(|s| s.wanted).unwrap_or(true),
                })
            })
            .collect();
        json::print_json(&json_files)
    } else {
        table::print_torrent_files(&name, &files, &stats);
        Ok(())
    }
}
