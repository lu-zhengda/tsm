use crate::config::Config;

pub struct CompletionContext {
    pub name: String,
    pub id: i64,
    pub download_dir: String,
    pub size: i64,
    pub ratio: f64,
}

pub fn expand_template(template: &str, ctx: &CompletionContext) -> String {
    template
        .replace("{name}", &ctx.name)
        .replace("{id}", &ctx.id.to_string())
        .replace("{download_dir}", &ctx.download_dir)
        .replace("{size}", &ctx.size.to_string())
        .replace("{ratio}", &format!("{:.2}", ctx.ratio))
}

pub fn fire_completion(
    ctx: &CompletionContext,
    config: &Config,
    on_complete_override: Option<&str>,
) {
    let script_template = on_complete_override.or(config.on_complete_script.as_deref());

    if let Some(template) = script_template {
        let expanded = expand_template(template, ctx);
        // Split the expanded string into command and args
        let parts: Vec<&str> = expanded.split_whitespace().collect();
        if let Some((cmd, args)) = parts.split_first() {
            match std::process::Command::new(cmd).args(args).spawn() {
                Ok(_) => {}
                Err(e) => eprintln!("Warning: notification script failed: {e}"),
            }
        }
    }

    if let Some(url) = &config.on_complete_webhook {
        let payload = serde_json::json!({
            "event": "torrent_complete",
            "name": ctx.name,
            "id": ctx.id,
            "download_dir": ctx.download_dir,
            "size": ctx.size,
            "ratio": ctx.ratio,
        });
        let body = serde_json::to_vec(&payload).unwrap_or_default();
        let agent: ureq::Agent = ureq::Agent::new_with_defaults();
        match agent
            .post(url)
            .header("Content-Type", "application/json")
            .send(&body[..])
        {
            Ok(_) => {}
            Err(e) => eprintln!("Warning: notification webhook failed: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx() -> CompletionContext {
        CompletionContext {
            name: "ubuntu-24.04.iso".to_string(),
            id: 42,
            download_dir: "/downloads".to_string(),
            size: 3_000_000_000,
            ratio: 1.53,
        }
    }

    #[test]
    fn test_expand_all_vars() {
        let ctx = make_ctx();
        let result = expand_template(
            "/bin/script.sh {name} {id} {download_dir} {size} {ratio}",
            &ctx,
        );
        assert_eq!(
            result,
            "/bin/script.sh ubuntu-24.04.iso 42 /downloads 3000000000 1.53"
        );
    }

    #[test]
    fn test_expand_no_vars() {
        let ctx = make_ctx();
        let result = expand_template("/bin/script.sh", &ctx);
        assert_eq!(result, "/bin/script.sh");
    }

    #[test]
    fn test_expand_repeated_vars() {
        let ctx = make_ctx();
        let result = expand_template("{name} - {name}", &ctx);
        assert_eq!(result, "ubuntu-24.04.iso - ubuntu-24.04.iso");
    }

    #[test]
    fn test_expand_unknown_vars_left_as_is() {
        let ctx = make_ctx();
        let result = expand_template("{unknown} {name}", &ctx);
        assert_eq!(result, "{unknown} ubuntu-24.04.iso");
    }

    #[test]
    fn test_expand_empty_template() {
        let ctx = make_ctx();
        let result = expand_template("", &ctx);
        assert_eq!(result, "");
    }
}
