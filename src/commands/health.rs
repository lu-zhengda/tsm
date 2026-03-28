use crate::client::TransmissionClient;
use crate::error::Error;
use crate::output::json;
use crate::rpc::methods;

pub fn execute(client: &TransmissionClient, json_output: bool) -> Result<(), Error> {
    // 1. Connectivity
    let session = methods::session_get(client)?;
    let version = session
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // 2. Disk space
    let download_dir = session
        .get("download-dir")
        .and_then(|v| v.as_str())
        .unwrap_or("/tmp");
    let free = methods::free_space(client, download_dir)?;
    let free_gb = free.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let disk_ok = free_gb >= 1.0;

    // 3. Port test
    let peer_port = session
        .get("peer-port")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let port_open = methods::port_test(client)?;

    if json_output {
        let value = serde_json::json!({
            "connectivity": { "ok": true, "version": version },
            "disk_space": {
                "ok": disk_ok,
                "path": download_dir,
                "free_bytes": free.size_bytes,
            },
            "port": {
                "port": peer_port,
                "open": port_open,
            },
        });
        return json::print_json(&value);
    }

    println!("Connectivity:  OK (Transmission {version})");

    if disk_ok {
        println!(
            "Disk Space:    OK ({:.1} GB free on {download_dir})",
            free_gb
        );
    } else {
        println!(
            "Disk Space:    WARNING ({:.1} GB free on {download_dir})",
            free_gb
        );
    }

    if port_open {
        println!("Port ({peer_port}):    OPEN");
    } else {
        println!("Port ({peer_port}):    CLOSED");
    }

    Ok(())
}
