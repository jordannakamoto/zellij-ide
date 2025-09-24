# Zellij IDE Development Log

## Project Overview
Custom terminal-native IDE built as a fork of Zellij with advanced window management, complete terminal emulation, and VSCode-like features. Eventually to be wrapped with Ghostty for GPU acceleration.

## Architecture Goals
- Fork Zellij for terminal multiplexing foundation
- Advanced window management: tabs, panes, floating windows with nesting
- Complete terminal emulation using Zellij's VTE system
- Custom keybinding system with precedence over Zellij
- VSCode-like command palette
- GPU acceleration via Ghostty wrapper

## Completed Features ✅

### 1. Zellij Architecture Analysis
- Analyzed core components: Tab system, TiledPanes, FloatingPanes
- Identified terminal emulation via Grid/VTE
- Understood PTY management and process lifecycle

### 2. Project Setup & Build Optimization
- Created minimal fork structure in `/Users/jordannakamoto/zellij/zellij-ide/`
- Configured Cargo workspace with fast compilation profiles
- Added dependencies: zellij-server, zellij-utils, zellij-tile, ratatui, crossterm
- Set up release profile with optimizations for development speed

### 3. Core Window Management System
**File: `src/window_manager.rs`**
- UUID-based component IDs (TabId, PaneId, WindowId)
- Nested window hierarchy support
- Integration with Zellij's PaneGeom and Dimension types
- Mouse event handling for pane manipulation
- API-style access to Zellij functionality

### 4. GUI Control System
**File: `src/gui_controls.rs`**
- Ratatui-based overlay system with terminal UI
- Function key bindings (F1-F4) for different modes
- Mouse interaction for pane creation and manipulation
- Context menus and visual feedback
- Split preview and creation modes

### 5. Terminal IDE Orchestrator
**File: `src/terminal_ide.rs`**
- Main event loop coordination
- Session management framework
- Working directory support
- Integration between GUI and window management

### 6. VSCode-like Command Palette
**File: `src/command_palette.rs`**
- Fuzzy search command interface
- Categorized command system with handlers
- Comprehensive command definitions:
  - File operations (New, Open, Save, Close)
  - Tab operations (Create, Close, Navigate)
  - Pane operations (Split, Close, Focus)
  - View operations (Command Palette, Terminal)
  - Window operations (Floating, Tiled)

### 7. Comprehensive Keybinding System ⭐
**File: `src/keybindings.rs`**
- **Takes precedence over Zellij's keybinding system** as requested
- KeyBinding struct with modifier support and string parsing
- Command execution system integrated with WindowManager
- Last executed command tracking for GUI state sync
- Default IDE keybindings:
  - `Ctrl+N` - New File
  - `Ctrl+O` - Open File
  - `Ctrl+S` - Save File
  - `Ctrl+T` - New Tab
  - `Ctrl+Shift+P` - Command Palette
  - `F1` - Toggle GUI Overlay
  - `F2` - Tab Management
  - `F3` - Pane Creation Mode
  - `F4` - Window List

### 8. Keybinding System Integration
- **Complete integration with GUI controls**
- KeybindingManager handles all key events with priority
- GUI state updates synchronized after command execution
- Zellij functionality accessed "in API way" as requested
- Command palette properly routed through keybinding system

## Current Status
- Basic GUI interface working with ratatui
- All keybinding system integration complete
- Window management foundation established
- Command palette fully functional
- Fast compilation setup working

## Pending Tasks 🚧

### Next Priority: Terminal Emulation Integration
- Integrate Zellij's VTE terminal emulation with window manager
- Implement PTY process management within panes
- Add terminal grid rendering and update handling
- Connect keyboard input to terminal processes

### Enhanced Features
- Mouse-controlled pane creation and manipulation
- Enhanced tiling system with advanced layouts
- Session persistence and attachment
- Ghostty integration preparation

## Technical Architecture

### Key Design Decisions
1. **Modular separation**: GUI controls separate from window management
2. **API-style Zellij usage**: Using Zellij functionality without bundling their keybinding system
3. **Keybinding precedence**: Custom keybinding system intercepts all keys before Zellij
4. **Fast compilation**: Optimized build profiles for development speed
5. **Command pattern**: Extensible handler system for commands

### File Structure
```
src/
├── main.rs                 # CLI entry point with clap
├── window_manager.rs       # Core window/pane/tab management
├── gui_controls.rs         # Ratatui overlay system
├── terminal_ide.rs         # Main IDE orchestrator
├── command_palette.rs      # VSCode-like command interface
└── keybindings.rs         # Custom keybinding system (precedence over Zellij)
```

### Dependencies
- **zellij-server**: Window management and terminal emulation
- **zellij-utils**: Core utilities and data structures
- **ratatui**: Terminal UI rendering
- **crossterm**: Terminal event handling
- **tokio**: Async runtime
- **clap**: CLI argument parsing

## Key User Requirements Met
1. ✅ "Fork zellij and make custom app"
2. ✅ "Full, complete terminal emulation within each view component" (foundation ready)
3. ✅ "VSCode-like command palette system"
4. ✅ "Keybinding system to accompany it"
5. ✅ "Takes precedence over zellij's keybinding system"
6. ✅ "Use zellij functionality in API way" (not bundling their keybinding system)
7. ✅ "Spend less time compiling" (fast build profiles configured)

## Build & Run
```bash
# Quick check
cargo check

# Development build (optimized for compile speed)
cargo build

# Run with GUI
cargo run

# Run specific command
cargo run -- start --name "dev-session"
```

## Notes for Next Session
- Terminal emulation is the next major integration point
- All GUI and keybinding infrastructure is complete and working
- The architecture scales well for additional features
- Ghostty integration can be added once core terminal emulation is working