use crate::actor::ActorManager;
use egui;
use uuid::Uuid;
use std::collections::HashMap;

/// View primitive - can house actors and be arranged in panels/tabs/windows
#[derive(Clone)]
pub struct View {
    pub id: Uuid,
    pub title: String,
    pub view_type: ViewType,
    pub actor_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub enum ViewType {
    /// A single view containing an actor
    Leaf,

    /// A tab container with multiple views
    Tabs {
        children: Vec<Uuid>,
        active_tab: usize,
    },

    /// Split container (like Zellij's panes)
    Split {
        direction: SplitDirection,
        children: Vec<Uuid>,
        ratios: Vec<f32>,
    },

    /// Floating window
    Floating {
        position: egui::Pos2,
        size: egui::Vec2,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum SplitDirection {
    Horizontal, // Split left/right
    Vertical,   // Split top/bottom
}

/// Resize state for tracking active resize operations
#[derive(Debug, Clone)]
struct ResizeState {
    pub view_id: Uuid,
    pub resize_type: ResizeType,
    pub start_pos: egui::Pos2,
    pub original_size: egui::Vec2,
}

#[derive(Debug, Clone, Copy)]
enum ResizeType {
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BorderEdge {
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Manages the view hierarchy
pub struct ViewManager {
    pub views: Vec<View>,
    root_view: Uuid,
    active_view: Uuid,
    view_sizes: HashMap<Uuid, egui::Vec2>,
    active_resize: Option<ResizeState>,
}

impl ViewManager {
    pub fn new() -> Self {
        let root_view = View {
            id: Uuid::new_v4(),
            title: "Main".to_string(),
            view_type: ViewType::Leaf,
            actor_id: None,
        };

        let root_id = root_view.id;

        Self {
            views: vec![root_view],
            root_view: root_id,
            active_view: root_id,
            view_sizes: HashMap::new(),
            active_resize: None,
        }
    }

    pub fn get_view(&self, id: Uuid) -> Option<&View> {
        self.views.iter().find(|v| v.id == id)
    }

    pub fn get_view_mut(&mut self, id: Uuid) -> Option<&mut View> {
        self.views.iter_mut().find(|v| v.id == id)
    }

    pub fn active_view(&self) -> Uuid {
        self.active_view
    }

    pub fn set_active_view(&mut self, id: Uuid) {
        self.active_view = id;
    }

    pub fn attach_actor_to_view(&mut self, view_id: Uuid, actor_id: Uuid) {
        if let Some(view) = self.get_view_mut(view_id) {
            view.actor_id = Some(actor_id);
        }
    }

    pub fn split_view(&mut self, view_id: Uuid, direction: SplitDirection) -> Option<Uuid> {
        // Find the view to split
        let view_idx = self.views.iter().position(|v| v.id == view_id)?;

        // Create two new child views
        let child1 = View {
            id: Uuid::new_v4(),
            title: "Split 1".to_string(),
            view_type: ViewType::Leaf,
            actor_id: self.views[view_idx].actor_id,
        };

        let child2 = View {
            id: Uuid::new_v4(),
            title: "Split 2".to_string(),
            view_type: ViewType::Leaf,
            actor_id: None,
        };

        let child1_id = child1.id;
        let child2_id = child2.id;

        // Add children to view list
        self.views.push(child1);
        self.views.push(child2);

        // Convert parent to split container
        if let Some(parent) = self.get_view_mut(view_id) {
            parent.view_type = ViewType::Split {
                direction,
                children: vec![child1_id, child2_id],
                ratios: vec![0.5, 0.5],
            };
            parent.actor_id = None;
        }

        Some(child2_id)
    }

    pub fn render(&mut self, ui: &mut egui::Ui, actors: &mut ActorManager) {
        let root_view = self.root_view;
        self.render_view(root_view, ui, actors);
    }

    fn render_view_with_borders<F>(&mut self, ui: &mut egui::Ui, view_id: Uuid, is_active: bool, content_renderer: F)
    where
        F: FnOnce(&mut Self, &mut egui::Ui),
    {
        let border_width = 8.0; // Invisible border width for resize detection
        let available_rect = ui.available_rect_before_wrap();

        // Get or initialize view size
        let current_size = self.view_sizes.get(&view_id).copied().unwrap_or(available_rect.size());

        // Update stored size if it has changed significantly
        if (current_size - available_rect.size()).length() > 1.0 {
            self.view_sizes.insert(view_id, available_rect.size());
        }

        // Create resize areas around the content
        let content_rect = available_rect.shrink(border_width);

        // Handle resize interactions on borders
        let mut hovered_edges = Vec::new();  // Track which edges are being hovered

        // Check for active resize or start new resize
        if let Some(resize_state) = &self.active_resize {
            if resize_state.view_id == view_id {
                // Handle ongoing resize
                let current_pos = ui.ctx().pointer_latest_pos().unwrap_or(resize_state.start_pos);
                let delta = current_pos - resize_state.start_pos;

                // Calculate new size based on resize type
                let mut new_size = resize_state.original_size;
                match resize_state.resize_type {
                    ResizeType::Right | ResizeType::TopRight | ResizeType::BottomRight => {
                        new_size.x = (resize_state.original_size.x + delta.x).max(50.0);
                    }
                    ResizeType::Left | ResizeType::TopLeft | ResizeType::BottomLeft => {
                        new_size.x = (resize_state.original_size.x - delta.x).max(50.0);
                    }
                    _ => {}
                }
                match resize_state.resize_type {
                    ResizeType::Bottom | ResizeType::BottomLeft | ResizeType::BottomRight => {
                        new_size.y = (resize_state.original_size.y + delta.y).max(50.0);
                    }
                    ResizeType::Top | ResizeType::TopLeft | ResizeType::TopRight => {
                        new_size.y = (resize_state.original_size.y - delta.y).max(50.0);
                    }
                    _ => {}
                }

                // Update view size
                self.view_sizes.insert(view_id, new_size);

                // End resize if mouse released
                if !ui.ctx().input(|i| i.pointer.primary_down()) {
                    self.active_resize = None;
                }
            }
        } else {
            // Check for new resize starts
            macro_rules! handle_resize_border {
                ($border:expr, $sense:expr, $cursor:expr, $resize_type:expr, $edge_name:expr) => {
                    let response = ui.allocate_rect($border, $sense);
                    if response.hovered() {
                        ui.ctx().set_cursor_icon($cursor);
                        hovered_edges.push($edge_name);
                    }
                    if response.drag_started() {
                        self.active_resize = Some(ResizeState {
                            view_id,
                            resize_type: $resize_type,
                            start_pos: ui.ctx().pointer_latest_pos().unwrap_or(egui::Pos2::ZERO),
                            original_size: current_size,
                        });
                    }
                };
            }

            // Top border
            let top_border = egui::Rect::from_min_max(
                available_rect.min,
                egui::pos2(available_rect.max.x, available_rect.min.y + border_width)
            );
            handle_resize_border!(top_border, egui::Sense::drag(), egui::CursorIcon::ResizeVertical, ResizeType::Top, BorderEdge::Top);

            // Bottom border
            let bottom_border = egui::Rect::from_min_max(
                egui::pos2(available_rect.min.x, available_rect.max.y - border_width),
                available_rect.max
            );
            handle_resize_border!(bottom_border, egui::Sense::drag(), egui::CursorIcon::ResizeVertical, ResizeType::Bottom, BorderEdge::Bottom);

            // Left border
            let left_border = egui::Rect::from_min_max(
                available_rect.min,
                egui::pos2(available_rect.min.x + border_width, available_rect.max.y)
            );
            handle_resize_border!(left_border, egui::Sense::drag(), egui::CursorIcon::ResizeHorizontal, ResizeType::Left, BorderEdge::Left);

            // Right border
            let right_border = egui::Rect::from_min_max(
                egui::pos2(available_rect.max.x - border_width, available_rect.min.y),
                available_rect.max
            );
            handle_resize_border!(right_border, egui::Sense::drag(), egui::CursorIcon::ResizeHorizontal, ResizeType::Right, BorderEdge::Right);

            // Corner resize handles
            let tl_corner = egui::Rect::from_min_size(available_rect.min, egui::vec2(border_width, border_width));
            handle_resize_border!(tl_corner, egui::Sense::drag(), egui::CursorIcon::ResizeNwSe, ResizeType::TopLeft, BorderEdge::TopLeft);

            let tr_corner = egui::Rect::from_min_size(
                egui::pos2(available_rect.max.x - border_width, available_rect.min.y),
                egui::vec2(border_width, border_width)
            );
            handle_resize_border!(tr_corner, egui::Sense::drag(), egui::CursorIcon::ResizeNeSw, ResizeType::TopRight, BorderEdge::TopRight);

            let bl_corner = egui::Rect::from_min_size(
                egui::pos2(available_rect.min.x, available_rect.max.y - border_width),
                egui::vec2(border_width, border_width)
            );
            handle_resize_border!(bl_corner, egui::Sense::drag(), egui::CursorIcon::ResizeNeSw, ResizeType::BottomLeft, BorderEdge::BottomLeft);

            let br_corner = egui::Rect::from_min_size(
                egui::pos2(available_rect.max.x - border_width, available_rect.max.y - border_width),
                egui::vec2(border_width, border_width)
            );
            handle_resize_border!(br_corner, egui::Sense::drag(), egui::CursorIcon::ResizeNwSe, ResizeType::BottomRight, BorderEdge::BottomRight);
        }

        // Visual feedback for borders
        let border_stroke_width = 2.0;
        let hover_color = egui::Color32::from_rgb(100, 150, 255);
        let resize_color = egui::Color32::from_rgb(50, 200, 100);
        let active_color = egui::Color32::from_rgba_unmultiplied(100, 150, 255, 60);

        // Show full border when actively resizing
        if let Some(resize_state) = &self.active_resize {
            if resize_state.view_id == view_id {
                ui.painter().rect_stroke(
                    available_rect.shrink(1.0),
                    egui::Rounding::same(2.0),
                    egui::Stroke::new(border_stroke_width, resize_color)
                );
            }
        }
        // Show edge borders on hover (only when not resizing)
        else if !hovered_edges.is_empty() {
            let inner_rect = available_rect.shrink(1.0);

            for edge in &hovered_edges {
                match edge {
                    BorderEdge::Top => {
                        ui.painter().line_segment(
                            [inner_rect.left_top(), inner_rect.right_top()],
                            egui::Stroke::new(border_stroke_width, hover_color)
                        );
                    },
                    BorderEdge::Bottom => {
                        ui.painter().line_segment(
                            [inner_rect.left_bottom(), inner_rect.right_bottom()],
                            egui::Stroke::new(border_stroke_width, hover_color)
                        );
                    },
                    BorderEdge::Left => {
                        ui.painter().line_segment(
                            [inner_rect.left_top(), inner_rect.left_bottom()],
                            egui::Stroke::new(border_stroke_width, hover_color)
                        );
                    },
                    BorderEdge::Right => {
                        ui.painter().line_segment(
                            [inner_rect.right_top(), inner_rect.right_bottom()],
                            egui::Stroke::new(border_stroke_width, hover_color)
                        );
                    },
                    BorderEdge::TopLeft | BorderEdge::TopRight | BorderEdge::BottomLeft | BorderEdge::BottomRight => {
                        // For corner hovers, show the full border
                        ui.painter().rect_stroke(
                            inner_rect,
                            egui::Rounding::same(2.0),
                            egui::Stroke::new(border_stroke_width, hover_color)
                        );
                    },
                }
            }
        }
        // Subtle border for active view when not hovering or resizing
        else if is_active {
            ui.painter().rect_stroke(
                available_rect.shrink(1.0),
                egui::Rounding::same(2.0),
                egui::Stroke::new(1.0, active_color)
            );
        }

        // Render the content in the inner area
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(content_rect), |ui| {
            content_renderer(self, ui);
        });
    }

    fn render_view(&mut self, view_id: Uuid, ui: &mut egui::Ui, actors: &mut ActorManager) {
        let Some(view) = self.get_view(view_id).cloned() else { return };
        let is_active = view_id == self.active_view;

        // Apply universal border rendering to all view types
        self.render_view_with_borders(ui, view_id, is_active, |view_manager, ui| {
            match view.view_type {
                ViewType::Leaf => {
                    // Render the actor content
                    if let Some(actor_id) = view.actor_id {
                        if let Some(actor) = actors.get_actor_mut(actor_id) {
                            actor.render(ui);
                        } else {
                            ui.centered_and_justified(|ui| {
                                ui.label("Empty View");
                            });
                        }
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("Empty View - No Actor");
                        });
                    }
                },

                ViewType::Split { direction, children, ratios } => {
                    match direction {
                        SplitDirection::Horizontal => {
                            ui.horizontal(|ui| {
                                for (i, &child_id) in children.iter().enumerate() {
                                    let width = ui.available_width() * ratios.get(i).unwrap_or(&0.5);
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(width, ui.available_height()),
                                        egui::Layout::default(),
                                        |ui| view_manager.render_view(child_id, ui, actors)
                                    );

                                    if i < children.len() - 1 {
                                        ui.separator();
                                    }
                                }
                            });
                        },
                        SplitDirection::Vertical => {
                            ui.vertical(|ui| {
                                for (i, &child_id) in children.iter().enumerate() {
                                    let height = ui.available_height() * ratios.get(i).unwrap_or(&0.5);
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(ui.available_width(), height),
                                        egui::Layout::default(),
                                        |ui| view_manager.render_view(child_id, ui, actors)
                                    );

                                    if i < children.len() - 1 {
                                        ui.separator();
                                    }
                                }
                            });
                        },
                    }
                },

                ViewType::Tabs { children, active_tab } => {
                    ui.vertical(|ui| {
                        // Tab bar
                        ui.horizontal(|ui| {
                            for (i, &child_id) in children.iter().enumerate() {
                                let child = view_manager.get_view(child_id);
                                let title = child.map(|v| v.title.as_str()).unwrap_or("Tab");

                                if ui.selectable_label(i == active_tab, title).clicked() {
                                    // Would need mutable access to change active_tab
                                }
                            }
                        });

                        ui.separator();

                        // Active tab content
                        if let Some(&active_child) = children.get(active_tab) {
                            view_manager.render_view(active_child, ui, actors);
                        }
                    });
                },

                ViewType::Floating { position, size: _ } => {
                    // Floating windows - for now just show placeholder
                    ui.centered_and_justified(|ui| {
                        ui.label(format!("Floating window at {:?}", position));
                    });
                },
            }
        });
    }
}