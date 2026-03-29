use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct RpcRequest {
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
}

impl RpcRequest {
    pub fn new(method: impl Into<String>, arguments: Option<Value>) -> Self {
        Self {
            method: method.into(),
            arguments,
        }
    }
}

// Transmission RPC response format
#[derive(Debug, Deserialize)]
pub struct LegacyRpcResponse {
    pub result: Option<String>,
    pub arguments: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Torrent {
    pub id: i64,
    pub name: String,
    pub status: i64,
    #[serde(rename = "totalSize")]
    pub total_size: i64,
    #[serde(rename = "percentDone")]
    pub percent_done: f64,
    #[serde(rename = "rateDownload")]
    pub rate_download: i64,
    #[serde(rename = "rateUpload")]
    pub rate_upload: i64,
    pub eta: i64,
    #[serde(rename = "uploadRatio")]
    pub upload_ratio: f64,
    #[serde(rename = "addedDate")]
    pub added_date: i64,
    #[serde(default)]
    pub labels: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TorrentDetail {
    pub id: i64,
    pub name: String,
    pub status: i64,
    #[serde(rename = "totalSize")]
    pub total_size: i64,
    #[serde(rename = "percentDone")]
    pub percent_done: f64,
    #[serde(rename = "rateDownload")]
    pub rate_download: i64,
    #[serde(rename = "rateUpload")]
    pub rate_upload: i64,
    pub eta: i64,
    #[serde(rename = "uploadRatio")]
    pub upload_ratio: f64,
    #[serde(rename = "addedDate")]
    pub added_date: i64,
    #[serde(rename = "hashString")]
    pub hash_string: String,
    pub comment: Option<String>,
    #[serde(rename = "downloadDir")]
    pub download_dir: String,
    #[serde(rename = "pieceCount")]
    pub piece_count: i64,
    #[serde(rename = "pieceSize")]
    pub piece_size: i64,
    #[serde(rename = "downloadedEver")]
    pub downloaded_ever: i64,
    #[serde(rename = "uploadedEver")]
    pub uploaded_ever: i64,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub trackers: Vec<Tracker>,
    #[serde(default)]
    pub peers: Vec<Peer>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Tracker {
    pub announce: String,
    pub id: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Peer {
    pub address: String,
    #[serde(rename = "clientName")]
    pub client_name: String,
    #[serde(rename = "rateToClient")]
    pub rate_to_client: i64,
    #[serde(rename = "rateToPeer")]
    pub rate_to_peer: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TrackerStat {
    pub announce: String,
    pub host: String,
    pub id: i64,
    #[serde(rename = "lastAnnounceResult")]
    pub last_announce_result: String,
    #[serde(rename = "lastAnnounceSucceeded")]
    pub last_announce_succeeded: bool,
    #[serde(rename = "seederCount")]
    pub seeder_count: i64,
    #[serde(rename = "leecherCount")]
    pub leecher_count: i64,
    pub tier: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TorrentFile {
    pub name: String,
    pub length: i64,
    #[serde(rename = "bytesCompleted")]
    pub bytes_completed: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SessionStats {
    #[serde(rename = "activeTorrentCount")]
    pub active_torrent_count: i64,
    #[serde(rename = "downloadSpeed")]
    pub download_speed: i64,
    #[serde(rename = "pausedTorrentCount")]
    pub paused_torrent_count: i64,
    #[serde(rename = "torrentCount")]
    pub torrent_count: i64,
    #[serde(rename = "uploadSpeed")]
    pub upload_speed: i64,
    #[serde(rename = "cumulative-stats")]
    pub cumulative_stats: Option<CumulativeStats>,
    #[serde(rename = "current-stats")]
    pub current_stats: Option<CumulativeStats>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CumulativeStats {
    #[serde(rename = "downloadedBytes")]
    pub downloaded_bytes: i64,
    #[serde(rename = "uploadedBytes")]
    pub uploaded_bytes: i64,
    #[serde(rename = "filesAdded")]
    pub files_added: i64,
    #[serde(rename = "sessionCount")]
    pub session_count: i64,
    #[serde(rename = "secondsActive")]
    pub seconds_active: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FreeSpace {
    pub path: String,
    #[serde(rename = "size-bytes")]
    pub size_bytes: i64,
    #[serde(rename = "total-size")]
    pub total_size: Option<i64>,
}
