use std::io::{self, Write};
use std::time::Duration;

use crossterm::{
    ExecutableCommand, cursor, event,
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    terminal,
};

use crate::client::TransmissionClient;
use crate::error::Error;
use crate::output::table::{format_eta, format_size, format_speed, status_string};
use crate::rpc::methods;

pub fn run_top(client: &TransmissionClient, interval_secs: u64) -> Result<(), Error> {
    let mut stdout = io::stdout();

    // Install panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = terminal::disable_raw_mode();
        let _ = io::stdout().execute(terminal::LeaveAlternateScreen);
        let _ = io::stdout().execute(cursor::Show);
        original_hook(info);
    }));

    terminal::enable_raw_mode()
        .map_err(|e| Error::Config(format!("Failed to enable raw mode: {e}")))?;
    stdout
        .execute(terminal::EnterAlternateScreen)
        .map_err(|e| Error::Config(format!("Failed to enter alternate screen: {e}")))?;
    stdout
        .execute(cursor::Hide)
        .map_err(|e| Error::Config(format!("Failed to hide cursor: {e}")))?;

    let result = top_loop(client, interval_secs, &mut stdout);

    // Always restore terminal
    let _ = stdout.execute(cursor::Show);
    let _ = stdout.execute(terminal::LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();

    result
}

fn top_loop(
    client: &TransmissionClient,
    interval_secs: u64,
    stdout: &mut io::Stdout,
) -> Result<(), Error> {
    let mut scroll_offset: usize = 0;

    loop {
        // Fetch data
        let stats = methods::session_stats(client)?;
        let mut torrents = methods::torrent_get_list(client)?;

        // Sort: downloading first, then seeding, then rest
        torrents.sort_by(|a, b| {
            let priority = |status: i64| -> u8 {
                match status {
                    4 => 0, // Downloading
                    6 => 1, // Seeding
                    _ => 2,
                }
            };
            priority(a.status)
                .cmp(&priority(b.status))
                .then(b.rate_download.cmp(&a.rate_download))
        });

        let (cols, rows) = terminal::size().unwrap_or((80, 24));
        let max_rows = rows as usize - 4; // header + separator + footer + status line

        // Clamp scroll offset
        if scroll_offset >= torrents.len() {
            scroll_offset = torrents.len().saturating_sub(1);
        }

        // Render
        stdout
            .execute(cursor::MoveTo(0, 0))
            .map_err(|e| Error::Io(io::Error::other(e)))?;
        stdout
            .execute(terminal::Clear(terminal::ClearType::All))
            .map_err(|e| Error::Io(io::Error::other(e)))?;

        // Header
        write!(
            stdout,
            "{}tsm top{} | Total: {} | Active: {} | Paused: {} | Down: {} | Up: {}\r\n",
            SetAttribute(Attribute::Bold),
            SetAttribute(Attribute::Reset),
            stats.torrent_count,
            stats.active_torrent_count,
            stats.paused_torrent_count,
            format_speed(stats.download_speed),
            format_speed(stats.upload_speed),
        )
        .ok();

        // Column header
        let name_width = (cols as usize).saturating_sub(65).max(10);
        write!(
            stdout,
            "{:>5} {:>12} {:>name_width$} {:>8} {:>13} {:>9} {:>9} {:>6}\r\n",
            "ID", "Status", "Name", "Size", "Progress", "Down", "Up", "ETA",
        )
        .ok();

        // Torrent rows
        let visible = &torrents[scroll_offset..];
        for (i, t) in visible.iter().enumerate() {
            if i >= max_rows {
                break;
            }

            let status = status_string(t.status);
            let name = truncate(&t.name, name_width);
            let progress = format_bar(t.percent_done, 8);

            // Color the status
            let color = match t.status {
                0 => Color::Yellow,
                1 | 2 => Color::Cyan,
                3 | 4 => Color::Blue,
                5 | 6 => Color::Green,
                _ => Color::White,
            };

            write!(
                stdout,
                "{:>5} {}{:>12}{} {:>name_width$} {:>8} {} {:>9} {:>9} {:>6}\r\n",
                t.id,
                SetForegroundColor(color),
                status,
                ResetColor,
                name,
                format_size(t.total_size),
                progress,
                format_speed(t.rate_download),
                format_speed(t.rate_upload),
                format_eta(t.eta),
            )
            .ok();
        }

        // Footer
        let showing = visible.len().min(max_rows);
        write!(
            stdout,
            "\r\n{}q{} quit | {}\u{2191}\u{2193}{} scroll | Showing {}-{} of {} | Refresh: {}s",
            SetAttribute(Attribute::Bold),
            SetAttribute(Attribute::Reset),
            SetAttribute(Attribute::Bold),
            SetAttribute(Attribute::Reset),
            scroll_offset + 1,
            scroll_offset + showing,
            torrents.len(),
            interval_secs,
        )
        .ok();

        stdout.flush().ok();

        // Wait for input or timeout
        if event::poll(Duration::from_secs(interval_secs)).unwrap_or(false)
            && let Ok(event::Event::Key(key)) = event::read()
        {
            match key.code {
                event::KeyCode::Char('q') | event::KeyCode::Esc => break,
                event::KeyCode::Down | event::KeyCode::Char('j') => {
                    if scroll_offset + max_rows < torrents.len() {
                        scroll_offset += 1;
                    }
                }
                event::KeyCode::Up | event::KeyCode::Char('k') => {
                    scroll_offset = scroll_offset.saturating_sub(1);
                }
                event::KeyCode::PageDown => {
                    scroll_offset =
                        (scroll_offset + max_rows).min(torrents.len().saturating_sub(max_rows));
                }
                event::KeyCode::PageUp => {
                    scroll_offset = scroll_offset.saturating_sub(max_rows);
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max.saturating_sub(3)).collect();
        format!("{t}...")
    }
}

fn format_bar(percent: f64, width: usize) -> String {
    let filled = (percent * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!(
        "[{}{}] {:>3.0}%",
        "=".repeat(filled),
        " ".repeat(empty),
        percent * 100.0
    )
}
