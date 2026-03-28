use comfy_table::Color;

pub fn status_color(status: i64) -> Option<Color> {
    match status {
        0 => Some(Color::Yellow),   // Stopped
        1 | 2 => Some(Color::Cyan), // Check Wait / Checking
        3 => Some(Color::Blue),     // DL Wait (queued)
        4 => Some(Color::Blue),     // Downloading
        5 => Some(Color::Green),    // Seed Wait (queued)
        6 => Some(Color::Green),    // Seeding
        _ => None,
    }
}

pub fn format_progress_bar(percent: f64, width: usize) -> String {
    let filled = (percent * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!(
        "[{}{}] {:.0}%",
        "=".repeat(filled),
        " ".repeat(empty),
        percent * 100.0
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_color() {
        assert_eq!(status_color(0), Some(Color::Yellow));
        assert_eq!(status_color(1), Some(Color::Cyan));
        assert_eq!(status_color(2), Some(Color::Cyan));
        assert_eq!(status_color(3), Some(Color::Blue));
        assert_eq!(status_color(4), Some(Color::Blue));
        assert_eq!(status_color(5), Some(Color::Green));
        assert_eq!(status_color(6), Some(Color::Green));
        assert_eq!(status_color(99), None);
    }

    #[test]
    fn test_progress_bar_empty() {
        assert_eq!(format_progress_bar(0.0, 10), "[          ] 0%");
    }

    #[test]
    fn test_progress_bar_full() {
        assert_eq!(format_progress_bar(1.0, 10), "[==========] 100%");
    }

    #[test]
    fn test_progress_bar_partial() {
        assert_eq!(format_progress_bar(0.5, 10), "[=====     ] 50%");
    }

    #[test]
    fn test_progress_bar_quarter() {
        assert_eq!(format_progress_bar(0.25, 8), "[==      ] 25%");
    }
}
