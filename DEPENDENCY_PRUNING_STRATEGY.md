# Zellij Dependency Pruning Strategy

## Current Problem
**Target directory**: 3.9GB for 2,217 lines of source code
**Root cause**: Pulling in entire Zellij ecosystem when only using tiny utilities

## Actual Usage Analysis

### What You're Actually Using
```rust
// From zellij_utils only:
use zellij_utils::pane_size::{PaneGeom, Size, Viewport, Dimension};
use zellij_utils::data::{Direction, FloatingPaneCoordinates};
use zellij_utils::input::layout::SplitSize;
```

### What You're NOT Using But Pulling In
- **zellij-server**: Entire server, client communication, PTY management, IPC
- **zellij-tile**: Plugin system, WASM runtime, tile protocols
- **Transitive deps**: Hundreds of crates for server/client functionality

## Pruning Strategy

### Option 1: Minimal Zellij Dependencies (Recommended)
Replace current Cargo.toml dependencies:

```toml
# REMOVE these heavy dependencies:
# zellij-server = { path = "../zellij-server/", version = "0.44.0" }
# zellij-tile = { path = "../zellij-tile/", version = "0.44.0" }

# KEEP only:
zellij-utils = { path = "../zellij-utils/", version = "0.44.0", default-features = false }

# Add specific features if zellij-utils supports them:
# zellij-utils = { path = "../zellij-utils/", features = ["pane-geometry", "data-types"] }
```

**Expected savings**: ~2.5GB target directory reduction

### Option 2: Copy Minimal Utilities
Create `src/zellij_types.rs` and copy only the structs you need:

```rust
// Copy these specific types instead of depending on zellij-utils
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Left, Right, Up, Down,
}

#[derive(Debug, Clone)]
pub struct Size {
    pub cols: usize,
    pub rows: usize,
}

// Copy only PaneGeom, Viewport, Dimension, FloatingPaneCoordinates, SplitSize
```

**Expected savings**: ~3GB+ target directory reduction

### Option 3: Feature-Gated Dependencies
Add feature flags to Cargo.toml:

```toml
[features]
default = ["minimal"]
minimal = []
full-zellij = ["zellij-server", "zellij-tile"]
server-integration = ["zellij-server"]
plugin-support = ["zellij-tile"]

[dependencies]
zellij-server = { path = "../zellij-server/", version = "0.44.0", optional = true }
zellij-utils = { path = "../zellij-utils/", version = "0.44.0" }
zellij-tile = { path = "../zellij-tile/", version = "0.44.0", optional = true }
```

## Implementation Plan

### Phase 1: Remove Unused Dependencies (5 minutes)
```bash
# Test that you're not actually using zellij-server or zellij-tile
cargo check --no-default-features
```

### Phase 2: Update Cargo.toml (2 minutes)
Remove zellij-server and zellij-tile dependencies entirely.

### Phase 3: Verify Build (1 minute)
```bash
cargo build --release
du -sh target/  # Should be <1GB now
```

### Phase 4: Optional - Copy Minimal Types
If zellij-utils still brings in too much, copy just the 5 types you need.

## Expected Performance Impact

### Before Pruning
- Target directory: 3.9GB
- Build time: 2.7s (fast-check)
- Dependencies: ~400+ crates
- Binary size: Unknown (couldn't measure due to build issues)

### After Pruning (Estimated)
- Target directory: <1GB (75% reduction)
- Build time: <1s (fast-check)
- Dependencies: ~50 crates
- Binary size: <10MB
- Cold build: <30s (vs current timeout >5min)

## Risk Assessment

### Low Risk
- Removing zellij-server: You don't use any server functionality
- Removing zellij-tile: You don't use any plugin functionality

### Medium Risk
- Reducing zellij-utils features: Might need to copy some utility functions

### Zero Risk
- The current approach is pulling in 10x more than needed

## Quick Win Implementation

Execute this immediately for instant improvement:

```toml
# In Cargo.toml, comment out:
# zellij-server = { path = "../zellij-server/", version = "0.44.0" }
# zellij-tile = { path = "../zellij-tile/", version = "0.44.0" }

# Keep only:
zellij-utils = { path = "../zellij-utils/", version = "0.44.0" }
```

This single change should reduce your target directory by 60-80% while maintaining all current functionality.