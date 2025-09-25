use egui::{self, Vec2, Pos2, Rect};
use uuid::Uuid;
use std::collections::HashMap;

/// Scene-based view that supports pan and zoom
#[derive(Debug, Clone)]
pub struct SceneView {
    pub id: Uuid,
    pub title: String,
    pub transform: SceneTransform,
    pub actor_id: Option<Uuid>,
    pub bounds: Option<Rect>, // Optional bounds for the scene
}

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
        // Pan position remains unchanged - no offset adjustment
    }

    /// Pan the view by a delta in viewport coordinates
    pub fn pan(&mut self, delta: Vec2) {
        self.offset += delta;
    }

    /// Pan with axis locking - restricts to one axis unless free_pan is true
    pub fn pan_with_axis_lock(&mut self, delta: Vec2, free_pan: bool) {
        if free_pan {
            // Free panning when Cmd is held
            self.offset += delta;
        } else {
            // Axis-locked panning - choose the dominant axis
            if delta.x.abs() > delta.y.abs() {
                // Horizontal movement is dominant
                self.offset += Vec2::new(delta.x, 0.0);
            } else {
                // Vertical movement is dominant
                self.offset += Vec2::new(0.0, delta.y);
            }
        }
    }

    /// Transform a point from scene coordinates to viewport coordinates
    pub fn scene_to_viewport(&self, scene_pos: Pos2, viewport_center: Pos2) -> Pos2 {
        let transformed = scene_pos.to_vec2() * self.zoom + self.offset;
        (transformed + viewport_center.to_vec2()).to_pos2()
    }

    /// Transform a point from viewport coordinates to scene coordinates
    pub fn viewport_to_scene(&self, viewport_pos: Pos2, viewport_center: Pos2) -> Pos2 {
        let relative_to_center = viewport_pos - viewport_center;
        let scene_coords = (relative_to_center - self.offset) / self.zoom;
        scene_coords.to_pos2()
    }

    /// Get the visible scene rectangle in scene coordinates
    pub fn visible_scene_rect(&self, viewport_rect: Rect) -> Rect {
        let top_left = self.viewport_to_scene(viewport_rect.min, viewport_rect.center());
        let bottom_right = self.viewport_to_scene(viewport_rect.max, viewport_rect.center());
        Rect::from_min_max(top_left, bottom_right)
    }

    /// Reset transform to default
    pub fn reset(&mut self) {
        self.zoom = 1.0;
        self.offset = Vec2::ZERO;
    }
}

/// Manager for scene-based views with pan/zoom
pub struct SceneManager {
    pub views: Vec<SceneView>,
    pub active_view: Option<Uuid>,
    /// Per-view interaction state
    interaction_state: HashMap<Uuid, ViewInteractionState>,
}

#[derive(Debug)]
struct ViewInteractionState {
    is_dragging: bool,
    drag_start: Option<Pos2>,
    last_mouse_pos: Option<Pos2>,
}

impl Default for ViewInteractionState {
    fn default() -> Self {
        Self {
            is_dragging: false,
            drag_start: None,
            last_mouse_pos: None,
        }
    }
}

impl SceneManager {
    pub fn new() -> Self {
        let root_view = SceneView {
            id: Uuid::new_v4(),
            title: "Main".to_string(),
            transform: SceneTransform::new(),
            actor_id: None,
            bounds: None,
        };

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

    pub fn set_active_view(&mut self, id: Uuid) {
        self.active_view = Some(id);
    }

    pub fn attach_actor_to_view(&mut self, view_id: Uuid, actor_id: Uuid) {
        if let Some(view) = self.get_view_mut(view_id) {
            view.actor_id = Some(actor_id);
        }
    }

    /// Render a scene view with pan/zoom controls
    pub fn render_scene_view(
        &mut self,
        view_id: Uuid,
        ui: &mut egui::Ui,
        content_renderer: impl FnOnce(&mut egui::Ui, &SceneTransform),
    ) {
        let available_rect = ui.available_rect_before_wrap();

        // Handle input for pan and zoom
        let response = ui.allocate_rect(available_rect, egui::Sense::click_and_drag());

        // Handle pinch gestures for zoom
        let zoom_factor = if response.hovered() {
            let zoom_delta = ui.ctx().input(|i| i.zoom_delta());
            if zoom_delta != 1.0 {
                Some((zoom_delta, response.hover_pos()))
            } else { None }
        } else { None };

        // Handle both scroll wheel and click-drag for panning
        let mut pan_delta = None;

        // Check if Cmd key is held for free panning
        let cmd_held = ui.ctx().input(|i| i.modifiers.command);

        // Only handle scroll wheel panning if NOT zooming (to avoid conflict with pinch)
        if response.hovered() && zoom_factor.is_none() {
            let scroll_delta = ui.ctx().input(|i| i.smooth_scroll_delta);
            if scroll_delta != egui::Vec2::ZERO {
                // Invert scroll direction for natural panning
                pan_delta = Some((-scroll_delta, None, cmd_held));
            }
        }

        // Click and drag panning (overrides scroll wheel if dragging)
        if response.dragged() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                // Get last mouse position
                let last_pos = self.interaction_state.get(&view_id)
                    .and_then(|s| s.last_mouse_pos);

                if let Some(last_pos) = last_pos {
                    pan_delta = Some((mouse_pos - last_pos, Some(mouse_pos), cmd_held));
                } else {
                    pan_delta = Some((egui::Vec2::ZERO, Some(mouse_pos), cmd_held));
                }
            }
        }

        // Now apply changes to view and interaction state
        if let Some(view) = self.get_view_mut(view_id) {
            // Apply zoom with fixed pan position
            if let Some((factor, _)) = zoom_factor {
                view.transform.zoom_fixed_pan(factor);
            }

            // Apply pan with axis locking
            if let Some((delta, _, free_pan)) = pan_delta {
                view.transform.pan_with_axis_lock(delta, free_pan);
            }
        }

        // Update interaction state for click-drag
        let interaction = self.interaction_state.entry(view_id).or_default();
        if let Some((_, Some(mouse_pos), _)) = pan_delta {
            interaction.last_mouse_pos = Some(mouse_pos);
            interaction.is_dragging = true;
        } else if !response.dragged() {
            interaction.is_dragging = false;
            interaction.last_mouse_pos = None;
        }

        // Get transform for content rendering
        let transform = if let Some(view) = self.get_view(view_id) {
            view.transform.clone()
        } else {
            SceneTransform::default()
        };

        // Render content with transform
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(available_rect), |ui| {
            // Apply clipping to the view area
            ui.set_clip_rect(available_rect);

            // Render content with transform applied
            content_renderer(ui, &transform);

            // Overlay controls (always in viewport coordinates)
            ui.allocate_new_ui(
                egui::UiBuilder::new().max_rect(
                    Rect::from_min_size(available_rect.min, Vec2::new(200.0, 60.0))
                ),
                |ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_black_alpha(100))
                        .rounding(4.0)
                        .inner_margin(6.0)
                        .show(ui, |ui| {
                            ui.small(format!("Zoom: {:.1}x", transform.zoom));
                            ui.small(format!("Pan: ({:.0}, {:.0})", transform.offset.x, transform.offset.y));
                            // Show panning mode
                            let cmd_held = ui.ctx().input(|i| i.modifiers.command);
                            if cmd_held {
                                ui.small("Free Pan (⌘)");
                            } else {
                                ui.small("Axis-Locked Pan");
                            }
                            if ui.small_button("Reset").clicked() {
                                if let Some(view) = self.get_view_mut(view_id) {
                                    view.transform.reset();
                                }
                            }
                        });
                }
            );
        });
    }

    /// Get the transform for a specific view (useful for custom rendering)
    pub fn get_transform(&self, view_id: Uuid) -> Option<&SceneTransform> {
        self.get_view(view_id).map(|v| &v.transform)
    }
}