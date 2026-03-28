use crate::cli::LabelAction;
use crate::client::TransmissionClient;
use crate::error::Error;
use crate::output::json;
use crate::rpc::methods;

pub fn execute(
    client: &TransmissionClient,
    action: &LabelAction,
    json_output: bool,
) -> Result<(), Error> {
    match action {
        LabelAction::Add { id, label } => add_label(client, *id, label),
        LabelAction::Remove { id, label } => remove_label(client, *id, label),
        LabelAction::List { id } => list_labels(client, *id, json_output),
    }
}

fn get_labels(client: &TransmissionClient, id: i64) -> Result<Vec<String>, Error> {
    let detail = methods::torrent_get_detail(client, id)?;
    Ok(detail.labels.clone())
}

fn add_label(client: &TransmissionClient, id: i64, label: &str) -> Result<(), Error> {
    let mut labels = get_labels(client, id)?;
    if labels.iter().any(|l| l == label) {
        println!("Label '{label}' already exists on torrent {id}");
        return Ok(());
    }
    labels.push(label.to_string());
    methods::torrent_set_labels(client, id, labels)?;
    println!("Label '{label}' added to torrent {id}");
    Ok(())
}

fn remove_label(client: &TransmissionClient, id: i64, label: &str) -> Result<(), Error> {
    let mut labels = get_labels(client, id)?;
    let original_len = labels.len();
    labels.retain(|l| l != label);
    if labels.len() == original_len {
        println!("Label '{label}' not found on torrent {id}");
        return Ok(());
    }
    methods::torrent_set_labels(client, id, labels)?;
    println!("Label '{label}' removed from torrent {id}");
    Ok(())
}

fn list_labels(client: &TransmissionClient, id: i64, json_output: bool) -> Result<(), Error> {
    let labels = get_labels(client, id)?;
    if json_output {
        json::print_json(&labels)
    } else if labels.is_empty() {
        println!("No labels on torrent {id}");
        Ok(())
    } else {
        for label in &labels {
            println!("{label}");
        }
        Ok(())
    }
}
