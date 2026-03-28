# tsm

A CLI for [Transmission](https://transmissionbt.com/) BitTorrent client.

## Install

### From GitHub

```bash
cargo install --git https://github.com/lu-zhengda/tsm --tag v0.1.0
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
# Login (saves credentials to ~/.config/tsm/config.toml with 600 permissions)
tsm login

# List all torrents
tsm list
tsm list --filter downloading --sort speed-down
tsm list --json

# Add a torrent
tsm add path/to/file.torrent
tsm add "magnet:?xt=urn:btih:..." --paused
tsm add https://example.com/file.torrent --download-dir /data

# Manage torrents
tsm start 42
tsm stop 42
tsm remove 42
tsm remove 42 --delete   # also delete data
tsm verify 42

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

# Shell completions
tsm completions bash >> ~/.bashrc
tsm completions zsh >> ~/.zshrc
tsm completions fish > ~/.config/fish/completions/tsm.fish
```

## Configuration

### Login

The easiest way to configure `tsm`:

```bash
tsm login                       # configure default profile
tsm login --profile seedbox     # configure a named profile
```

This interactively prompts for host, port, and credentials, tests the connection, then saves to `~/.config/tsm/config.toml` with `600` permissions.

### Config Resolution

Config is resolved in order (highest priority first): CLI flags > env vars > config file.

**CLI flags** override everything:

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
tsm stats --json | jq '.torrentCount'
```

## License

MIT
