use comfy_table::{Cell, ContentArrangement, Table};

use super::color;
use crate::rpc::types::*;

pub fn format_size(bytes: i64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let b = bytes as f64;
    if b >= TB {
        format!("{:.1} TB", b / TB)
    } else if b >= GB {
        format!("{:.1} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

pub fn format_speed(bytes_per_sec: i64) -> String {
    if bytes_per_sec == 0 {
        return "-".to_string();
    }
    format!("{}/s", format_size(bytes_per_sec))
}

pub fn format_eta(eta: i64) -> String {
    if eta < 0 {
        return "-".to_string();
    }
    if eta == 0 {
        return "Done".to_string();
    }

    let hours = eta / 3600;
    let minutes = (eta % 3600) / 60;

    if hours > 24 {
        let days = hours / 24;
        format!("{days}d")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else if minutes > 0 {
        format!("{minutes}m")
    } else {
        format!("{eta}s")
    }
}

pub fn format_ratio(ratio: f64) -> String {
    if ratio < 0.0 {
        "-".to_string()
    } else {
        format!("{ratio:.2}")
    }
}

pub fn status_string(status: i64) -> &'static str {
    match status {
        0 => "Stopped",
        1 => "Check Wait",
        2 => "Checking",
        3 => "DL Wait",
        4 => "Downloading",
        5 => "Seed Wait",
        6 => "Seeding",
        _ => "Unknown",
    }
}

pub fn print_torrent_list(torrents: &[Torrent], no_color: bool) {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        "ID", "Status", "Name", "Size", "Progress", "Down", "Up", "ETA", "Ratio",
    ]);

    for t in torrents {
        let status_cell = if no_color {
            Cell::new(status_string(t.status))
        } else if let Some(c) = color::status_color(t.status) {
            Cell::new(status_string(t.status)).fg(c)
        } else {
            Cell::new(status_string(t.status))
        };

        let progress = color::format_progress_bar(t.percent_done, 10);

        table.add_row(vec![
            Cell::new(t.id),
            status_cell,
            Cell::new(truncate_name(&t.name, 40)),
            Cell::new(format_size(t.total_size)),
            Cell::new(progress),
            Cell::new(format_speed(t.rate_download)),
            Cell::new(format_speed(t.rate_upload)),
            Cell::new(format_eta(t.eta)),
            Cell::new(format_ratio(t.upload_ratio)),
        ]);
    }

    println!("{table}");
}

pub fn print_torrent_detail(t: &TorrentDetail) {
    println!("Name:        {}", t.name);
    println!("ID:          {}", t.id);
    println!("Hash:        {}", t.hash_string);
    println!("Status:      {}", status_string(t.status));
    println!("Size:        {}", format_size(t.total_size));
    println!("Progress:    {:.1}%", t.percent_done * 100.0);
    println!("Downloaded:  {}", format_size(t.downloaded_ever));
    println!("Uploaded:    {}", format_size(t.uploaded_ever));
    println!("Ratio:       {}", format_ratio(t.upload_ratio));
    println!("Down Speed:  {}", format_speed(t.rate_download));
    println!("Up Speed:    {}", format_speed(t.rate_upload));
    println!("ETA:         {}", format_eta(t.eta));
    println!("Location:    {}", t.download_dir);
    println!(
        "Pieces:      {} x {}",
        t.piece_count,
        format_size(t.piece_size)
    );

    if !t.labels.is_empty() {
        println!("Labels:      {}", t.labels.join(", "));
    }
    if let Some(comment) = &t.comment
        && !comment.is_empty()
    {
        println!("Comment:     {comment}");
    }

    if !t.trackers.is_empty() {
        println!("\nTrackers:");
        for tracker in &t.trackers {
            println!("  {}", tracker.announce);
        }
    }

    if !t.peers.is_empty() {
        println!("\nPeers ({}):", t.peers.len());
        let mut peer_table = Table::new();
        peer_table.set_content_arrangement(ContentArrangement::Dynamic);
        peer_table.set_header(vec!["Address", "Client", "Down", "Up"]);
        for peer in &t.peers {
            peer_table.add_row(vec![
                peer.address.clone(),
                peer.client_name.clone(),
                format_speed(peer.rate_to_client),
                format_speed(peer.rate_to_peer),
            ]);
        }
        println!("{peer_table}");
    }
}

pub fn print_torrent_files(name: &str, files: &[TorrentFile], stats: &[TorrentFileStat]) {
    println!("Torrent: {name}");
    println!();

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["#", "Name", "Size", "Done", "Priority", "Wanted"]);

    for (i, f) in files.iter().enumerate() {
        let progress = if f.length > 0 {
            format!(
                "{:.0}%",
                (f.bytes_completed as f64 / f.length as f64) * 100.0
            )
        } else {
            "0%".to_string()
        };

        let stat = stats.get(i);
        let priority = match stat.map(|s| s.priority).unwrap_or(0) {
            1 => "High",
            -1 => "Low",
            _ => "Normal",
        };
        let wanted = if stat.map(|s| s.wanted).unwrap_or(true) {
            "Yes"
        } else {
            "No"
        };

        table.add_row(vec![
            i.to_string(),
            f.name.clone(),
            format_size(f.length),
            progress,
            priority.to_string(),
            wanted.to_string(),
        ]);
    }

    println!("{table}");
}

pub fn priority_string(priority: i64) -> &'static str {
    match priority {
        1 => "High",
        -1 => "Low",
        _ => "Normal",
    }
}

pub fn print_session_info(session: &serde_json::Value) {
    let get_str = |key: &str| -> String {
        session
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("-")
            .to_string()
    };
    let get_i64 = |key: &str| -> String {
        session
            .get(key)
            .and_then(|v| v.as_i64())
            .map(|v| v.to_string())
            .unwrap_or("-".to_string())
    };
    let get_bool = |key: &str| -> String {
        session
            .get(key)
            .and_then(|v| v.as_bool())
            .map(|v| if v { "Yes" } else { "No" }.to_string())
            .unwrap_or("-".to_string())
    };

    println!("Version:           {}", get_str("version"));
    println!("RPC Version:       {}", get_i64("rpc-version"));
    println!("Download Dir:      {}", get_str("download-dir"));
    println!("Peer Port:         {}", get_i64("peer-port"));
    println!("Port Forwarding:   {}", get_bool("port-forwarding-enabled"));
    println!("DHT Enabled:       {}", get_bool("dht-enabled"));
    println!("PEX Enabled:       {}", get_bool("pex-enabled"));
    println!("Encryption:        {}", get_str("encryption"));

    let dl_limited = session
        .get("speed-limit-down-enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let ul_limited = session
        .get("speed-limit-up-enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if dl_limited {
        println!("Download Limit:    {} KB/s", get_i64("speed-limit-down"));
    } else {
        println!("Download Limit:    Unlimited");
    }

    if ul_limited {
        println!("Upload Limit:      {} KB/s", get_i64("speed-limit-up"));
    } else {
        println!("Upload Limit:      Unlimited");
    }
}

pub fn print_session_stats(stats: &SessionStats) {
    println!("Active Torrents:   {}", stats.active_torrent_count);
    println!("Paused Torrents:   {}", stats.paused_torrent_count);
    println!("Total Torrents:    {}", stats.torrent_count);
    println!("Down Speed:        {}", format_speed(stats.download_speed));
    println!("Up Speed:          {}", format_speed(stats.upload_speed));

    if let Some(cumulative) = &stats.cumulative_stats {
        println!();
        println!("Cumulative Stats:");
        println!(
            "  Downloaded:      {}",
            format_size(cumulative.downloaded_bytes)
        );
        println!(
            "  Uploaded:        {}",
            format_size(cumulative.uploaded_bytes)
        );
        println!("  Files Added:     {}", cumulative.files_added);
        println!("  Sessions:        {}", cumulative.session_count);
        println!(
            "  Active Time:     {}",
            format_duration(cumulative.seconds_active)
        );
    }

    if let Some(current) = &stats.current_stats {
        println!();
        println!("Current Session:");
        println!(
            "  Downloaded:      {}",
            format_size(current.downloaded_bytes)
        );
        println!("  Uploaded:        {}", format_size(current.uploaded_bytes));
        println!(
            "  Active Time:     {}",
            format_duration(current.seconds_active)
        );
    }
}

pub fn print_free_space(space: &FreeSpace) {
    println!("Path:  {}", space.path);
    println!("Free:  {}", format_size(space.size_bytes));
    if let Some(total) = space.total_size {
        println!("Total: {}", format_size(total));
    }
}

pub fn print_speed_info(session: &serde_json::Value) {
    let dl_limited = session
        .get("speed-limit-down-enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let ul_limited = session
        .get("speed-limit-up-enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let alt_enabled = session
        .get("alt-speed-enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if dl_limited {
        let limit = session
            .get("speed-limit-down")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        println!("Download Limit:  {limit} KB/s");
    } else {
        println!("Download Limit:  Unlimited");
    }

    if ul_limited {
        let limit = session
            .get("speed-limit-up")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        println!("Upload Limit:    {limit} KB/s");
    } else {
        println!("Upload Limit:    Unlimited");
    }

    println!(
        "Alt-Speed Mode:  {}",
        if alt_enabled { "Enabled" } else { "Disabled" }
    );

    if alt_enabled {
        let alt_down = session
            .get("alt-speed-down")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let alt_up = session
            .get("alt-speed-up")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        println!("  Alt Down:      {alt_down} KB/s");
        println!("  Alt Up:        {alt_up} KB/s");
    }
}

fn truncate_name(name: &str, max_chars: usize) -> String {
    let char_count = name.chars().count();
    if char_count <= max_chars {
        name.to_string()
    } else {
        let truncated: String = name.chars().take(max_chars - 3).collect();
        format!("{truncated}...")
    }
}

fn format_duration(seconds: i64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
        assert_eq!(format_size(1073741824), "1.0 GB");
        assert_eq!(format_size(1099511627776), "1.0 TB");
    }

    #[test]
    fn test_format_speed() {
        assert_eq!(format_speed(0), "-");
        assert_eq!(format_speed(1024), "1.0 KB/s");
        assert_eq!(format_speed(2097152), "2.0 MB/s");
    }

    #[test]
    fn test_format_eta() {
        assert_eq!(format_eta(-1), "-");
        assert_eq!(format_eta(0), "Done");
        assert_eq!(format_eta(30), "30s");
        assert_eq!(format_eta(120), "2m");
        assert_eq!(format_eta(3661), "1h 1m");
        assert_eq!(format_eta(90000), "1d");
    }

    #[test]
    fn test_format_ratio() {
        assert_eq!(format_ratio(-1.0), "-");
        assert_eq!(format_ratio(0.0), "0.00");
        assert_eq!(format_ratio(1.5), "1.50");
        assert_eq!(format_ratio(2.345), "2.35");
    }

    #[test]
    fn test_status_string() {
        assert_eq!(status_string(0), "Stopped");
        assert_eq!(status_string(4), "Downloading");
        assert_eq!(status_string(6), "Seeding");
        assert_eq!(status_string(99), "Unknown");
    }

    #[test]
    fn test_truncate_name() {
        assert_eq!(truncate_name("short", 40), "short");
        assert_eq!(
            truncate_name("this is a very long name that exceeds the limit", 20),
            "this is a very lo..."
        );
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0m");
        assert_eq!(format_duration(120), "2m");
        assert_eq!(format_duration(3661), "1h 1m");
        assert_eq!(format_duration(90061), "1d 1h 1m");
    }
}
