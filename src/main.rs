use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod window_manager;
mod terminal_ide;
mod gui_controls;
mod command_palette;
mod keybindings;

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

    // Start the terminal IDE
    let mut ide = TerminalIDE::new(cli.gui_controls, cli.directory).await?;

    match cli.command {
        Some(Commands::Start { name }) => {
            ide.start_session(name).await?;
        },
        Some(Commands::Attach { name }) => {
            ide.attach_session(name).await?;
        },
        Some(Commands::List) => {
            ide.list_sessions().await?;
        },
        None => {
            // Default to starting a new session
            ide.start_session(None).await?;
        },
    }

    Ok(())
}