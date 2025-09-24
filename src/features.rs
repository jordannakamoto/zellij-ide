use anyhow::Result;
use std::collections::HashMap;

use crate::command_palette::{Command, CommandContext, CommandResult};

/// Feature trait - defines the interface for all IDE features
pub trait Feature: Send + Sync {
    /// Get the feature name
    fn name(&self) -> &str;

    /// Get commands this feature provides for the command palette
    fn get_commands(&self) -> Vec<Command>;

    /// Execute a command from this feature
    fn execute_command(&self, command_id: &str, _context: CommandContext) -> Result<CommandResult>;

    /// Initialize the feature (called on startup)
    fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// Cleanup the feature (called on shutdown)
    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Feature registry - manages all IDE features
pub struct FeatureRegistry {
    features: HashMap<String, Box<dyn Feature>>,
}

impl FeatureRegistry {
    pub fn new() -> Self {
        Self {
            features: HashMap::new(),
        }
    }

    /// Register a feature
    pub fn register<F: Feature + 'static>(&mut self, feature: F) {
        let name = feature.name().to_string();
        self.features.insert(name, Box::new(feature));
    }

    /// Get all commands from all features
    pub fn get_all_commands(&self) -> Vec<Command> {
        let mut commands = Vec::new();
        for feature in self.features.values() {
            commands.extend(feature.get_commands());
        }
        commands
    }

    /// Execute a command by finding the appropriate feature
    pub fn execute_command(&self, command_id: &str, context: CommandContext) -> Result<CommandResult> {
        // Try to find a feature that can handle this command
        for feature in self.features.values() {
            for command in feature.get_commands() {
                if command.id == command_id {
                    return feature.execute_command(command_id, context);
                }
            }
        }

        Err(anyhow::anyhow!("No feature found to handle command: {}", command_id))
    }

    /// Initialize all features
    pub fn initialize_all(&mut self) -> Result<()> {
        for feature in self.features.values_mut() {
            feature.initialize()?;
        }
        Ok(())
    }

    /// Cleanup all features
    pub fn cleanup_all(&mut self) -> Result<()> {
        for feature in self.features.values_mut() {
            feature.cleanup()?;
        }
        Ok(())
    }
}

impl Default for FeatureRegistry {
    fn default() -> Self {
        Self::new()
    }
}