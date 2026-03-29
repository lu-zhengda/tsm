use comfy_table::{ContentArrangement, Table};

use crate::cli::TrackerAction;
use crate::client::TransmissionClient;
use crate::error::Error;
use crate::rpc::methods;

pub fn execute(
    client: &TransmissionClient,
    action: &TrackerAction,
    json_mode: bool,
) -> Result<(), Error> {
    match action {
        TrackerAction::List { id } => execute_list(client, *id, json_mode),
        TrackerAction::Add { id, url } => execute_add(client, *id, url),
        TrackerAction::Remove { id, url } => execute_remove(client, *id, url),
        TrackerAction::Replace { from, to } => execute_replace(client, from, to, json_mode),
    }
}

fn execute_list(client: &TransmissionClient, id: i64, json_mode: bool) -> Result<(), Error> {
    let (name, stats) = methods::torrent_get_tracker_stats(client, id)?;

    if json_mode {
        let json_stats: Vec<serde_json::Value> = stats
            .iter()
            .map(|s| {
                serde_json::json!({
                    "announce": s.announce,
                    "host": s.host,
                    "tier": s.tier,
                    "seeders": s.seeder_count,
                    "leechers": s.leecher_count,
                    "last_result": s.last_announce_result,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json_stats).unwrap_or_default()
        );
        return Ok(());
    }

    println!("Trackers for: {name} (ID: {id})");
    if stats.is_empty() {
        println!("  No trackers.");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        "Tier",
        "Announce URL",
        "Seeders",
        "Leechers",
        "Last Result",
    ]);

    for s in &stats {
        table.add_row(vec![
            s.tier.to_string(),
            s.announce.clone(),
            s.seeder_count.to_string(),
            s.leecher_count.to_string(),
            s.last_announce_result.clone(),
        ]);
    }

    println!("{table}");
    Ok(())
}

fn execute_add(client: &TransmissionClient, id: i64, url: &str) -> Result<(), Error> {
    let current = methods::torrent_get_tracker_list(client, id)?;
    let new_list = append_tracker(&current, url);
    methods::torrent_set_tracker_list(client, id, &new_list)?;
    println!("Added tracker to torrent {id}: {url}");
    Ok(())
}

fn execute_remove(client: &TransmissionClient, id: i64, url: &str) -> Result<(), Error> {
    let current = methods::torrent_get_tracker_list(client, id)?;
    let new_list = remove_tracker(&current, url);
    if new_list == current {
        println!("Tracker not found on torrent {id}: {url}");
        return Ok(());
    }
    methods::torrent_set_tracker_list(client, id, &new_list)?;
    println!("Removed tracker from torrent {id}: {url}");
    Ok(())
}

fn execute_replace(
    client: &TransmissionClient,
    from: &str,
    to: &str,
    json_mode: bool,
) -> Result<(), Error> {
    let torrents = methods::torrent_get_list(client)?;
    let mut replaced_count = 0u32;

    for t in &torrents {
        let current = methods::torrent_get_tracker_list(client, t.id)?;
        if current.contains(from) {
            let new_list = current.replace(from, to);
            methods::torrent_set_tracker_list(client, t.id, &new_list)?;
            replaced_count += 1;
            if !json_mode {
                println!("Replaced tracker on ID {}: {}", t.id, t.name);
            }
        }
    }

    if json_mode {
        println!(
            "{}",
            serde_json::json!({ "replaced_count": replaced_count })
        );
    } else if replaced_count == 0 {
        println!("No torrents had tracker: {from}");
    } else {
        println!("Replaced tracker on {replaced_count} torrent(s).");
    }
    Ok(())
}

/// Append a tracker URL as a new tier (separated by \n\n)
fn append_tracker(tracker_list: &str, url: &str) -> String {
    if tracker_list.is_empty() {
        url.to_string()
    } else {
        format!("{tracker_list}\n\n{url}")
    }
}

/// Remove a tracker URL, preserving tier structure
fn remove_tracker(tracker_list: &str, url: &str) -> String {
    let tiers: Vec<&str> = tracker_list.split("\n\n").collect();
    let filtered: Vec<String> = tiers
        .iter()
        .map(|tier| {
            tier.lines()
                .filter(|line| line.trim() != url)
                .collect::<Vec<_>>()
                .join("\n")
        })
        .filter(|tier| !tier.is_empty())
        .collect();
    filtered.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_tracker_empty() {
        let result = append_tracker("", "udp://tracker.example.com:1337/announce");
        assert_eq!(result, "udp://tracker.example.com:1337/announce");
    }

    #[test]
    fn test_append_tracker_existing() {
        let result = append_tracker(
            "udp://existing.com:1337/announce",
            "udp://new.com:1337/announce",
        );
        assert_eq!(
            result,
            "udp://existing.com:1337/announce\n\nudp://new.com:1337/announce"
        );
    }

    #[test]
    fn test_remove_tracker_single_tier() {
        let list = "udp://a.com:1337/announce\nudp://b.com:1337/announce";
        let result = remove_tracker(list, "udp://a.com:1337/announce");
        assert_eq!(result, "udp://b.com:1337/announce");
    }

    #[test]
    fn test_remove_tracker_multi_tier() {
        let list = "udp://a.com:1337/announce\n\nudp://b.com:1337/announce";
        let result = remove_tracker(list, "udp://b.com:1337/announce");
        assert_eq!(result, "udp://a.com:1337/announce");
    }

    #[test]
    fn test_remove_tracker_not_found() {
        let list = "udp://a.com:1337/announce";
        let result = remove_tracker(list, "udp://notfound.com:1337/announce");
        assert_eq!(result, "udp://a.com:1337/announce");
    }

    #[test]
    fn test_remove_tracker_preserves_tiers() {
        let list = "udp://a.com/announce\nudp://b.com/announce\n\nudp://c.com/announce";
        let result = remove_tracker(list, "udp://b.com/announce");
        assert_eq!(result, "udp://a.com/announce\n\nudp://c.com/announce");
    }
}
