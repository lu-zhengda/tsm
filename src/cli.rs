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
        /// Filter by status
        #[arg(long)]
        filter: Option<FilterStatus>,

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

    /// List files in a torrent
    Files {
        /// Torrent ID
        id: i64,
    },

    /// Show or set speed limits
    Speed {
        /// Set download speed limit (KB/s)
        #[arg(long)]
        set_down: Option<i64>,

        /// Set upload speed limit (KB/s)
        #[arg(long)]
        set_up: Option<i64>,

        /// Enable alt-speed mode
        #[arg(long, default_value_t = false)]
        alt_on: bool,

        /// Disable alt-speed mode
        #[arg(long, default_value_t = false)]
        alt_off: bool,

        /// Remove all speed limits
        #[arg(long, default_value_t = false)]
        no_limit: bool,
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
