use crate::view_system::{ViewSystem, Transformable, TransformInfo};
use egui::{self, Vec2, Pos2};
use uuid::Uuid;
use std::collections::HashMap;
use std::any::Any;

/// Transform state for pan and zoom
#[derive(Debug, Clone)]
pub struct SceneTransform {
    /// Zoom level (1.0 = normal, >1.0 = zoomed in, <1.0 = zoomed out)
    pub zoom: f32,
    /// Pan offset in scene coordinates
    pub offset: Vec2,
    /// Minimum zoom level
    pub min_zoom: f32,
    /// Maximum zoom level
    pub max_zoom: f32,
}

impl Default for SceneTransform {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            offset: Vec2::ZERO,
            min_zoom: 0.1,
            max_zoom: 10.0,
        }
    }
}

impl SceneTransform {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply zoom change with fixed pan position
    pub fn zoom_fixed_pan(&mut self, zoom_delta: f32) {
        self.zoom = (self.zoom * zoom_delta).clamp(self.min_zoom, self.max_zoom);
    }

    /// Pan with axis locking - restricts to one axis unless free_pan is true
    pub fn pan_with_axis_lock(&mut self, delta: Vec2, free_pan: bool) {
        if free_pan {
            // Free pan mode: allow movement in both X and Y directions simultaneously
            self.offset += delta;
        } else {
            // Axis-locked pan mode: restrict movement to the dominant axis
            if delta.x.abs() > delta.y.abs() {
                self.offset += Vec2::new(delta.x, 0.0);
            } else {
                self.offset += Vec2::new(0.0, delta.y);
            }
        }
    }

    /// Reset transform to default
    pub fn reset(&mut self) {
        self.zoom = 1.0;
        self.offset = Vec2::ZERO;
    }
}

/// Scene-based view
#[derive(Debug, Clone)]
pub struct SceneView {
    pub id: Uuid,
    pub title: String,
    pub transform: SceneTransform,
    pub actor_id: Option<Uuid>,
}

impl SceneView {
    pub fn new(title: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            transform: SceneTransform::new(),
            actor_id: None,
        }
    }
}

/// Interaction state for scene views
#[derive(Debug, Default)]
struct ViewInteractionState {
    is_dragging: bool,
    last_mouse_pos: Option<Pos2>,
}

/// Scene-based view system implementation
pub struct SceneSystem {
    views: Vec<SceneView>,
    active_view: Option<Uuid>,
    interaction_state: HashMap<Uuid, ViewInteractionState>,
}

impl SceneSystem {
    pub fn new() -> Self {
        let root_view = SceneView::new("Main".to_string());
        let root_id = root_view.id;

        let mut interaction_state = HashMap::new();
        interaction_state.insert(root_id, ViewInteractionState::default());

        Self {
            views: vec![root_view],
            active_view: Some(root_id),
            interaction_state,
        }
    }

    pub fn get_view(&self, id: Uuid) -> Option<&SceneView> {
        self.views.iter().find(|v| v.id == id)
    }

    pub fn get_view_mut(&mut self, id: Uuid) -> Option<&mut SceneView> {
        self.views.iter_mut().find(|v| v.id == id)
    }

    /// Render a specific scene view with input handling
    fn render_scene_view(
        &mut self,
        view_id: Uuid,
        ui: &mut egui::Ui,
        content_renderer: &dyn Fn(&mut egui::Ui, &SceneTransform),
    ) {
        let available_rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(available_rect, egui::Sense::click_and_drag());

        // Check if Cmd key is held for free panning (this locks zoom)
        let cmd_held = ui.ctx().input(|i| i.modifiers.command);

        // Handle pinch gestures for zoom (disabled during free pan mode)
        let zoom_factor = if response.hovered() && !cmd_held {
            let zoom_delta = ui.ctx().input(|i| i.zoom_delta());
            if zoom_delta != 1.0 {
                Some(zoom_delta)
            } else { None }
        } else { None };

        // Handle panning input
        let mut pan_delta = None;

        // Scroll wheel panning (when not zooming and cmd not held)
        // Note: When Cmd is held, we rely on drag panning for free pan
        if response.hovered() && zoom_factor.is_none() && !cmd_held {
            let scroll_delta = ui.ctx().input(|i| i.smooth_scroll_delta);
            if scroll_delta != egui::Vec2::ZERO {
                pan_delta = Some((-scroll_delta, false)); // Always axis-locked for scroll
            }
        }

        // Click and drag panning
        if response.dragged() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                let last_pos = self.interaction_state.get(&view_id)
                    .and_then(|s| s.last_mouse_pos);

                if let Some(last_pos) = last_pos {
                    pan_delta = Some((mouse_pos - last_pos, cmd_held));
                }

                // Update interaction state
                let interaction = self.interaction_state.entry(view_id).or_default();
                interaction.last_mouse_pos = Some(mouse_pos);
                interaction.is_dragging = true;
            }
        } else {
            // Clear interaction state when not dragging
            if let Some(interaction) = self.interaction_state.get_mut(&view_id) {
                interaction.is_dragging = false;
                interaction.last_mouse_pos = None;
            }
        }

        // Apply transformations
        if let Some(view) = self.get_view_mut(view_id) {
            if let Some(zoom_delta) = zoom_factor {
                view.transform.zoom_fixed_pan(zoom_delta);
            }

            if let Some((delta, free_pan)) = pan_delta {
                view.transform.pan_with_axis_lock(delta, free_pan);
            }
        }

        // Get transform for rendering
        let transform = if let Some(view) = self.get_view(view_id) {
            view.transform.clone()
        } else {
            SceneTransform::default()
        };

        // Render content with transform
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(available_rect), |ui| {
            ui.set_clip_rect(available_rect);
            content_renderer(ui, &transform);

            // Transform overlay is now handled by the widget system
        });
    }
}

impl ViewSystem for SceneSystem {
    fn create_view(&mut self, title: String) -> Uuid {
        let view = SceneView::new(title);
        let id = view.id;
        self.views.push(view);
        self.interaction_state.insert(id, ViewInteractionState::default());
        id
    }

    fn active_view(&self) -> Option<Uuid> {
        self.active_view
    }

    fn set_active_view(&mut self, view_id: Uuid) -> bool {
        if self.views.iter().any(|v| v.id == view_id) {
            self.active_view = Some(view_id);
            true
        } else {
            false
        }
    }

    fn attach_actor_to_view(&mut self, view_id: Uuid, actor_id: Uuid) -> bool {
        if let Some(view) = self.get_view_mut(view_id) {
            view.actor_id = Some(actor_id);
            true
        } else {
            false
        }
    }

    fn detach_actor_from_view(&mut self, view_id: Uuid) -> bool {
        if let Some(view) = self.get_view_mut(view_id) {
            view.actor_id = None;
            true
        } else {
            false
        }
    }

    fn get_view_actor(&self, view_id: Uuid) -> Option<Uuid> {
        self.get_view(view_id).and_then(|v| v.actor_id)
    }

    fn render_views(&mut self, ui: &mut egui::Ui, render_actor: &dyn Fn(&mut egui::Ui, Uuid)) {
        if let Some(active_view_id) = self.active_view {
            let actor_id = self.get_view_actor(active_view_id);

            self.render_scene_view(active_view_id, ui, &|ui, transform| {
                if let Some(actor_id) = actor_id {
                    // Apply transform to actor rendering
                    let available_rect = ui.available_rect_before_wrap();
                    let center = available_rect.center();

                    let child_rect = egui::Rect::from_center_size(
                        center + transform.offset,
                        available_rect.size() * transform.zoom
                    );

                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(child_rect), |ui| {
                        render_actor(ui, actor_id);
                    });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Empty Scene");
                    });
                }
            });
        }
    }

    fn handle_input(&mut self, _ui: &mut egui::Ui) -> bool {
        // Input handling is integrated into render_views for scene system
        false
    }

    fn view_count(&self) -> usize {
        self.views.len()
    }

    fn remove_view(&mut self, view_id: Uuid) -> bool {
        if let Some(pos) = self.views.iter().position(|v| v.id == view_id) {
            self.views.remove(pos);
            self.interaction_state.remove(&view_id);

            // If we removed the active view, set a new active view
            if self.active_view == Some(view_id) {
                self.active_view = self.views.first().map(|v| v.id);
            }

            true
        } else {
            false
        }
    }

    fn get_view_ids(&self) -> Vec<Uuid> {
        self.views.iter().map(|v| v.id).collect()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Transformable for SceneSystem {
    fn zoom(&mut self, factor: f32, _center: Option<Pos2>) {
        if let Some(active_view_id) = self.active_view {
            if let Some(view) = self.get_view_mut(active_view_id) {
                view.transform.zoom_fixed_pan(factor);
            }
        }
    }

    fn pan(&mut self, delta: Vec2) {
        if let Some(active_view_id) = self.active_view {
            if let Some(view) = self.get_view_mut(active_view_id) {
                view.transform.pan_with_axis_lock(delta, false);
            }
        }
    }

    fn reset_transform(&mut self) {
        if let Some(active_view_id) = self.active_view {
            if let Some(view) = self.get_view_mut(active_view_id) {
                view.transform.reset();
            }
        }
    }

    fn get_transform_info(&self) -> TransformInfo {
        if let Some(active_view_id) = self.active_view {
            if let Some(view) = self.get_view(active_view_id) {
                return TransformInfo {
                    zoom: view.transform.zoom,
                    pan_offset: view.transform.offset,
                    can_zoom: true,
                    can_pan: true,
                };
            }
        }

        TransformInfo {
            zoom: 1.0,
            pan_offset: Vec2::ZERO,
            can_zoom: false,
            can_pan: false,
        }
    }
}