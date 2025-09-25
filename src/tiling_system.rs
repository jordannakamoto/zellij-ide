use crate::view_system::{ViewSystem, Transformable, TransformInfo};
use egui::{self, Vec2, Pos2, Rect};
use uuid::Uuid;
use std::collections::HashMap;
use std::any::Any;

/// Tiling layout direction
#[derive(Debug, Clone, Copy)]
pub enum TileDirection {
    Horizontal,
    Vertical,
}

/// A tile in the tiling system
#[derive(Debug, Clone)]
pub struct Tile {
    pub id: Uuid,
    pub title: String,
    pub actor_id: Option<Uuid>,
    pub rect: Rect,
    pub is_focused: bool,
}

impl Tile {
    pub fn new(title: String, rect: Rect) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            actor_id: None,
            rect,
            is_focused: false,
        }
    }
}

/// Tiling-based view system (placeholder implementation)
/// This demonstrates how we can have different view management approaches
pub struct TilingSystem {
    tiles: HashMap<Uuid, Tile>,
    active_tile: Option<Uuid>,
    layout_direction: TileDirection,
    tile_margin: f32,
}

impl TilingSystem {
    pub fn new() -> Self {
        let mut system = Self {
            tiles: HashMap::new(),
            active_tile: None,
            layout_direction: TileDirection::Horizontal,
            tile_margin: 4.0,
        };

        // Create initial tile
        let initial_id = system.create_view("Main".to_string());
        system.set_active_view(initial_id);

        system
    }

    pub fn set_layout_direction(&mut self, direction: TileDirection) {
        self.layout_direction = direction;
    }

    pub fn set_tile_margin(&mut self, margin: f32) {
        self.tile_margin = margin;
    }

    fn recalculate_layout(&mut self, available_rect: Rect) {
        let tile_count = self.tiles.len();
        if tile_count == 0 {
            return;
        }

        let total_margin = self.tile_margin * (tile_count as f32 - 1.0);

        let tiles: Vec<_> = self.tiles.values_mut().collect();

        match self.layout_direction {
            TileDirection::Horizontal => {
                let tile_width = (available_rect.width() - total_margin) / tile_count as f32;
                for (i, tile) in tiles.into_iter().enumerate() {
                    let x = available_rect.min.x + (tile_width + self.tile_margin) * i as f32;
                    tile.rect = Rect::from_min_size(
                        Pos2::new(x, available_rect.min.y),
                        Vec2::new(tile_width, available_rect.height())
                    );
                }
            }
            TileDirection::Vertical => {
                let tile_height = (available_rect.height() - total_margin) / tile_count as f32;
                for (i, tile) in tiles.into_iter().enumerate() {
                    let y = available_rect.min.y + (tile_height + self.tile_margin) * i as f32;
                    tile.rect = Rect::from_min_size(
                        Pos2::new(available_rect.min.x, y),
                        Vec2::new(available_rect.width(), tile_height)
                    );
                }
            }
        }
    }

    pub fn get_tile(&self, id: Uuid) -> Option<&Tile> {
        self.tiles.get(&id)
    }

    pub fn get_tile_mut(&mut self, id: Uuid) -> Option<&mut Tile> {
        self.tiles.get_mut(&id)
    }
}

impl ViewSystem for TilingSystem {
    fn create_view(&mut self, title: String) -> Uuid {
        let tile = Tile::new(title, Rect::ZERO);
        let id = tile.id;
        self.tiles.insert(id, tile);
        id
    }

    fn active_view(&self) -> Option<Uuid> {
        self.active_tile
    }

    fn set_active_view(&mut self, view_id: Uuid) -> bool {
        if self.tiles.contains_key(&view_id) {
            // Unfocus all tiles
            for tile in self.tiles.values_mut() {
                tile.is_focused = false;
            }

            // Focus the selected tile
            if let Some(tile) = self.tiles.get_mut(&view_id) {
                tile.is_focused = true;
                self.active_tile = Some(view_id);
                return true;
            }
        }
        false
    }

    fn attach_actor_to_view(&mut self, view_id: Uuid, actor_id: Uuid) -> bool {
        if let Some(tile) = self.tiles.get_mut(&view_id) {
            tile.actor_id = Some(actor_id);
            true
        } else {
            false
        }
    }

    fn detach_actor_from_view(&mut self, view_id: Uuid) -> bool {
        if let Some(tile) = self.tiles.get_mut(&view_id) {
            tile.actor_id = None;
            true
        } else {
            false
        }
    }

    fn get_view_actor(&self, view_id: Uuid) -> Option<Uuid> {
        self.tiles.get(&view_id).and_then(|tile| tile.actor_id)
    }

    fn render_views(&mut self, ui: &mut egui::Ui, render_actor: &dyn Fn(&mut egui::Ui, Uuid)) {
        let available_rect = ui.available_rect_before_wrap();

        // Recalculate layout
        self.recalculate_layout(available_rect);

        // Collect tile data to avoid borrowing issues
        let tiles_data: Vec<_> = self.tiles.iter()
            .map(|(&id, tile)| (id, tile.clone()))
            .collect();

        // Render each tile
        for (tile_id, tile) in tiles_data {
            let tile_response = ui.allocate_rect(tile.rect, egui::Sense::click());

            // Handle tile selection
            if tile_response.clicked() {
                self.set_active_view(tile_id);
            }

            // Draw tile background
            let fill_color = if tile.is_focused {
                egui::Color32::from_gray(60)
            } else {
                egui::Color32::from_gray(40)
            };

            ui.painter().rect_filled(
                tile.rect,
                egui::Rounding::same(2.0),
                fill_color
            );

            // Draw tile border
            let stroke_color = if tile.is_focused {
                egui::Color32::from_rgb(100, 150, 255)
            } else {
                egui::Color32::from_gray(80)
            };

            ui.painter().rect_stroke(
                tile.rect,
                egui::Rounding::same(2.0),
                egui::Stroke::new(1.0, stroke_color)
            );

            // Render tile content
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(tile.rect.shrink(4.0)), |ui| {
                if let Some(actor_id) = tile.actor_id {
                    render_actor(ui, actor_id);
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(&tile.title);
                    });
                }
            });
        }
    }

    fn handle_input(&mut self, ui: &mut egui::Ui) -> bool {
        // Handle keyboard shortcuts for tile management
        let handled = ui.ctx().input(|i| {
            if i.key_pressed(egui::Key::Tab) && i.modifiers.ctrl {
                // Cycle through tiles
                if let Some(current_id) = self.active_tile {
                    let tile_ids: Vec<_> = self.tiles.keys().cloned().collect();
                    if let Some(current_index) = tile_ids.iter().position(|&id| id == current_id) {
                        let next_index = (current_index + 1) % tile_ids.len();
                        let next_id = tile_ids[next_index];
                        return Some(next_id);
                    }
                }
            }
            None
        });

        if let Some(next_id) = handled {
            self.set_active_view(next_id);
            true
        } else {
            false
        }
    }

    fn view_count(&self) -> usize {
        self.tiles.len()
    }

    fn remove_view(&mut self, view_id: Uuid) -> bool {
        if self.tiles.remove(&view_id).is_some() {
            // If we removed the active tile, select another one
            if self.active_tile == Some(view_id) {
                self.active_tile = self.tiles.keys().next().cloned();
                if let Some(new_active) = self.active_tile {
                    self.set_active_view(new_active);
                }
            }
            true
        } else {
            false
        }
    }

    fn get_view_ids(&self) -> Vec<Uuid> {
        self.tiles.keys().cloned().collect()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Transformable for TilingSystem {
    fn zoom(&mut self, _factor: f32, _center: Option<Pos2>) {
        // Tiling system doesn't support zoom by default
        // Could be extended to zoom individual tiles
    }

    fn pan(&mut self, _delta: Vec2) {
        // Tiling system doesn't support pan by default
        // Could be extended to pan individual tiles
    }

    fn reset_transform(&mut self) {
        // No-op for basic tiling system
    }

    fn get_transform_info(&self) -> TransformInfo {
        TransformInfo {
            zoom: 1.0,
            pan_offset: Vec2::ZERO,
            can_zoom: false,
            can_pan: false,
        }
    }
}