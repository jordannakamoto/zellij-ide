use anyhow::Result;
use std::process::{Command, Stdio};
use std::path::PathBuf;

pub mod commands;

pub struct EditorFeature {
    editor_binary: String,
}

impl EditorFeature {
    pub fn new() -> Self {
        Self {
            editor_binary: "micro".to_string(),
        }
    }

    /// Launch a new editor terminal with micro
    pub fn launch_editor(&self, file_path: Option<PathBuf>) -> Result<()> {
        let mut cmd = Command::new(&self.editor_binary);

        if let Some(path) = file_path {
            cmd.arg(path);
        }

        cmd.stdin(Stdio::inherit())
           .stdout(Stdio::inherit())
           .stderr(Stdio::inherit());

        let mut child = cmd.spawn()?;
        child.wait()?;

        Ok(())
    }

    /// Launch editor in a new terminal pane
    pub fn launch_editor_pane(&self, file_path: Option<PathBuf>) -> Result<()> {
        // For now, we'll use the terminal directly
        // Later we'll integrate with Zellij's pane system
        self.launch_editor(file_path)
    }

}

impl Default for EditorFeature {
    fn default() -> Self {
        Self::new()
    }
}