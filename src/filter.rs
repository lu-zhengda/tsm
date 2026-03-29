use crate::error::Error;
use crate::rpc::types::Torrent;

#[derive(Debug, PartialEq)]
pub struct FilterExpr {
    conditions: Vec<FilterCondition>,
}

#[derive(Debug, PartialEq)]
enum FilterCondition {
    RatioGt(f64),
    RatioLt(f64),
    RatioEq(f64),
    SizeGt(i64),
    SizeLt(i64),
    ProgressGt(f64),
    ProgressLt(f64),
    ProgressEq(f64),
    AgeGt(i64), // seconds since added
    AgeLt(i64),
    LabelMatch(String),
    NameContains(String),
    StatusMatch(String),
}

pub fn parse_filter(input: &str) -> Result<FilterExpr, Error> {
    let trimmed = input.trim();

    // Backward compat: bare status keywords
    if let Some(expr) = try_legacy_status(trimmed) {
        return Ok(expr);
    }

    let parts: Vec<&str> = split_and(trimmed);
    let mut conditions = Vec::new();
    for part in parts {
        conditions.push(parse_condition(part.trim())?);
    }

    if conditions.is_empty() {
        return Err(Error::Filter("Empty filter expression".to_string()));
    }

    Ok(FilterExpr { conditions })
}

pub fn matches(torrent: &Torrent, expr: &FilterExpr) -> bool {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    expr.conditions
        .iter()
        .all(|c| match_condition(torrent, c, now))
}

fn match_condition(t: &Torrent, cond: &FilterCondition, now: i64) -> bool {
    match cond {
        FilterCondition::RatioGt(v) => t.upload_ratio > *v,
        FilterCondition::RatioLt(v) => t.upload_ratio < *v,
        FilterCondition::RatioEq(v) => (t.upload_ratio - v).abs() < 0.01,
        FilterCondition::SizeGt(v) => t.total_size > *v,
        FilterCondition::SizeLt(v) => t.total_size < *v,
        FilterCondition::ProgressGt(v) => (t.percent_done * 100.0) > *v,
        FilterCondition::ProgressLt(v) => (t.percent_done * 100.0) < *v,
        FilterCondition::ProgressEq(v) => ((t.percent_done * 100.0) - v).abs() < 0.5,
        FilterCondition::AgeGt(secs) => (now - t.added_date) > *secs,
        FilterCondition::AgeLt(secs) => (now - t.added_date) < *secs,
        FilterCondition::LabelMatch(label) => {
            let label_lower = label.to_lowercase();
            t.labels.iter().any(|l| l.to_lowercase() == label_lower)
        }
        FilterCondition::NameContains(sub) => t.name.to_lowercase().contains(&sub.to_lowercase()),
        FilterCondition::StatusMatch(status) => {
            let codes = status_to_codes(status);
            codes.contains(&t.status)
        }
    }
}

fn try_legacy_status(input: &str) -> Option<FilterExpr> {
    let lower = input.to_lowercase();
    let valid = [
        "downloading",
        "seeding",
        "paused",
        "stopped",
        "checking",
        "queued",
    ];
    if valid.contains(&lower.as_str()) {
        Some(FilterExpr {
            conditions: vec![FilterCondition::StatusMatch(lower)],
        })
    } else {
        None
    }
}

fn status_to_codes(status: &str) -> Vec<i64> {
    match status {
        "downloading" => vec![4],
        "seeding" => vec![6],
        "paused" | "stopped" => vec![0],
        "checking" => vec![1, 2],
        "queued" => vec![3, 5],
        _ => vec![],
    }
}

fn split_and(input: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut rest = input;

    loop {
        // Find case-insensitive " AND " (with spaces)
        let lower = rest.to_lowercase();
        if let Some(pos) = lower.find(" and ") {
            result.push(&rest[..pos]);
            rest = &rest[pos + 5..];
        } else {
            result.push(rest);
            break;
        }
    }

    result
}

fn parse_condition(token: &str) -> Result<FilterCondition, Error> {
    // Try key:value format first
    if let Some((key, value)) = token.split_once(':') {
        return match key.to_lowercase().as_str() {
            "label" => Ok(FilterCondition::LabelMatch(value.to_string())),
            "name" => Ok(FilterCondition::NameContains(value.to_string())),
            "status" => Ok(FilterCondition::StatusMatch(value.to_lowercase())),
            _ => Err(Error::Filter(format!("Unknown field: '{key}'"))),
        };
    }

    // Try field<op>value format
    let (field, op, value_str) = parse_field_op_value(token)?;

    match field {
        "ratio" => {
            let v: f64 = value_str
                .parse()
                .map_err(|_| Error::Filter(format!("Invalid ratio value: '{value_str}'")))?;
            match op {
                '>' => Ok(FilterCondition::RatioGt(v)),
                '<' => Ok(FilterCondition::RatioLt(v)),
                '=' => Ok(FilterCondition::RatioEq(v)),
                _ => unreachable!(),
            }
        }
        "size" => {
            let v = parse_size(value_str)?;
            match op {
                '>' => Ok(FilterCondition::SizeGt(v)),
                '<' => Ok(FilterCondition::SizeLt(v)),
                _ => Err(Error::Filter("Size only supports > and <".to_string())),
            }
        }
        "progress" => {
            let v: f64 = value_str
                .parse()
                .map_err(|_| Error::Filter(format!("Invalid progress value: '{value_str}'")))?;
            match op {
                '>' => Ok(FilterCondition::ProgressGt(v)),
                '<' => Ok(FilterCondition::ProgressLt(v)),
                '=' => Ok(FilterCondition::ProgressEq(v)),
                _ => unreachable!(),
            }
        }
        "age" => {
            let v = parse_duration(value_str)?;
            match op {
                '>' => Ok(FilterCondition::AgeGt(v)),
                '<' => Ok(FilterCondition::AgeLt(v)),
                _ => Err(Error::Filter("Age only supports > and <".to_string())),
            }
        }
        _ => Err(Error::Filter(format!("Unknown field: '{field}'"))),
    }
}

fn parse_field_op_value(token: &str) -> Result<(&str, char, &str), Error> {
    for (i, c) in token.char_indices() {
        if c == '>' || c == '<' || c == '=' {
            let field = &token[..i];
            let value = &token[i + 1..];
            if field.is_empty() || value.is_empty() {
                return Err(Error::Filter(format!("Invalid condition: '{token}'")));
            }
            return Ok((field, c, value));
        }
    }
    Err(Error::Filter(format!(
        "Invalid condition: '{token}'. Expected format: field>value, field<value, field:value"
    )))
}

fn parse_size(s: &str) -> Result<i64, Error> {
    let s = s.trim();
    let (num_str, multiplier) = if let Some(n) = s.strip_suffix("TB") {
        (n, 1024_i64 * 1024 * 1024 * 1024)
    } else if let Some(n) = s.strip_suffix("GB") {
        (n, 1024_i64 * 1024 * 1024)
    } else if let Some(n) = s.strip_suffix("MB") {
        (n, 1024_i64 * 1024)
    } else if let Some(n) = s.strip_suffix("KB") {
        (n, 1024_i64)
    } else {
        (s, 1_i64) // bytes
    };

    let num: f64 = num_str
        .trim()
        .parse()
        .map_err(|_| Error::Filter(format!("Invalid size: '{s}'")))?;
    Ok((num * multiplier as f64) as i64)
}

fn parse_duration(s: &str) -> Result<i64, Error> {
    let s = s.trim();
    let (num_str, multiplier) = if let Some(n) = s.strip_suffix('d') {
        (n, 86400_i64)
    } else if let Some(n) = s.strip_suffix('h') {
        (n, 3600_i64)
    } else if let Some(n) = s.strip_suffix('m') {
        (n, 60_i64)
    } else {
        (s, 1_i64) // seconds
    };

    let num: i64 = num_str
        .trim()
        .parse()
        .map_err(|_| Error::Filter(format!("Invalid duration: '{s}'")))?;
    Ok(num * multiplier)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_torrent(name: &str, status: i64, ratio: f64, size: i64, progress: f64) -> Torrent {
        Torrent {
            id: 1,
            name: name.to_string(),
            status,
            total_size: size,
            percent_done: progress,
            rate_download: 0,
            rate_upload: 0,
            eta: -1,
            upload_ratio: ratio,
            added_date: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
                - 3600, // 1 hour ago
            labels: vec!["movies".to_string(), "hd".to_string()],
        }
    }

    #[test]
    fn test_parse_legacy_status() {
        let expr = parse_filter("downloading").unwrap();
        assert_eq!(expr.conditions.len(), 1);
    }

    #[test]
    fn test_parse_status_colon() {
        let expr = parse_filter("status:seeding").unwrap();
        assert_eq!(expr.conditions.len(), 1);
    }

    #[test]
    fn test_parse_ratio_gt() {
        let expr = parse_filter("ratio>2.0").unwrap();
        assert_eq!(expr.conditions, vec![FilterCondition::RatioGt(2.0)]);
    }

    #[test]
    fn test_parse_size_suffix() {
        let expr = parse_filter("size>500MB").unwrap();
        assert_eq!(
            expr.conditions,
            vec![FilterCondition::SizeGt(500 * 1024 * 1024)]
        );
    }

    #[test]
    fn test_parse_age_duration() {
        let expr = parse_filter("age<7d").unwrap();
        assert_eq!(expr.conditions, vec![FilterCondition::AgeLt(7 * 86400)]);
    }

    #[test]
    fn test_parse_compound() {
        let expr = parse_filter("ratio>2.0 AND label:movies").unwrap();
        assert_eq!(expr.conditions.len(), 2);
    }

    #[test]
    fn test_parse_case_insensitive_and() {
        let expr = parse_filter("ratio>1 and name:ubuntu").unwrap();
        assert_eq!(expr.conditions.len(), 2);
    }

    #[test]
    fn test_matches_ratio() {
        let t = make_torrent("test", 6, 2.5, 1000, 1.0);
        assert!(matches(&t, &parse_filter("ratio>2.0").unwrap()));
        assert!(!matches(&t, &parse_filter("ratio>3.0").unwrap()));
    }

    #[test]
    fn test_matches_label() {
        let t = make_torrent("test", 6, 1.0, 1000, 1.0);
        assert!(matches(&t, &parse_filter("label:movies").unwrap()));
        assert!(!matches(&t, &parse_filter("label:linux").unwrap()));
    }

    #[test]
    fn test_matches_name_case_insensitive() {
        let t = make_torrent("Ubuntu Server", 4, 0.0, 1000, 0.5);
        assert!(matches(&t, &parse_filter("name:ubuntu").unwrap()));
        assert!(matches(&t, &parse_filter("name:SERVER").unwrap()));
    }

    #[test]
    fn test_matches_compound_all_must_pass() {
        let t = make_torrent("test", 6, 2.5, 1000, 1.0);
        assert!(matches(
            &t,
            &parse_filter("ratio>2.0 AND label:movies").unwrap()
        ));
        assert!(!matches(
            &t,
            &parse_filter("ratio>2.0 AND label:linux").unwrap()
        ));
    }

    #[test]
    fn test_parse_invalid_field() {
        assert!(parse_filter("foo>2").is_err());
    }

    #[test]
    fn test_parse_invalid_operator() {
        assert!(parse_filter("ratio!2.0").is_err());
    }

    #[test]
    fn test_matches_progress() {
        let t = make_torrent("test", 4, 0.0, 1000, 0.5); // 50%
        assert!(matches(&t, &parse_filter("progress>25").unwrap()));
        assert!(!matches(&t, &parse_filter("progress>75").unwrap()));
    }

    #[test]
    fn test_matches_status() {
        let t = make_torrent("test", 4, 0.0, 1000, 0.5);
        assert!(matches(&t, &parse_filter("status:downloading").unwrap()));
        assert!(!matches(&t, &parse_filter("status:seeding").unwrap()));
    }

    #[test]
    fn test_matches_size() {
        let t = make_torrent("test", 4, 0.0, 2 * 1024 * 1024 * 1024, 0.5); // 2 GB
        assert!(matches(&t, &parse_filter("size>1GB").unwrap()));
        assert!(!matches(&t, &parse_filter("size>3GB").unwrap()));
    }
}
