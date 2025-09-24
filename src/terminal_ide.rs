use anyhow::Result;
use std::io;

use crate::window_manager::WindowManager;
use crate::command_palette::CommandPalette;
use crate::keybindings::KeybindingManager;
use crate::features::FeatureRegistry;

/// Main IDE application
pub struct TerminalIDE {
    window_manager: WindowManager,
    command_palette: CommandPalette,
    keybinding_manager: KeybindingManager,
    feature_registry: FeatureRegistry,
    show_command_palette: bool,
    show_gui: bool,
}

impl TerminalIDE {
    pub fn new() -> Self {
        // Get terminal size for display area
        let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
        let display_area = zellij_utils::pane_size::Size {
            cols: cols as usize,
            rows: rows as usize,
        };

        let window_manager = WindowManager::new(display_area, true);

        // Create command system for command palette
        let command_system = crate::command_palette::create_command_system();
        let command_palette = CommandPalette::new(command_system);

        let keybinding_manager = KeybindingManager::new();
        let feature_registry = FeatureRegistry::new();

        Self {
            window_manager,
            command_palette,
            keybinding_manager,
            feature_registry,
            show_command_palette: false,
            show_gui: true,
        }
    }

    /// Render the main IDE interface
    fn render_main_interface(&self) -> String {
        let mut content = String::new();

        content.push_str("🚀 Zellij IDE - Terminal Native Development Environment\n");
        content.push_str("═══════════════════════════════════════════════════════\n\n");
        content.push_str("Welcome to Zellij IDE! A terminal-native IDE with advanced window management.\n\n");
        content.push_str("📋 Available Commands:\n");
        content.push_str("  Cmd+P / Ctrl+P - Command Palette\n");
        content.push_str("  F1 - Toggle GUI Overlay\n");
        content.push_str("  Esc - Exit menus\n");
        content.push_str("  Ctrl+C - Exit IDE\n\n");

        if self.show_gui {
            content.push_str("✨ Press Cmd+P to see the command palette! ✨");
        } else {
            content.push_str("GUI overlay hidden. Press F1 to show it again.");
        }

        content
    }

    /// Render the command palette
    fn render_command_palette(&self) -> String {
        let mut content = String::new();

        content.push_str("🎨 Command Palette\n");
        content.push_str("═══════════════════\n\n");
        content.push_str("⚡ Available Commands:\n\n");

        content.push_str("📁  File: New, Open, Save\n");
        content.push_str("✏️   Edit: Cut, Copy, Paste\n");
        content.push_str("👁️   View: Toggle panels, Zoom\n");
        content.push_str("▦  Pane: Split horizontal, Split vertical\n");
        content.push_str("📑  Tab: New tab, Close tab\n");
        content.push_str("💻  Terminal: New terminal, Run command\n\n");

        content.push_str("Press Esc to close • Use ↑↓ to navigate • Enter to select");

        content
    }

    /// Simple run method for now
    pub async fn run() -> Result<()> {
        let ide = Self::new();

        // Clear screen and show initial interface
        print!("\x1B[2J\x1B[H"); // Clear screen and move cursor to top

        if ide.show_command_palette {
            println!("{}", ide.render_command_palette());
        } else {
            println!("{}", ide.render_main_interface());
        }

        println!("\nPress Enter to exit...");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(())
    }
}

/// Default trait implementation for easier instantiation
impl Default for TerminalIDE {
    fn default() -> Self {
        Self::new()
    }
}