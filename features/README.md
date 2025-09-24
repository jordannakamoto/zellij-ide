# Features Directory

This directory contains modular features for the Zellij IDE. Each feature is self-contained and can be developed independently.

## Feature Structure

Each feature directory should contain:
- `mod.rs` - Main module definition
- `config.rs` - Feature-specific configuration
- `commands.rs` - Commands exposed to the command palette
- `ui.rs` - UI components and rendering logic
- `tests.rs` - Feature tests

## Available Features

- **editor/** - Text editor integration (Micro-based)
- **terminal/** - Terminal management and spawning
- **file_explorer/** - File system navigation and management
- **command_palette/** - Command search and execution
- **themes/** - Theme system and customization

## Feature Integration

Features are registered in `src/main.rs` and can communicate through:
- Shared state via the main application context
- Events through the event system
- Commands through the command palette system