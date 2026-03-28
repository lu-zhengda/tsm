use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use notify::{EventKind, RecursiveMode, Watcher};

use crate::client::TransmissionClient;
use crate::config::Config;
use crate::error::Error;
use crate::rpc::methods;

pub fn execute(
    client: &TransmissionClient,
    dir: &str,
    paused: bool,
    download_dir: Option<&str>,
    delete_after_add: bool,
    notify_config: Option<&Config>,
) -> Result<(), Error> {
    let dir_path = Path::new(dir);
    if !dir_path.is_dir() {
        return Err(Error::Config(format!("Not a directory: {dir}")));
    }

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .map_err(|e| Error::Config(format!("Failed to set signal handler: {e}")))?;

    println!("Watching {dir} for .torrent files (Ctrl+C to stop)");

    // Process existing .torrent files
    process_existing(client, dir_path, paused, download_dir, delete_after_add);

    // Set up filesystem watcher
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    })
    .map_err(|e| Error::Config(format!("Failed to create watcher: {e}")))?;

    watcher
        .watch(dir_path, RecursiveMode::NonRecursive)
        .map_err(|e| Error::Config(format!("Failed to watch directory: {e}")))?;

    // Track completed torrents for notifications
    let mut completed_ids: HashSet<i64> = HashSet::new();
    if notify_config.is_some() {
        // Seed with already-completed torrents so we don't fire for them
        if let Ok(torrents) = methods::torrent_get_list(client) {
            for t in &torrents {
                if t.percent_done >= 1.0 {
                    completed_ids.insert(t.id);
                }
            }
        }
    }
    let mut poll_counter: u32 = 0;

    while running.load(Ordering::SeqCst) {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => {
                if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                    for path in &event.paths {
                        if is_torrent_file(path) {
                            add_torrent(client, path, paused, download_dir, delete_after_add);
                        }
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        // Check for newly completed torrents every ~30 seconds
        if let Some(config) = notify_config {
            poll_counter += 1;
            if poll_counter >= 30 {
                poll_counter = 0;
                check_completions(client, config, &mut completed_ids);
            }
        }
    }

    println!("\nStopped watching.");
    Ok(())
}

fn check_completions(
    client: &TransmissionClient,
    config: &Config,
    completed_ids: &mut HashSet<i64>,
) {
    let torrents = match methods::torrent_get_list(client) {
        Ok(t) => t,
        Err(_) => return,
    };

    for t in &torrents {
        if t.percent_done >= 1.0 && !completed_ids.contains(&t.id) {
            completed_ids.insert(t.id);
            println!("Completed: {} (ID: {})", t.name, t.id);
            crate::notify_hook::fire_completion(&t.name, t.id, config);
        }
    }
}

fn process_existing(
    client: &TransmissionClient,
    dir: &Path,
    paused: bool,
    download_dir: Option<&str>,
    delete_after_add: bool,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Warning: cannot read directory: {e}");
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if is_torrent_file(&path) {
            add_torrent(client, &path, paused, download_dir, delete_after_add);
        }
    }
}

fn is_torrent_file(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("torrent"))
}

fn add_torrent(
    client: &TransmissionClient,
    path: &Path,
    paused: bool,
    download_dir: Option<&str>,
    delete_after_add: bool,
) {
    let path_str = match path.to_str() {
        Some(s) => s,
        None => {
            eprintln!("Warning: invalid path: {}", path.display());
            return;
        }
    };

    match methods::torrent_add(client, path_str, paused, download_dir) {
        Ok(result) => {
            let name = result
                .get("torrent-added")
                .or_else(|| result.get("torrent-duplicate"))
                .and_then(|t| t.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("unknown");

            if result.get("torrent-duplicate").is_some() {
                println!("Duplicate: {name}");
            } else {
                let id = result
                    .get("torrent-added")
                    .and_then(|t| t.get("id"))
                    .and_then(|i| i.as_i64())
                    .unwrap_or(0);
                println!("Added: {name} (ID: {id})");

                if delete_after_add && let Err(e) = std::fs::remove_file(path) {
                    eprintln!("Warning: could not delete {}: {e}", path.display());
                }
            }
        }
        Err(e) => {
            eprintln!("Error adding {}: {e}", path.display());
        }
    }
}
