use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::window_manager::WindowManager;

/// Command execution context - provides access to all IDE systems
pub struct CommandContext<'a> {
    pub window_manager: &'a mut WindowManager,
    // Future: Add more context like file system, git, LSP, etc.
    // pub file_system: &'a mut FileSystem,
    // pub lsp_client: &'a mut LspClient,
    // pub git: &'a mut GitClient,
}

/// Command handler trait - allows for extensible command system
pub trait CommandHandler: Send + Sync {
    fn execute(&self, context: CommandContext) -> Result<CommandResult>;
    fn can_handle(&self, command_id: &str) -> bool;
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
        Self { success: true, message: None, should_close_palette: true }
    }

    pub fn success_with_message(message: String) -> Self {
        Self { success: true, message: Some(message), should_close_palette: true }
    }

    pub fn error(message: String) -> Self {
        Self { success: false, message: Some(message), should_close_palette: false }
    }
}

/// Command definition with metadata
#[derive(Debug, Clone)]
pub struct Command {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: CommandCategory,
    pub keybinding: Option<String>,
    pub tags: Vec<String>,
    pub enabled: bool,
}

impl Command {
    pub fn new(id: &str, name: &str, description: &str, category: CommandCategory) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            category,
            keybinding: None,
            tags: Vec::new(),
            enabled: true,
        }
    }

    pub fn with_keybinding(mut self, keybinding: &str) -> Self {
        self.keybinding = Some(keybinding.to_string());
        self
    }

    pub fn with_tags(mut self, tags: Vec<&str>) -> Self {
        self.tags = tags.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

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
    Git,
    LSP,
    Extension,
    Help,
}

impl CommandCategory {
    fn icon(&self) -> &'static str {
        match self {
            CommandCategory::File => "📁",
            CommandCategory::Edit => "✏️",
            CommandCategory::View => "👁️",
            CommandCategory::Pane => "▦",
            CommandCategory::Tab => "📑",
            CommandCategory::Window => "🪟",
            CommandCategory::Session => "🔗",
            CommandCategory::Debug => "🐛",
            CommandCategory::Terminal => "💻",
            CommandCategory::Git => "🌿",
            CommandCategory::LSP => "🧠",
            CommandCategory::Extension => "🧩",
            CommandCategory::Help => "❓",
        }
    }

    fn color(&self) -> Color {
        match self {
            CommandCategory::File => Color::Blue,
            CommandCategory::Edit => Color::Green,
            CommandCategory::View => Color::Cyan,
            CommandCategory::Pane => Color::Yellow,
            CommandCategory::Tab => Color::Magenta,
            CommandCategory::Window => Color::LightBlue,
            CommandCategory::Session => Color::LightGreen,
            CommandCategory::Debug => Color::Red,
            CommandCategory::Terminal => Color::Gray,
            CommandCategory::Git => Color::LightMagenta,
            CommandCategory::LSP => Color::LightYellow,
            CommandCategory::Extension => Color::LightRed,
            CommandCategory::Help => Color::LightCyan,
        }
    }
}

/// Main command system - acts as middleware for all IDE operations
pub struct CommandSystem {
    commands: HashMap<String, Command>,
    handlers: Vec<Box<dyn CommandHandler>>,
    command_history: Vec<String>,
    favorites: Vec<String>,
}

impl CommandSystem {
    pub fn new() -> Self {
        let mut system = Self {
            commands: HashMap::new(),
            handlers: Vec::new(),
            command_history: Vec::new(),
            favorites: Vec::new(),
        };

        system.register_core_handlers();
        system.register_core_commands();
        system
    }

    /// Register a command handler
    pub fn register_handler(&mut self, handler: Box<dyn CommandHandler>) {
        self.handlers.push(handler);
    }

    /// Register a command
    pub fn register_command(&mut self, command: Command) {
        self.commands.insert(command.id.clone(), command);
    }

    /// Batch register commands
    pub fn register_commands(&mut self, commands: Vec<Command>) {
        for command in commands {
            self.register_command(command);
        }
    }

    /// Execute a command by ID
    pub fn execute_command(&mut self, command_id: &str, window_manager: &mut WindowManager) -> Result<CommandResult> {
        // Add to history
        self.command_history.push(command_id.to_string());
        if self.command_history.len() > 100 {
            self.command_history.remove(0);
        }

        // Find handler
        for handler in &self.handlers {
            if handler.can_handle(command_id) {
                let context = CommandContext { window_manager };
                return handler.execute(context);
            }
        }

        Ok(CommandResult::error(format!("No handler found for command: {}", command_id)))
    }

    /// Get all commands
    pub fn get_commands(&self) -> Vec<&Command> {
        self.commands.values().filter(|cmd| cmd.enabled).collect()
    }

    /// Get command by ID
    pub fn get_command(&self, id: &str) -> Option<&Command> {
        self.commands.get(id)
    }

    /// Search commands with fuzzy matching
    pub fn search_commands(&self, query: &str) -> Vec<&Command> {
        if query.is_empty() {
            return self.get_commands();
        }

        let query_lower = query.to_lowercase();
        let mut results: Vec<(&Command, u32)> = Vec::new();

        for command in self.get_commands() {
            let mut score = 0u32;

            // Exact name match (highest priority)
            if command.name.to_lowercase() == query_lower {
                score += 1000;
            }
            // Name starts with query
            else if command.name.to_lowercase().starts_with(&query_lower) {
                score += 500;
            }
            // Name contains query
            else if command.name.to_lowercase().contains(&query_lower) {
                score += 200;
            }

            // Description contains query
            if command.description.to_lowercase().contains(&query_lower) {
                score += 50;
            }

            // Tag matches
            for tag in &command.tags {
                if tag.to_lowercase().contains(&query_lower) {
                    score += 30;
                }
            }

            // ID matches (for advanced users)
            if command.id.to_lowercase().contains(&query_lower) {
                score += 20;
            }

            // Category matches
            if format!("{:?}", command.category).to_lowercase().contains(&query_lower) {
                score += 10;
            }

            if score > 0 {
                results.push((command, score));
            }
        }

        // Sort by score (highest first), then by name
        results.sort_by(|a, b| {
            b.1.cmp(&a.1).then_with(|| a.0.name.cmp(&b.0.name))
        });

        results.into_iter().map(|(cmd, _)| cmd).collect()
    }

    fn register_core_commands(&mut self) {
        let commands = vec![
            // File operations
            Command::new("file.new", "New File", "Create a new file", CommandCategory::File)
                .with_keybinding("Ctrl+N")
                .with_tags(vec!["create", "new"]),

            Command::new("file.open", "Open File", "Open a file browser", CommandCategory::File)
                .with_keybinding("Ctrl+O")
                .with_tags(vec!["open", "browse"]),

            Command::new("file.save", "Save File", "Save the current file", CommandCategory::File)
                .with_keybinding("Ctrl+S")
                .with_tags(vec!["save", "write"]),

            // Pane operations
            Command::new("pane.split_right", "Split Pane Right", "Split the current pane to the right", CommandCategory::Pane)
                .with_keybinding("Ctrl+Shift+Right")
                .with_tags(vec!["split", "right", "horizontal"]),

            Command::new("pane.split_down", "Split Pane Down", "Split the current pane downward", CommandCategory::Pane)
                .with_keybinding("Ctrl+Shift+Down")
                .with_tags(vec!["split", "down", "vertical"]),

            Command::new("pane.close", "Close Pane", "Close the current pane", CommandCategory::Pane)
                .with_keybinding("Ctrl+W")
                .with_tags(vec!["close", "kill"]),

            Command::new("pane.focus_left", "Focus Left Pane", "Move focus to the pane on the left", CommandCategory::Pane)
                .with_keybinding("Ctrl+Left")
                .with_tags(vec!["focus", "left", "navigate"]),

            Command::new("pane.focus_right", "Focus Right Pane", "Move focus to the pane on the right", CommandCategory::Pane)
                .with_keybinding("Ctrl+Right")
                .with_tags(vec!["focus", "right", "navigate"]),

            Command::new("pane.focus_up", "Focus Up Pane", "Move focus to the pane above", CommandCategory::Pane)
                .with_keybinding("Ctrl+Up")
                .with_tags(vec!["focus", "up", "navigate"]),

            Command::new("pane.focus_down", "Focus Down Pane", "Move focus to the pane below", CommandCategory::Pane)
                .with_keybinding("Ctrl+Down")
                .with_tags(vec!["focus", "down", "navigate"]),

            // Tab operations
            Command::new("tab.new", "New Tab", "Create a new tab", CommandCategory::Tab)
                .with_keybinding("Ctrl+T")
                .with_tags(vec!["new", "create"]),

            Command::new("tab.close", "Close Tab", "Close the current tab", CommandCategory::Tab)
                .with_keybinding("Ctrl+Shift+W")
                .with_tags(vec!["close", "kill"]),

            Command::new("tab.next", "Next Tab", "Switch to the next tab", CommandCategory::Tab)
                .with_keybinding("Ctrl+Tab")
                .with_tags(vec!["next", "switch"]),

            Command::new("tab.previous", "Previous Tab", "Switch to the previous tab", CommandCategory::Tab)
                .with_keybinding("Ctrl+Shift+Tab")
                .with_tags(vec!["previous", "switch"]),

            // View operations
            Command::new("view.toggle_overlay", "Toggle GUI Overlay", "Show/hide the GUI overlay", CommandCategory::View)
                .with_keybinding("F1")
                .with_tags(vec!["toggle", "gui", "overlay"]),

            Command::new("view.command_palette", "Show Command Palette", "Open the command palette", CommandCategory::View)
                .with_keybinding("Ctrl+Shift+P")
                .with_tags(vec!["command", "palette", "search"]),

            // Session operations
            Command::new("session.new", "New Session", "Create a new Zellij session", CommandCategory::Session)
                .with_tags(vec!["new", "create", "session"]),

            Command::new("session.attach", "Attach to Session", "Attach to an existing session", CommandCategory::Session)
                .with_tags(vec!["attach", "connect", "session"]),

            Command::new("session.detach", "Detach Session", "Detach from current session", CommandCategory::Session)
                .with_tags(vec!["detach", "disconnect"]),

            // Terminal operations
            Command::new("terminal.new", "New Terminal", "Create a new terminal pane", CommandCategory::Terminal)
                .with_keybinding("Ctrl+`")
                .with_tags(vec!["new", "terminal", "shell"]),

            Command::new("terminal.clear", "Clear Terminal", "Clear the current terminal", CommandCategory::Terminal)
                .with_keybinding("Ctrl+L")
                .with_tags(vec!["clear", "clean"]),

            // Future Git operations (placeholder)
            Command::new("git.status", "Git Status", "Show git status", CommandCategory::Git)
                .with_tags(vec!["status", "git"])
                .disabled(),

            Command::new("git.commit", "Git Commit", "Make a git commit", CommandCategory::Git)
                .with_tags(vec!["commit", "git"])
                .disabled(),
        ];

        self.register_commands(commands);
    }

    fn register_core_handlers(&mut self) {
        self.register_handler(Box::new(CoreCommandHandler));
        // Future: Add more specialized handlers
        // self.register_handler(Box::new(FileSystemHandler));
        // self.register_handler(Box::new(GitHandler));
        // self.register_handler(Box::new(LSPHandler));
    }
}

/// Core command handler for basic IDE operations
struct CoreCommandHandler;

impl CommandHandler for CoreCommandHandler {
    fn can_handle(&self, command_id: &str) -> bool {
        matches!(command_id,
            "pane.split_right" | "pane.split_down" | "pane.close" |
            "pane.focus_left" | "pane.focus_right" | "pane.focus_up" | "pane.focus_down" |
            "tab.new" | "tab.close" | "tab.next" | "tab.previous" |
            "view.toggle_overlay" | "view.command_palette" |
            "session.detach" | "terminal.new" | "terminal.clear"
        )
    }

    fn execute(&self, context: CommandContext) -> Result<CommandResult> {
        match context.window_manager {
            wm => {
                // Note: These are placeholder implementations
                // They would call actual WindowManager methods
                match "placeholder" {
                    "pane.split_right" => {
                        // wm.split_current_pane(Direction::Right)?;
                        Ok(CommandResult::success_with_message("Pane split right".to_string()))
                    },
                    "tab.new" => {
                        wm.create_tab(None)?;
                        Ok(CommandResult::success_with_message("New tab created".to_string()))
                    },
                    _ => Ok(CommandResult::success())
                }
            }
        }
    }
}

/// Command palette UI component
pub struct CommandPalette {
    pub visible: bool,
    query: String,
    filtered_commands: Vec<Command>,
    selected_index: usize,
    list_state: ListState,
    command_system: Arc<Mutex<CommandSystem>>,
    last_message: Option<String>,
}

impl CommandPalette {
    pub fn new(command_system: Arc<Mutex<CommandSystem>>) -> Self {
        let mut palette = Self {
            visible: false,
            query: String::new(),
            filtered_commands: Vec::new(),
            selected_index: 0,
            list_state: ListState::default(),
            command_system,
            last_message: None,
        };

        palette.update_filtered_commands();
        palette
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.query.clear();
        self.selected_index = 0;
        self.last_message = None;
        self.update_filtered_commands();
        self.update_list_state();
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.query.clear();
        self.last_message = None;
    }

    pub fn handle_key(&mut self, key: KeyEvent, window_manager: &mut WindowManager) -> Result<bool> {
        if !self.visible {
            return Ok(true);
        }

        match key.code {
            KeyCode::Esc => {
                self.hide();
            },
            KeyCode::Enter => {
                if let Some(command) = self.filtered_commands.get(self.selected_index) {
                    let command_id = command.id.clone();
                    let result = if let Ok(mut system) = self.command_system.lock() {
                        system.execute_command(&command_id, window_manager)
                    } else {
                        Ok(CommandResult::error("Failed to lock command system".to_string()))
                    };

                    match result {
                        Ok(cmd_result) => {
                            self.last_message = cmd_result.message;
                            if cmd_result.should_close_palette {
                                self.hide();
                            }
                        },
                        Err(e) => {
                            self.last_message = Some(format!("Error: {}", e));
                        }
                    }
                }
            },
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    self.update_list_state();
                }
            },
            KeyCode::Down => {
                if self.selected_index + 1 < self.filtered_commands.len() {
                    self.selected_index += 1;
                    self.update_list_state();
                }
            },
            KeyCode::Backspace => {
                self.query.pop();
                self.selected_index = 0;
                self.update_filtered_commands();
                self.update_list_state();
            },
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.query.push(c);
                self.selected_index = 0;
                self.update_filtered_commands();
                self.update_list_state();
            },
            _ => {},
        }

        Ok(true)
    }

    fn update_filtered_commands(&mut self) {
        if let Ok(system) = self.command_system.lock() {
            self.filtered_commands = system.search_commands(&self.query).into_iter().cloned().collect();
        }
    }

    fn update_list_state(&mut self) {
        self.list_state.select(Some(self.selected_index));
    }

    pub fn render(&mut self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        let size = frame.area();
        let popup_area = Rect {
            x: size.width / 6,
            y: size.height / 6,
            width: (size.width * 2) / 3,
            height: (size.height * 2) / 3,
        };

        frame.render_widget(Clear, popup_area);

        let main_block = Block::default()
            .title("🎯 Command Palette")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        frame.render_widget(main_block, popup_area);

        let inner = popup_area.inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search box
                Constraint::Min(1),    // Command list
                Constraint::Length(if self.last_message.is_some() { 3 } else { 0 }), // Status message
            ].as_ref())
            .split(inner);

        // Search input
        let search_text = if self.query.is_empty() {
            "Type to search commands..."
        } else {
            &self.query
        };

        let search_paragraph = Paragraph::new(search_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("🔍 Search")
                    .style(Style::default().fg(Color::Yellow))
            )
            .style(if self.query.is_empty() {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            });

        frame.render_widget(search_paragraph, chunks[0]);

        // Command list
        if self.filtered_commands.is_empty() {
            let no_results = Paragraph::new("No commands found")
                .block(Block::default().borders(Borders::ALL).title("Commands"))
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(no_results, chunks[1]);
        } else {
            let items: Vec<ListItem> = self.filtered_commands
                .iter()
                .map(|cmd| {
                    let keybinding = cmd.keybinding
                        .as_ref()
                        .map(|kb| format!(" ({})", kb))
                        .unwrap_or_default();

                    ListItem::new(Line::from(vec![
                        Span::styled(cmd.category.icon(), Style::default().fg(cmd.category.color())),
                        Span::raw(" "),
                        Span::styled(&cmd.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        Span::styled(keybinding, Style::default().fg(Color::DarkGray)),
                        Span::raw("\n  "),
                        Span::styled(&cmd.description, Style::default().fg(Color::Gray)),
                    ]))
                })
                .collect();

            let total_commands = if let Ok(system) = self.command_system.lock() {
                system.get_commands().len()
            } else {
                0
            };

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("📋 Commands ({}/{})", self.filtered_commands.len(), total_commands))
                )
                .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
                .highlight_symbol("→ ");

            frame.render_stateful_widget(list, chunks[1], &mut self.list_state);
        }

        // Status message
        if let Some(message) = &self.last_message {
            let status_paragraph = Paragraph::new(message.as_str())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Status")
                        .style(Style::default().fg(Color::Green))
                )
                .style(Style::default().fg(Color::White));

            frame.render_widget(status_paragraph, chunks[2]);
        }
    }
}

/// Global command system instance
pub fn create_command_system() -> Arc<Mutex<CommandSystem>> {
    Arc::new(Mutex::new(CommandSystem::new()))
}