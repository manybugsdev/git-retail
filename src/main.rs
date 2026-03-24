mod ssh;
mod web;

use clap::{Parser, Subcommand};

/// git-retail – local-oriented git subcommand for managing bare repositories
/// via an SSH host named `git-retailer`.
#[derive(Parser)]
#[command(name = "git-retail", about = "Manage bare git repositories via SSH")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the web UI on port 7411
    Up,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Up => web::start().await,
    }
}

