/*!
# Widget System Documentation

The widget system provides a flexible way to add overlay UI components to the IDE.
Widgets are rendered on top of the main content and can be positioned anywhere in the UI.

## Architecture

### Core Components

- `Widget` trait: Defines the interface for all widgets
- `WidgetManager`: Manages widget lifecycle and rendering
- `WidgetPosition`: Controls where widgets are positioned
- `WidgetVisibility`: Controls when widgets are shown

### Widget Trait

All widgets must implement the `Widget` trait:

```rust
pub trait Widget {
    fn id(&self) -> &str;
    fn render(&mut self, ui: &mut egui::Ui, ctx: &WidgetContext);
    fn is_visible(&self) -> bool;
    fn set_visible(&mut self, visible: bool);
    fn position(&self) -> WidgetPosition;
}
```

### Creating New Widgets

1. Create a new struct that implements the `Widget` trait
2. Add it to the `WidgetManager` using `add_widget()`
3. The widget will automatically be rendered and positioned

Example:

```rust
pub struct MyCustomWidget {
    visible: bool,
    position: WidgetPosition,
}

impl Widget for MyCustomWidget {
    fn id(&self) -> &str { "my_custom_widget" }

    fn render(&mut self, ui: &mut egui::Ui, ctx: &WidgetContext) {
        egui::Frame::none()
            .fill(egui::Color32::from_black_alpha(100))
            .show(ui, |ui| {
                ui.label("My Custom Widget");
            });
    }

    // ... other trait methods
}

// Add to manager:
widget_manager.add_widget(Box::new(MyCustomWidget::new()));
```

### Widget Positioning

Widgets can be positioned using `WidgetPosition`:

- `TopLeft`: Top-left corner
- `TopRight`: Top-right corner
- `BottomLeft`: Bottom-left corner
- `BottomRight`: Bottom-right corner
- `Center`: Center of the screen
- `Custom(offset)`: Custom offset from top-left

### Widget Context

The `WidgetContext` provides widgets with access to IDE state:

- `view_system`: Current view system reference
- `transform_info`: Current transform information (zoom, pan)
- `input_state`: Keyboard/mouse state

## Built-in Widgets

### TransformWidget

Shows current zoom and pan information with controls:
- Displays zoom level and pan offset
- Shows current panning mode (axis-locked vs free)
- Provides reset button
- Only visible when using transformable view systems

## Adding New Widget Types

To add a new type of widget:

1. Define the widget struct
2. Implement the `Widget` trait
3. Add any necessary context data to `WidgetContext`
4. Register the widget with `WidgetManager`

Common widget patterns:
- **Info widgets**: Display read-only information
- **Control widgets**: Provide interactive controls
- **Status widgets**: Show current state
- **Tool widgets**: Provide tool-specific UI

*/

use egui::{self, Vec2, Pos2, Rect};
use crate::view_system::{ViewSystem, TransformInfo};
use std::collections::HashMap;

/// Position for widget placement
#[derive(Debug, Clone, Copy)]
pub enum WidgetPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
    Custom(Vec2), // Offset from top-left
}

impl WidgetPosition {
    /// Calculate the actual position given the widget size and available rect
    pub fn calculate_rect(&self, widget_size: Vec2, available_rect: Rect) -> Rect {
        let pos = match self {
            WidgetPosition::TopLeft => available_rect.min,
            WidgetPosition::TopRight => Pos2::new(
                available_rect.max.x - widget_size.x,
                available_rect.min.y
            ),
            WidgetPosition::BottomLeft => Pos2::new(
                available_rect.min.x,
                available_rect.max.y - widget_size.y
            ),
            WidgetPosition::BottomRight => Pos2::new(
                available_rect.max.x - widget_size.x,
                available_rect.max.y - widget_size.y
            ),
            WidgetPosition::Center => available_rect.center() - widget_size * 0.5,
            WidgetPosition::Custom(offset) => available_rect.min + *offset,
        };

        Rect::from_min_size(pos, widget_size)
    }
}

/// Context provided to widgets during rendering
pub struct WidgetContext<'a> {
    pub view_system: &'a dyn ViewSystem,
    pub transform_info: TransformInfo,
    pub available_rect: Rect,
    pub is_hovered: bool,
}

/// Base trait for all widgets
pub trait Widget {
    /// Unique identifier for this widget
    fn id(&self) -> &str;

    /// Render the widget
    fn render(&mut self, ui: &mut egui::Ui, ctx: &WidgetContext);

    /// Check if widget should be visible
    fn is_visible(&self) -> bool;

    /// Set widget visibility
    fn set_visible(&mut self, visible: bool);

    /// Get widget position
    fn position(&self) -> WidgetPosition;

    /// Get the size this widget wants to occupy
    fn desired_size(&self) -> Vec2 {
        Vec2::new(200.0, 80.0) // Default size
    }

    /// Check if this widget should auto-hide when not needed
    fn auto_hide(&self) -> bool {
        false
    }
}

/// Transform widget showing zoom and pan information
pub struct TransformWidget {
    visible: bool,
    position: WidgetPosition,
    last_transform: Option<TransformInfo>,
}

impl TransformWidget {
    pub fn new() -> Self {
        Self {
            visible: true,
            position: WidgetPosition::TopLeft,
            last_transform: None,
        }
    }
}

impl Widget for TransformWidget {
    fn id(&self) -> &str {
        "transform_widget"
    }

    fn render(&mut self, ui: &mut egui::Ui, ctx: &WidgetContext) {
        let widget_rect = self.position().calculate_rect(self.desired_size(), ctx.available_rect);

        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(widget_rect), |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_black_alpha(100))
                .rounding(4.0)
                .inner_margin(6.0)
                .show(ui, |ui| {
                    let transform = &ctx.transform_info;

                    ui.small(format!("Zoom: {:.1}x", transform.zoom));
                    ui.small(format!("Pan: ({:.0}, {:.0})",
                        transform.pan_offset.x, transform.pan_offset.y));

                    // Show panning mode if pan is supported
                    if transform.can_pan {
                        let cmd_held = ui.ctx().input(|i| i.modifiers.command);
                        if cmd_held {
                            ui.small("Free Pan (⌘)");
                            ui.small("Zoom Locked");
                        } else {
                            ui.small("Axis-Locked Pan");
                        }
                    }

                    // Reset button only if transform is supported
                    if transform.can_zoom || transform.can_pan {
                        if ui.small_button("Reset").clicked() {
                            // Signal that reset was requested
                            // This will be handled by the view system
                        }
                    }
                });
        });

        // Store current transform for comparison
        self.last_transform = Some(ctx.transform_info.clone());
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn position(&self) -> WidgetPosition {
        self.position
    }

    fn desired_size(&self) -> Vec2 {
        Vec2::new(200.0, 80.0)
    }

    fn auto_hide(&self) -> bool {
        true // Hide when no transform capability
    }
}

/// Widget manager handles lifecycle of all widgets
pub struct WidgetManager {
    widgets: HashMap<String, Box<dyn Widget>>,
    enabled: bool,
}

impl WidgetManager {
    pub fn new() -> Self {
        let mut manager = Self {
            widgets: HashMap::new(),
            enabled: true,
        };

        // Add built-in widgets
        manager.add_widget(Box::new(TransformWidget::new()));

        manager
    }

    /// Add a widget to the manager
    pub fn add_widget(&mut self, widget: Box<dyn Widget>) {
        let id = widget.id().to_string();
        self.widgets.insert(id, widget);
    }

    /// Remove a widget by ID
    pub fn remove_widget(&mut self, id: &str) -> bool {
        self.widgets.remove(id).is_some()
    }

    /// Get a widget by ID
    pub fn get_widget(&self, id: &str) -> Option<&dyn Widget> {
        self.widgets.get(id).map(|w| w.as_ref())
    }

    /// Get a mutable widget by ID (internal use only)
    fn get_widget_mut_internal(&mut self, id: &str) -> Option<&mut Box<dyn Widget>> {
        self.widgets.get_mut(id)
    }

    /// Show/hide a specific widget
    pub fn set_widget_visible(&mut self, id: &str, visible: bool) {
        if let Some(widget) = self.widgets.get_mut(id) {
            widget.set_visible(visible);
        }
    }

    /// Toggle widgets system on/off
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Render all visible widgets
    pub fn render_widgets(&mut self, ui: &mut egui::Ui, ctx: &WidgetContext) {
        if !self.enabled {
            return;
        }

        // Update auto-hide widgets based on context
        for widget in self.widgets.values_mut() {
            if widget.auto_hide() {
                // Auto-hide widgets that don't make sense for current context
                let should_show = match widget.id() {
                    "transform_widget" => ctx.transform_info.can_zoom || ctx.transform_info.can_pan,
                    _ => true,
                };
                widget.set_visible(should_show);
            }
        }

        // Render all visible widgets
        for widget in self.widgets.values_mut() {
            if widget.is_visible() {
                widget.render(ui, ctx);
            }
        }
    }

    /// Get list of all widget IDs
    pub fn widget_ids(&self) -> Vec<String> {
        self.widgets.keys().cloned().collect()
    }

    /// Handle reset requests from widgets
    pub fn handle_reset_request(&self) -> bool {
        // Check if any widget requested a reset
        // For now, we'll check if the transform widget's reset button was clicked
        // In the future, we could have a more sophisticated event system
        false // Placeholder - would be implemented with proper event handling
    }
}

impl Default for WidgetManager {
    fn default() -> Self {
        Self::new()
    }
}