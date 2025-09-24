# Zellij IDE

A terminal-native IDE built on Zellij's window management system with enhanced GUI controls and mouse interaction.

## Features

### Core Window Management
- **Nested Tabs**: Tabs can contain other tabs, creating hierarchical workspaces
- **Tiled Panes**: Advanced tiling with intelligent splitting and resizing
- **Floating Windows**: Draggable floating windows that can contain their own tab systems
- **Full Terminal Emulation**: Complete VTE terminal emulation in every pane

### GUI Control System
- **Visual Overlays**: F1-F4 hotkeys for different GUI modes
- **Mouse Interaction**: Click-to-split, drag-to-resize, right-click context menus
- **Pane Creation Mode**: Visual pane creation with split previews
- **Context Menus**: Right-click menus for panes, tabs, and windows

### Enhanced Features
- **Smart Splitting**: Mouse-position-aware pane splitting
- **Visual Feedback**: Split previews and hover effects
- **Easy Navigation**: Tab bar, window list, and pane management GUIs

## Architecture

```
zellij-ide/
├── src/
│   ├── main.rs              # CLI and app entry point
│   ├── terminal_ide.rs      # Main IDE orchestrator
│   ├── window_manager.rs    # Core window/pane/tab management
│   └── gui_controls.rs      # GUI overlay system
├── gui-controls/            # Separate crate for GUI components
└── Cargo.toml              # Workspace configuration
```

### Key Components

1. **WindowManager**: Core logic for tabs, panes, and floating windows
2. **GuiControls**: TUI overlay system using ratatui for visual controls
3. **TerminalIDE**: Main orchestrator integrating window management with terminal emulation
4. **Mouse Interaction**: Advanced mouse handling for pane creation and manipulation

## Usage

### Basic Commands
```bash
# Start new IDE session
zellij-ide start --name "my-project"

# Start with GUI controls
zellij-ide start --gui-controls

# Attach to existing session
zellij-ide attach my-project
```

### Keyboard Shortcuts
- `F1`: Toggle GUI overlay
- `F2`: Tab management
- `F3`: Pane creation mode
- `F4`: Window list
- `Ctrl+T`: Quick new tab
- `Ctrl+S`: Quick split mode
- `Esc`: Exit current GUI mode

### Mouse Controls
- **Left Click**: Focus pane/tab/window
- **Right Click**: Context menu
- **Drag**: Move floating windows or resize panes
- **Split Mode Click**: Create new pane at click position

## Integration with Zellij

This IDE uses Zellij's core components:
- **Terminal Grid**: Full VTE terminal emulation via `zellij-server/panes/grid.rs`
- **PTY Management**: Process spawning and management from `zellij-server/pty.rs`
- **Pane Geometry**: Layout calculations using `zellij-utils/pane_size.rs`

## Future: Ghostty Integration

The IDE is designed to be wrapped by Ghostty terminal emulator:
- Zellij IDE handles window management and terminal multiplexing
- Ghostty provides GPU acceleration and advanced terminal features
- Clean separation allows for easy integration

## Development Status

- [x] Core window management architecture
- [x] GUI control system design
- [x] Mouse interaction framework
- [ ] Terminal emulation integration
- [ ] Enhanced tiling system
- [ ] Session persistence
- [ ] Ghostty wrapper integration

## Building

```bash
# Build the IDE
cargo build --release

# Run with GUI controls
cargo run -- start --gui-controls
```

## Dependencies

- **Core**: Uses Zellij's server, utils, and tile crates for terminal emulation
- **GUI**: ratatui and crossterm for terminal UI rendering
- **Async**: tokio for async runtime
- **Data**: serde for serialization, uuid for IDs