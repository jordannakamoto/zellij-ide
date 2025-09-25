/*!
# Command Palette System Documentation

The command palette provides a unified, extensible way to execute commands across the IDE.
It supports global IDE commands, actor-specific commands, and configurable command groups.

## Architecture

### Core Components

- `Command`: Individual executable action
- `CommandProvider`: Source of commands (global, actor-specific, or custom)
- `CommandPalette`: Headless command execution engine
- `CommandPaletteWidget`: Optional UI widget for command palette
- `CommandGroup`: Logical grouping of related commands

### Command Structure

Commands are lightweight, serializable actions:

```rust
pub struct Command {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub shortcut: Option<String>,
    pub enabled: bool,
    pub action: CommandAction,
    pub group: Option<String>,
}
```

### Command Providers

The system supports multiple command providers:

1. **Global Provider**: IDE-wide commands (file operations, view switching, etc.)
2. **Actor Provider**: Actor-specific commands when actor has focus
3. **Custom Provider**: User-defined command sources

### Command Groups

Commands can be organized into logical groups for better UX:

```rust
pub struct CommandGroup {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: i32,
    pub enabled: bool,
    pub activation_context: GroupActivation,
}
```

### Group Activation Context

Groups can be activated based on different contexts:

- `Always`: Always available
- `ActorType(String)`: Only when specific actor type has focus
- `ActorGroup(Vec<String>)`: When any actor in group has focus
- `ViewSystem(String)`: Only with specific view system
- `Custom(Box<dyn Fn(&CommandContext) -> bool>)`: Custom activation logic

## Usage Examples

### Creating Global Commands

```rust
let global_provider = GlobalCommandProvider::new();
global_provider.add_command(Command {
    id: "file.new".to_string(),
    title: "New File".to_string(),
    description: Some("Create a new file".to_string()),
    category: "File".to_string(),
    shortcut: Some("Ctrl+N".to_string()),
    enabled: true,
    action: CommandAction::Global(GlobalAction::NewFile),
    group: Some("file_operations".to_string()),
});
```

### Creating Actor-Specific Commands

```rust
impl CommandProvider for CodeEditorActor {
    fn get_commands(&self, _ctx: &CommandContext) -> Vec<Command> {
        vec![
            Command {
                id: "editor.format".to_string(),
                title: "Format Code".to_string(),
                category: "Editor".to_string(),
                action: CommandAction::Actor(self.id(), "format".to_string()),
                ..Default::default()
            }
        ]
    }
}
```

### Headless Execution

```rust
// Execute command by ID
palette.execute_command("file.new", &context)?;

// Search and execute
let commands = palette.search_commands("format", &context);
if let Some(cmd) = commands.first() {
    palette.execute_command(&cmd.id, &context)?;
}
```

### Group Configuration

```rust
let group = CommandGroup {
    id: "editor_tools".to_string(),
    title: "Editor Tools".to_string(),
    activation_context: GroupActivation::ActorGroup(vec![
        "CodeEditorActor".to_string(),
        "MarkdownEditorActor".to_string(),
    ]),
    ..Default::default()
};
```

## Adding New Command Types

1. **Define the action** in `CommandAction` enum
2. **Implement execution** in the command palette
3. **Create provider** or add to existing provider
4. **Register with palette**

## Integration with Widgets

The command palette can optionally show a UI widget, but is designed to work
headlessly for programmatic access, keybindings, and API integration.

*/

use crate::actor::ActorManager;
use crate::view_system::ViewContainer;
use crate::widgets::{Widget, WidgetContext, WidgetPosition};
use egui::{self, Vec2};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use anyhow::Result;

/// Represents a single executable command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub shortcut: Option<String>,
    pub enabled: bool,
    #[serde(skip)]
    pub action: CommandAction,
    pub group: Option<String>,
}

impl Default for Command {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            description: None,
            category: "General".to_string(),
            shortcut: None,
            enabled: true,
            action: CommandAction::NoOp,
            group: None,
        }
    }
}

/// Different types of command actions
#[derive(Clone)]
pub enum CommandAction {
    /// No operation (placeholder)
    NoOp,
    /// Global IDE action
    Global(GlobalAction),
    /// Actor-specific action
    Actor(Uuid, String),
    /// Custom action with closure
    Custom(Arc<dyn Fn(&CommandContext) -> Result<()> + Send + Sync>),
}

impl std::fmt::Debug for CommandAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandAction::NoOp => write!(f, "NoOp"),
            CommandAction::Global(action) => write!(f, "Global({:?})", action),
            CommandAction::Actor(id, action) => write!(f, "Actor({}, {})", id, action),
            CommandAction::Custom(_) => write!(f, "Custom(<closure>)"),
        }
    }
}

impl Default for CommandAction {
    fn default() -> Self {
        CommandAction::NoOp
    }
}

/// Global IDE actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GlobalAction {
    NewFile,
    NewTab,
    CloseTab,
    ToggleWidgets,
    SwitchViewSystem(String),
    ShowCommandPalette,
    ExitApplication,
    ResetTransform,
}

/// Context provided to commands during execution
pub struct CommandContext<'a> {
    pub focused_actor: Option<Uuid>,
    pub view_container: &'a ViewContainer,
    pub actor_manager: &'a ActorManager,
    pub current_view_system: String,
}

/// Command group for logical organization
#[derive(Debug, Clone)]
pub struct CommandGroup {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: i32,
    pub enabled: bool,
    pub activation_context: GroupActivation,
}

/// Context for when command groups should be active
#[derive(Debug, Clone)]
pub enum GroupActivation {
    /// Always available
    Always,
    /// Only when specific actor type has focus
    ActorType(String),
    /// When any actor in the specified group has focus
    ActorGroup(Vec<String>),
    /// Only with specific view system
    ViewSystem(String),
    /// Custom activation logic
    Custom(String), // ID for custom logic
}

/// Trait for providing commands
pub trait CommandProvider {
    /// Get commands available from this provider
    fn get_commands(&self, ctx: &CommandContext) -> Vec<Command>;

    /// Get the provider ID
    fn provider_id(&self) -> String;

    /// Check if this provider should be active in the current context
    fn is_active(&self, ctx: &CommandContext) -> bool {
        true // Default: always active
    }
}

/// Global command provider for IDE-wide actions
pub struct GlobalCommandProvider {
    commands: Vec<Command>,
}

impl GlobalCommandProvider {
    pub fn new() -> Self {
        let mut provider = Self {
            commands: Vec::new(),
        };

        // Add default global commands
        provider.add_default_commands();
        provider
    }

    fn add_default_commands(&mut self) {
        self.commands.extend([
            Command {
                id: "file.new".to_string(),
                title: "New File".to_string(),
                description: Some("Create a new file".to_string()),
                category: "File".to_string(),
                shortcut: Some("Ctrl+N".to_string()),
                action: CommandAction::Global(GlobalAction::NewFile),
                group: Some("file_operations".to_string()),
                ..Default::default()
            },
            Command {
                id: "tab.new".to_string(),
                title: "New Tab".to_string(),
                description: Some("Create a new tab".to_string()),
                category: "File".to_string(),
                shortcut: Some("Ctrl+T".to_string()),
                action: CommandAction::Global(GlobalAction::NewTab),
                group: Some("file_operations".to_string()),
                ..Default::default()
            },
            Command {
                id: "tab.close".to_string(),
                title: "Close Tab".to_string(),
                description: Some("Close current tab".to_string()),
                category: "File".to_string(),
                shortcut: Some("Ctrl+W".to_string()),
                action: CommandAction::Global(GlobalAction::CloseTab),
                group: Some("file_operations".to_string()),
                ..Default::default()
            },
            Command {
                id: "view.toggle_widgets".to_string(),
                title: "Toggle Widgets".to_string(),
                description: Some("Show/hide all widgets".to_string()),
                category: "View".to_string(),
                action: CommandAction::Global(GlobalAction::ToggleWidgets),
                group: Some("view_operations".to_string()),
                ..Default::default()
            },
            Command {
                id: "view.scene_system".to_string(),
                title: "Switch to Scene System".to_string(),
                description: Some("Use scene-based view system".to_string()),
                category: "View".to_string(),
                action: CommandAction::Global(GlobalAction::SwitchViewSystem("scene".to_string())),
                group: Some("view_operations".to_string()),
                ..Default::default()
            },
            Command {
                id: "view.tiling_system".to_string(),
                title: "Switch to Tiling System".to_string(),
                description: Some("Use tiling-based view system".to_string()),
                category: "View".to_string(),
                action: CommandAction::Global(GlobalAction::SwitchViewSystem("tiling".to_string())),
                group: Some("view_operations".to_string()),
                ..Default::default()
            },
            Command {
                id: "palette.show".to_string(),
                title: "Show Command Palette".to_string(),
                description: Some("Open command palette".to_string()),
                category: "Tools".to_string(),
                shortcut: Some("Ctrl+Shift+P".to_string()),
                action: CommandAction::Global(GlobalAction::ShowCommandPalette),
                group: Some("tools".to_string()),
                ..Default::default()
            },
            Command {
                id: "transform.reset".to_string(),
                title: "Reset Transform".to_string(),
                description: Some("Reset zoom and pan to defaults".to_string()),
                category: "View".to_string(),
                action: CommandAction::Global(GlobalAction::ResetTransform),
                group: Some("transform_operations".to_string()),
                ..Default::default()
            },
        ]);
    }

    pub fn add_command(&mut self, command: Command) {
        self.commands.push(command);
    }
}

impl CommandProvider for GlobalCommandProvider {
    fn get_commands(&self, _ctx: &CommandContext) -> Vec<Command> {
        self.commands.clone()
    }

    fn provider_id(&self) -> String {
        "global".to_string()
    }
}

/// Actor-specific command provider
pub struct ActorCommandProvider {
    actor_commands: HashMap<String, Vec<Command>>, // actor_type -> commands
}

impl ActorCommandProvider {
    pub fn new() -> Self {
        let mut provider = Self {
            actor_commands: HashMap::new(),
        };

        // Add default actor commands
        provider.add_code_editor_commands();
        provider
    }

    fn add_code_editor_commands(&mut self) {
        let commands = vec![
            Command {
                id: "editor.format".to_string(),
                title: "Format Code".to_string(),
                description: Some("Format the current code".to_string()),
                category: "Editor".to_string(),
                shortcut: Some("Alt+Shift+F".to_string()),
                action: CommandAction::Actor(Uuid::nil(), "format".to_string()),
                group: Some("editor_tools".to_string()),
                ..Default::default()
            },
            Command {
                id: "editor.save".to_string(),
                title: "Save File".to_string(),
                description: Some("Save the current file".to_string()),
                category: "Editor".to_string(),
                shortcut: Some("Ctrl+S".to_string()),
                action: CommandAction::Actor(Uuid::nil(), "save".to_string()),
                group: Some("editor_tools".to_string()),
                ..Default::default()
            },
            Command {
                id: "editor.find".to_string(),
                title: "Find in File".to_string(),
                description: Some("Search within the current file".to_string()),
                category: "Editor".to_string(),
                shortcut: Some("Ctrl+F".to_string()),
                action: CommandAction::Actor(Uuid::nil(), "find".to_string()),
                group: Some("editor_tools".to_string()),
                ..Default::default()
            },
        ];

        self.actor_commands.insert("CodeEditorActor".to_string(), commands);
    }

    pub fn add_actor_commands(&mut self, actor_type: String, commands: Vec<Command>) {
        self.actor_commands.insert(actor_type, commands);
    }
}

impl CommandProvider for ActorCommandProvider {
    fn get_commands(&self, ctx: &CommandContext) -> Vec<Command> {
        if let Some(focused_actor) = ctx.focused_actor {
            // Get actor info and dynamically generate commands from API
            let actors_info = ctx.actor_manager.get_actors_info();
            if let Some(actor_info) = actors_info.iter().find(|info| info.id == focused_actor) {
                // Get API methods from the actor
                let api_methods = ctx.actor_manager.get_actor_api_methods(focused_actor);

                // Convert API methods to commands
                return api_methods.into_iter().map(|api_method| Command {
                    id: format!("actor.{}.{}", actor_info.actor_type, api_method.name),
                    title: api_method.description.clone(),
                    description: Some(format!("API: {}", api_method.description)),
                    category: format!("{} ({})", api_method.category, actor_info.actor_type),
                    shortcut: None,
                    enabled: true,
                    action: CommandAction::Actor(focused_actor, api_method.name),
                    group: Some(format!("{}_api", actor_info.actor_type.to_lowercase())),
                }).collect();
            }
        }
        Vec::new()
    }

    fn provider_id(&self) -> String {
        "actor".to_string()
    }

    fn is_active(&self, ctx: &CommandContext) -> bool {
        ctx.focused_actor.is_some()
    }
}

/// Main command palette system
pub struct CommandPalette {
    providers: HashMap<String, Box<dyn CommandProvider>>,
    groups: HashMap<String, CommandGroup>,
    custom_activators: HashMap<String, Box<dyn Fn(&CommandContext) -> bool + Send + Sync>>,
    search_query: String,
    is_visible: bool,
}

impl CommandPalette {
    pub fn new() -> Self {
        let mut palette = Self {
            providers: HashMap::new(),
            groups: HashMap::new(),
            custom_activators: HashMap::new(),
            search_query: String::new(),
            is_visible: false,
        };

        // Register default providers
        palette.register_provider(Box::new(GlobalCommandProvider::new()));
        palette.register_provider(Box::new(ActorCommandProvider::new()));

        // Register default groups
        palette.add_default_groups();

        palette
    }

    fn add_default_groups(&mut self) {
        let groups = vec![
            CommandGroup {
                id: "file_operations".to_string(),
                title: "File Operations".to_string(),
                description: Some("File and tab management".to_string()),
                priority: 100,
                enabled: true,
                activation_context: GroupActivation::Always,
            },
            CommandGroup {
                id: "view_operations".to_string(),
                title: "View Operations".to_string(),
                description: Some("View and layout management".to_string()),
                priority: 90,
                enabled: true,
                activation_context: GroupActivation::Always,
            },
            CommandGroup {
                id: "editor_tools".to_string(),
                title: "Editor Tools".to_string(),
                description: Some("Text editing tools".to_string()),
                priority: 80,
                enabled: true,
                activation_context: GroupActivation::ActorGroup(vec![
                    "CodeEditorActor".to_string(),
                    "MarkdownEditorActor".to_string(),
                ]),
            },
            CommandGroup {
                id: "transform_operations".to_string(),
                title: "Transform Operations".to_string(),
                description: Some("Zoom and pan controls".to_string()),
                priority: 70,
                enabled: true,
                activation_context: GroupActivation::ViewSystem("scene".to_string()),
            },
        ];

        for group in groups {
            self.groups.insert(group.id.clone(), group);
        }
    }

    /// Register a command provider
    pub fn register_provider(&mut self, provider: Box<dyn CommandProvider>) {
        let id = provider.provider_id();
        self.providers.insert(id, provider);
    }

    /// Add a custom group activator
    pub fn add_custom_activator(
        &mut self,
        id: String,
        activator: Box<dyn Fn(&CommandContext) -> bool + Send + Sync>
    ) {
        self.custom_activators.insert(id, activator);
    }

    /// Get all available commands in the current context
    pub fn get_available_commands(&self, ctx: &CommandContext) -> Vec<Command> {
        let mut commands = Vec::new();

        // Collect commands from active providers
        for provider in self.providers.values() {
            if provider.is_active(ctx) {
                commands.extend(provider.get_commands(ctx));
            }
        }

        // Filter by active groups
        commands.into_iter().filter(|cmd| {
            if let Some(group_id) = &cmd.group {
                if let Some(group) = self.groups.get(group_id) {
                    return self.is_group_active(group, ctx);
                }
            }
            true // No group = always available
        }).collect()
    }

    /// Search commands by query
    pub fn search_commands(&self, query: &str, ctx: &CommandContext) -> Vec<Command> {
        let available = self.get_available_commands(ctx);

        if query.is_empty() {
            return available;
        }

        let query_lower = query.to_lowercase();
        available.into_iter().filter(|cmd| {
            cmd.title.to_lowercase().contains(&query_lower) ||
            cmd.category.to_lowercase().contains(&query_lower) ||
            cmd.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&query_lower)) ||
            cmd.id.to_lowercase().contains(&query_lower)
        }).collect()
    }

    /// Execute a command by ID
    pub fn execute_command(&self, command_id: &str, ctx: &CommandContext) -> Result<CommandExecutionResult> {
        let available = self.get_available_commands(ctx);

        if let Some(command) = available.iter().find(|c| c.id == command_id) {
            if !command.enabled {
                return Ok(CommandExecutionResult::Disabled);
            }

            match &command.action {
                CommandAction::NoOp => Ok(CommandExecutionResult::Success),
                CommandAction::Global(action) => {
                    // Global actions would be executed by the IDE state
                    Ok(CommandExecutionResult::GlobalAction(action.clone()))
                }
                CommandAction::Actor(actor_id, action) => {
                    // Actor actions would be forwarded to the specific actor
                    Ok(CommandExecutionResult::ActorAction(*actor_id, action.clone()))
                }
                CommandAction::Custom(func) => {
                    func(ctx)?;
                    Ok(CommandExecutionResult::Success)
                }
            }
        } else {
            Ok(CommandExecutionResult::NotFound)
        }
    }

    /// Check if a command group is active in the current context
    fn is_group_active(&self, group: &CommandGroup, ctx: &CommandContext) -> bool {
        if !group.enabled {
            return false;
        }

        match &group.activation_context {
            GroupActivation::Always => true,
            GroupActivation::ActorType(_actor_type) => {
                // Would need to check focused actor type
                ctx.focused_actor.is_some() // Placeholder
            }
            GroupActivation::ActorGroup(_actor_types) => {
                // Would need to check if focused actor is in the group
                ctx.focused_actor.is_some() // Placeholder
            }
            GroupActivation::ViewSystem(system_name) => {
                ctx.current_view_system == *system_name
            }
            GroupActivation::Custom(activator_id) => {
                if let Some(activator) = self.custom_activators.get(activator_id) {
                    activator(ctx)
                } else {
                    false
                }
            }
        }
    }

    /// Show/hide the command palette
    pub fn set_visible(&mut self, visible: bool) {
        self.is_visible = visible;
        if visible {
            self.search_query.clear();
        }
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn toggle_visibility(&mut self) {
        self.set_visible(!self.is_visible);
    }

    /// Get current search query
    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    /// Set search query
    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }
}

/// Result of command execution
#[derive(Debug, Clone)]
pub enum CommandExecutionResult {
    Success,
    NotFound,
    Disabled,
    GlobalAction(GlobalAction),
    ActorAction(Uuid, String),
    Error(String),
}

/// Widget for displaying command palette UI
pub struct CommandPaletteWidget {
    visible: bool,
    position: WidgetPosition,
    max_results: usize,
    selected_index: usize,
}

impl CommandPaletteWidget {
    pub fn new() -> Self {
        Self {
            visible: false,
            position: WidgetPosition::Center,
            max_results: 10,
            selected_index: 0,
        }
    }
}

impl Widget for CommandPaletteWidget {
    fn id(&self) -> &str {
        "command_palette"
    }

    fn render(&mut self, ui: &mut egui::Ui, ctx: &WidgetContext) {
        if !self.visible {
            return;
        }

        let widget_size = Vec2::new(600.0, 400.0);
        let widget_rect = self.position().calculate_rect(widget_size, ctx.available_rect);

        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(widget_rect), |ui| {
            egui::Window::new("Command Palette")
                .resizable(false)
                .collapsible(false)
                .title_bar(false)
                .show(ui.ctx(), |ui| {
                    ui.vertical(|ui| {
                        // Search input
                        ui.horizontal(|ui| {
                            ui.label("❯");
                            let response = ui.text_edit_singleline(&mut String::new()); // Placeholder
                            // Focus handling would be implemented here
                            if response.gained_focus() {
                                // Request focus for the text input
                            }
                        });

                        ui.separator();

                        // Command list (placeholder)
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.label("Commands will appear here...");
                            ui.small("(Implementation connected to IDE state)");
                        });
                    });
                });
        });
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
        if visible {
            self.selected_index = 0;
        }
    }

    fn position(&self) -> WidgetPosition {
        self.position
    }

    fn desired_size(&self) -> Vec2 {
        Vec2::new(600.0, 400.0)
    }
}

impl Default for CommandPaletteWidget {
    fn default() -> Self {
        Self::new()
    }
}