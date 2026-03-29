use comfy_table::{Cell, ContentArrangement, Table};

use crate::cli::PolicyAction;
use crate::client::TransmissionClient;
use crate::config::{Config, SeedPolicy};
use crate::error::Error;
use crate::rpc::methods;

pub fn execute(
    client: &TransmissionClient,
    action: &PolicyAction,
    config: &Config,
    json_mode: bool,
) -> Result<(), Error> {
    match action {
        PolicyAction::List => execute_list(config, json_mode),
        PolicyAction::Apply { dry_run } => execute_apply(client, config, *dry_run, json_mode),
    }
}

fn execute_list(config: &Config, json_mode: bool) -> Result<(), Error> {
    if config.policies.is_empty() {
        if json_mode {
            println!("[]");
        } else {
            println!("No seeding policies configured.");
            println!("Add [[policies]] sections to your config.toml file.");
        }
        return Ok(());
    }

    if json_mode {
        let policies: Vec<serde_json::Value> = config
            .policies
            .iter()
            .map(|p| {
                serde_json::json!({
                    "name": p.name,
                    "match_label": p.match_label,
                    "seed_ratio": p.seed_ratio,
                    "seed_idle_minutes": p.seed_idle_minutes,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&policies).unwrap_or_default()
        );
        return Ok(());
    }

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["Name", "Match Label", "Seed Ratio", "Idle Minutes"]);

    for p in &config.policies {
        table.add_row(vec![
            Cell::new(&p.name),
            Cell::new(&p.match_label),
            Cell::new(
                p.seed_ratio
                    .map(|r| format!("{r:.1}"))
                    .unwrap_or_else(|| "-".to_string()),
            ),
            Cell::new(
                p.seed_idle_minutes
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            ),
        ]);
    }

    println!("{table}");
    Ok(())
}

fn find_matching_policy<'a>(
    labels: &[String],
    policies: &'a [SeedPolicy],
) -> Option<&'a SeedPolicy> {
    policies.iter().find(|p| {
        labels
            .iter()
            .any(|l| l.eq_ignore_ascii_case(&p.match_label))
    })
}

fn execute_apply(
    client: &TransmissionClient,
    config: &Config,
    dry_run: bool,
    json_mode: bool,
) -> Result<(), Error> {
    if config.policies.is_empty() {
        if !json_mode {
            println!("No seeding policies configured.");
        }
        return Ok(());
    }

    let torrents = methods::torrent_get_list(client)?;
    let mut match_count = 0u32;
    let mut json_changes: Vec<serde_json::Value> = Vec::new();

    for t in &torrents {
        if let Some(policy) = find_matching_policy(&t.labels, &config.policies) {
            match_count += 1;
            if dry_run {
                if json_mode {
                    json_changes.push(serde_json::json!({
                        "id": t.id,
                        "name": t.name,
                        "policy": policy.name,
                        "seed_ratio": policy.seed_ratio,
                        "seed_idle_minutes": policy.seed_idle_minutes,
                    }));
                } else {
                    println!(
                        "ID {}: \"{}\" matches policy \"{}\" (label: {}) -> ratio: {}, idle: {}",
                        t.id,
                        t.name,
                        policy.name,
                        policy.match_label,
                        policy
                            .seed_ratio
                            .map(|r| format!("{r:.1}"))
                            .unwrap_or_else(|| "unchanged".to_string()),
                        policy
                            .seed_idle_minutes
                            .map(|m| format!("{m} min"))
                            .unwrap_or_else(|| "unchanged".to_string()),
                    );
                }
            } else {
                methods::torrent_set_seed_limits(
                    client,
                    t.id,
                    policy.seed_ratio,
                    policy.seed_idle_minutes,
                )?;
                if json_mode {
                    json_changes.push(serde_json::json!({
                        "id": t.id,
                        "name": t.name,
                        "policy": policy.name,
                        "applied": true,
                    }));
                } else {
                    println!(
                        "Applied policy \"{}\" to ID {}: {}",
                        policy.name, t.id, t.name
                    );
                }
            }
        }
    }

    if json_mode {
        println!(
            "{}",
            serde_json::to_string_pretty(&json_changes).unwrap_or_default()
        );
    } else if match_count == 0 {
        println!("No torrents matched any policy.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_policy(name: &str, label: &str, ratio: Option<f64>, idle: Option<i64>) -> SeedPolicy {
        SeedPolicy {
            name: name.to_string(),
            match_label: label.to_string(),
            seed_ratio: ratio,
            seed_idle_minutes: idle,
        }
    }

    #[test]
    fn test_find_matching_policy_matches_label() {
        let policies = vec![
            make_policy("movies", "4k", Some(2.5), Some(4320)),
            make_policy("tv", "tv", Some(1.5), Some(1440)),
        ];
        let labels = vec!["4k".to_string(), "action".to_string()];
        let matched = find_matching_policy(&labels, &policies);
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().name, "movies");
    }

    #[test]
    fn test_find_matching_policy_first_match_wins() {
        let policies = vec![
            make_policy("first", "shared", Some(1.0), None),
            make_policy("second", "shared", Some(2.0), None),
        ];
        let labels = vec!["shared".to_string()];
        let matched = find_matching_policy(&labels, &policies);
        assert_eq!(matched.unwrap().name, "first");
    }

    #[test]
    fn test_find_matching_policy_no_match() {
        let policies = vec![make_policy("movies", "4k", Some(2.5), None)];
        let labels = vec!["tv".to_string()];
        assert!(find_matching_policy(&labels, &policies).is_none());
    }

    #[test]
    fn test_find_matching_policy_empty_policies() {
        let policies: Vec<SeedPolicy> = vec![];
        let labels = vec!["4k".to_string()];
        assert!(find_matching_policy(&labels, &policies).is_none());
    }

    #[test]
    fn test_find_matching_policy_empty_labels() {
        let policies = vec![make_policy("movies", "4k", Some(2.5), None)];
        let labels: Vec<String> = vec![];
        assert!(find_matching_policy(&labels, &policies).is_none());
    }

    #[test]
    fn test_find_matching_policy_case_insensitive() {
        let policies = vec![make_policy("movies", "4K", Some(2.5), None)];
        let labels = vec!["4k".to_string()];
        assert!(find_matching_policy(&labels, &policies).is_some());
    }
}
