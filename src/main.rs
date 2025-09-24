use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod window_manager;
mod terminal_ide;
mod command_palette;
mod keybindings;
mod features;

use terminal_ide::TerminalIDE;

#[derive(Parser)]
#[clap(name = "zellij-ide")]
#[clap(about = "A terminal-native IDE with advanced window management")]
#[clap(version = "0.1.0")]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,

    /// Enable GUI controls overlay
    #[clap(long, default_value = "true")]
    gui_controls: bool,

    /// Starting directory
    #[clap(short, long)]
    directory: Option<PathBuf>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Start the IDE session
    Start {
        /// Session name
        #[clap(short, long)]
        name: Option<String>,
    },
    /// Attach to existing session
    Attach {
        /// Session name
        name: String,
    },
    /// List active sessions
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    env_logger::init();

    match cli.command {
        Some(Commands::Start { name }) => {
            log::info!("Starting IDE session: {:?}", name);
            TerminalIDE::run().await?;
        },
        Some(Commands::Attach { name }) => {
            log::info!("Attach not yet implemented, starting new session: {}", name);
            TerminalIDE::run().await?;
        },
        Some(Commands::List) => {
            println!("Session listing not yet implemented");
        },
        None => {
            // Default to starting IDE
            TerminalIDE::run().await?;
        },
    }

    Ok(())
}