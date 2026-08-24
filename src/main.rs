use clap::{Parser, Subcommand};
use fh::update;

/// Utilities for managing development environments with flake.
#[derive(Parser)]
#[command(name = "fh", version, about)]
struct Cli {
    /// Increase logging verbosity.
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Update flake dependencies
    Update {
        /// Recurse into subdirectories.
        #[arg(short, long)]
        recurse: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let default_level = if cli.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_level))
        .init();

    match cli.action {
        Action::Update { recurse } => update::run(recurse),
    }
}