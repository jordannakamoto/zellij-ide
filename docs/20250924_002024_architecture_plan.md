# Long-term Architecture Plan for Zellij IDE

## Current Structure (Flat - Not Scalable)
```
src/
├── main.rs                 # Entry point
├── terminal_ide.rs         # Main orchestrator
├── gui_controls.rs         # GUI overlay system
├── window_manager.rs       # Pane/tab management
├── command_palette.rs      # Command system
├── keybindings.rs          # Key handling
└── features.rs             # Feature flags
```

## Proposed Long-term Architecture

### Core Modules (Domain-Driven Design)
```
src/
├── main.rs
├── lib.rs                  # Public API
├── app/                    # Application orchestration
│   ├── mod.rs
│   ├── terminal_ide.rs     # Main app controller
│   └── event_loop.rs       # Central event handling
├── terminal/               # Terminal management
│   ├── mod.rs
│   ├── pty_manager.rs      # PTY process handling
│   ├── terminal_state.rs   # Terminal data structures
│   └── shell_integration.rs # Shell-specific features
├── ui/                     # User interface (bubbletea-rs enhanced)
│   ├── mod.rs
│   ├── gui_controls.rs     # Main GUI system (bubbletea integration)
│   ├── command_palette.rs  # Command interface (bubbletea widgets)
│   ├── themes.rs           # lipgloss color schemes and styling
│   └── components/         # Reusable bubbletea components
│       ├── mod.rs
│       ├── tab_bar.rs      # Gradient tab bar with animations
│       ├── status_bar.rs   # Rich status with progress indicators
│       └── dialogs.rs      # Modal dialogs with blur effects
├── window/                 # Window management
│   ├── mod.rs
│   ├── manager.rs          # Main window manager
│   ├── pane.rs             # Pane data structures
│   ├── layout.rs           # Layout algorithms
│   └── floating.rs         # Floating window support
├── input/                  # Input handling
│   ├── mod.rs
│   ├── keybindings.rs      # Key mapping
│   ├── mouse.rs            # Mouse handling
│   └── shortcuts.rs        # Configurable shortcuts
├── config/                 # Configuration system
│   ├── mod.rs
│   ├── settings.rs         # User settings
│   ├── layouts.rs          # Layout templates
│   └── themes.rs           # Theme definitions
├── features/               # Feature modules
│   ├── mod.rs
│   ├── git_integration.rs  # Git features (future)
│   ├── file_browser.rs     # File explorer (future)
│   └── debugging.rs        # Debug support (future)
└── utils/                  # Shared utilities
    ├── mod.rs
    ├── ids.rs              # ID generation
    └── messaging.rs        # Event/message types
```

## Implementation Strategy for PTY Integration

### Phase 1: Current Integration (Keep Flat Structure)
For the immediate PTY work, keep the flat structure but organize better:

```rust
// src/terminal_ide.rs - Add PTY management
pub struct TerminalIDE {
    window_manager: WindowManager,
    gui_controls: Option<GuiControls>,
    pty_manager: PtyManager,        // NEW
    working_directory: Option<PathBuf>,
}

// src/window_manager.rs - Real terminal integration
pub struct TerminalState {
    pub terminal_id: u32,           // Real terminal
    pub child_pid: RawFd,
    pub pty_bus: PtyBus,           // Communication
    pub title: String,
}
```

### Phase 2: Gradual Refactoring (Post-MVP)
Once PTY integration works, gradually move to modular structure:

1. **Extract terminal management**: `src/terminal/` module
2. **Split UI concerns**: `src/ui/` module
3. **Separate window management**: `src/window/` module
4. **Add configuration system**: `src/config/` module

## Benefits of Modular Architecture

### Maintainability
- **Single Responsibility**: Each module handles one concern
- **Clear Boundaries**: Easy to understand what goes where
- **Testability**: Mock individual modules for testing

### Extensibility
- **Plugin System**: Easy to add new features
- **Theme System**: Modular UI theming
- **Configuration**: Structured settings management

### Performance
- **Lazy Loading**: Load modules only when needed
- **Parallel Development**: Team members can work on different modules
- **Selective Compilation**: Feature flags per module

## Recommended Migration Path

### Immediate (With PTY Work)
```rust
// Keep flat structure, add PTY
src/terminal_ide.rs     → Add PtyManager integration
src/window_manager.rs   → Replace TerminalState
src/gui_controls.rs     → Connect to real terminals
```

### Short-term (Next 2-4 weeks)
```rust
// Extract terminal concerns
src/terminal/
├── pty_manager.rs      → Move from terminal_ide.rs
├── terminal_state.rs   → Move from window_manager.rs
└── shell_integration.rs → New shell features
```

### Medium-term (1-3 months)
```rust
// Full modular structure
src/app/        → Application orchestration
src/ui/         → All GUI components
src/window/     → Window management
src/config/     → Configuration system
```

## Dependencies & Integration Points

### Zellij Integration Strategy
```rust
// Terminal module interfaces with zellij-server
src/terminal/pty_manager.rs:
  - use zellij_server::pty::Pty
  - use zellij_server::thread_bus::Bus

// UI module stays independent
src/ui/gui_controls.rs:
  - No direct zellij dependencies
  - Communication via internal messages
```

### Event System Architecture
```rust
// Central event bus
pub enum IDEEvent {
    Terminal(TerminalEvent),
    Window(WindowEvent),
    UI(UIEvent),
    Input(InputEvent),
}

// Each module publishes/subscribes to relevant events
```

## Quick Implementation Decision

**For the PTY integration work right now**: Keep the flat structure. The refactoring can happen later.

**Immediate priority**: Get working terminals by integrating zellij PTY management into the current files.

**Long-term vision**: Migrate to the modular architecture once the core functionality is stable.

Ready to proceed with PTY integration in the current structure?