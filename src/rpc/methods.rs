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
) -> Result<(String, Vec<TorrentFile>), Error> {
    let params = json!({ "fields": ["id", "name", "files"], "ids": [id] });
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

    Ok((name, files))
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
