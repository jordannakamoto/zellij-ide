use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, MouseEvent, MouseEventKind, MouseButton},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame, Terminal,
};
use std::io::{self, Stdout};

use crate::window_manager::{WindowManager, TabId, PaneId, WindowId};
use crate::command_palette::{CommandPalette, create_command_system};
use crate::keybindings::KeybindingManager;

/// GUI overlay system for visual window management controls
pub struct GuiControls {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    show_overlay: bool,
    active_menu: Option<Menu>,
    context_menu_position: Option<(u16, u16)>,
    pane_creation_mode: bool,
    split_preview: Option<SplitPreview>,
    command_palette: CommandPalette,
    keybinding_manager: KeybindingManager,
}

#[derive(Debug, Clone)]
pub enum Menu {
    TabBar,
    ContextMenu(ContextMenuType),
    PaneCreator,
    WindowList,
}

#[derive(Debug, Clone)]
pub enum ContextMenuType {
    Pane(PaneId),
    Tab(TabId),
    Window(WindowId),
    Empty,
}

#[derive(Debug, Clone)]
pub struct SplitPreview {
    pub pane_id: PaneId,
    pub direction: crate::window_manager::MouseEventType,
    pub position: (u16, u16),
    pub preview_rect: Rect,
}

impl GuiControls {
    pub fn new() -> Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;

        // Initialize command system
        let command_system = create_command_system();
        let command_palette = CommandPalette::new(command_system);

        // Initialize keybinding system
        let keybinding_manager = KeybindingManager::new();

        Ok(Self {
            terminal,
            show_overlay: true, // Show overlay by default so user sees something
            active_menu: None,
            context_menu_position: None,
            pane_creation_mode: false,
            split_preview: None,
            command_palette,
            keybinding_manager,
        })
    }

    /// Show the initial interface with help text
    pub fn show_initial_interface(&mut self, window_manager: &WindowManager) -> Result<()> {
        self.show_overlay = true;
        self.render(window_manager)?;
        Ok(())
    }

    /// Main event loop for GUI controls
    pub fn handle_events(&mut self, window_manager: &mut WindowManager) -> Result<bool> {
        if !event::poll(std::time::Duration::from_millis(16))? {
            return Ok(true);
        }

        match event::read()? {
            Event::Key(key) => {
                if !self.handle_key_event(key, window_manager)? {
                    return Ok(false); // Signal to exit
                }
            },
            Event::Mouse(mouse) => self.handle_mouse_event(mouse, window_manager)?,
            Event::Resize(_, _) => {
                // Handle terminal resize
            },
            _ => {},
        }

        Ok(true)
    }

    fn handle_key_event(&mut self, key: KeyEvent, window_manager: &mut WindowManager) -> Result<bool> {
        // Handle command palette first (it has its own key handling when visible)
        if self.command_palette.visible {
            return self.command_palette.handle_key(key, window_manager);
        }

        // Our keybinding system takes precedence over any Zellij keybinding system
        // This ensures we intercept all keys before they reach Zellij's handlers
        if self.keybinding_manager.handle_key_event(key, window_manager)? {
            // Keybinding was handled, check for special GUI state changes
            self.update_gui_state_after_command(window_manager);
            return Ok(true);
        }

        // Fallback handling for keys not bound in our system
        match key.code {
            // Exit menus (always available)
            KeyCode::Esc => {
                self.active_menu = None;
                self.pane_creation_mode = false;
                self.split_preview = None;
            },
            // Exit IDE (always available as safety)
            KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                return Ok(false); // Signal to exit
            },
            _ => {
                // If we reach here, the key wasn't handled by our system
                // We deliberately don't pass it to Zellij's keybinding system
                // as per user requirement to use Zellij functionality "in like an api way"
            },
        }

        Ok(true)
    }

    /// Updates GUI state after a command has been executed
    fn update_gui_state_after_command(&mut self, _window_manager: &WindowManager) {
        // Check if command palette should be shown/hidden
        if let Some(last_command) = self.keybinding_manager.get_last_executed_command() {
            match last_command.as_str() {
                "toggle_gui_overlay" => {
                    self.show_overlay = !self.show_overlay;
                },
                "show_tab_management" => {
                    self.active_menu = Some(Menu::TabBar);
                    self.show_overlay = true;
                },
                "toggle_pane_creation_mode" => {
                    self.pane_creation_mode = !self.pane_creation_mode;
                    self.show_overlay = true;
                },
                "show_window_list" => {
                    self.active_menu = Some(Menu::WindowList);
                    self.show_overlay = true;
                },
                "show_command_palette" => {
                    self.command_palette.show();
                },
                _ => {},
            }
        }
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent, window_manager: &mut WindowManager) -> Result<()> {
        let position = (mouse.column, mouse.row);

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if self.pane_creation_mode {
                    self.handle_pane_creation_click(position, window_manager)?;
                } else {
                    // Regular click handling
                    window_manager.handle_mouse_event(crate::window_manager::MouseEvent {
                        position,
                        event_type: crate::window_manager::MouseEventType::LeftClick,
                    })?;
                }
            },
            MouseEventKind::Down(MouseButton::Right) => {
                self.show_context_menu(position, window_manager)?;
            },
            MouseEventKind::Drag(MouseButton::Left) => {
                if let Some(preview) = &mut self.split_preview {
                    // Update split preview
                    preview.position = position;
                    // Note: split preview update would happen here but requires more complex handling
                }
            },
            _ => {},
        }

        Ok(())
    }

    fn show_context_menu(&mut self, position: (u16, u16), _window_manager: &mut WindowManager) -> Result<()> {
        self.context_menu_position = Some(position);
        self.active_menu = Some(Menu::ContextMenu(ContextMenuType::Empty));
        self.show_overlay = true;
        Ok(())
    }

    fn initiate_split_mode(&mut self, _window_manager: &mut WindowManager) -> Result<()> {
        self.pane_creation_mode = true;
        self.show_overlay = true;
        // Could set up split preview here
        Ok(())
    }

    fn handle_pane_creation_click(&mut self, position: (u16, u16), window_manager: &mut WindowManager) -> Result<()> {
        // Determine split direction based on click position relative to current pane
        // For now, simple logic: if closer to right edge, split right; if closer to bottom, split down
        let direction = self.determine_split_direction(position);

        // Create new pane (placeholder pane ID)
        let pane_id = PaneId::new();
        window_manager.split_pane_at_position(pane_id, position, direction)?;

        self.pane_creation_mode = false;
        Ok(())
    }

    fn determine_split_direction(&self, position: (u16, u16)) -> zellij_utils::data::Direction {
        // Simple heuristic: if x > y, split vertically (right), else split horizontally (down)
        if position.0 > position.1 {
            zellij_utils::data::Direction::Right
        } else {
            zellij_utils::data::Direction::Down
        }
    }

    /// Render the GUI overlay
    pub fn render(&mut self, _window_manager: &WindowManager) -> Result<()> {
        let show_overlay = self.show_overlay;
        let active_menu = self.active_menu.clone();
        let pane_creation_mode = self.pane_creation_mode;

        // Prepare command palette rendering
        let palette_visible = self.command_palette.visible;

        self.terminal.draw(|f| {
            render_main_interface(f, show_overlay, &active_menu, pane_creation_mode);
        })?;

        // Render command palette separately if visible
        if palette_visible {
            self.terminal.draw(|f| {
                self.command_palette.render(f);
            })?;
        }

        Ok(())
    }
}

fn render_main_interface(frame: &mut Frame, _show_overlay: bool, active_menu: &Option<Menu>, pane_creation_mode: bool) {
    let area = frame.area();

    // Create main layout
    let main_block = Block::default()
        .title("🚀 Zellij IDE - Terminal Native Development Environment")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(main_block, area);

    // Create inner area for content
    let inner = area.inner(ratatui::layout::Margin { vertical: 2, horizontal: 2 });

    // Show help text and status
    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled("🎯 Welcome to Zellij IDE!", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Keyboard Controls:"),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("F1", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - Toggle GUI overlay"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("F2", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - Tab management"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("F3", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - Pane creation mode"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("F4", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - Window list"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Ctrl+T", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - New tab"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Ctrl+S", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - Split mode"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" - Exit menus"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Ctrl+Shift+P", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::raw(" - Command Palette"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Ctrl+C", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" - Exit IDE"),
        ]),
        Line::from(""),
        Line::from(Span::styled("🔧 Status: Architecture complete, terminal integration pending", Style::default().fg(Color::Blue))),
        Line::from(""),
        Line::from(Span::styled("Press F1 to start exploring the interface!", Style::default().fg(Color::Green))),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .alignment(Alignment::Left)
        .wrap(ratatui::widgets::Wrap { trim: true });

    frame.render_widget(help_paragraph, inner);

    // Show overlay-specific content if active
    match active_menu {
        Some(Menu::TabBar) => {
            render_tab_bar_overlay(frame, area);
        },
        Some(Menu::WindowList) => {
            render_window_list_overlay(frame, area);
        },
        _ => {},
    }

    if pane_creation_mode {
        render_pane_creation_help(frame, area);
    }
}

fn render_tab_bar_overlay(frame: &mut Frame, area: Rect) {
    let popup_area = ratatui::layout::Rect {
        x: area.width / 4,
        y: 3,
        width: area.width / 2,
        height: 8,
    };

    let block = Block::default()
        .title("📑 Tab Management")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let tabs_text = vec![
        Line::from("📝 Tab 1 (Active)"),
        Line::from("📝 Tab 2"),
        Line::from("➕ Create New Tab"),
        Line::from(""),
        Line::from("Use arrow keys to navigate, Enter to select"),
    ];

    let paragraph = Paragraph::new(tabs_text)
        .block(block)
        .alignment(Alignment::Left);

    frame.render_widget(Clear, popup_area);
    frame.render_widget(paragraph, popup_area);
}

fn render_window_list_overlay(frame: &mut Frame, area: Rect) {
    let popup_area = ratatui::layout::Rect {
        x: area.width / 3,
        y: 4,
        width: area.width / 3,
        height: 10,
    };

    let block = Block::default()
        .title("🪟 Window List")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Magenta));

    let windows_text = vec![
        Line::from("🖥️  Terminal 1 (Active)"),
        Line::from("🖥️  Terminal 2"),
        Line::from("🌊 Floating: Editor"),
        Line::from("🌊 Floating: Debug Console"),
        Line::from(""),
        Line::from("➕ Create New Window"),
        Line::from(""),
        Line::from("Press Esc to close this menu"),
    ];

    let paragraph = Paragraph::new(windows_text)
        .block(block)
        .alignment(Alignment::Left);

    frame.render_widget(Clear, popup_area);
    frame.render_widget(paragraph, popup_area);
}

fn render_pane_creation_help(frame: &mut Frame, area: Rect) {
    let help_area = ratatui::layout::Rect {
        x: 0,
        y: area.height - 3,
        width: area.width,
        height: 3,
    };

    let block = Block::default()
        .title("✂️ Pane Creation Mode")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let help_text = vec![
        Line::from("Click anywhere to create a new pane at that location"),
        Line::from("Right-click for context menu • Esc to exit creation mode"),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, help_area);
}

impl Drop for GuiControls {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = stdout.execute(LeaveAlternateScreen);
    }
}