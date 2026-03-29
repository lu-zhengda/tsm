---
name: tsm
description: >
  Manage Transmission BitTorrent client via the tsm CLI. Use when the user asks
  to add torrents, check download status, manage trackers, set speed limits,
  apply seeding policies, or automate torrent workflows. Requires a running
  Transmission daemon with RPC enabled.
license: MIT
compatibility: macOS, Linux (Transmission daemon required)
user-invocable: true
argument-hint: "<command or natural language request>"
---

# tsm — Transmission CLI

A CLI for managing Transmission BitTorrent client via RPC. All commands output structured data with `--json` for easy parsing.

## Setup

```bash
# Install
brew install lu-zhengda/tap/tsm

# Configure connection (interactive)
tsm login                    # plaintext password in config
tsm login --keychain         # macOS Keychain (recommended)
tsm login --profile seedbox  # named profile for multiple servers

# Verify
tsm health
```

Config file: `~/.config/tsm/config.toml` (or `~/Library/Application Support/tsm/config.toml` on macOS).
Env vars: `TSM_HOST`, `TSM_PORT`, `TSM_USERNAME`, `TSM_PASSWORD`, `TSM_PROFILE`.

## Commands

### Listing and Searching

```bash
tsm list                                          # all torrents
tsm list --json                                   # JSON output
tsm list --ids-only                               # one ID per line (pipe-friendly)
tsm list --filter downloading                     # by status
tsm list --filter "ratio>2.0 AND label:movies"    # advanced filter
tsm list --filter "size>1GB AND age<7d"           # size and age
tsm list --sort speed-down                        # sort by field
tsm search "ubuntu"                               # name search
```

Filter fields: `ratio`, `size` (KB/MB/GB/TB), `progress`, `age` (d/h/m).
Key:value: `label:x`, `name:x`, `status:x`. Combinator: `AND`.

### Adding Torrents

```bash
tsm add path/to/file.torrent
tsm add "magnet:?xt=urn:btih:..."
tsm add https://example.com/file.torrent
tsm add <source> --paused                         # add without starting
tsm add <source> --download-dir /path             # custom download location
```

### Torrent Control

```bash
tsm start <id>              # start (also accepts "all")
tsm stop <id>               # stop (also accepts "all")
tsm verify <id>             # verify data integrity
tsm reannounce <id>         # force tracker reannounce
tsm move <id> /new/path     # relocate data
tsm remove <id>             # remove from client
tsm remove <id> --delete    # remove and delete data (DESTRUCTIVE)
```

### Sequential Download

```bash
tsm sequential <id>         # enable (default)
tsm sequential <id> --on    # enable explicitly
tsm sequential <id> --off   # disable
```

### Torrent Details and Files

```bash
tsm info <id>               # detailed info
tsm info <id> --json        # JSON output
tsm files <id>              # list files with priority and wanted status
tsm files <id> --json       # JSON file list

# Per-file priority
tsm files <id> --priority high --priority-indices 0,1
tsm files <id> --priority low --priority-indices 3
tsm files <id> --priority normal --priority-indices 0,1,2,3

# Skip/unskip files
tsm files <id> --skip 2,3        # don't download these files
tsm files <id> --unskip 2        # resume downloading this file
```

### Labels

```bash
tsm label add <id> <label>
tsm label remove <id> <label>
tsm label list <id>
```

### Tracker Management

```bash
tsm tracker list <id>                              # list with stats
tsm tracker add <id> <url>                         # add tracker
tsm tracker remove <id> <url>                      # remove tracker
tsm tracker replace --from <old-url> --to <new-url>  # bulk replace across ALL torrents
```

### Speed Limits

```bash
# Session-level
tsm speed                                # show current limits
tsm speed --set-down 1000 --set-up 500   # set limits (KB/s)
tsm speed --no-limit                     # remove all limits
tsm speed --alt-on                       # enable alt-speed mode
tsm speed --alt-off                      # disable alt-speed mode

# Per-torrent
tsm speed <id>                                     # show torrent limits
tsm speed <id> --set-down 500 --set-up 100         # set limits
tsm speed <id> --priority high                     # high/normal/low
tsm speed <id> --no-honor-global                   # exempt from session limits
tsm speed <id> --no-limit                          # remove torrent limits
```

### Seeding Policies

Policies are defined in config.toml:

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
tsm policy list                # show configured policies
tsm policy apply --dry-run     # preview what would change
tsm policy apply               # apply to matching torrents
```

### Monitoring

```bash
tsm health                     # connectivity, disk, port check
tsm session                    # server info
tsm stats                      # cumulative statistics
tsm free                       # disk space
tsm top                        # live TUI dashboard (q to quit)
tsm top --interval 5           # custom refresh interval
```

### Watch Directory

```bash
tsm watch ~/Downloads                              # auto-add .torrent files
tsm watch ~/Downloads --paused --delete-after-add   # add paused, remove .torrent
tsm watch ~/Downloads --notify                      # completion notifications
tsm watch ~/Downloads --on-complete "script.sh {name} {id} {download_dir}"
```

Template variables: `{name}`, `{id}`, `{download_dir}`, `{size}`, `{ratio}`.

### Other

```bash
tsm config                     # show resolved configuration
tsm completions bash           # shell completions (bash/zsh/fish)
```

## JSON Mode

All read commands support `--json`. Always use `--json` when you need to parse output programmatically.

```bash
tsm list --json | jq '.[].name'
tsm list --ids-only | xargs -I{} tsm info {} --json
tsm health --json
tsm --json tracker list <id>
tsm --json policy list
tsm files <id> --json
```

## Common Automation Patterns

### Add torrent and prioritize video files
```bash
ID=$(tsm add "magnet:..." --json | jq -r '."torrent-added".id')
tsm files $ID --json | jq -r '.[] | select(.name | test("\\.(mkv|mp4|avi)$")) | .index' | \
  tr '\n' ',' | sed 's/,$//' | xargs -I{} tsm files $ID --priority high --priority-indices {}
```

### Clean up well-seeded torrents
```bash
tsm list --json | jq -r '.[] | select(.uploadRatio > 3.0) | .id' | \
  while read id; do echo "Removing ID $id"; tsm remove $id; done
```

### Apply policies and report
```bash
tsm policy apply --dry-run --json | jq '.[] | "\(.name): \(.policy)"'
```

### Replace dead tracker everywhere
```bash
tsm tracker replace --from "udp://dead-tracker.com:1337/announce" --to "udp://new-tracker.com:1337/announce"
```

### Monitor active downloads
```bash
tsm list --filter downloading --json | jq '.[] | {name, progress: (.percentDone * 100 | floor), speed: .rateDownload}'
```

## Safety Rules

- **NEVER** run `tsm remove <id> --delete` without explicit user confirmation — this permanently deletes downloaded data.
- **NEVER** run `tsm remove` in a loop without showing the user what will be removed first.
- Prefer `--dry-run` before `tsm policy apply`.
- Use read-only commands (`list`, `info`, `files`, `health`, `stats`) freely.
- Mutations (`start`, `stop`, `add`, `remove`, `speed`, `label`, `tracker`, `policy apply`) should be confirmed when acting on multiple torrents.
- When in doubt, show the user what you plan to do before executing.

## Profiles

```bash
tsm --profile seedbox list     # use named profile
TSM_PROFILE=seedbox tsm list   # via env var
```

## Exit Codes

- `0` — success
- `1` — connection/RPC/IO error
- `2` — config/filter/keychain error
- `3` — torrent not found
