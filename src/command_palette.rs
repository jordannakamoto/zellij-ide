use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::window_manager::WindowManager;

/// Command execution context - provides access to all IDE systems
pub struct CommandContext<'a> {
    pub window_manager: &'a mut WindowManager,
}

/// Result of command execution
#[derive(Debug)]
pub struct CommandResult {
    pub success: bool,
    pub message: Option<String>,
    pub should_close_palette: bool,
}

impl CommandResult {
    pub fn success() -> Self {
        Self {
            success: true,
            message: None,
            should_close_palette: true,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            message: Some(message),
            should_close_palette: false,
        }
    }
}

/// Trait for handling command execution
pub trait CommandHandler: Send + Sync {
    fn execute(&self, context: CommandContext) -> Result<CommandResult>;
    fn can_handle(&self, command_id: &str) -> bool;
}

/// Categories for organizing commands
#[derive(Debug, Clone, PartialEq)]
pub enum CommandCategory {
    File,
    Edit,
    View,
    Pane,
    Tab,
    Window,
    Session,
    Debug,
    Terminal,
    Editor,
    Git,
    LSP,
    Extension,
    Help,
}

/// A command that can be executed
#[derive(Debug, Clone)]
pub struct Command {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: CommandCategory,
    pub keybinding: Option<String>,
}

impl Command {
    pub fn new(id: &str, name: &str, description: &str, category: CommandCategory) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            category,
            keybinding: None,
        }
    }

    pub fn with_keybinding(mut self, keybinding: &str) -> Self {
        self.keybinding = Some(keybinding.to_string());
        self
    }
}

/// System for managing and executing commands
pub struct CommandSystem {
    commands: Vec<Command>,
    handlers: Vec<Box<dyn CommandHandler>>,
    command_history: Vec<String>,
    favorites: Vec<String>,
}

impl CommandSystem {
    pub fn new() -> Self {
        let mut system = Self {
            commands: Vec::new(),
            handlers: Vec::new(),
            command_history: Vec::new(),
            favorites: Vec::new(),
        };

        // Register default commands
        system.register_default_commands();
        system
    }

    fn register_default_commands(&mut self) {
        use CommandCategory::*;

        let commands = vec![
            Command::new("file.new", "New File", "Create a new file", File),
            Command::new("file.open", "Open File", "Open an existing file", File),
            Command::new("file.save", "Save File", "Save the current file", File),
            Command::new("edit.cut", "Cut", "Cut selected text", Edit),
            Command::new("edit.copy", "Copy", "Copy selected text", Edit),
            Command::new("edit.paste", "Paste", "Paste from clipboard", Edit),
            Command::new("view.command_palette", "Show Command Palette", "Show the command palette", View)
                .with_keybinding("Cmd+P"),
            Command::new("pane.split_horizontal", "Split Horizontally", "Split the current pane horizontally", Pane),
            Command::new("pane.split_vertical", "Split Vertically", "Split the current pane vertically", Pane),
            Command::new("tab.new", "New Tab", "Create a new tab", Tab),
            Command::new("tab.close", "Close Tab", "Close the current tab", Tab),
            Command::new("terminal.new", "New Terminal", "Open a new terminal", Terminal),
        ];

        self.commands.extend(commands);
    }

    pub fn execute_command(&mut self, command_id: &str, _window_manager: &mut WindowManager) -> Result<CommandResult> {
        // Find the command
        if let Some(command) = self.commands.iter().find(|c| c.id == command_id) {
            // For now, just log the command execution
            log::info!("Executing command: {} ({})", command.name, command.id);

            // Add to history
            self.command_history.push(command_id.to_string());

            // Return success for now
            Ok(CommandResult::success())
        } else {
            Ok(CommandResult::error(format!("Command not found: {}", command_id)))
        }
    }

    pub fn get_commands(&self) -> &[Command] {
        &self.commands
    }

    pub fn get_command(&self, id: &str) -> Option<&Command> {
        self.commands.iter().find(|c| c.id == id)
    }

    pub fn search_commands(&self, query: &str) -> Vec<&Command> {
        self.commands
            .iter()
            .filter(|c| {
                c.name.to_lowercase().contains(&query.to_lowercase())
                    || c.description.to_lowercase().contains(&query.to_lowercase())
            })
            .collect()
    }
}

/// The command palette widget
pub struct CommandPalette {
    command_system: CommandSystem,
    visible: bool,
    query: String,
    filtered_commands: Vec<Command>,
    selected_index: usize,
    last_message: Option<String>,
}

impl CommandPalette {
    pub fn new(command_system: CommandSystem) -> Self {
        let filtered_commands = command_system.get_commands().to_vec();
        Self {
            command_system,
            visible: false,
            query: String::new(),
            filtered_commands,
            selected_index: 0,
            last_message: None,
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.query.clear();
        self.selected_index = 0;
        self.update_filtered_commands();
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.query.clear();
        self.last_message = None;
    }

    pub fn is_active(&self) -> bool {
        self.visible
    }

    pub fn handle_key_event(&mut self, _key_event: KeyEvent) -> Result<()> {
        // TODO: Implement key handling for search and navigation
        Ok(())
    }

    pub fn set_commands(&mut self, _commands: Vec<Command>) {
        // TODO: Update commands from features
        self.update_filtered_commands();
    }

    pub fn handle_key(&mut self, key: KeyEvent, window_manager: &mut WindowManager) -> Result<bool> {
        if !self.visible {
            return Ok(false);
        }

        match key.code {
            KeyCode::Esc => {
                self.hide();
                return Ok(true);
            }
            KeyCode::Enter => {
                if !self.filtered_commands.is_empty() && self.selected_index < self.filtered_commands.len() {
                    let command = &self.filtered_commands[self.selected_index];
                    let result = self.command_system.execute_command(&command.id, window_manager)?;

                    if let Some(message) = result.message {
                        self.last_message = Some(message);
                    }

                    if result.should_close_palette {
                        self.hide();
                    }
                }
                return Ok(true);
            }
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                return Ok(true);
            }
            KeyCode::Down => {
                if self.selected_index < self.filtered_commands.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
                return Ok(true);
            }
            KeyCode::Char(c) => {
                self.query.push(c);
                self.update_filtered_commands();
                self.selected_index = 0;
                return Ok(true);
            }
            KeyCode::Backspace => {
                self.query.pop();
                self.update_filtered_commands();
                self.selected_index = 0;
                return Ok(true);
            }
            _ => {}
        }

        Ok(false)
    }

    fn update_filtered_commands(&mut self) {
        if self.query.is_empty() {
            self.filtered_commands = self.command_system.get_commands().to_vec();
        } else {
            self.filtered_commands = self.command_system
                .search_commands(&self.query)
                .into_iter()
                .cloned()
                .collect();
        }
    }

    pub fn render(&mut self) -> String {
        if !self.visible {
            return String::new();
        }

        let mut content = String::new();
        content.push_str("Command Palette\n");
        content.push_str("═══════════════\n");
        content.push_str(&format!("Search: {}\n\n", self.query));

        for (i, command) in self.filtered_commands.iter().enumerate() {
            let marker = if i == self.selected_index { ">" } else { " " };
            let keybinding = command.keybinding.as_deref().unwrap_or("");
            content.push_str(&format!("{} {} - {} {}\n", marker, command.name, command.description, keybinding));
        }

        if let Some(message) = &self.last_message {
            content.push_str(&format!("\nMessage: {}", message));
        }

        content
    }
}

/// Create a default command system
pub fn create_command_system() -> CommandSystem {
    CommandSystem::new()
}