use crate::client::TransmissionClient;
use crate::error::Error;
use crate::output::tui;

pub fn execute(client: &TransmissionClient, interval: u64) -> Result<(), Error> {
    tui::run_top(client, interval)
}
