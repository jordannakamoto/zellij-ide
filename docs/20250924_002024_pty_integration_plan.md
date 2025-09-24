# PTY Integration Implementation Plan

## Current State
- **GUI System**: Complete ✅
- **Command Palette**: Complete ✅
- **Window Manager**: Empty shells with placeholder `TerminalState` ❌
- **PTY Management**: None - just TODOs ❌
- **Terminal Rendering**: None ❌

## Goal
Replace placeholder terminal system with real zellij-server PTY management to get working terminals.

## Architecture Overview

### Zellij-Server Components We'll Use
1. **`Pty` struct** (`/zellij-server/src/pty.rs:209`): Main PTY manager
2. **PTY Thread** (`pty_thread_main`): Background thread for terminal I/O
3. **Terminal spawning** (`spawn_terminal`): Create new terminal processes
4. **Bus system**: Message passing between components

### What We'll Replace
```rust
// BEFORE (src/window_manager.rs:84-88)
pub struct TerminalState {
    pub pty_id: Option<u32>,  // Always None
    pub title: String,        // Static string
}

// AFTER - Real terminal integration
pub struct TerminalState {
    pub terminal_id: u32,           // Actual terminal ID
    pub child_pid: RawFd,           // Real process FD
    pub title: String,              // Dynamic from process
    pub pty_bus: PtyBus,            // Communication channel
}
```

## Implementation Steps

### Phase 1: Add Zellij PTY Dependencies (5 min)
1. Update imports in `src/window_manager.rs`:
   ```rust
   use zellij_server::pty::{Pty, PtyInstruction};
   use zellij_server::thread_bus::{Bus, ThreadSenders};
   ```

2. Add PTY thread management to `TerminalIDE`

### Phase 2: Replace TerminalState (10 min)
1. Remove placeholder `TerminalState`
2. Integrate real zellij PTY structs
3. Update `TiledPane` to hold actual terminal data

### Phase 3: Implement Terminal Spawning (15 min)
1. Replace `create_terminal_pane()` TODO with real implementation
2. Use `pty.spawn_terminal()` to create actual shell processes
3. Connect terminal ID to pane management

### Phase 4: Terminal Data Flow (20 min)
1. Replace `handle_terminal_events()` TODO
2. Set up PTY read/write loop
3. Connect terminal output to GUI rendering
4. Handle keyboard input → terminal input

### Phase 5: Integration Testing (10 min)
1. Test terminal creation
2. Verify shell commands work
3. Test pane switching between terminals

## Key Integration Points

### 1. Terminal Creation Flow
```rust
// Current (broken):
create_pane() → TerminalState { pty_id: None }

// New (working):
create_pane() → spawn_terminal() → real shell process
```

### 2. Event Loop Integration
```rust
// Current:
handle_terminal_events() → // TODO: placeholder

// New:
handle_terminal_events() → read PTY → update terminal grid → render
```

### 3. Message Bus Architecture
```rust
TerminalIDE → WindowManager → PTY Thread → Shell Process
     ↑                                          ↓
GUI Events ←————— Terminal Output ←——————————————
```

## Files to Modify

### Primary Changes
1. **`src/terminal_ide.rs`**: Add PTY thread management, replace TODOs
2. **`src/window_manager.rs`**: Replace `TerminalState`, integrate real terminals
3. **`src/gui_controls.rs`**: Connect terminal rendering to actual data

### Secondary Changes
4. **`Cargo.toml`**: Ensure zellij-server dependency is properly configured
5. **`src/main.rs`**: Initialize PTY system on startup

## Risk Assessment

### Low Risk
- Adding zellij PTY imports
- Replacing placeholder structs
- Terminal creation (well-tested in zellij)

### Medium Risk
- Message bus integration (complex but documented)
- Event loop coordination (async/sync boundary)

### High Risk
- None - we're using proven zellij components

## Success Criteria
- [ ] `cargo run` starts with working shell
- [ ] Can type commands and see output
- [ ] Pane creation spawns new shell
- [ ] Tab switching works between terminals
- [ ] Command palette can create new terminals

## Estimated Time: ~60 minutes
- Phase 1: 5 min
- Phase 2: 10 min
- Phase 3: 15 min
- Phase 4: 20 min
- Phase 5: 10 min

Ready to start with Phase 1?