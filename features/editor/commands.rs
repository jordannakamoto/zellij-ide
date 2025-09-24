use anyhow::Result;
use crate::command_palette::{Command, CommandCategory, CommandContext, CommandResult};
use crate::features::Feature;
use super::EditorFeature;

impl Feature for EditorFeature {
    fn name(&self) -> &str {
        "editor"
    }

    fn get_commands(&self) -> Vec<Command> {
        vec![
            Command::new(
                "editor.new_file",
                "New File",
                "Create a new file in the editor",
                CommandCategory::Editor
            )
            .with_keybinding("Ctrl+N")
            .with_tags(vec!["new", "file", "create"]),

            Command::new(
                "editor.open_file",
                "Open File",
                "Open an existing file in the editor",
                CommandCategory::Editor
            )
            .with_keybinding("Ctrl+O")
            .with_tags(vec!["open", "file", "load"]),

            Command::new(
                "editor.new_pane",
                "New Editor Pane",
                "Open a new editor in a split pane",
                CommandCategory::Editor
            )
            .with_keybinding("Ctrl+E")
            .with_tags(vec!["editor", "pane", "split"]),

            Command::new(
                "editor.launch",
                "Launch Editor",
                "Launch Micro editor in current terminal",
                CommandCategory::Editor
            )
            .with_keybinding("Ctrl+Shift+E")
            .with_tags(vec!["micro", "launch", "terminal"]),
        ]
    }

    fn execute_command(&self, command_id: &str, _context: CommandContext) -> Result<CommandResult> {
        match command_id {
            "editor.new_file" => {
                self.launch_editor(None)?;
                Ok(CommandResult::success_with_message("Launched new file in editor".to_string()))
            },
            "editor.open_file" => {
                // TODO: Add file picker dialog
                self.launch_editor(None)?;
                Ok(CommandResult::success_with_message("Launched editor for file selection".to_string()))
            },
            "editor.new_pane" => {
                self.launch_editor_pane(None)?;
                Ok(CommandResult::success_with_message("Launched editor in new pane".to_string()))
            },
            "editor.launch" => {
                self.launch_editor(None)?;
                Ok(CommandResult::success_with_message("Launched Micro editor".to_string()))
            },
            _ => Err(anyhow::anyhow!("Unknown editor command: {}", command_id)),
        }
    }
}