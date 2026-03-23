use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "Reagis Citadel")]
#[command(version = "1.0")]
#[command(about = "A secure, Rust-powered 'Citadel' for local AI agents", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Run {
        #[arg(short, long, default_value = "default")]
        session: String,
    },
    List,
}