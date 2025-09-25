use crate::actor::{Actor, ActorMessage, ActorAPI, ApiMethod, ApiParameter, ApiParams, ApiResult};
use async_trait::async_trait;
use egui::{self, Color32, FontId, Vec2, RichText, FontFamily};
use uuid::Uuid;
use std::any::Any;
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::{Read, Write};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use vte::{Parser, Perform};
use serde_json;

/// High-performance terminal emulator actor using VTE parser
/// Supports full ANSI escape sequences and terminal features
pub struct TerminalActor {
    id: Uuid,
    name: String,

    // Terminal grid system
    terminal_grid: Arc<Mutex<TerminalGrid>>,

    // VTE parser for escape sequences
    parser: Arc<Mutex<Parser>>,

    // PTY handling
    pty_master: Option<Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>>,
    writer: Option<Arc<Mutex<Box<dyn Write + Send>>>>,

    // Rendering
    font_size: f32,
    char_width: f32,
    line_height: f32,

    // Colors
    colors: TerminalColors,

    // Scroll state
    scroll_offset: usize,
    auto_scroll: bool,
}

/// Terminal grid that stores characters and their attributes
#[derive(Clone)]
pub struct TerminalGrid {
    pub cells: Vec<Vec<TerminalCell>>,
    pub cursor: TerminalCursor,
    pub size: (usize, usize), // (cols, rows)
    pub title: String,
    pub dirty_lines: Vec<bool>, // Track which lines need re-rendering
}

#[derive(Clone)]
pub struct TerminalCell {
    pub ch: char,
    pub fg: Color32,
    pub bg: Color32,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
}

#[derive(Clone)]
pub struct TerminalCursor {
    pub x: usize,
    pub y: usize,
    pub visible: bool,
}

#[derive(Clone)]
pub struct TerminalColors {
    pub black: Color32,
    pub red: Color32,
    pub green: Color32,
    pub yellow: Color32,
    pub blue: Color32,
    pub magenta: Color32,
    pub cyan: Color32,
    pub white: Color32,
    pub bright_black: Color32,
    pub bright_red: Color32,
    pub bright_green: Color32,
    pub bright_yellow: Color32,
    pub bright_blue: Color32,
    pub bright_magenta: Color32,
    pub bright_cyan: Color32,
    pub bright_white: Color32,
    pub foreground: Color32,
    pub background: Color32,
}

impl Default for TerminalColors {
    fn default() -> Self {
        Self {
            black: Color32::from_rgb(40, 44, 52),
            red: Color32::from_rgb(224, 108, 117),
            green: Color32::from_rgb(152, 195, 121),
            yellow: Color32::from_rgb(229, 192, 123),
            blue: Color32::from_rgb(97, 175, 239),
            magenta: Color32::from_rgb(198, 120, 221),
            cyan: Color32::from_rgb(86, 182, 194),
            white: Color32::from_rgb(171, 178, 191),
            bright_black: Color32::from_rgb(92, 99, 112),
            bright_red: Color32::from_rgb(240, 113, 120),
            bright_green: Color32::from_rgb(166, 218, 149),
            bright_yellow: Color32::from_rgb(229, 192, 123),
            bright_blue: Color32::from_rgb(103, 173, 228),
            bright_magenta: Color32::from_rgb(224, 108, 117),
            bright_cyan: Color32::from_rgb(152, 195, 121),
            bright_white: Color32::from_rgb(208, 208, 208),
            foreground: Color32::from_rgb(171, 178, 191),
            background: Color32::from_rgb(40, 44, 52),
        }
    }
}

impl Default for TerminalCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Color32::from_rgb(171, 178, 191),
            bg: Color32::from_rgb(40, 44, 52),
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
        }
    }
}

impl TerminalGrid {
    pub fn new(cols: usize, rows: usize) -> Self {
        let cells = vec![vec![TerminalCell::default(); cols]; rows];
        let dirty_lines = vec![true; rows];

        Self {
            cells,
            cursor: TerminalCursor {
                x: 0,
                y: 0,
                visible: true,
            },
            size: (cols, rows),
            title: "Terminal".to_string(),
            dirty_lines,
        }
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        let old_cols = self.size.0;
        let old_rows = self.size.1;

        // Resize existing rows
        for row in &mut self.cells {
            row.resize(cols, TerminalCell::default());
        }

        // Add or remove rows
        if rows > old_rows {
            for _ in old_rows..rows {
                self.cells.push(vec![TerminalCell::default(); cols]);
            }
        } else {
            self.cells.truncate(rows);
        }

        self.dirty_lines.resize(rows, true);
        self.size = (cols, rows);

        // Clamp cursor position
        self.cursor.x = self.cursor.x.min(cols.saturating_sub(1));
        self.cursor.y = self.cursor.y.min(rows.saturating_sub(1));
    }

    pub fn put_char(&mut self, ch: char, fg: Color32, bg: Color32, bold: bool) {
        if self.cursor.y < self.size.1 && self.cursor.x < self.size.0 {
            self.cells[self.cursor.y][self.cursor.x] = TerminalCell {
                ch,
                fg,
                bg,
                bold,
                italic: false,
                underline: false,
                strikethrough: false,
            };
            self.dirty_lines[self.cursor.y] = true;
        }
    }

    pub fn move_cursor(&mut self, x: usize, y: usize) {
        self.cursor.x = x.min(self.size.0.saturating_sub(1));
        self.cursor.y = y.min(self.size.1.saturating_sub(1));
    }

    pub fn clear_line(&mut self, y: usize) {
        if y < self.size.1 {
            for cell in &mut self.cells[y] {
                *cell = TerminalCell::default();
            }
            self.dirty_lines[y] = true;
        }
    }

    pub fn scroll_up(&mut self, lines: usize) {
        for _ in 0..lines {
            self.cells.remove(0);
            self.cells.push(vec![TerminalCell::default(); self.size.0]);
        }
        // Mark all lines as dirty after scrolling
        self.dirty_lines.fill(true);
    }
}

/// VTE performer that updates the terminal grid
struct TerminalPerformer {
    grid: Arc<Mutex<TerminalGrid>>,
    colors: TerminalColors,
    current_fg: Color32,
    current_bg: Color32,
    bold: bool,
}

impl TerminalPerformer {
    fn new(grid: Arc<Mutex<TerminalGrid>>, colors: TerminalColors) -> Self {
        Self {
            grid,
            current_fg: colors.foreground,
            current_bg: colors.background,
            bold: false,
            colors,
        }
    }
}

impl Perform for TerminalPerformer {
    fn print(&mut self, c: char) {
        let mut grid = self.grid.lock().unwrap();
        grid.put_char(c, self.current_fg, self.current_bg, self.bold);

        // Move cursor forward
        if grid.cursor.x < grid.size.0 - 1 {
            grid.cursor.x += 1;
        } else if grid.cursor.y < grid.size.1 - 1 {
            grid.cursor.x = 0;
            grid.cursor.y += 1;
        } else {
            // Scroll up when we reach bottom-right
            grid.scroll_up(1);
            grid.cursor.x = 0;
        }
    }

    fn execute(&mut self, byte: u8) {
        let mut grid = self.grid.lock().unwrap();
        match byte {
            b'\n' => { // Line Feed
                if grid.cursor.y < grid.size.1 - 1 {
                    grid.cursor.y += 1;
                } else {
                    grid.scroll_up(1);
                }
            },
            b'\r' => { // Carriage Return
                grid.cursor.x = 0;
            },
            b'\t' => { // Tab
                let next_tab = ((grid.cursor.x / 8) + 1) * 8;
                grid.cursor.x = next_tab.min(grid.size.0 - 1);
            },
            b'\x08' => { // Backspace
                if grid.cursor.x > 0 {
                    grid.cursor.x -= 1;
                }
            },
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &vte::Params, _intermediates: &[u8], _ignore: bool, c: char) {
        let mut grid = self.grid.lock().unwrap();

        match c {
            'H' | 'f' => { // Cursor Position
                let row = *params.iter().nth(0).and_then(|p| p.get(0)).unwrap_or(&1) as usize;
                let col = *params.iter().nth(1).and_then(|p| p.get(0)).unwrap_or(&1) as usize;
                grid.move_cursor(col.saturating_sub(1), row.saturating_sub(1));
            },
            'A' => { // Cursor Up
                let n = *params.iter().nth(0).and_then(|p| p.get(0)).unwrap_or(&1) as usize;
                grid.cursor.y = grid.cursor.y.saturating_sub(n);
            },
            'B' => { // Cursor Down
                let n = *params.iter().nth(0).and_then(|p| p.get(0)).unwrap_or(&1) as usize;
                grid.cursor.y = (grid.cursor.y + n).min(grid.size.1 - 1);
            },
            'C' => { // Cursor Forward
                let n = *params.iter().nth(0).and_then(|p| p.get(0)).unwrap_or(&1) as usize;
                grid.cursor.x = (grid.cursor.x + n).min(grid.size.0 - 1);
            },
            'D' => { // Cursor Back
                let n = *params.iter().nth(0).and_then(|p| p.get(0)).unwrap_or(&1) as usize;
                grid.cursor.x = grid.cursor.x.saturating_sub(n);
            },
            'K' => { // Erase in Line
                let n = *params.iter().nth(0).and_then(|p| p.get(0)).unwrap_or(&0);
                match n {
                    0 => { // Clear from cursor to end of line
                        let cursor_x = grid.cursor.x;
                        let cursor_y = grid.cursor.y;
                        let size_0 = grid.size.0;
                        for x in cursor_x..size_0 {
                            grid.cells[cursor_y][x] = TerminalCell::default();
                        }
                    },
                    1 => { // Clear from start of line to cursor
                        let cursor_x = grid.cursor.x;
                        let cursor_y = grid.cursor.y;
                        for x in 0..=cursor_x {
                            grid.cells[cursor_y][x] = TerminalCell::default();
                        }
                    },
                    2 => { // Clear entire line
                        let cursor_y = grid.cursor.y;
                        grid.clear_line(cursor_y);
                    },
                    _ => {}
                }
                let cursor_y = grid.cursor.y;
                grid.dirty_lines[cursor_y] = true;
            },
            'J' => { // Erase in Display
                let n = *params.iter().nth(0).and_then(|p| p.get(0)).unwrap_or(&0);
                match n {
                    0 => { // Clear from cursor to end of screen
                        let cursor_x = grid.cursor.x;
                        let cursor_y = grid.cursor.y;
                        let size_0 = grid.size.0;
                        let size_1 = grid.size.1;

                        // Clear current line from cursor
                        for x in cursor_x..size_0 {
                            grid.cells[cursor_y][x] = TerminalCell::default();
                        }
                        // Clear all lines below
                        for y in (cursor_y + 1)..size_1 {
                            grid.clear_line(y);
                        }
                    },
                    1 => { // Clear from start of screen to cursor
                        let cursor_x = grid.cursor.x;
                        let cursor_y = grid.cursor.y;

                        // Clear all lines above
                        for y in 0..cursor_y {
                            grid.clear_line(y);
                        }
                        // Clear current line to cursor
                        for x in 0..=cursor_x {
                            grid.cells[cursor_y][x] = TerminalCell::default();
                        }
                    },
                    2 => { // Clear entire screen
                        for y in 0..grid.size.1 {
                            grid.clear_line(y);
                        }
                        grid.cursor = TerminalCursor { x: 0, y: 0, visible: true };
                    },
                    _ => {}
                }
            },
            'm' => { // Select Graphic Rendition (colors, bold, etc.)
                for param in params.iter() {
                    for &p in param {
                        match p {
                            0 => { // Reset
                                self.current_fg = self.colors.foreground;
                                self.current_bg = self.colors.background;
                                self.bold = false;
                            },
                            1 => self.bold = true,
                            22 => self.bold = false,
                            30 => self.current_fg = self.colors.black,
                            31 => self.current_fg = self.colors.red,
                            32 => self.current_fg = self.colors.green,
                            33 => self.current_fg = self.colors.yellow,
                            34 => self.current_fg = self.colors.blue,
                            35 => self.current_fg = self.colors.magenta,
                            36 => self.current_fg = self.colors.cyan,
                            37 => self.current_fg = self.colors.white,
                            39 => self.current_fg = self.colors.foreground,
                            40 => self.current_bg = self.colors.black,
                            41 => self.current_bg = self.colors.red,
                            42 => self.current_bg = self.colors.green,
                            43 => self.current_bg = self.colors.yellow,
                            44 => self.current_bg = self.colors.blue,
                            45 => self.current_bg = self.colors.magenta,
                            46 => self.current_bg = self.colors.cyan,
                            47 => self.current_bg = self.colors.white,
                            49 => self.current_bg = self.colors.background,
                            90 => self.current_fg = self.colors.bright_black,
                            91 => self.current_fg = self.colors.bright_red,
                            92 => self.current_fg = self.colors.bright_green,
                            93 => self.current_fg = self.colors.bright_yellow,
                            94 => self.current_fg = self.colors.bright_blue,
                            95 => self.current_fg = self.colors.bright_magenta,
                            96 => self.current_fg = self.colors.bright_cyan,
                            97 => self.current_fg = self.colors.bright_white,
                            _ => {}
                        }
                    }
                }
            },
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {
        // Handle escape sequences
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        // Handle OSC sequences (like setting terminal title)
        if !params.is_empty() && params[0] == b"0" && params.len() > 1 {
            if let Ok(title) = std::str::from_utf8(params[1]) {
                let mut grid = self.grid.lock().unwrap();
                grid.title = title.to_string();
            }
        }
    }

    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _c: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
}

impl TerminalActor {
    pub fn new() -> Self {
        let id = Uuid::new_v4();
        let colors = TerminalColors::default();

        // Create terminal grid (80x24 is standard)
        let terminal_grid = Arc::new(Mutex::new(TerminalGrid::new(80, 24)));

        // Create VTE parser
        let parser = Arc::new(Mutex::new(Parser::new()));

        let mut actor = Self {
            id,
            name: "Terminal".to_string(),
            terminal_grid,
            parser,
            pty_master: None,
            writer: None,
            font_size: 14.0,
            char_width: 8.4,  // Approximate monospace width
            line_height: 18.0,
            colors: colors.clone(),
            scroll_offset: 0,
            auto_scroll: true,
        };

        actor.spawn_shell();
        actor
    }

    fn spawn_shell(&mut self) {
        let pty_system = native_pty_system();

        let pty_size = PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pair = pty_system.openpty(pty_size).expect("Failed to open PTY");

        // Spawn shell
        let cmd = CommandBuilder::new("/bin/bash");
        let _child = pair.slave.spawn_command(cmd)
            .expect("Failed to spawn shell");

        // Set up reader thread with VTE parser
        let mut reader = pair.master.try_clone_reader()
            .expect("Failed to clone reader");
        let grid = self.terminal_grid.clone();
        let parser = self.parser.clone();
        let colors = self.colors.clone();

        thread::spawn(move || {
            let mut performer = TerminalPerformer::new(grid, colors);
            let mut parser = parser.lock().unwrap();

            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(size) if size > 0 => {
                        for &byte in &buf[..size] {
                            parser.advance(&mut performer, byte);
                        }
                    },
                    _ => break,
                }
            }
        });

        // Store writer for input
        self.writer = Some(Arc::new(Mutex::new(pair.master.take_writer()
            .expect("Failed to get writer"))));
        self.pty_master = Some(Arc::new(Mutex::new(pair.master)));
    }

    pub fn write_to_terminal(&mut self, text: &str) {
        if let Some(writer_arc) = &self.writer {
            if let Ok(mut writer) = writer_arc.lock() {
                let _ = writer.write_all(text.as_bytes());
                let _ = writer.flush();
            }
        }
    }

    pub fn resize_terminal(&mut self, cols: u16, rows: u16) {
        // Resize PTY
        if let Some(master_arc) = &self.pty_master {
            if let Ok(master) = master_arc.lock() {
                let _ = master.resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                });
            }
        }

        // Resize terminal grid
        let mut grid = self.terminal_grid.lock().unwrap();
        grid.resize(cols as usize, rows as usize);
    }
}

#[async_trait]
impl Actor for TerminalActor {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> String {
        let grid = self.terminal_grid.lock().unwrap();
        if grid.title != "Terminal" {
            format!("{}: {}", self.name, grid.title)
        } else {
            self.name.clone()
        }
    }

    async fn handle_message(&mut self, message: ActorMessage) -> anyhow::Result<()> {
        match message {
            ActorMessage::TextInput(text) => {
                self.write_to_terminal(&text);
            },
            ActorMessage::KeyEvent { key, modifiers } => {
                match key {
                    egui::Key::Enter => self.write_to_terminal("\r"),
                    egui::Key::Backspace => self.write_to_terminal("\x08"),
                    egui::Key::Tab => self.write_to_terminal("\t"),
                    egui::Key::ArrowUp => self.write_to_terminal("\x1b[A"),
                    egui::Key::ArrowDown => self.write_to_terminal("\x1b[B"),
                    egui::Key::ArrowLeft => self.write_to_terminal("\x1b[D"),
                    egui::Key::ArrowRight => self.write_to_terminal("\x1b[C"),
                    egui::Key::C if modifiers.ctrl => self.write_to_terminal("\x03"),
                    egui::Key::D if modifiers.ctrl => self.write_to_terminal("\x04"),
                    egui::Key::Z if modifiers.ctrl => self.write_to_terminal("\x1a"),
                    egui::Key::L if modifiers.ctrl => self.write_to_terminal("\x0c"),
                    _ => {}
                }
            },
            ActorMessage::Resize { width, height } => {
                let cols = (width / self.char_width) as u16;
                let rows = (height / self.line_height) as u16;
                self.resize_terminal(cols, rows);
            },
            _ => {}
        }
        Ok(())
    }

    fn update(&mut self, ctx: &egui::Context) {
        ctx.request_repaint();
    }

    fn render(&mut self, ui: &mut egui::Ui) {
        let available_rect = ui.available_rect_before_wrap();

        // Update font metrics
        let font_id = FontId::new(self.font_size, FontFamily::Monospace);

        // Paint background
        ui.painter().rect_filled(
            available_rect,
            0.0,
            self.colors.background,
        );

        // Get terminal dimensions first
        let new_cols = (available_rect.width() / self.char_width) as u16;
        let new_rows = (available_rect.height() / self.line_height) as u16;

        // Handle keyboard input first (without borrowing self)
        let mut input_events = Vec::new();
        ui.input(|i| {
            for event in &i.events {
                match event {
                    egui::Event::Text(text) => {
                        input_events.push(text.clone());
                    },
                    egui::Event::Key { key, pressed, modifiers, .. } if *pressed => {
                        match key {
                            egui::Key::Enter => input_events.push("\r".to_string()),
                            egui::Key::Backspace => input_events.push("\x7f".to_string()),
                            egui::Key::Tab => input_events.push("\t".to_string()),
                            egui::Key::ArrowUp => input_events.push("\x1b[A".to_string()),
                            egui::Key::ArrowDown => input_events.push("\x1b[B".to_string()),
                            egui::Key::ArrowLeft => input_events.push("\x1b[D".to_string()),
                            egui::Key::ArrowRight => input_events.push("\x1b[C".to_string()),
                            egui::Key::C if modifiers.ctrl => input_events.push("\x03".to_string()),
                            egui::Key::D if modifiers.ctrl => input_events.push("\x04".to_string()),
                            egui::Key::Z if modifiers.ctrl => input_events.push("\x1a".to_string()),
                            egui::Key::L if modifiers.ctrl => input_events.push("\x0c".to_string()),
                            _ => {}
                        }
                    },
                    _ => {}
                }
            }
        });

        // Process input events
        for input in input_events {
            self.write_to_terminal(&input);
        }

        // Render terminal grid
        {
            let grid = self.terminal_grid.lock().unwrap();

            egui::ScrollArea::vertical()
                .id_salt(self.id) // Use id_salt instead of deprecated id_source
                .auto_shrink([false; 2])
                .show(ui, |ui| {

                    for (row_idx, row) in grid.cells.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;

                            for (col_idx, cell) in row.iter().enumerate() {
                                let cell_rect = egui::Rect::from_min_size(
                                    ui.cursor().min,
                                    Vec2::new(self.char_width, self.line_height)
                                );

                                // Draw cell background if different from terminal background
                                if cell.bg != self.colors.background {
                                    ui.painter().rect_filled(cell_rect, 0.0, cell.bg);
                                }

                                // Draw cursor
                                if grid.cursor.visible &&
                                   row_idx == grid.cursor.y &&
                                   col_idx == grid.cursor.x {
                                    ui.painter().rect_stroke(
                                        cell_rect,
                                        0.0,
                                        egui::Stroke::new(1.0, self.colors.foreground)
                                    );
                                }

                                // Draw character
                                let mut text = RichText::new(cell.ch.to_string())
                                    .font(font_id.clone())
                                    .color(cell.fg);

                                if cell.bold {
                                    text = text.strong();
                                }

                                ui.allocate_ui_with_layout(
                                    Vec2::new(self.char_width, self.line_height),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.label(text);
                                    }
                                );
                            }
                        });
                    }
                });

            // Auto-resize terminal based on available space
            if new_cols != grid.size.0 as u16 || new_rows != grid.size.1 as u16 {
                drop(grid); // Release the lock before calling resize_terminal
                self.resize_terminal(new_cols, new_rows);
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ActorAPI for TerminalActor {
    fn actor_type(&self) -> String {
        "TerminalActor".to_string()
    }

    fn get_api_methods(&self) -> Vec<ApiMethod> {
        vec![
            ApiMethod {
                name: "write".to_string(),
                description: "Write text to the terminal".to_string(),
                parameters: vec![
                    ApiParameter {
                        name: "text".to_string(),
                        param_type: "string".to_string(),
                        description: "Text to write to the terminal".to_string(),
                        required: true,
                        default_value: None,
                    }
                ],
                return_type: "void".to_string(),
                category: "input".to_string(),
            },
            ApiMethod {
                name: "clear".to_string(),
                description: "Clear the terminal screen".to_string(),
                parameters: vec![],
                return_type: "void".to_string(),
                category: "display".to_string(),
            },
            ApiMethod {
                name: "resize".to_string(),
                description: "Resize the terminal".to_string(),
                parameters: vec![
                    ApiParameter {
                        name: "cols".to_string(),
                        param_type: "number".to_string(),
                        description: "Number of columns".to_string(),
                        required: true,
                        default_value: None,
                    },
                    ApiParameter {
                        name: "rows".to_string(),
                        param_type: "number".to_string(),
                        description: "Number of rows".to_string(),
                        required: true,
                        default_value: None,
                    }
                ],
                return_type: "void".to_string(),
                category: "display".to_string(),
            },
            ApiMethod {
                name: "get_title".to_string(),
                description: "Get the terminal title".to_string(),
                parameters: vec![],
                return_type: "string".to_string(),
                category: "info".to_string(),
            },
            ApiMethod {
                name: "get_size".to_string(),
                description: "Get terminal dimensions".to_string(),
                parameters: vec![],
                return_type: "object".to_string(),
                category: "info".to_string(),
            },
        ]
    }

    fn execute_api_method(&mut self, method: &str, params: ApiParams) -> anyhow::Result<ApiResult> {
        match method {
            "write" => {
                let text: String = params.get("text")?;
                self.write_to_terminal(&text);
                Ok(ApiResult::Success)
            },
            "clear" => {
                self.write_to_terminal("\x1b[2J\x1b[H"); // ANSI clear screen and home cursor
                Ok(ApiResult::Success)
            },
            "resize" => {
                let cols: u16 = params.get::<f64>("cols")? as u16;
                let rows: u16 = params.get::<f64>("rows")? as u16;
                self.resize_terminal(cols, rows);
                Ok(ApiResult::Success)
            },
            "get_title" => {
                let grid = self.terminal_grid.lock().unwrap();
                Ok(ApiResult::Value(serde_json::Value::String(grid.title.clone())))
            },
            "get_size" => {
                let grid = self.terminal_grid.lock().unwrap();
                let size = serde_json::json!({
                    "cols": grid.size.0,
                    "rows": grid.size.1
                });
                Ok(ApiResult::Value(size))
            },
            _ => Err(anyhow::anyhow!("Unknown method: {}", method))
        }
    }

    fn get_capabilities(&self) -> Vec<String> {
        vec![
            "terminal_emulation".to_string(),
            "ansi_colors".to_string(),
            "shell_integration".to_string(),
            "vt100_compatibility".to_string(),
            "pty_support".to_string(),
        ]
    }

    fn get_state(&self) -> std::collections::HashMap<String, serde_json::Value> {
        let mut state = std::collections::HashMap::new();

        if let Ok(grid) = self.terminal_grid.lock() {
            state.insert("cols".to_string(), serde_json::Value::Number(serde_json::Number::from(grid.size.0)));
            state.insert("rows".to_string(), serde_json::Value::Number(serde_json::Number::from(grid.size.1)));
            state.insert("title".to_string(), serde_json::Value::String(grid.title.clone()));
            state.insert("cursor_visible".to_string(), serde_json::Value::Bool(grid.cursor.visible));
            state.insert("cursor_x".to_string(), serde_json::Value::Number(serde_json::Number::from(grid.cursor.x)));
            state.insert("cursor_y".to_string(), serde_json::Value::Number(serde_json::Number::from(grid.cursor.y)));
        }

        state.insert("font_size".to_string(), serde_json::json!(self.font_size));
        state.insert("auto_scroll".to_string(), serde_json::Value::Bool(self.auto_scroll));
        state.insert("scroll_offset".to_string(), serde_json::Value::Number(serde_json::Number::from(self.scroll_offset)));

        state
    }
}