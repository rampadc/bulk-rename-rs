use std::fs;
use std::path::Path;
use egui::{Align, Id, Layout};
use resolve_path::PathResolveExt;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    directory_path: String,
    working_path: String,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            directory_path: "~".resolve().to_str().unwrap().to_string(),
            working_path: "~".resolve().to_str().unwrap().to_string(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn is_valid_path(path_str: &str) -> bool {
        let path = Path::new(path_str);
        path.exists()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                if ui.button(format!("{}", egui_phosphor::regular::ARROW_SQUARE_UP)).clicked() {
                    match fs::canonicalize(format!("{}/..", &self.directory_path)) {
                        Ok(path) => {
                            if path.exists() {
                                self.directory_path = path.to_str().unwrap_or("").to_string();
                                self.working_path = self.directory_path.clone();
                            }
                        }
                        Err(err) => {
                            eprintln!("Error going up a directory: {}", err);
                            // Handle the error, e.g., display an error message to the user
                        }
                    }
                }

                let available_width =
                    ui.available_width() - ui.spacing().item_spacing.x * 2.0 - ui.style().spacing.button_padding.x * 4.0;
                let directory_edit =
                    ui.add(
                        egui::TextEdit::singleline(&mut self.working_path)
                            .hint_text("Directory path")
                            .desired_width(available_width)
                    );
                if directory_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let path = Path::new(&self.working_path);
                    if !path.exists() {
                        eprintln!("Directory does not exists: {}", self.working_path);
                        self.working_path = self.directory_path.clone();
                    } else {
                        self.directory_path = self.working_path.clone();
                    }
                }

                if ui.button(format!("{}", egui_phosphor::regular::FOLDER_OPEN)).clicked() {
                    if let Some(path) = rfd::FileDialog::new().set_directory(&self.directory_path).pick_folder() {
                        self.directory_path = path.display().to_string();
                        self.working_path = self.directory_path.clone();
                    } else {
                        eprintln!("Error selecting a directory");
                        // Handle the error, e.g., display an error message to the user
                    }
                }
            });
            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
