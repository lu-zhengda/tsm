use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};

use crate::client::TransmissionClient;
use crate::error::Error;
use crate::rpc::types::*;

const LIST_FIELDS: &[&str] = &[
    "id",
    "name",
    "status",
    "totalSize",
    "percentDone",
    "rateDownload",
    "rateUpload",
    "eta",
    "uploadRatio",
    "addedDate",
    "labels",
];

const DETAIL_FIELDS: &[&str] = &[
    "id",
    "name",
    "status",
    "totalSize",
    "percentDone",
    "rateDownload",
    "rateUpload",
    "eta",
    "uploadRatio",
    "addedDate",
    "hashString",
    "comment",
    "downloadDir",
    "pieceCount",
    "pieceSize",
    "downloadedEver",
    "uploadedEver",
    "labels",
    "trackers",
    "peers",
];

pub fn torrent_get_list(client: &TransmissionClient) -> Result<Vec<Torrent>, Error> {
    let params = json!({ "fields": LIST_FIELDS });
    let result = client.request("torrent-get", Some(params))?;

    let torrents_value = result
        .get("torrents")
        .ok_or_else(|| Error::Rpc("Missing 'torrents' field in response".to_string()))?;

    serde_json::from_value(torrents_value.clone())
        .map_err(|e| Error::Rpc(format!("Failed to parse torrents: {e}")))
}

pub fn torrent_get_detail(client: &TransmissionClient, id: i64) -> Result<TorrentDetail, Error> {
    let params = json!({ "fields": DETAIL_FIELDS, "ids": [id] });
    let result = client.request("torrent-get", Some(params))?;

    let torrents_value = result
        .get("torrents")
        .ok_or_else(|| Error::Rpc("Missing 'torrents' field in response".to_string()))?;

    let torrents: Vec<TorrentDetail> = serde_json::from_value(torrents_value.clone())
        .map_err(|e| Error::Rpc(format!("Failed to parse torrent detail: {e}")))?;

    torrents
        .into_iter()
        .next()
        .ok_or_else(|| Error::TorrentNotFound(id.to_string()))
}

pub fn torrent_get_files(
    client: &TransmissionClient,
    id: i64,
) -> Result<(String, Vec<TorrentFile>, Vec<TorrentFileStat>), Error> {
    let params = json!({ "fields": ["id", "name", "files", "priorities", "wanted"], "ids": [id] });
    let result = client.request("torrent-get", Some(params))?;

    let torrents_value = result
        .get("torrents")
        .ok_or_else(|| Error::Rpc("Missing 'torrents' field in response".to_string()))?;

    let torrents = torrents_value
        .as_array()
        .ok_or_else(|| Error::Rpc("Expected array of torrents".to_string()))?;

    let torrent = torrents
        .first()
        .ok_or_else(|| Error::TorrentNotFound(id.to_string()))?;

    let name = torrent
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let files: Vec<TorrentFile> = torrent
        .get("files")
        .map(|f| serde_json::from_value(f.clone()).unwrap_or_default())
        .unwrap_or_default();

    // Build file stats from priorities and wanted arrays
    let priorities: Vec<i64> = torrent
        .get("priorities")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect())
        .unwrap_or_default();

    let wanted: Vec<bool> = torrent
        .get("wanted")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|v| {
                    // Handle both bool and integer representations
                    v.as_bool().unwrap_or_else(|| v.as_i64().unwrap_or(1) != 0)
                })
                .collect()
        })
        .unwrap_or_default();

    let stats: Vec<TorrentFileStat> = (0..files.len())
        .map(|i| TorrentFileStat {
            priority: priorities.get(i).copied().unwrap_or(0),
            wanted: wanted.get(i).copied().unwrap_or(true),
        })
        .collect();

    Ok((name, files, stats))
}

pub fn torrent_set_file_properties(
    client: &TransmissionClient,
    id: i64,
    priority_high: Option<&[usize]>,
    priority_normal: Option<&[usize]>,
    priority_low: Option<&[usize]>,
    files_wanted: Option<&[usize]>,
    files_unwanted: Option<&[usize]>,
) -> Result<Value, Error> {
    let mut params = serde_json::Map::new();
    params.insert("ids".to_string(), json!([id]));

    if let Some(indices) = priority_high {
        params.insert("priority-high".to_string(), json!(indices));
    }
    if let Some(indices) = priority_normal {
        params.insert("priority-normal".to_string(), json!(indices));
    }
    if let Some(indices) = priority_low {
        params.insert("priority-low".to_string(), json!(indices));
    }
    if let Some(indices) = files_wanted {
        params.insert("files-wanted".to_string(), json!(indices));
    }
    if let Some(indices) = files_unwanted {
        params.insert("files-unwanted".to_string(), json!(indices));
    }

    client.request("torrent-set", Some(Value::Object(params)))
}

pub fn torrent_add(
    client: &TransmissionClient,
    source: &str,
    paused: bool,
    download_dir: Option<&str>,
) -> Result<Value, Error> {
    let mut params = serde_json::Map::new();

    // Detect if source is a file path or magnet/URL
    if source.starts_with("magnet:")
        || source.starts_with("http://")
        || source.starts_with("https://")
    {
        params.insert("filename".to_string(), json!(source));
    } else {
        // Validate file size before reading (torrent files should be small)
        let metadata = std::fs::metadata(source).map_err(Error::Io)?;
        const MAX_TORRENT_SIZE: u64 = 10 * 1024 * 1024; // 10 MB
        if metadata.len() > MAX_TORRENT_SIZE {
            return Err(Error::Config(format!(
                "File too large to be a torrent file: {} bytes (max {})",
                metadata.len(),
                MAX_TORRENT_SIZE
            )));
        }
        let file_bytes = std::fs::read(source).map_err(Error::Io)?;
        let encoded = STANDARD.encode(&file_bytes);
        params.insert("metainfo".to_string(), json!(encoded));
    }

    if paused {
        params.insert("paused".to_string(), json!(true));
    }

    if let Some(dir) = download_dir {
        params.insert("download-dir".to_string(), json!(dir));
    }

    client.request("torrent-add", Some(Value::Object(params)))
}

pub fn torrent_start(client: &TransmissionClient, ids: Value) -> Result<Value, Error> {
    let params = json!({ "ids": ids });
    client.request("torrent-start", Some(params))
}

pub fn torrent_stop(client: &TransmissionClient, ids: Value) -> Result<Value, Error> {
    let params = json!({ "ids": ids });
    client.request("torrent-stop", Some(params))
}

pub fn torrent_remove(
    client: &TransmissionClient,
    id: i64,
    delete_data: bool,
) -> Result<Value, Error> {
    let params = json!({
        "ids": [id],
        "delete-local-data": delete_data,
    });
    client.request("torrent-remove", Some(params))
}

pub fn torrent_verify(client: &TransmissionClient, id: i64) -> Result<Value, Error> {
    let params = json!({ "ids": [id] });
    client.request("torrent-verify", Some(params))
}

pub fn session_get(client: &TransmissionClient) -> Result<Value, Error> {
    client.request("session-get", None)
}

pub fn session_set(client: &TransmissionClient, params: Value) -> Result<Value, Error> {
    client.request("session-set", Some(params))
}

pub fn session_stats(client: &TransmissionClient) -> Result<SessionStats, Error> {
    let result = client.request("session-stats", None)?;
    serde_json::from_value(result)
        .map_err(|e| Error::Rpc(format!("Failed to parse session stats: {e}")))
}

pub fn free_space(client: &TransmissionClient, path: &str) -> Result<FreeSpace, Error> {
    let params = json!({ "path": path });
    let result = client.request("free-space", Some(params))?;
    serde_json::from_value(result)
        .map_err(|e| Error::Rpc(format!("Failed to parse free space: {e}")))
}

pub fn torrent_set_location(
    client: &TransmissionClient,
    id: i64,
    location: &str,
) -> Result<Value, Error> {
    let params = json!({ "ids": [id], "location": location, "move": true });
    client.request("torrent-set-location", Some(params))
}

pub fn torrent_set_labels(
    client: &TransmissionClient,
    id: i64,
    labels: Vec<String>,
) -> Result<Value, Error> {
    let params = json!({ "ids": [id], "labels": labels });
    client.request("torrent-set", Some(params))
}

pub fn torrent_set_sequential(
    client: &TransmissionClient,
    id: i64,
    enable: bool,
) -> Result<Value, Error> {
    let params = json!({ "ids": [id], "sequential_download": enable });
    client.request("torrent-set", Some(params))
}

pub fn torrent_get_sequential(
    client: &TransmissionClient,
    id: i64,
) -> Result<(String, Option<bool>), Error> {
    let params = json!({ "fields": ["id", "name", "sequential_download"], "ids": [id] });
    let result = client.request("torrent-get", Some(params))?;

    let torrents = result
        .get("torrents")
        .and_then(|v| v.as_array())
        .ok_or_else(|| Error::Rpc("Missing 'torrents' field".to_string()))?;

    let torrent = torrents
        .first()
        .ok_or_else(|| Error::TorrentNotFound(id.to_string()))?;

    let name = torrent
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let sequential = torrent.get("sequential_download").and_then(|v| v.as_bool());

    Ok((name, sequential))
}

pub fn torrent_reannounce(client: &TransmissionClient, id: i64) -> Result<Value, Error> {
    let params = json!({ "ids": [id] });
    client.request("torrent-reannounce", Some(params))
}

pub fn torrent_get_tracker_stats(
    client: &TransmissionClient,
    id: i64,
) -> Result<(String, Vec<TrackerStat>), Error> {
    let params = json!({ "fields": ["id", "name", "trackerStats", "trackerList"], "ids": [id] });
    let result = client.request("torrent-get", Some(params))?;

    let torrents = result
        .get("torrents")
        .and_then(|v| v.as_array())
        .ok_or_else(|| Error::Rpc("Missing 'torrents' field".to_string()))?;

    let torrent = torrents
        .first()
        .ok_or_else(|| Error::TorrentNotFound(id.to_string()))?;

    let name = torrent
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let stats: Vec<TrackerStat> = torrent
        .get("trackerStats")
        .map(|s| serde_json::from_value(s.clone()).unwrap_or_default())
        .unwrap_or_default();

    Ok((name, stats))
}

pub fn torrent_get_tracker_list(client: &TransmissionClient, id: i64) -> Result<String, Error> {
    let params = json!({ "fields": ["id", "trackerList"], "ids": [id] });
    let result = client.request("torrent-get", Some(params))?;

    let torrents = result
        .get("torrents")
        .and_then(|v| v.as_array())
        .ok_or_else(|| Error::Rpc("Missing 'torrents' field".to_string()))?;

    let torrent = torrents
        .first()
        .ok_or_else(|| Error::TorrentNotFound(id.to_string()))?;

    Ok(torrent
        .get("trackerList")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string())
}

pub fn torrent_set_tracker_list(
    client: &TransmissionClient,
    id: i64,
    tracker_list: &str,
) -> Result<Value, Error> {
    let params = json!({ "ids": [id], "trackerList": tracker_list });
    client.request("torrent-set", Some(params))
}

pub fn torrent_set_bandwidth(
    client: &TransmissionClient,
    id: i64,
    download_limit: Option<i64>,
    upload_limit: Option<i64>,
    priority: Option<i64>,
    honors_session: Option<bool>,
    no_limit: bool,
) -> Result<Value, Error> {
    let mut params = serde_json::Map::new();
    params.insert("ids".to_string(), json!([id]));

    if no_limit {
        params.insert("downloadLimited".to_string(), json!(false));
        params.insert("uploadLimited".to_string(), json!(false));
    }

    if let Some(dl) = download_limit {
        params.insert("downloadLimit".to_string(), json!(dl));
        params.insert("downloadLimited".to_string(), json!(true));
    }

    if let Some(ul) = upload_limit {
        params.insert("uploadLimit".to_string(), json!(ul));
        params.insert("uploadLimited".to_string(), json!(true));
    }

    if let Some(p) = priority {
        params.insert("bandwidthPriority".to_string(), json!(p));
    }

    if let Some(honors) = honors_session {
        params.insert("honorsSessionLimits".to_string(), json!(honors));
    }

    client.request("torrent-set", Some(Value::Object(params)))
}

pub fn torrent_get_bandwidth(client: &TransmissionClient, id: i64) -> Result<Value, Error> {
    let params = json!({
        "fields": [
            "id", "name",
            "downloadLimit", "downloadLimited",
            "uploadLimit", "uploadLimited",
            "bandwidthPriority", "honorsSessionLimits"
        ],
        "ids": [id]
    });
    let result = client.request("torrent-get", Some(params))?;

    let torrents = result
        .get("torrents")
        .and_then(|v| v.as_array())
        .ok_or_else(|| Error::Rpc("Missing 'torrents' field".to_string()))?;

    torrents
        .first()
        .cloned()
        .ok_or_else(|| Error::TorrentNotFound(id.to_string()))
}

pub fn torrent_set_seed_limits(
    client: &TransmissionClient,
    id: i64,
    ratio: Option<f64>,
    idle_minutes: Option<i64>,
) -> Result<Value, Error> {
    let mut params = serde_json::Map::new();
    params.insert("ids".to_string(), json!([id]));

    if let Some(r) = ratio {
        params.insert("seedRatioLimit".to_string(), json!(r));
        params.insert("seedRatioMode".to_string(), json!(1)); // 1 = per-torrent
    }

    if let Some(idle) = idle_minutes {
        params.insert("seedIdleLimit".to_string(), json!(idle));
        params.insert("seedIdleMode".to_string(), json!(1)); // 1 = per-torrent
    }

    client.request("torrent-set", Some(Value::Object(params)))
}

pub fn port_test(client: &TransmissionClient) -> Result<bool, Error> {
    let result = client.request("port-test", None)?;
    Ok(result
        .get("port-is-open")
        .and_then(|v| v.as_bool())
        .unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encode() {
        assert_eq!(STANDARD.encode(b"Hello"), "SGVsbG8=");
        assert_eq!(STANDARD.encode(b"Hi"), "SGk=");
        assert_eq!(STANDARD.encode(b""), "");
        assert_eq!(STANDARD.encode(b"abc"), "YWJj");
    }
}
