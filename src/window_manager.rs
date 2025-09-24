use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use uuid::Uuid;
use zellij_utils::pane_size::{PaneGeom, Size, Viewport, Dimension};
use zellij_utils::data::{Direction, FloatingPaneCoordinates};
use zellij_utils::input::layout::SplitSize;

/// Core window manager that handles tabs, panes, and floating windows
pub struct WindowManager {
    /// Active tabs indexed by tab ID
    tabs: HashMap<TabId, Tab>,
    /// Active tab ID
    active_tab_id: Option<TabId>,
    /// Display area size
    display_area: Size,
    /// Current viewport
    viewport: Viewport,
    /// GUI control overlay enabled
    gui_controls_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TabId(Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct PaneId(Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowId(Uuid);

/// A tab contains both tiled and floating panes
pub struct Tab {
    id: TabId,
    name: String,
    tiled_panes: TiledPaneManager,
    floating_windows: FloatingWindowManager,
    active_pane_id: Option<PaneId>,
}

/// Manages tiled pane layout with enhanced splitting
pub struct TiledPaneManager {
    panes: BTreeMap<PaneId, TiledPane>,
    layout_tree: LayoutTree,
    active_pane_id: Option<PaneId>,
}

/// Manages floating windows that can contain their own tabs/panes
pub struct FloatingWindowManager {
    windows: HashMap<WindowId, FloatingWindow>,
    z_order: Vec<WindowId>,
    active_window_id: Option<WindowId>,
}

/// A floating window can contain tabs and panes
pub struct FloatingWindow {
    id: WindowId,
    position: FloatingPaneCoordinates,
    tabs: HashMap<TabId, Tab>, // Nested tabs within floating windows
    active_tab_id: Option<TabId>,
    is_maximized: bool,
}

/// A tiled pane with terminal emulation
pub struct TiledPane {
    id: PaneId,
    geometry: PaneGeom,
    terminal_state: TerminalState,
    is_focused: bool,
}

/// Tree structure for managing tiled pane layouts
#[derive(Debug, Clone)]
pub enum LayoutTree {
    Leaf(PaneId),
    Split {
        direction: Direction,
        ratio: f32,
        children: Vec<LayoutTree>,
    },
}

/// Terminal emulation state (placeholder for now)
pub struct TerminalState {
    // Will integrate with Zellij's Grid and VTE
    pub pty_id: Option<u32>,
    pub title: String,
}

impl WindowManager {
    pub fn new(display_area: Size, gui_controls: bool) -> Self {
        Self {
            tabs: HashMap::new(),
            active_tab_id: None,
            display_area,
            viewport: Viewport::default(),
            gui_controls_enabled: gui_controls,
        }
    }

    pub fn handle_resize(&mut self, width: u16, height: u16) -> Result<()> {
        self.display_area = Size {
            cols: width as usize,
            rows: height as usize,
        };
        // TODO: Update all pane geometries based on new size
        Ok(())
    }

    /// Create a new tab
    pub fn create_tab(&mut self, name: Option<String>) -> Result<TabId> {
        let id = TabId(Uuid::new_v4());
        let tab_name = name.unwrap_or_else(|| format!("Tab {}", self.tabs.len() + 1));

        let tab = Tab {
            id,
            name: tab_name,
            tiled_panes: TiledPaneManager::new(),
            floating_windows: FloatingWindowManager::new(),
            active_pane_id: None,
        };

        self.tabs.insert(id, tab);
        self.active_tab_id = Some(id);

        // Create initial pane in the new tab
        self.create_tiled_pane_in_tab(id, None)?;

        Ok(id)
    }

    /// Create a new tiled pane in a specific tab
    pub fn create_tiled_pane_in_tab(&mut self, tab_id: TabId, split_direction: Option<Direction>) -> Result<PaneId> {
        let tab = self.tabs.get_mut(&tab_id).ok_or_else(|| anyhow::anyhow!("Tab not found"))?;
        tab.tiled_panes.create_pane(split_direction, &self.viewport)
    }

    /// Create a floating window
    pub fn create_floating_window(&mut self, coordinates: Option<FloatingPaneCoordinates>) -> Result<WindowId> {
        let active_tab = self.get_active_tab_mut()?;
        active_tab.floating_windows.create_window(coordinates)
    }

    /// Split pane with mouse interaction
    pub fn split_pane_at_position(&mut self, pane_id: PaneId, position: (u16, u16), direction: Direction) -> Result<PaneId> {
        // Enhanced splitting with visual feedback
        let tab = self.get_active_tab_mut()?;
        tab.tiled_panes.split_pane_at_position(pane_id, position, direction)
    }

    /// Handle mouse events for window management
    pub fn handle_mouse_event(&mut self, event: MouseEvent) -> Result<()> {
        match event.event_type {
            MouseEventType::LeftClick => self.handle_left_click(event.position),
            MouseEventType::RightClick => self.handle_right_click(event.position),
            MouseEventType::Drag => self.handle_drag(event.position),
            _ => Ok(()),
        }
    }

    fn handle_left_click(&mut self, _position: (u16, u16)) -> Result<()> {
        // Determine what was clicked and focus/select it
        // Could be a pane, tab, window, or GUI control
        Ok(())
    }

    fn handle_right_click(&mut self, _position: (u16, u16)) -> Result<()> {
        // Show context menu for creating panes, splitting, etc.
        Ok(())
    }

    fn handle_drag(&mut self, _position: (u16, u16)) -> Result<()> {
        // Handle dragging floating windows or resizing panes
        Ok(())
    }

    fn get_active_tab_mut(&mut self) -> Result<&mut Tab> {
        let id = self.active_tab_id.ok_or_else(|| anyhow::anyhow!("No active tab"))?;
        self.tabs.get_mut(&id).ok_or_else(|| anyhow::anyhow!("Active tab not found"))
    }

    /// Get GUI overlay state for rendering
    pub fn gui_controls_enabled(&self) -> bool {
        self.gui_controls_enabled
    }

    /// Toggle GUI controls
    pub fn toggle_gui_controls(&mut self) {
        self.gui_controls_enabled = !self.gui_controls_enabled;
    }
}

// Mouse event types for our enhanced interaction
#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub position: (u16, u16),
    pub event_type: MouseEventType,
}

#[derive(Debug, Clone)]
pub enum MouseEventType {
    LeftClick,
    RightClick,
    MiddleClick,
    Drag,
    Scroll(i8),
}

impl TiledPaneManager {
    fn new() -> Self {
        Self {
            panes: BTreeMap::new(),
            layout_tree: LayoutTree::Leaf(PaneId(Uuid::new_v4())), // Will be properly initialized
            active_pane_id: None,
        }
    }

    fn create_pane(&mut self, split_direction: Option<Direction>, viewport: &Viewport) -> Result<PaneId> {
        let id = PaneId(Uuid::new_v4());
        let geometry = if self.panes.is_empty() {
            // First pane takes full area
            PaneGeom {
                x: 0,
                y: 0,
                rows: Dimension::fixed(viewport.rows),
                cols: Dimension::fixed(viewport.cols),
                stacked: None,
                is_pinned: false,
                logical_position: None,
            }
        } else {
            // Split existing pane
            self.calculate_split_geometry(split_direction, viewport)?
        };

        let pane = TiledPane {
            id,
            geometry,
            terminal_state: TerminalState {
                pty_id: None,
                title: format!("Terminal {}", self.panes.len() + 1),
            },
            is_focused: true,
        };

        self.panes.insert(id, pane);
        self.active_pane_id = Some(id);

        Ok(id)
    }

    fn split_pane_at_position(&mut self, pane_id: PaneId, _position: (u16, u16), direction: Direction) -> Result<PaneId> {
        // Enhanced splitting logic with mouse position awareness
        let new_id = PaneId(Uuid::new_v4());

        // TODO: Implement sophisticated splitting based on mouse position
        // For now, simple split
        if let Some(pane) = self.panes.get(&pane_id) {
            let mut new_geometry = pane.geometry.clone();
            match direction {
                Direction::Right => {
                    // Split horizontally - simplified for now
                    new_geometry.cols = Dimension::fixed(50);
                },
                Direction::Down => {
                    // Split vertically - simplified for now
                    new_geometry.rows = Dimension::fixed(25);
                },
                _ => return Err(anyhow::anyhow!("Unsupported split direction")),
            }

            let new_pane = TiledPane {
                id: new_id,
                geometry: new_geometry,
                terminal_state: TerminalState {
                    pty_id: None,
                    title: format!("Terminal {}", self.panes.len() + 1),
                },
                is_focused: false,
            };

            self.panes.insert(new_id, new_pane);
        }

        Ok(new_id)
    }

    fn calculate_split_geometry(&self, _split_direction: Option<Direction>, viewport: &Viewport) -> Result<PaneGeom> {
        // Placeholder - will implement sophisticated layout calculation
        Ok(PaneGeom {
            x: 0,
            y: 0,
            rows: Dimension::fixed(viewport.rows),
            cols: Dimension::fixed(viewport.cols / 2),
            stacked: None,
            is_pinned: false,
            logical_position: None,
        })
    }
}

impl FloatingWindowManager {
    fn new() -> Self {
        Self {
            windows: HashMap::new(),
            z_order: Vec::new(),
            active_window_id: None,
        }
    }

    fn create_window(&mut self, coordinates: Option<FloatingPaneCoordinates>) -> Result<WindowId> {
        let id = WindowId(Uuid::new_v4());
        let coords = coordinates.unwrap_or_else(|| FloatingPaneCoordinates {
            x: Some(SplitSize::Fixed(10)),
            y: Some(SplitSize::Fixed(10)),
            width: Some(SplitSize::Fixed(80)),
            height: Some(SplitSize::Fixed(24)),
            pinned: Some(false),
        });

        let window = FloatingWindow {
            id,
            position: coords,
            tabs: HashMap::new(),
            active_tab_id: None,
            is_maximized: false,
        };

        self.windows.insert(id, window);
        self.z_order.push(id);
        self.active_window_id = Some(id);

        Ok(id)
    }
}

// Generate new UUIDs for IDs
impl TabId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl PaneId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl WindowId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}