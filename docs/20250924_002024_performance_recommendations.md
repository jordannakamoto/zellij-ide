# Zellij-IDE Performance Recommendations

## Fork-Specific Optimizations

### 1. Command Palette System (src/command_palette.rs)
- **Current**: Arc<Mutex<CommandSystem>> with Vec<Box<dyn CommandHandler>>
- **Optimization**: Replace with mpsc channel architecture
```rust
// Instead of Arc<Mutex<CommandSystem>>
pub struct CommandPalette {
    command_tx: mpsc::UnboundedSender<CommandRequest>,
    result_rx: mpsc::UnboundedReceiver<CommandResult>,
    // ... other fields
}
```

### 2. Fuzzy Search Performance
- **Current**: O(n) linear search through all commands
- **Optimization**: Use trie or radix tree for command indexing
- **Cache**: Store last 10 search results with LRU eviction

### 3. GUI Controls Rendering (src/gui_controls.rs)
- **Issue**: Double terminal.draw() calls (lines 254-263)
- **Fix**: Single draw call with layered rendering
- **Memory**: Pre-allocate Line/Span vectors for help text

### 4. Window Manager (src/window_manager.rs)
- **HashMap hotpath**: BTreeMap<PaneId, TiledPane> causes frequent allocations
- **Solution**: Use SlotMap for O(1) pane access
- **Geometry calculations**: Cache PaneGeom calculations, only recalc on resize

### 5. Keybinding System Bottleneck
- **Critical**: KeyCode/KeyModifiers serialization blocking builds
- **Solution**: Custom serde implementation or switch to bincode
- **Alternative**: Use integer constants instead of enum serialization

## Memory Usage Optimizations

### Stack Allocations
```rust
// Hot paths in command palette
use smallvec::SmallVec;
type CommandVec = SmallVec<[Command; 8]>; // Stack-allocated for <8 commands
```

### String Interning
```rust
// For frequently used command names/descriptions
use string_cache::DefaultAtom;
pub struct Command {
    id: DefaultAtom,
    name: DefaultAtom,
    // ...
}
```

### Lazy Loading
- Don't initialize all CommandHandlers at startup
- Load handlers on-demand when first command from category is used

## Build Performance

### Current Issues
- 3.9GB target directory for 2,217 lines of source
- Pulling entire Zellij ecosystem (server, client, plugins)
- Heavy proc-macro usage from dependencies

### Solutions
1. Feature flags for optional components
2. Selective Zellij utility imports
3. Consider copying minimal Zellij utilities instead of depending on full crates

## Runtime Performance Targets
- Command palette open: <16ms (1 frame at 60fps)
- Search results: <50ms for 1000+ commands
- Pane splitting: <100ms visual feedback
- Memory usage: <50MB RSS for basic session