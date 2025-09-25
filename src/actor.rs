use async_trait::async_trait;
use egui;
use uuid::Uuid;
use std::any::Any;
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};

/// Actor API interface for external operations
pub trait ActorAPI {
    /// Get actor type name (e.g., "CodeEditorActor", "TerminalActor")
    fn actor_type(&self) -> String;

    /// Get available API methods for this actor
    fn get_api_methods(&self) -> Vec<ApiMethod>;

    /// Execute an API method with parameters
    fn execute_api_method(&mut self, method: &str, params: ApiParams) -> Result<ApiResult>;

    /// Check if actor can handle a specific method
    fn can_handle_method(&self, method: &str) -> bool {
        self.get_api_methods().iter().any(|m| m.name == method)
    }

    /// Get actor capabilities/features
    fn get_capabilities(&self) -> Vec<String> {
        Vec::new() // Default: no special capabilities
    }

    /// Get actor state for external inspection
    fn get_state(&self) -> HashMap<String, serde_json::Value> {
        HashMap::new() // Default: no exposed state
    }
}

/// API method definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMethod {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ApiParameter>,
    pub return_type: String,
    pub category: String,
}

/// API parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiParameter {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
}

/// Parameters for API method calls
#[derive(Debug, Clone, Default)]
pub struct ApiParams {
    pub params: HashMap<String, serde_json::Value>,
}

impl ApiParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_param<T: Serialize>(mut self, name: &str, value: T) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.params.insert(name.to_string(), json_value);
        }
        self
    }

    pub fn get<T>(&self, name: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let value = self.params.get(name)
            .ok_or_else(|| anyhow!("Parameter '{}' not found", name))?
            .clone();
        serde_json::from_value(value)
            .map_err(|_| anyhow!("Failed to deserialize parameter '{}'", name))
    }

    pub fn get_optional<T>(&self, name: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.get(name).ok()
    }
}

/// Result from API method execution
#[derive(Debug, Clone)]
pub enum ApiResult {
    Success,
    Value(serde_json::Value),
    Error(String),
}

impl<T: Serialize> From<Result<T>> for ApiResult {
    fn from(result: Result<T>) -> Self {
        match result {
            Ok(value) => {
                if let Ok(json) = serde_json::to_value(value) {
                    // Special case for unit type
                    if json == serde_json::Value::Null {
                        ApiResult::Success
                    } else {
                        ApiResult::Value(json)
                    }
                } else {
                    ApiResult::Success
                }
            }
            Err(e) => ApiResult::Error(e.to_string()),
        }
    }
}

/// Actor trait - inspired by Zellij's architecture
/// Each actor is an independent unit that can:
/// - Receive messages
/// - Update its state
/// - Render to egui
/// - Provide API interface
#[async_trait]
pub trait Actor: Send + Sync + ActorAPI {
    /// Unique identifier for this actor
    fn id(&self) -> Uuid;

    /// Human-readable name for this actor
    fn name(&self) -> String;

    /// Handle incoming messages
    async fn handle_message(&mut self, message: ActorMessage) -> anyhow::Result<()>;

    /// Update the actor's state (called each frame)
    fn update(&mut self, ctx: &egui::Context);

    /// Render the actor's UI
    fn render(&mut self, ui: &mut egui::Ui);

    /// Get actor as Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Get mutable actor as Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Messages that can be sent to actors
#[derive(Debug, Clone)]
pub enum ActorMessage {
    /// Focus this actor
    Focus,

    /// Unfocus this actor
    Unfocus,

    /// Resize event
    Resize { width: f32, height: f32 },

    /// Text input
    TextInput(String),

    /// Key event
    KeyEvent { key: egui::Key, modifiers: egui::Modifiers },

    /// Custom message (actor-specific)
    Custom(String, serde_json::Value),
}

/// Manager for actors - tracks and routes messages
pub struct ActorManager {
    pub actors: Vec<Box<dyn Actor>>,
    focused_actor: Option<Uuid>,
}

impl ActorManager {
    pub fn new() -> Self {
        Self {
            actors: Vec::new(),
            focused_actor: None,
        }
    }

    pub fn register_actor(&mut self, actor: Box<dyn Actor>) {
        let id = actor.id();
        self.actors.push(actor);

        // Auto-focus first actor
        if self.focused_actor.is_none() {
            self.focused_actor = Some(id);
        }
    }

    pub fn get_actor(&self, id: Uuid) -> Option<&dyn Actor> {
        self.actors.iter()
            .find(|a| a.id() == id)
            .map(|a| a.as_ref())
    }

    pub fn get_actor_mut(&mut self, id: Uuid) -> Option<&mut Box<dyn Actor>> {
        self.actors.iter_mut()
            .find(|a| a.id() == id)
    }

    pub fn focused_actor(&self) -> Option<Uuid> {
        self.focused_actor
    }

    pub fn set_focus(&mut self, id: Uuid) {
        // For now, just track focus without async messaging
        // We can add proper async handling later
        self.focused_actor = Some(id);
    }

    pub async fn broadcast_message(&mut self, message: ActorMessage) {
        for actor in &mut self.actors {
            let _ = actor.handle_message(message.clone()).await;
        }
    }

    /// Execute API method on specific actor
    pub fn execute_actor_api(&mut self, actor_id: Uuid, method: &str, params: ApiParams) -> Result<ApiResult> {
        if let Some(actor) = self.get_actor_mut(actor_id) {
            actor.execute_api_method(method, params)
        } else {
            Err(anyhow!("Actor with ID {} not found", actor_id))
        }
    }

    /// Get API methods for specific actor
    pub fn get_actor_api_methods(&self, actor_id: Uuid) -> Vec<ApiMethod> {
        if let Some(actor) = self.get_actor(actor_id) {
            actor.get_api_methods()
        } else {
            Vec::new()
        }
    }

    /// Get all actors with their types and capabilities
    pub fn get_actors_info(&self) -> Vec<ActorInfo> {
        self.actors.iter().map(|actor| ActorInfo {
            id: actor.id(),
            name: actor.name(),
            actor_type: actor.actor_type(),
            capabilities: actor.get_capabilities(),
            is_focused: self.focused_actor == Some(actor.id()),
        }).collect()
    }

    /// Find actors by type
    pub fn find_actors_by_type(&self, actor_type: &str) -> Vec<Uuid> {
        self.actors.iter()
            .filter(|actor| actor.actor_type() == actor_type)
            .map(|actor| actor.id())
            .collect()
    }

    /// Find actors with specific capability
    pub fn find_actors_with_capability(&self, capability: &str) -> Vec<Uuid> {
        self.actors.iter()
            .filter(|actor| actor.get_capabilities().contains(&capability.to_string()))
            .map(|actor| actor.id())
            .collect()
    }
}

/// Information about an actor
#[derive(Debug, Clone, Serialize)]
pub struct ActorInfo {
    pub id: Uuid,
    pub name: String,
    pub actor_type: String,
    pub capabilities: Vec<String>,
    pub is_focused: bool,
}