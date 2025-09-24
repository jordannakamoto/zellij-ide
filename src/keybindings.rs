use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::window_manager::WindowManager;

/// A key combination that can trigger commands
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyBinding {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    pub fn new(key: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { key, modifiers }
    }

    /// Create a simple key binding with no modifiers
    pub fn key(key: KeyCode) -> Self {
        Self::new(key, KeyModifiers::NONE)
    }

    /// Create a Ctrl+key binding
    pub fn ctrl(key: KeyCode) -> Self {
        Self::new(key, KeyModifiers::CONTROL)
    }

    /// Create a Shift+key binding
    pub fn shift(key: KeyCode) -> Self {
        Self::new(key, KeyModifiers::SHIFT)
    }

    /// Create a Ctrl+Shift+key binding
    pub fn ctrl_shift(key: KeyCode) -> Self {
        Self::new(key, KeyModifiers::CONTROL | KeyModifiers::SHIFT)
    }

    /// Create a Alt+key binding
    pub fn alt(key: KeyCode) -> Self {
        Self::new(key, KeyModifiers::ALT)
    }

    /// Check if this key binding matches a key event
    pub fn matches(&self, event: &KeyEvent) -> bool {
        self.key == event.code && self.modifiers == event.modifiers
    }

    /// Parse a key binding from a string like "Ctrl+P" or "F1"
    pub fn from_string(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('+').collect();
        let mut modifiers = KeyModifiers::NONE;
        let mut key_str = s;

        if parts.len() > 1 {
            key_str = parts.last().unwrap();
            for part in &parts[..parts.len() - 1] {
                match part.to_lowercase().as_str() {
                    "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
                    "shift" => modifiers |= KeyModifiers::SHIFT,
                    "alt" => modifiers |= KeyModifiers::ALT,
                    _ => return Err(anyhow::anyhow!("Unknown modifier: {}", part)),
                }
            }
        }

        let key = match key_str.to_lowercase().as_str() {
            "f1" => KeyCode::F(1),
            "f2" => KeyCode::F(2),
            "f3" => KeyCode::F(3),
            "f4" => KeyCode::F(4),
            "f5" => KeyCode::F(5),
            "f6" => KeyCode::F(6),
            "f7" => KeyCode::F(7),
            "f8" => KeyCode::F(8),
            "f9" => KeyCode::F(9),
            "f10" => KeyCode::F(10),
            "f11" => KeyCode::F(11),
            "f12" => KeyCode::F(12),
            "enter" | "return" => KeyCode::Enter,
            "space" => KeyCode::Char(' '),
            "escape" | "esc" => KeyCode::Esc,
            "tab" => KeyCode::Tab,
            "backspace" => KeyCode::Backspace,
            "delete" | "del" => KeyCode::Delete,
            "home" => KeyCode::Home,
            "end" => KeyCode::End,
            "pageup" => KeyCode::PageUp,
            "pagedown" => KeyCode::PageDown,
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "insert" => KeyCode::Insert,
            s if s.len() == 1 => KeyCode::Char(s.chars().next().unwrap()),
            _ => return Err(anyhow::anyhow!("Unknown key: {}", key_str)),
        };

        Ok(Self::new(key, modifiers))
    }
}

impl fmt::Display for KeyBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();

        if self.modifiers.contains(KeyModifiers::CONTROL) {
            parts.push("Ctrl");
        }
        if self.modifiers.contains(KeyModifiers::SHIFT) {
            parts.push("Shift");
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            parts.push("Alt");
        }

        let key_str = match self.key {
            KeyCode::F(n) => format!("F{}", n),
            KeyCode::Char(c) => c.to_uppercase().to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Esc => "Escape".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::Delete => "Delete".to_string(),
            KeyCode::Home => "Home".to_string(),
            KeyCode::End => "End".to_string(),
            KeyCode::PageUp => "PageUp".to_string(),
            KeyCode::PageDown => "PageDown".to_string(),
            KeyCode::Up => "Up".to_string(),
            KeyCode::Down => "Down".to_string(),
            KeyCode::Left => "Left".to_string(),
            KeyCode::Right => "Right".to_string(),
            KeyCode::Insert => "Insert".to_string(),
            _ => format!("{:?}", self.key),
        };

        parts.push(&key_str);
        write!(f, "{}", parts.join("+"))
    }
}

/// Represents a command that can be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub action: CommandAction,
}

/// Different types of actions a command can perform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandAction {
    /// Built-in IDE actions
    CreateTab,
    CloseTab,
    NextTab,
    PreviousTab,
    SplitHorizontal,
    SplitVertical,
    ClosePane,
    FocusNextPane,
    FocusPreviousPane,
    ToggleFullscreen,
    CreateFloatingWindow,
    ShowCommandPalette,
    ToggleTerminal,

    /// File operations
    NewFile,
    OpenFile,
    SaveFile,
    SaveAs,
    CloseFile,

    /// Editor operations
    Find,
    Replace,
    GoToLine,
    SelectAll,
    Copy,
    Cut,
    Paste,
    Undo,
    Redo,

    /// View operations
    ZoomIn,
    ZoomOut,
    ResetZoom,
    ToggleSidebar,
    ToggleStatusBar,

    /// Custom command with arbitrary string
    Custom(String),
}

impl Command {
    pub fn new(id: &str, name: &str, description: &str, category: &str, action: CommandAction) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            action,
        }
    }

    /// Execute the command
    pub fn execute(&self, window_manager: &mut WindowManager) -> Result<()> {
        match &self.action {
            CommandAction::CreateTab => {
                window_manager.create_tab(None)?;
            },
            CommandAction::SplitHorizontal => {
                // TODO: Implement horizontal split
                log::info!("Executing split horizontal");
            },
            CommandAction::SplitVertical => {
                // TODO: Implement vertical split
                log::info!("Executing split vertical");
            },
            CommandAction::ShowCommandPalette => {
                // This is handled by the GUI system
                log::info!("Show command palette");
            },
            CommandAction::Custom(cmd) => {
                log::info!("Executing custom command: {}", cmd);
                // TODO: Implement custom command execution
            },
            _ => {
                log::info!("Executing command: {}", self.name);
                // TODO: Implement other commands as needed
            },
        }
        Ok(())
    }
}

/// Manages keybindings and command execution
pub struct KeybindingManager {
    /// Map from keybinding to command ID
    bindings: HashMap<KeyBinding, String>,
    /// Map from command ID to command
    commands: HashMap<String, Command>,
    /// Command categories for organization
    categories: HashMap<String, Vec<String>>,
    /// Track the last executed command ID
    last_executed_command: Option<String>,
}

impl KeybindingManager {
    pub fn new() -> Self {
        let mut manager = Self {
            bindings: HashMap::new(),
            commands: HashMap::new(),
            categories: HashMap::new(),
            last_executed_command: None,
        };

        manager.load_default_bindings();
        manager
    }

    /// Load default IDE keybindings
    pub fn load_default_bindings(&mut self) {
        // File operations
        self.register_command_and_binding(
            Command::new("file.new", "New File", "Create a new file", "File", CommandAction::NewFile),
            KeyBinding::ctrl(KeyCode::Char('n')),
        );
        self.register_command_and_binding(
            Command::new("file.open", "Open File", "Open an existing file", "File", CommandAction::OpenFile),
            KeyBinding::ctrl(KeyCode::Char('o')),
        );
        self.register_command_and_binding(
            Command::new("file.save", "Save File", "Save the current file", "File", CommandAction::SaveFile),
            KeyBinding::ctrl(KeyCode::Char('s')),
        );
        self.register_command_and_binding(
            Command::new("file.close", "Close File", "Close the current file", "File", CommandAction::CloseFile),
            KeyBinding::ctrl(KeyCode::Char('w')),
        );

        // Tab operations
        self.register_command_and_binding(
            Command::new("tab.new", "New Tab", "Create a new tab", "Tab", CommandAction::CreateTab),
            KeyBinding::ctrl(KeyCode::Char('t')),
        );
        self.register_command_and_binding(
            Command::new("tab.close", "Close Tab", "Close the current tab", "Tab", CommandAction::CloseTab),
            KeyBinding::ctrl_shift(KeyCode::Char('w')),
        );
        self.register_command_and_binding(
            Command::new("tab.next", "Next Tab", "Switch to the next tab", "Tab", CommandAction::NextTab),
            KeyBinding::ctrl(KeyCode::Tab),
        );

        // Pane operations
        self.register_command_and_binding(
            Command::new("pane.split_horizontal", "Split Horizontally", "Split the current pane horizontally", "Pane", CommandAction::SplitHorizontal),
            KeyBinding::ctrl_shift(KeyCode::Char('h')),
        );
        self.register_command_and_binding(
            Command::new("pane.split_vertical", "Split Vertically", "Split the current pane vertically", "Pane", CommandAction::SplitVertical),
            KeyBinding::ctrl_shift(KeyCode::Char('v')),
        );
        self.register_command_and_binding(
            Command::new("pane.close", "Close Pane", "Close the current pane", "Pane", CommandAction::ClosePane),
            KeyBinding::ctrl_shift(KeyCode::Char('x')),
        );

        // Editor operations
        self.register_command_and_binding(
            Command::new("editor.find", "Find", "Find text in the current file", "Editor", CommandAction::Find),
            KeyBinding::ctrl(KeyCode::Char('f')),
        );
        self.register_command_and_binding(
            Command::new("editor.replace", "Replace", "Find and replace text", "Editor", CommandAction::Replace),
            KeyBinding::ctrl(KeyCode::Char('h')),
        );
        self.register_command_and_binding(
            Command::new("editor.select_all", "Select All", "Select all text", "Editor", CommandAction::SelectAll),
            KeyBinding::ctrl(KeyCode::Char('a')),
        );
        self.register_command_and_binding(
            Command::new("editor.copy", "Copy", "Copy selected text", "Editor", CommandAction::Copy),
            KeyBinding::ctrl(KeyCode::Char('c')),
        );
        self.register_command_and_binding(
            Command::new("editor.cut", "Cut", "Cut selected text", "Editor", CommandAction::Cut),
            KeyBinding::ctrl(KeyCode::Char('x')),
        );
        self.register_command_and_binding(
            Command::new("editor.paste", "Paste", "Paste from clipboard", "Editor", CommandAction::Paste),
            KeyBinding::ctrl(KeyCode::Char('v')),
        );

        // View operations
        self.register_command_and_binding(
            Command::new("view.command_palette", "Show Command Palette", "Show the command palette", "View", CommandAction::ShowCommandPalette),
            KeyBinding::ctrl_shift(KeyCode::Char('p')),
        );
        self.register_command_and_binding(
            Command::new("view.toggle_terminal", "Toggle Terminal", "Show/hide integrated terminal", "View", CommandAction::ToggleTerminal),
            KeyBinding::ctrl(KeyCode::Char('`')),
        );

        // Function key bindings for GUI controls
        self.register_command_and_binding(
            Command::new("toggle_gui_overlay", "Toggle GUI Overlay", "Toggle the GUI overlay", "GUI", CommandAction::Custom("toggle_gui_overlay".to_string())),
            KeyBinding::key(KeyCode::F(1)),
        );
        self.register_command_and_binding(
            Command::new("show_tab_management", "Tab Management", "Show tab management menu", "GUI", CommandAction::Custom("show_tab_management".to_string())),
            KeyBinding::key(KeyCode::F(2)),
        );
        self.register_command_and_binding(
            Command::new("toggle_pane_creation_mode", "Pane Creation Mode", "Enter pane creation mode", "GUI", CommandAction::Custom("toggle_pane_creation_mode".to_string())),
            KeyBinding::key(KeyCode::F(3)),
        );
        self.register_command_and_binding(
            Command::new("show_window_list", "Window List", "Show window list", "GUI", CommandAction::Custom("show_window_list".to_string())),
            KeyBinding::key(KeyCode::F(4)),
        );
        self.register_command_and_binding(
            Command::new("show_command_palette", "Show Command Palette", "Show the command palette", "GUI", CommandAction::Custom("show_command_palette".to_string())),
            KeyBinding::ctrl_shift(KeyCode::Char('p')),
        );
    }

    /// Register a command and its keybinding
    pub fn register_command_and_binding(&mut self, command: Command, keybinding: KeyBinding) {
        let command_id = command.id.clone();
        let category = command.category.clone();

        // Add to commands map
        self.commands.insert(command_id.clone(), command);

        // Add keybinding
        self.bindings.insert(keybinding, command_id.clone());

        // Add to category
        self.categories.entry(category).or_insert_with(Vec::new).push(command_id);
    }

    /// Add or update a keybinding for a command
    pub fn bind_key(&mut self, command_id: &str, keybinding: KeyBinding) -> Result<()> {
        if !self.commands.contains_key(command_id) {
            return Err(anyhow::anyhow!("Command '{}' not found", command_id));
        }

        self.bindings.insert(keybinding, command_id.to_string());
        Ok(())
    }

    /// Remove a keybinding
    pub fn unbind_key(&mut self, keybinding: &KeyBinding) {
        self.bindings.remove(keybinding);
    }

    /// Handle a key event and execute the bound command if any
    pub fn handle_key_event(&mut self, event: KeyEvent, window_manager: &mut WindowManager) -> Result<bool> {
        for (binding, command_id) in &self.bindings {
            if binding.matches(&event) {
                if let Some(command) = self.commands.get(command_id) {
                    log::info!("Executing command: {} ({})", command.name, binding);
                    command.execute(window_manager)?;
                    self.last_executed_command = Some(command_id.clone());
                    return Ok(true); // Command was handled
                }
            }
        }
        Ok(false) // No command matched
    }

    /// Get all commands for the command palette
    pub fn get_all_commands(&self) -> Vec<&Command> {
        self.commands.values().collect()
    }

    /// Get the last executed command ID
    pub fn get_last_executed_command(&self) -> Option<&String> {
        self.last_executed_command.as_ref()
    }

    /// Get commands by category
    pub fn get_commands_by_category(&self, category: &str) -> Vec<&Command> {
        if let Some(command_ids) = self.categories.get(category) {
            command_ids.iter()
                .filter_map(|id| self.commands.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all categories
    pub fn get_categories(&self) -> Vec<&String> {
        self.categories.keys().collect()
    }

    /// Search commands by name or description
    pub fn search_commands(&self, query: &str) -> Vec<&Command> {
        let query_lower = query.to_lowercase();
        self.commands.values()
            .filter(|cmd| {
                cmd.name.to_lowercase().contains(&query_lower) ||
                cmd.description.to_lowercase().contains(&query_lower) ||
                cmd.category.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Get the keybinding for a command
    pub fn get_keybinding(&self, command_id: &str) -> Option<&KeyBinding> {
        self.bindings.iter()
            .find(|(_, id)| id.as_str() == command_id)
            .map(|(binding, _)| binding)
    }

    /// Get the command for a keybinding
    pub fn get_command_for_binding(&self, binding: &KeyBinding) -> Option<&Command> {
        self.bindings.get(binding)
            .and_then(|id| self.commands.get(id))
    }

    /// Load keybindings from JSON configuration
    pub fn load_from_config(&mut self, _config: &str) -> Result<()> {
        // TODO: Implement configuration loading
        Ok(())
    }

    /// Save keybindings to JSON configuration
    pub fn save_to_config(&self) -> Result<String> {
        // TODO: Implement configuration saving
        Ok("{}".to_string())
    }
}

impl Default for KeybindingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keybinding_parsing() {
        assert_eq!(
            KeyBinding::from_string("Ctrl+P").unwrap(),
            KeyBinding::ctrl(KeyCode::Char('p'))
        );

        assert_eq!(
            KeyBinding::from_string("Ctrl+Shift+P").unwrap(),
            KeyBinding::ctrl_shift(KeyCode::Char('p'))
        );

        assert_eq!(
            KeyBinding::from_string("F1").unwrap(),
            KeyBinding::key(KeyCode::F(1))
        );

        assert_eq!(
            KeyBinding::from_string("Enter").unwrap(),
            KeyBinding::key(KeyCode::Enter)
        );
    }

    #[test]
    fn test_keybinding_display() {
        assert_eq!(
            KeyBinding::ctrl(KeyCode::Char('p')).to_string(),
            "Ctrl+P"
        );

        assert_eq!(
            KeyBinding::ctrl_shift(KeyCode::Char('p')).to_string(),
            "Ctrl+Shift+P"
        );

        assert_eq!(
            KeyBinding::key(KeyCode::F(1)).to_string(),
            "F1"
        );
    }

    #[test]
    fn test_keybinding_matching() {
        let binding = KeyBinding::ctrl(KeyCode::Char('p'));

        let matching_event = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL);
        assert!(binding.matches(&matching_event));

        let non_matching_event = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::SHIFT);
        assert!(!binding.matches(&non_matching_event));
    }
}