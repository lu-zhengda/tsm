use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "tsm",
    about = "A CLI for Transmission BitTorrent client",
    version
)]
pub struct Cli {
    /// Transmission host
    #[arg(long, global = true)]
    pub host: Option<String>,

    /// Transmission port
    #[arg(long, global = true)]
    pub port: Option<u16>,

    /// Auth username
    #[arg(long, global = true)]
    pub username: Option<String>,

    /// Auth password (visible in process list — prefer `tsm login` or TSM_PASSWORD env var)
    #[arg(long, global = true)]
    pub password: Option<String>,

    /// Output as JSON
    #[arg(long, global = true, default_value_t = false)]
    pub json: bool,

    /// Config file path
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// Config profile name
    #[arg(long, global = true)]
    pub profile: Option<String>,

    /// Disable colored output
    #[arg(long = "no-color", global = true)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// List all torrents
    List {
        /// Filter expression (e.g., "downloading", "ratio>2.0 AND label:movies")
        #[arg(long)]
        filter: Option<String>,

        /// Sort by field
        #[arg(long)]
        sort: Option<SortField>,

        /// Output only torrent IDs (one per line)
        #[arg(long)]
        ids_only: bool,
    },

    /// Search torrents by name
    Search {
        /// Search query (case-insensitive substring match)
        query: String,

        /// Sort by field
        #[arg(long)]
        sort: Option<SortField>,
    },

    /// Move torrent data to a new location
    #[command(name = "move")]
    Move {
        /// Torrent ID
        id: i64,

        /// New location path
        path: String,
    },

    /// Manage torrent labels
    Label {
        #[command(subcommand)]
        action: LabelAction,
    },

    /// Add a torrent (file path, magnet link, or URL)
    Add {
        /// Torrent file path, magnet link, or URL
        source: String,

        /// Add in paused state
        #[arg(long, default_value_t = false)]
        paused: bool,

        /// Download directory
        #[arg(long)]
        download_dir: Option<String>,
    },

    /// Start torrent(s)
    Start {
        /// Torrent ID or "all"
        target: String,
    },

    /// Stop torrent(s)
    Stop {
        /// Torrent ID or "all"
        target: String,
    },

    /// Remove a torrent
    Remove {
        /// Torrent ID
        id: i64,

        /// Also delete local data
        #[arg(long, default_value_t = false)]
        delete: bool,
    },

    /// Verify torrent data
    Verify {
        /// Torrent ID
        id: i64,
    },

    /// Show detailed torrent info
    Info {
        /// Torrent ID
        id: i64,
    },

    /// List files in a torrent, or set file priority/skip
    Files {
        /// Torrent ID
        id: i64,

        /// Set file priority (requires --priority-indices)
        #[arg(long, requires = "priority_indices")]
        priority: Option<FilePriority>,

        /// File indices for priority change (comma-separated, 0-based)
        #[arg(long = "priority-indices", value_delimiter = ',')]
        priority_indices: Option<Vec<usize>>,

        /// Skip files (comma-separated indices, 0-based)
        #[arg(long, value_delimiter = ',')]
        skip: Option<Vec<usize>>,

        /// Unskip files (comma-separated indices, 0-based)
        #[arg(long, value_delimiter = ',')]
        unskip: Option<Vec<usize>>,
    },

    /// Show or set speed limits (session-level or per-torrent)
    Speed {
        /// Torrent ID (omit for session-level)
        id: Option<i64>,

        /// Set download speed limit (KB/s)
        #[arg(long)]
        set_down: Option<i64>,

        /// Set upload speed limit (KB/s)
        #[arg(long)]
        set_up: Option<i64>,

        /// Enable alt-speed mode (session only)
        #[arg(long, default_value_t = false)]
        alt_on: bool,

        /// Disable alt-speed mode (session only)
        #[arg(long, default_value_t = false)]
        alt_off: bool,

        /// Remove all speed limits
        #[arg(long, default_value_t = false)]
        no_limit: bool,

        /// Bandwidth priority: high, normal, or low (per-torrent only)
        #[arg(long)]
        priority: Option<BandwidthPriority>,

        /// Exempt torrent from session speed limits (per-torrent only)
        #[arg(long, default_value_t = false)]
        no_honor_global: bool,
    },

    /// Show session info
    Session,

    /// Show cumulative stats
    Stats,

    /// Check free disk space
    Free {
        /// Path to check (defaults to download dir)
        path: Option<String>,
    },

    /// Watch a directory and auto-add .torrent files
    Watch {
        /// Directory to watch
        dir: String,

        /// Add torrents in paused state
        #[arg(long)]
        paused: bool,

        /// Download directory for added torrents
        #[arg(long)]
        download_dir: Option<String>,

        /// Delete .torrent files after adding
        #[arg(long)]
        delete_after_add: bool,

        /// Enable completion notifications
        #[arg(long)]
        notify: bool,

        /// Run command on torrent completion (template vars: {name} {id} {download_dir} {size} {ratio})
        #[arg(long)]
        on_complete: Option<String>,
    },

    /// Live dashboard with auto-refresh
    Top {
        /// Refresh interval in seconds
        #[arg(long, default_value_t = 2)]
        interval: u64,
    },

    /// Show resolved configuration
    #[command(name = "config")]
    ConfigShow,

    /// Check connectivity, disk space, and port status
    Health,

    /// Toggle sequential download mode (Transmission 4.0+)
    Sequential {
        /// Torrent ID
        id: i64,

        /// Enable sequential download
        #[arg(long, conflicts_with = "off")]
        on: bool,

        /// Disable sequential download
        #[arg(long, conflicts_with = "on")]
        off: bool,
    },

    /// Force tracker reannounce
    Reannounce {
        /// Torrent ID
        id: i64,
    },

    /// Manage torrent trackers
    Tracker {
        #[command(subcommand)]
        action: TrackerAction,
    },

    /// Manage seeding policies
    Policy {
        #[command(subcommand)]
        action: PolicyAction,
    },

    /// Save connection credentials to config file
    Login {
        /// Profile name to save as
        #[arg(long, default_value = "default")]
        profile: String,
    },

    /// Generate shell completions
    Completions {
        /// Shell type
        shell: clap_complete::Shell,
    },
}

#[derive(Subcommand, Debug)]
pub enum LabelAction {
    /// Add a label to a torrent
    Add {
        /// Torrent ID
        id: i64,
        /// Label to add
        label: String,
    },
    /// Remove a label from a torrent
    Remove {
        /// Torrent ID
        id: i64,
        /// Label to remove
        label: String,
    },
    /// List labels on a torrent
    List {
        /// Torrent ID
        id: i64,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum FilterStatus {
    Downloading,
    Seeding,
    Paused,
    Stopped,
    Checking,
    Queued,
}

#[derive(Subcommand, Debug)]
pub enum TrackerAction {
    /// List trackers for a torrent
    List {
        /// Torrent ID
        id: i64,
    },
    /// Add a tracker to a torrent
    Add {
        /// Torrent ID
        id: i64,
        /// Tracker announce URL
        url: String,
    },
    /// Remove a tracker from a torrent
    Remove {
        /// Torrent ID
        id: i64,
        /// Tracker announce URL
        url: String,
    },
    /// Replace a tracker URL across all torrents
    Replace {
        /// Old tracker URL to replace
        #[arg(long)]
        from: String,
        /// New tracker URL
        #[arg(long)]
        to: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum PolicyAction {
    /// List configured seeding policies
    List,
    /// Apply matching policies to torrents
    Apply {
        /// Show what would change without applying
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum FilePriority {
    High,
    Normal,
    Low,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum BandwidthPriority {
    Low,
    Normal,
    High,
}

impl BandwidthPriority {
    pub fn to_rpc_value(&self) -> i64 {
        match self {
            BandwidthPriority::Low => -1,
            BandwidthPriority::Normal => 0,
            BandwidthPriority::High => 1,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum SortField {
    Name,
    Size,
    Progress,
    Ratio,
    SpeedDown,
    SpeedUp,
    Added,
}
