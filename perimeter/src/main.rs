use clap::Parser;
use perimeter::cli::{Cli, Commands};
use perimeter::session;
use colored::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run { session }) => {
            println!("{} {}", "Starting Citadel session:".green().bold(), session.cyan());
            perimeter::run_agent(&session).await?;
        }
        Some(Commands::List) => {
            let sessions = session::list_sessions()?;
            if sessions.is_empty() {
                println!("No saved sessions found.");
            } else {
                println!("{}", "Saved Sessions:".yellow().bold());
                for s in sessions {
                    println!("  - {}", s);
                }
            }
        }
        None => {
            // Default behavior if no command provided? Maybe show help.
            // Clap usually handles this if command is required, but here it's Option.
            println!("Use --help for usage instructions.");
        }
    }

    Ok(())
}