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

    /// Auth password
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
