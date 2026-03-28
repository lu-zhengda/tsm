use serde::Serialize;

use crate::error::Error;

pub fn print_json<T: Serialize>(data: &T) -> Result<(), Error> {
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| Error::Rpc(format!("Failed to serialize JSON output: {e}")))?;
    println!("{json}");
    Ok(())
}
