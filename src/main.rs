mod cli;
mod client;
mod commands;
mod config;
mod error;
mod filter;
mod notify_hook;
mod output;
mod rpc;

use std::io;
use std::process;

use clap::Parser;
use clap_complete::generate;

use cli::{Cli, Command};
use client::TransmissionClient;

fn main() {
    let cli = Cli::parse();

    if let Command::Completions { shell } = &cli.command {
        let mut cmd = <Cli as clap::CommandFactory>::command();
        generate(*shell, &mut cmd, "tsm", &mut io::stdout());
        return;
    }

    if let Command::Login { profile } = &cli.command {
        if let Err(e) = commands::login::execute(profile) {
            eprintln!("Error: {e}");
            process::exit(e.exit_code());
        }
        return;
    }

    let config = match config::resolve(&cli) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(e.exit_code());
        }
    };

    // Commands that don't need a server connection
    if let Command::ConfigShow = &cli.command {
        if let Err(e) = commands::config_cmd::execute(&config, config.json) {
            eprintln!("Error: {e}");
            process::exit(e.exit_code());
        }
        return;
    }

    let client = match TransmissionClient::new(&config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(e.exit_code());
        }
    };

    let result = match &cli.command {
        Command::List {
            filter,
            sort,
            ids_only,
        } => commands::list::execute(
            &client,
            filter,
            sort,
            *ids_only,
            config.json,
            config.no_color,
        ),
        Command::Search { query, sort } => {
            commands::search::execute(&client, query, sort, config.json, config.no_color)
        }
        Command::Move { id, path } => commands::relocate::execute(&client, *id, path),
        Command::Label { action } => commands::label::execute(&client, action, config.json),
        Command::Add {
            source,
            paused,
            download_dir,
        } => commands::add::execute(
            &client,
            source,
            *paused,
            download_dir.as_deref(),
            config.json,
        ),
        Command::Start { target } => commands::start_stop::execute_start(&client, target),
        Command::Stop { target } => commands::start_stop::execute_stop(&client, target),
        Command::Remove { id, delete } => commands::remove::execute(&client, *id, *delete),
        Command::Verify { id } => commands::start_stop::execute_verify(&client, *id),
        Command::Info { id } => commands::info::execute_info(&client, *id, config.json),
        Command::Files { id } => commands::info::execute_files(&client, *id, config.json),
        Command::Speed {
            set_down,
            set_up,
            alt_on,
            alt_off,
            no_limit,
        } => commands::speed::execute(
            &client,
            *set_down,
            *set_up,
            *alt_on,
            *alt_off,
            *no_limit,
            config.json,
        ),
        Command::Session => commands::session::execute_session(&client, config.json),
        Command::Stats => commands::session::execute_stats(&client, config.json),
        Command::Free { path } => {
            commands::session::execute_free(&client, path.as_deref(), config.json)
        }
        Command::Health => commands::health::execute(&client, config.json),
        Command::Watch {
            dir,
            paused,
            download_dir,
            delete_after_add,
            notify,
        } => commands::watch::execute(
            &client,
            dir,
            *paused,
            download_dir.as_deref(),
            *delete_after_add,
            if *notify { Some(&config) } else { None },
        ),
        Command::Top { interval } => commands::top::execute(&client, *interval),
        Command::Login { .. } | Command::Completions { .. } | Command::ConfigShow => {
            unreachable!()
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(e.exit_code());
    }
}
