use clap::{Parser, Subcommand};

mod update;

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
}

fn main() {
    let cli = Cli::parse();

    match cli.action {
        Action::Update { recurse } => {
            update::run(recurse);
        }
    }
}
