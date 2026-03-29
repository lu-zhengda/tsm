use crate::cli::SortField;
use crate::client::TransmissionClient;
use crate::error::Error;
use crate::filter;
use crate::output::{json, table};
use crate::rpc::methods;
use crate::rpc::types::Torrent;

pub fn execute(
    client: &TransmissionClient,
    filter_str: &Option<String>,
    sort: &Option<SortField>,
    ids_only: bool,
    json_output: bool,
    no_color: bool,
) -> Result<(), Error> {
    let mut torrents = methods::torrent_get_list(client)?;

    if let Some(f) = filter_str {
        let expr = filter::parse_filter(f)?;
        torrents.retain(|t| filter::matches(t, &expr));
    }

    if let Some(sort) = sort {
        sort_torrents(&mut torrents, sort);
    }

    if ids_only {
        for t in &torrents {
            println!("{}", t.id);
        }
        return Ok(());
    }

    if json_output {
        json::print_json(&torrents)
    } else {
        if torrents.is_empty() {
            println!("No torrents found.");
        } else {
            table::print_torrent_list(&torrents, no_color);
        }
        Ok(())
    }
}

pub(crate) fn sort_torrents(torrents: &mut [Torrent], sort: &SortField) {
    torrents.sort_by(|a, b| match sort {
        SortField::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        SortField::Size => a.total_size.cmp(&b.total_size),
        SortField::Progress => a
            .percent_done
            .partial_cmp(&b.percent_done)
            .unwrap_or(std::cmp::Ordering::Equal),
        SortField::Ratio => a
            .upload_ratio
            .partial_cmp(&b.upload_ratio)
            .unwrap_or(std::cmp::Ordering::Equal),
        SortField::SpeedDown => b.rate_download.cmp(&a.rate_download),
        SortField::SpeedUp => b.rate_upload.cmp(&a.rate_upload),
        SortField::Added => b.added_date.cmp(&a.added_date),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_torrent(id: i64, name: &str, status: i64, size: i64, progress: f64) -> Torrent {
        Torrent {
            id,
            name: name.to_string(),
            status,
            total_size: size,
            percent_done: progress,
            rate_download: 0,
            rate_upload: 0,
            eta: -1,
            upload_ratio: 0.0,
            added_date: id * 1000,
            labels: vec![],
        }
    }

    #[test]
    fn test_filter_downloading() {
        let mut torrents = vec![
            make_torrent(1, "a", 4, 100, 0.5),
            make_torrent(2, "b", 6, 200, 1.0),
            make_torrent(3, "c", 0, 300, 0.0),
        ];
        let expr = filter::parse_filter("downloading").unwrap();
        torrents.retain(|t| filter::matches(t, &expr));
        assert_eq!(torrents.len(), 1);
        assert_eq!(torrents[0].id, 1);
    }

    #[test]
    fn test_filter_seeding() {
        let mut torrents = vec![
            make_torrent(1, "a", 4, 100, 0.5),
            make_torrent(2, "b", 6, 200, 1.0),
        ];
        let expr = filter::parse_filter("seeding").unwrap();
        torrents.retain(|t| filter::matches(t, &expr));
        assert_eq!(torrents.len(), 1);
        assert_eq!(torrents[0].id, 2);
    }

    #[test]
    fn test_sort_by_name() {
        let mut torrents = vec![
            make_torrent(1, "Zebra", 4, 100, 0.5),
            make_torrent(2, "Apple", 4, 200, 0.8),
            make_torrent(3, "mango", 4, 150, 0.3),
        ];
        sort_torrents(&mut torrents, &SortField::Name);
        assert_eq!(torrents[0].name, "Apple");
        assert_eq!(torrents[1].name, "mango");
        assert_eq!(torrents[2].name, "Zebra");
    }

    #[test]
    fn test_sort_by_size() {
        let mut torrents = vec![
            make_torrent(1, "a", 4, 300, 0.5),
            make_torrent(2, "b", 4, 100, 0.8),
            make_torrent(3, "c", 4, 200, 0.3),
        ];
        sort_torrents(&mut torrents, &SortField::Size);
        assert_eq!(torrents[0].total_size, 100);
        assert_eq!(torrents[1].total_size, 200);
        assert_eq!(torrents[2].total_size, 300);
    }
}
