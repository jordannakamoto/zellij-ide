use crate::actor::{ActorManager, Actor};
use crate::view::{SplitDirection};
use crate::view_system::ViewContainer;
use crate::scene_system::SceneSystem;
use crate::tiling_system::TilingSystem;
use crate::code_editor_actor::CodeEditorActor;
use crate::widgets::{WidgetManager, WidgetContext};
use crate::view_system::Transformable;
use crate::terminal_actor::TerminalActor;
use egui;

/// Main IDE state - combines actors, view system, and widgets
pub struct IdeState {
    pub actors: ActorManager,
    pub view_container: ViewContainer,
    pub widget_manager: WidgetManager,
    tab_counter: usize,
}

impl IdeState {
    pub fn new() -> Self {
        let mut actors = ActorManager::new();

        // Create view container with scene system (could be swapped for tiling system)
        let scene_system = Box::new(SceneSystem::new());
        let mut view_container = ViewContainer::new(scene_system);

        // Add a test terminal actor
        let terminal_actor = Box::new(TerminalActor::new());
        let terminal_id = terminal_actor.id();
        actors.register_actor(terminal_actor);
        let terminal_view_id = view_container.system_mut().create_view("Terminal".to_string());
        view_container.system_mut().attach_actor_to_view(terminal_view_id, terminal_id);

        // Set terminal view as active so it shows up
        view_container.system_mut().set_active_view(terminal_view_id);

        // Add a test code editor actor (but don't set it as active)
        let code_editor = Box::new(CodeEditorActor::new("main.rs".to_string()));
        let editor_id = code_editor.id();
        actors.register_actor(code_editor);
        let editor_view_id = view_container.system_mut().create_view("Code Editor".to_string());
        view_container.system_mut().attach_actor_to_view(editor_view_id, editor_id);

        Self {
            actors,
            view_container,
            widget_manager: WidgetManager::new(),
            tab_counter: 1,
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        // Handle keyboard input for view switching
        if ui.input(|i| i.key_pressed(egui::Key::Tab) && i.modifiers.ctrl) {
            self.cycle_active_view();
        }

        // Handle view system input
        self.view_container.system_mut().handle_input(ui);

        // Render views using the view system
        // Use a separate scope to avoid borrowing issues
        let actors_ptr = &mut self.actors as *mut ActorManager;
        self.view_container.system_mut().render_views(ui, &|ui, actor_id| {
            unsafe {
                let actor_manager = &mut *actors_ptr;
                if let Some(actor) = actor_manager.get_actor_mut(actor_id) {
                    actor.render(ui);
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("No Actor");
                    });
                }
            }
        });

        // Render widgets on top of everything
        let available_rect = ui.available_rect_before_wrap();
        let transform_info = if let Some(transformable) = self.view_container.as_system::<crate::scene_system::SceneSystem>() {
            transformable.get_transform_info()
        } else {
            crate::view_system::TransformInfo {
                zoom: 1.0,
                pan_offset: egui::Vec2::ZERO,
                can_zoom: false,
                can_pan: false,
            }
        };

        let widget_ctx = WidgetContext {
            view_system: self.view_container.system(),
            transform_info,
            available_rect,
            is_hovered: ui.ui_contains_pointer(),
        };

        self.widget_manager.render_widgets(ui, &widget_ctx);

        // Update all actors
        for actor_idx in 0..self.actors.actors.len() {
            let ctx = ui.ctx().clone();
            if let Some(actor) = self.actors.actors.get_mut(actor_idx) {
                actor.update(&ctx);
            }
        }
    }

    pub fn new_tab(&mut self) {
        self.tab_counter += 1;

        // Create new editor actor
        let editor = CodeEditorActor::new(format!("untitled-{}.rs", self.tab_counter));
        let editor_id = editor.id();
        self.actors.register_actor(Box::new(editor));

        // Replace the current view's actor
        if let Some(active_view) = self.view_container.system().active_view() {
            self.view_container.system_mut().attach_actor_to_view(active_view, editor_id);
            self.actors.set_focus(editor_id);
        }
    }

    // Temporarily disabled - terminal actor has compilation issues
    // pub fn new_terminal(&mut self) {
    //     self.tab_counter += 1;

    //     // Create new terminal actor
    //     let terminal = TerminalActor::new();
    //     let terminal_id = terminal.id();
    //     self.actors.register_actor(Box::new(terminal));

    //     // Replace the current scene view's actor with terminal
    //     if let Some(active_view) = self.scenes.active_view {
    //         self.scenes.attach_actor_to_view(active_view, terminal_id);
    //         self.actors.set_focus(terminal_id);
    //     }
    // }

    pub fn close_active_tab(&mut self) {
        // Clear the active view
        if let Some(active_view) = self.view_container.system().active_view() {
            self.view_container.system_mut().detach_actor_from_view(active_view);
        }
    }

    pub fn split_active_view(&mut self, _direction: SplitDirection) {
        // For now, just create a new editor in the current scene
        // TODO: Implement proper split view support with scene system
        self.new_tab();
    }

    pub fn cycle_active_view(&mut self) {
        // Get all view IDs
        let view_ids = self.view_container.system().get_view_ids();

        if view_ids.is_empty() {
            return;
        }

        // Get current active view
        let current_active = self.view_container.system().active_view();

        // Find the index of the current active view
        let current_index = current_active
            .and_then(|active| view_ids.iter().position(|&id| id == active))
            .unwrap_or(0);

        // Cycle to the next view
        let next_index = (current_index + 1) % view_ids.len();
        let next_view_id = view_ids[next_index];

        // Set the next view as active
        self.view_container.system_mut().set_active_view(next_view_id);
        log::info!("Cycled to view: {:?}", next_view_id);
    }

    pub fn active_view_id(&self) -> String {
        format!("{:?}", self.view_container.system().active_view())
    }

    pub fn view_count(&self) -> usize {
        self.view_container.system().view_count()
    }

    /// Get reference to widget manager
    pub fn widget_manager(&self) -> &WidgetManager {
        &self.widget_manager
    }

    /// Get mutable reference to widget manager
    pub fn widget_manager_mut(&mut self) -> &mut WidgetManager {
        &mut self.widget_manager
    }

    /// Switch to scene-based view system
    pub fn switch_to_scene_system(&mut self) {
        // Preserve current actors
        let current_actors: Vec<_> = self.view_container.system().get_view_ids()
            .into_iter()
            .filter_map(|view_id| {
                self.view_container.system().get_view_actor(view_id)
            })
            .collect();

        // Create new scene system
        let scene_system = Box::new(SceneSystem::new());
        self.view_container = ViewContainer::new(scene_system);

        // Reattach actors to the new system
        for (i, actor_id) in current_actors.into_iter().enumerate() {
            if i == 0 {
                // Use the default view for the first actor
                if let Some(default_view) = self.view_container.system().active_view() {
                    self.view_container.system_mut().attach_actor_to_view(default_view, actor_id);
                }
            } else {
                // Create new views for additional actors
                let new_view = self.view_container.system_mut().create_view(format!("View {}", i + 1));
                self.view_container.system_mut().attach_actor_to_view(new_view, actor_id);
            }
        }

        log::info!("Switched to Scene System");
    }

    /// Switch to tiling-based view system
    pub fn switch_to_tiling_system(&mut self) {
        // Preserve current actors
        let current_actors: Vec<_> = self.view_container.system().get_view_ids()
            .into_iter()
            .filter_map(|view_id| {
                self.view_container.system().get_view_actor(view_id)
            })
            .collect();

        // Create new tiling system
        let tiling_system = Box::new(TilingSystem::new());
        self.view_container = ViewContainer::new(tiling_system);

        // Reattach actors to the new system
        for (i, actor_id) in current_actors.into_iter().enumerate() {
            if i == 0 {
                // Use the default view for the first actor
                if let Some(default_view) = self.view_container.system().active_view() {
                    self.view_container.system_mut().attach_actor_to_view(default_view, actor_id);
                }
            } else {
                // Create new tiles for additional actors
                let new_tile = self.view_container.system_mut().create_view(format!("Tile {}", i + 1));
                self.view_container.system_mut().attach_actor_to_view(new_tile, actor_id);
            }
        }

        log::info!("Switched to Tiling System");
    }
}