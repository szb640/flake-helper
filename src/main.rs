use clap::{Parser, Subcommand};

/// Utilities for managing development environments with flake.
#[derive(Parser)]
#[command(name = "fh", version, about)]
struct Cli {
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
    /// Populate the cache.
    Cache {
        /// Recurse into subdirectories.
        #[arg(short, long)]
        recurse: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.action {
        Action::Update { recurse } => {
            println!("Running update (recurse = {recurse})");
        }
        Action::Cache { recurse } => {
            println!("Running cache (recurse = {recurse})");
        }
    }
}
