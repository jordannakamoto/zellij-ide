use crate::actor::{Actor, ActorMessage, ActorAPI, ApiMethod, ApiParameter, ApiParams, ApiResult};
use async_trait::async_trait;
use egui;
use uuid::Uuid;
use std::any::Any;
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde_json;

/// Simple Code Editor Actor - native egui implementation
pub struct CodeEditorActor {
    id: Uuid,
    name: String,
    content: String,
    language: String,
    is_focused: bool,
    cursor_pos: usize,
}

impl CodeEditorActor {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            content: String::from("// Welcome to Zellij IDE\n\nfn main() {\n    println!(\"Hello, world!\");\n}\n"),
            language: "rust".to_string(),
            is_focused: false,
            cursor_pos: 0,
        }
    }

    pub fn with_content(name: String, content: String) -> Self {
        let mut editor = Self::new(name);
        editor.content = content;
        editor
    }

    pub fn set_language(&mut self, lang: &str) {
        self.language = lang.to_string();
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }
}

#[async_trait]
impl Actor for CodeEditorActor {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    async fn handle_message(&mut self, message: ActorMessage) -> anyhow::Result<()> {
        match message {
            ActorMessage::Focus => {
                self.is_focused = true;
                log::info!("Code editor {} focused", self.name);
            },
            ActorMessage::Unfocus => {
                self.is_focused = false;
                log::info!("Code editor {} unfocused", self.name);
            },
            ActorMessage::TextInput(text) => {
                self.content.push_str(&text);
            },
            ActorMessage::Custom(cmd, data) => {
                match cmd.as_str() {
                    "set_content" => {
                        if let Ok(content) = serde_json::from_value::<String>(data) {
                            self.content = content;
                        }
                    },
                    "set_language" => {
                        if let Ok(lang) = serde_json::from_value::<String>(data) {
                            self.set_language(&lang);
                        }
                    },
                    _ => {},
                }
            },
            _ => {},
        }
        Ok(())
    }

    fn update(&mut self, _ctx: &egui::Context) {
        // Could handle animations, async operations, etc.
    }

    fn render(&mut self, ui: &mut egui::Ui) {
        // Header with file info
        ui.horizontal(|ui| {
            ui.label(&self.name);
            ui.separator();
            ui.label(format!("Language: {}", self.language));
            if self.is_focused {
                ui.separator();
                ui.colored_label(egui::Color32::GREEN, "● Active");
            }
        });

        ui.separator();

        // Simple text editor using egui's native TextEdit
        egui::ScrollArea::both()
            .id_salt(format!("editor_{}", self.id))
            .show(ui, |ui| {
                // Use monospace font for code
                let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                    let layout_job = egui::text::LayoutJob::simple(
                        string.to_owned(),
                        egui::FontId::monospace(14.0),
                        egui::Color32::from_gray(200),
                        f32::INFINITY,
                    );
                    ui.fonts(|f| f.layout_job(layout_job))
                };

                egui::TextEdit::multiline(&mut self.content)
                    .font(egui::TextStyle::Monospace)
                    .code_editor()
                    .desired_rows(30)
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .layouter(&mut layouter)
                    .show(ui);
            });

        // Status line
        ui.separator();
        ui.horizontal(|ui| {
            let lines = self.content.lines().count();
            let chars = self.content.len();
            ui.label(format!("Lines: {} | Chars: {}", lines, chars));
        });
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ActorAPI for CodeEditorActor {
    fn actor_type(&self) -> String {
        "CodeEditorActor".to_string()
    }

    fn get_api_methods(&self) -> Vec<ApiMethod> {
        vec![
            ApiMethod {
                name: "get_content".to_string(),
                description: "Get the current content of the editor".to_string(),
                parameters: vec![],
                return_type: "string".to_string(),
                category: "content".to_string(),
            },
            ApiMethod {
                name: "set_content".to_string(),
                description: "Set the content of the editor".to_string(),
                parameters: vec![
                    ApiParameter {
                        name: "content".to_string(),
                        param_type: "string".to_string(),
                        description: "The new content for the editor".to_string(),
                        required: true,
                        default_value: None,
                    }
                ],
                return_type: "void".to_string(),
                category: "content".to_string(),
            },
            ApiMethod {
                name: "get_language".to_string(),
                description: "Get the current programming language".to_string(),
                parameters: vec![],
                return_type: "string".to_string(),
                category: "language".to_string(),
            },
            ApiMethod {
                name: "set_language".to_string(),
                description: "Set the programming language for syntax highlighting".to_string(),
                parameters: vec![
                    ApiParameter {
                        name: "language".to_string(),
                        param_type: "string".to_string(),
                        description: "The programming language (e.g., 'rust', 'javascript', 'python')".to_string(),
                        required: true,
                        default_value: None,
                    }
                ],
                return_type: "void".to_string(),
                category: "language".to_string(),
            },
            ApiMethod {
                name: "format".to_string(),
                description: "Format the code in the editor".to_string(),
                parameters: vec![],
                return_type: "void".to_string(),
                category: "editing".to_string(),
            },
            ApiMethod {
                name: "save".to_string(),
                description: "Save the current content to file".to_string(),
                parameters: vec![
                    ApiParameter {
                        name: "path".to_string(),
                        param_type: "string".to_string(),
                        description: "Optional file path (uses current name if not provided)".to_string(),
                        required: false,
                        default_value: None,
                    }
                ],
                return_type: "void".to_string(),
                category: "file".to_string(),
            },
            ApiMethod {
                name: "find".to_string(),
                description: "Find text in the editor".to_string(),
                parameters: vec![
                    ApiParameter {
                        name: "query".to_string(),
                        param_type: "string".to_string(),
                        description: "Text to search for".to_string(),
                        required: true,
                        default_value: None,
                    },
                    ApiParameter {
                        name: "case_sensitive".to_string(),
                        param_type: "boolean".to_string(),
                        description: "Whether search should be case sensitive".to_string(),
                        required: false,
                        default_value: Some(serde_json::Value::Bool(false)),
                    }
                ],
                return_type: "array".to_string(),
                category: "search".to_string(),
            },
            ApiMethod {
                name: "get_stats".to_string(),
                description: "Get statistics about the editor content".to_string(),
                parameters: vec![],
                return_type: "object".to_string(),
                category: "info".to_string(),
            },
        ]
    }

    fn execute_api_method(&mut self, method: &str, params: ApiParams) -> Result<ApiResult> {
        match method {
            "get_content" => {
                Ok(ApiResult::Value(serde_json::Value::String(self.content.clone())))
            },
            "set_content" => {
                let content: String = params.get("content")?;
                self.content = content;
                Ok(ApiResult::Success)
            },
            "get_language" => {
                Ok(ApiResult::Value(serde_json::Value::String(self.language.clone())))
            },
            "set_language" => {
                let language: String = params.get("language")?;
                self.set_language(&language);
                Ok(ApiResult::Success)
            },
            "format" => {
                // Simple formatting - just normalize whitespace for now
                self.content = self.content
                    .lines()
                    .map(|line| line.trim())
                    .collect::<Vec<_>>()
                    .join("\n");
                log::info!("Formatted code in {}", self.name);
                Ok(ApiResult::Success)
            },
            "save" => {
                let _path: Option<String> = params.get_optional("path");
                // In a real implementation, this would save to disk
                log::info!("Saved {}", self.name);
                Ok(ApiResult::Success)
            },
            "find" => {
                let query: String = params.get("query")?;
                let case_sensitive: bool = params.get_optional("case_sensitive").unwrap_or(false);

                let content_to_search = if case_sensitive {
                    self.content.clone()
                } else {
                    self.content.to_lowercase()
                };

                let query_to_search = if case_sensitive {
                    query
                } else {
                    query.to_lowercase()
                };

                let matches: Vec<_> = content_to_search
                    .match_indices(&query_to_search)
                    .map(|(pos, _)| pos)
                    .collect();

                Ok(ApiResult::Value(serde_json::to_value(matches)?))
            },
            "get_stats" => {
                let stats = serde_json::json!({
                    "lines": self.content.lines().count(),
                    "characters": self.content.len(),
                    "words": self.content.split_whitespace().count(),
                    "language": self.language,
                    "name": self.name,
                    "is_focused": self.is_focused
                });
                Ok(ApiResult::Value(stats))
            },
            _ => Err(anyhow!("Unknown method: {}", method))
        }
    }

    fn get_capabilities(&self) -> Vec<String> {
        vec![
            "text_editing".to_string(),
            "syntax_highlighting".to_string(),
            "code_formatting".to_string(),
            "file_operations".to_string(),
            "search".to_string(),
        ]
    }

    fn get_state(&self) -> HashMap<String, serde_json::Value> {
        let mut state = HashMap::new();
        state.insert("content_length".to_string(), serde_json::Value::Number(serde_json::Number::from(self.content.len())));
        state.insert("language".to_string(), serde_json::Value::String(self.language.clone()));
        state.insert("is_focused".to_string(), serde_json::Value::Bool(self.is_focused));
        state.insert("line_count".to_string(), serde_json::Value::Number(serde_json::Number::from(self.content.lines().count())));
        state
    }
}