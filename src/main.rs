mod actor;
mod view;
mod ide_state;
mod panels;
mod code_editor_actor;
mod config;
mod terminal_actor;
mod scene_view;
mod view_system;
mod scene_system;
mod tiling_system;
mod widgets;
mod command_palette;

use eframe::egui;
use env_logger;
use ide_state::IdeState;
use config::IdeConfig;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Zellij IDE")
            .with_decorations(false)  // Remove window decorations
            .with_transparent(true)   // Make background transparent
            .with_window_level(egui::WindowLevel::AlwaysOnTop), // Force window level
        ..Default::default()
    };

    eframe::run_native(
        "Zellij IDE",
        native_options,
        Box::new(|cc| Ok(Box::new(IdeApp::new(cc)))),
    )
}

struct IdeApp {
    state: IdeState,
    config: IdeConfig,
}

impl IdeApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = IdeConfig::load().unwrap_or_else(|e| {
            log::warn!("Failed to load config: {}, using defaults", e);
            IdeConfig::default()
        });

        Self {
            state: IdeState::new(),
            config,
        }
    }
}

impl eframe::App for IdeApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        if self.config.appearance.background_opacity == 0.0 {
            egui::Rgba::TRANSPARENT.to_array()
        } else {
            // Convert our background color to rgba array
            let color = self.config.background_color();
            [
                color.r() as f32 / 255.0,
                color.g() as f32 / 255.0,
                color.b() as f32 / 255.0,
                color.a() as f32 / 255.0,
            ]
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set visuals - background transparency is handled by clear_color() method
        ctx.set_visuals(egui::Visuals::dark());
        // Top menu bar - with configurable transparency
        let menu_fill = if self.config.appearance.menu_opacity > 0.0 {
            self.config.menu_background_color()
        } else {
            egui::Color32::TRANSPARENT
        };

        egui::TopBottomPanel::top("menu_bar")
            .frame(egui::Frame::none().fill(menu_fill))
            .show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Tab").clicked() {
                        self.state.new_tab();
                    }
                    // Temporarily disabled terminal due to compilation issues
                    // if ui.button("New Terminal").clicked() {
                    //     self.state.new_terminal();
                    // }
                    ui.separator();
                    if ui.button("Close Tab").clicked() {
                        self.state.close_active_tab();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Split Horizontal").clicked() {
                        self.state.split_active_view(panels::SplitDirection::Horizontal);
                    }
                    if ui.button("Split Vertical").clicked() {
                        self.state.split_active_view(panels::SplitDirection::Vertical);
                    }
                    ui.separator();
                    ui.menu_button("View System", |ui| {
                        if ui.button("Scene System").clicked() {
                            self.state.switch_to_scene_system();
                        }
                        if ui.button("Tiling System").clicked() {
                            self.state.switch_to_tiling_system();
                        }
                    });
                    ui.separator();
                    ui.menu_button("Widgets", |ui| {
                        let widgets_enabled = self.state.widget_manager().is_enabled();
                        if ui.checkbox(&mut widgets_enabled.clone(), "Show Widgets").changed() {
                            self.state.widget_manager_mut().set_enabled(!widgets_enabled);
                        }

                        ui.separator();

                        // Individual widget toggles
                        let transform_visible = self.state.widget_manager()
                            .get_widget("transform_widget")
                            .map(|w| w.is_visible())
                            .unwrap_or(false);
                        if ui.checkbox(&mut transform_visible.clone(), "Transform Widget").changed() {
                            self.state.widget_manager_mut().set_widget_visible("transform_widget", !transform_visible);
                        }
                    });
                });

                ui.menu_button("Settings", |ui| {
                    ui.label("Background Transparency");
                    let mut opacity = self.config.appearance.background_opacity;
                    if ui.add(egui::Slider::new(&mut opacity, 0.0..=1.0)
                        .text("Background")
                        .show_value(true)).changed() {
                        if let Err(e) = self.config.set_background_opacity(opacity) {
                            log::error!("Failed to save background opacity: {}", e);
                        }
                    }

                    ui.label("Menu Transparency");
                    let mut menu_opacity = self.config.appearance.menu_opacity;
                    if ui.add(egui::Slider::new(&mut menu_opacity, 0.0..=1.0)
                        .text("Menu")
                        .show_value(true)).changed() {
                        if let Err(e) = self.config.set_menu_opacity(menu_opacity) {
                            log::error!("Failed to save menu opacity: {}", e);
                        }
                    }

                    ui.separator();
                    if ui.button("Reset to Defaults").clicked() {
                        self.config = IdeConfig::default();
                        if let Err(e) = self.config.save() {
                            log::error!("Failed to save default config: {}", e);
                        }
                    }
                });
            });
        });

        // Main content area - using proper transparency approach
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                self.state.render(ui);
            });
    }
}