# tsm

A CLI for [Transmission](https://transmissionbt.com/) BitTorrent client.

Requires a running Transmission instance with RPC enabled — either `transmission-daemon` or the desktop app (Transmission-Qt/GTK) with remote access turned on. This tool connects to the [Transmission RPC API](https://github.com/transmission/transmission/blob/main/docs/rpc-spec.md) over HTTP/HTTPS.

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
tsm move 42 /new/path            # relocate torrent data

# Labels
tsm label add 42 linux
tsm label list 42
tsm label remove 42 linux

# Torrent details
tsm info 42
tsm files 42

# Speed control
tsm speed
tsm speed --set-down 1000 --set-up 500
tsm speed --no-limit
tsm speed --alt-on

# Session info
tsm session
tsm stats
tsm free

# Live dashboard
tsm top                           # q to quit, j/k to scroll
tsm top --interval 5

# Watch directory for .torrent files
tsm watch ~/Downloads
tsm watch ~/Downloads --paused --delete-after-add
tsm watch ~/Downloads --notify    # fire notifications on completion

# Health check
tsm health
tsm health --json

# Show resolved config
tsm config

# Shell completions
tsm completions bash >> ~/.bashrc
tsm completions zsh >> ~/.zshrc
tsm completions fish > ~/.config/fish/completions/tsm.fish
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

[notifications]
on_complete = "/path/to/script.sh"   # called with torrent name and ID as args
webhook = "https://hooks.example.com" # POSTs JSON on completion
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
```

## Options

```
--no-color    Disable colored output (also respects NO_COLOR env var)
--json        Output as JSON
--host        Transmission host
--port        Transmission port (443 auto-selects HTTPS)
--profile     Config profile name
```

## License

MIT
