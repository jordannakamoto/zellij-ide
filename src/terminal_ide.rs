use anyhow::Result;
use std::path::PathBuf;
use tokio::time::{sleep, Duration};
use zellij_utils::pane_size::Size;

use crate::window_manager::WindowManager;
use crate::gui_controls::GuiControls;

/// Main terminal IDE orchestrator
pub struct TerminalIDE {
    window_manager: WindowManager,
    gui_controls: Option<GuiControls>,
    working_directory: Option<PathBuf>,
}

impl TerminalIDE {
    pub async fn new(enable_gui_controls: bool, directory: Option<PathBuf>) -> Result<Self> {
        // Get terminal size
        let (cols, rows) = crossterm::terminal::size()?;
        let display_area = Size {
            cols: cols as usize,
            rows: rows as usize,
        };

        let window_manager = WindowManager::new(display_area, enable_gui_controls);

        let gui_controls = if enable_gui_controls {
            Some(GuiControls::new()?)
        } else {
            None
        };

        Ok(Self {
            window_manager,
            gui_controls,
            working_directory: directory,
        })
    }

    pub async fn start_session(&mut self, session_name: Option<String>) -> Result<()> {
        let name = session_name.unwrap_or_else(|| "default".to_string());
        log::info!("Starting IDE session: {}", name);

        // Create initial tab and pane
        let tab_id = self.window_manager.create_tab(Some("Main".to_string()))?;
        log::info!("Created initial tab: {:?}", tab_id);

        // Change to working directory if specified
        if let Some(ref dir) = self.working_directory {
            log::info!("Setting working directory to: {}", dir.display());
            std::env::set_current_dir(dir)?;
        }

        // Start the main event loop
        self.run_event_loop().await?;

        Ok(())
    }

    pub async fn attach_session(&mut self, session_name: String) -> Result<()> {
        log::info!("Attaching to session: {}", session_name);
        // TODO: Implement session persistence and attachment
        println!("Session attachment not yet implemented. Starting new session instead.");
        self.start_session(Some(session_name)).await
    }

    pub async fn list_sessions(&self) -> Result<()> {
        println!("Active sessions:");
        println!("  - No sessions found (persistence not yet implemented)");
        Ok(())
    }

    async fn run_event_loop(&mut self) -> Result<()> {
        log::info!("Starting IDE event loop");

        // Show initial interface immediately
        if let Some(ref mut gui_controls) = self.gui_controls {
            gui_controls.show_initial_interface(&self.window_manager)?;
        }

        loop {
            // Handle GUI events if enabled
            if let Some(ref mut gui_controls) = self.gui_controls {
                if !gui_controls.handle_events(&mut self.window_manager)? {
                    log::info!("GUI controls requested exit");
                    break;
                }

                // Always render GUI (not just when overlay is shown)
                gui_controls.render(&self.window_manager)?;
            } else {
                // If no GUI controls, just show a basic message and exit
                println!("Zellij IDE running without GUI controls. Press Ctrl+C to exit.");
                sleep(Duration::from_millis(1000)).await;
            }

            // Handle terminal/pane events
            self.handle_terminal_events().await?;

            // Small delay to prevent busy waiting
            sleep(Duration::from_millis(16)).await;
        }

        log::info!("IDE event loop ending");
        Ok(())
    }

    async fn handle_terminal_events(&mut self) -> Result<()> {
        // TODO: Integrate with Zellij's terminal handling
        // This would involve:
        // 1. Reading from PTY processes
        // 2. Updating terminal grids
        // 3. Handling terminal output
        // 4. Managing process lifecycle

        // For now, just a placeholder
        Ok(())
    }

    /// Create a new terminal pane
    pub fn create_terminal_pane(&mut self) -> Result<()> {
        let _pane_id = self.window_manager.create_tiled_pane_in_tab(
            // TODO: Get current tab ID
            crate::window_manager::TabId::new(),
            None
        )?;

        log::info!("Created new terminal pane: {:?}", _pane_id);
        Ok(())
    }

    /// Create a floating window with terminal
    pub fn create_floating_terminal(&mut self) -> Result<()> {
        let _window_id = self.window_manager.create_floating_window(None)?;
        log::info!("Created floating terminal window: {:?}", _window_id);
        Ok(())
    }

    /// Split current pane
    pub fn split_current_pane(&mut self, direction: zellij_utils::data::Direction) -> Result<()> {
        // TODO: Get current pane ID and split it
        let placeholder_pane_id = crate::window_manager::PaneId::new();
        let _new_pane_id = self.window_manager.split_pane_at_position(
            placeholder_pane_id,
            (0, 0),
            direction
        )?;

        log::info!("Split pane in direction: {:?}", direction);
        Ok(())
    }
}