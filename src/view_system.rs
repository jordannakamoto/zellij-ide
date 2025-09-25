use egui::{self, Vec2, Pos2};
use uuid::Uuid;
use std::any::Any;

/// Generic trait for view management systems
pub trait ViewSystem: Send + Sync {
    /// Create a new view and return its ID
    fn create_view(&mut self, title: String) -> Uuid;

    /// Get the currently active view ID
    fn active_view(&self) -> Option<Uuid>;

    /// Set the active view
    fn set_active_view(&mut self, view_id: Uuid) -> bool;

    /// Attach an actor to a view
    fn attach_actor_to_view(&mut self, view_id: Uuid, actor_id: Uuid) -> bool;

    /// Detach actor from a view
    fn detach_actor_from_view(&mut self, view_id: Uuid) -> bool;

    /// Get the actor ID for a view
    fn get_view_actor(&self, view_id: Uuid) -> Option<Uuid>;

    /// Render the view system
    fn render_views(&mut self, ui: &mut egui::Ui, render_actor: &dyn Fn(&mut egui::Ui, Uuid));

    /// Handle input for the view system
    fn handle_input(&mut self, ui: &mut egui::Ui) -> bool;

    /// Get view count
    fn view_count(&self) -> usize;

    /// Remove a view
    fn remove_view(&mut self, view_id: Uuid) -> bool;

    /// Get all view IDs
    fn get_view_ids(&self) -> Vec<Uuid>;

    /// Cast to Any for downcasting to specific view system types
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Transform operations that can be applied to views
pub trait Transformable {
    /// Apply zoom with optional center point
    fn zoom(&mut self, factor: f32, center: Option<Pos2>);

    /// Pan by a delta
    fn pan(&mut self, delta: Vec2);

    /// Reset transform to default
    fn reset_transform(&mut self);

    /// Get current transform state for display
    fn get_transform_info(&self) -> TransformInfo;
}

/// Information about current transform state
#[derive(Debug, Clone)]
pub struct TransformInfo {
    pub zoom: f32,
    pub pan_offset: Vec2,
    pub can_zoom: bool,
    pub can_pan: bool,
}

/// Input handling configuration
#[derive(Debug, Clone)]
pub struct InputConfig {
    pub enable_zoom: bool,
    pub enable_pan: bool,
    pub axis_locked_pan: bool,
    pub zoom_with_pinch: bool,
    pub pan_with_scroll: bool,
    pub pan_with_drag: bool,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            enable_zoom: true,
            enable_pan: true,
            axis_locked_pan: true,
            zoom_with_pinch: true,
            pan_with_scroll: true,
            pan_with_drag: true,
        }
    }
}

/// Generic view container that can hold different view system implementations
pub struct ViewContainer {
    system: Box<dyn ViewSystem>,
    input_config: InputConfig,
}

impl ViewContainer {
    pub fn new(system: Box<dyn ViewSystem>) -> Self {
        Self {
            system,
            input_config: InputConfig::default(),
        }
    }

    pub fn with_input_config(mut self, config: InputConfig) -> Self {
        self.input_config = config;
        self
    }

    pub fn system(&self) -> &dyn ViewSystem {
        self.system.as_ref()
    }

    pub fn system_mut(&mut self) -> &mut dyn ViewSystem {
        self.system.as_mut()
    }

    pub fn input_config(&self) -> &InputConfig {
        &self.input_config
    }

    pub fn set_input_config(&mut self, config: InputConfig) {
        self.input_config = config;
    }

    /// Try to cast to a specific view system type
    pub fn as_system<T: 'static>(&self) -> Option<&T> {
        self.system.as_any().downcast_ref::<T>()
    }

    /// Try to cast to a specific view system type (mutable)
    pub fn as_system_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.system.as_any_mut().downcast_mut::<T>()
    }
}