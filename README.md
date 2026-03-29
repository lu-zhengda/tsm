# tsm

A CLI for [Transmission](https://transmissionbt.com/) BitTorrent client.

Requires a running Transmission instance with RPC enabled — either `transmission-daemon` or the desktop app (Transmission-Qt/GTK) with remote access turned on. This tool connects to the [Transmission RPC API](https://github.com/transmission/transmission/blob/main/docs/rpc-spec.md) over HTTP/HTTPS.

## tsm vs. transmission-remote

Both tools talk to the same [Transmission RPC API](https://github.com/transmission/transmission/blob/main/docs/rpc-spec.md). They complement each other.

| Capability | [`transmission-remote`](https://formulae.brew.sh/formula/transmission-cli) | `tsm` |
|---|---|---|
| List / start / stop / remove torrents | Yes | Yes |
| Add torrents (file, magnet, URL) | Yes | Yes |
| Torrent info and file listing | Yes | Yes, with colored tables |
| Per-file priority and skip/unskip | Yes | Yes |
| Per-torrent speed limits and priority | Yes | Yes |
| Sequential download | Yes | Yes (`tsm sequential`) |
| Tracker add / remove | Yes | Yes, plus bulk replace across all torrents |
| Reannounce | Yes | Yes |
| Labels | Yes | Yes |
| Filter / search torrents | Basic (`-F`) | Advanced query language (`ratio>2.0 AND label:movies AND age<7d`) |
| Live TUI dashboard | No | Yes (`tsm top`) |
| Directory watch + auto-add | No | Yes (`tsm watch`) |
| Completion hooks with templates | Daemon-level script only | Template variables (`{name}`, `{download_dir}`, etc.) + inline `--on-complete` |
| Config-driven seeding policies | No | Yes (`tsm policy apply` by label) |
| Multi-profile config / login wizard | No (manual auth flags) | Yes (`tsm login`, `--profile`) |
| Health check (connectivity + disk + port) | No | Yes (`tsm health`) |
| JSON output on all commands | Raw RPC JSON | Structured JSON (`--json`) |
| Shell completions | No | Yes (bash, zsh, fish) |
| Daemon settings (encryption, DHT, PEX, LPD, cache) | Yes | No — use `transmission-remote` |
| Bandwidth groups | Yes | No |
| Alt-speed scheduling (time-based) | Yes | No (toggle only) |
| Port mapping / peer limits / blocklist | Yes | No |
| Rename torrent / find data | Yes | No |

**In short:** `transmission-remote` is the complete low-level interface for every daemon setting. `tsm` covers day-to-day torrent management with better output, automation, and scripting support. Pick whichever fits your workflow — they talk to the same server and can be used interchangeably.

## Install

### Homebrew (macOS/Linux)

```bash
brew install lu-zhengda/tap/tsm
```

### Prebuilt binaries

Download from [GitHub Releases](https://github.com/lu-zhengda/tsm/releases).

### Cargo

```bash
cargo install transmission-cli
```

### From source

```bash
git clone https://github.com/lu-zhengda/tsm.git
cd tsm
cargo install --path .
```

### Verify

```bash
tsm --version
```

## Quick Start

```bash
# Login (saves credentials with 600 permissions)
tsm login

# List all torrents (colored status + progress bars)
tsm list
tsm list --filter downloading --sort speed-down
tsm list --filter "ratio>2.0 AND label:movies"
tsm list --filter "size>1GB AND age<7d"
tsm list --json
tsm list --ids-only              # pipe-friendly: one ID per line

# Search by name
tsm search "ubuntu"

# Add a torrent
tsm add path/to/file.torrent
tsm add "magnet:?xt=urn:btih:..." --paused
tsm add https://example.com/file.torrent --download-dir /data

# Manage torrents
tsm start 42
tsm stop 42
tsm remove 42
tsm remove 42 --delete           # also delete data
tsm verify 42
tsm reannounce 42                # force tracker reannounce
tsm move 42 /new/path            # relocate torrent data
tsm sequential 42                # enable sequential download
tsm sequential 42 --off          # disable sequential download

# Labels
tsm label add 42 linux
tsm label list 42
tsm label remove 42 linux

# Torrent details and file management
tsm info 42
tsm files 42                                             # list with priority/wanted columns
tsm files 42 --priority high --priority-indices 0,1      # set file priority
tsm files 42 --skip 2,3                                  # skip files
tsm files 42 --unskip 2                                  # unskip files
```

## Tracker Management

```bash
# List trackers with seeder/leecher counts
tsm tracker list 42

# Add/remove trackers
tsm tracker add 42 "udp://tracker.opentrackr.org:1337/announce"
tsm tracker remove 42 "udp://tracker.opentrackr.org:1337/announce"

# Bulk replace a tracker URL across all torrents
tsm tracker replace --from "udp://old-tracker.com/announce" --to "udp://new-tracker.com/announce"
```

## Speed Control

```bash
# Session-level limits
tsm speed
tsm speed --set-down 1000 --set-up 500
tsm speed --no-limit
tsm speed --alt-on

# Per-torrent limits and priority
tsm speed 42 --set-down 500 --set-up 100
tsm speed 42 --priority high       # high, normal, or low
tsm speed 42 --no-honor-global     # exempt from session limits
tsm speed 42 --no-limit            # remove per-torrent limits
```

## Seeding Policies

Define per-label seeding rules in your config file:

```toml
[[policies]]
name = "hd-movies"
match_label = "4k"
seed_ratio = 2.5
seed_idle_minutes = 4320

[[policies]]
name = "tv-shows"
match_label = "tv"
seed_ratio = 1.5
seed_idle_minutes = 1440
```

```bash
tsm policy list                   # show configured policies
tsm policy apply --dry-run        # preview what would change
tsm policy apply                  # apply matching policies
```

## Watch & Notifications

```bash
# Watch directory for .torrent files
tsm watch ~/Downloads
tsm watch ~/Downloads --paused --delete-after-add

# Completion notifications
tsm watch ~/Downloads --notify

# Completion hooks with template variables
tsm watch ~/Downloads --on-complete "/path/to/script.sh {name} {id} {download_dir}"
```

Template variables: `{name}`, `{id}`, `{download_dir}`, `{size}`, `{ratio}`

Config-based hooks:

```toml
[notifications]
on_complete = "/path/to/script.sh {name} {id}"
webhook = "https://hooks.example.com"
```

## Monitoring

```bash
# Live dashboard
tsm top                           # q to quit, j/k to scroll
tsm top --interval 5

# Health check
tsm health
tsm health --json

# Session info
tsm session
tsm stats
tsm free
```

## Configuration

### Login

```bash
tsm login                       # configure default profile
tsm login --profile seedbox     # configure a named profile
```

Interactively prompts for host, port, and credentials, tests the connection, then saves to `~/.config/tsm/config.toml` with `600` permissions.

### Config Resolution

Highest priority first: CLI flags > env vars > config file.

**CLI flags:**

```bash
tsm --host myserver --port 9091 --username admin --password secret list
```

**Env vars:** `TSM_HOST`, `TSM_PORT`, `TSM_USERNAME`, `TSM_PASSWORD`, `TSM_PROFILE`

**Config file** (`~/.config/tsm/config.toml`):

```toml
[default]
host = "localhost"
port = 9091
username = "admin"
password = "secret"

[profiles.seedbox]
host = "seedbox.example.com"
port = 443
username = "user"
password = "pass"
```

```bash
tsm --profile seedbox list
```

## JSON Output

All read commands support `--json` for scripting:

```bash
tsm list --json | jq '.[].name'
tsm list --ids-only | xargs -I{} tsm info {} --json
tsm health --json
tsm --json policy list
tsm --json tracker list 42
```

## Options

```
--no-color    Disable colored output (also respects NO_COLOR env var)
--json        Output as JSON
--host        Transmission host
--port        Transmission port (443 auto-selects HTTPS)
--profile     Config profile name
```

## Shell Completions

```bash
tsm completions bash >> ~/.bashrc
tsm completions zsh >> ~/.zshrc
tsm completions fish > ~/.config/fish/completions/tsm.fish
```

## License

MIT
