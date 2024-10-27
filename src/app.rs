use chrono::{DateTime, Utc};
use egui::{Align, Layout, Response, RichText, SelectableLabel, Ui};
use egui_extras::Column;
use egui_selectable_table::{ColumnOperations, ColumnOrdering, SelectableRow, SelectableTable, SortOrder};
use mime_db::lookup;
use resolve_path::PathResolveExt;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::fs;
use std::path::Path;
use std::time::SystemTime;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    directory_path: String,
    working_path: String,
    path_changed: bool,

    is_first_load: bool,

    #[serde(skip)]
    file_browser_table: SelectableTable<FileBrowserRow, FileBrowserColumns, FileBrowserConfig>,
    #[serde(skip)]
    file_browser_config: FileBrowserConfig,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            directory_path: "~".resolve().to_str().unwrap().to_string(),
            working_path: "~".resolve().to_str().unwrap().to_string(),
            file_browser_table: SelectableTable::new(FileBrowserColumns::iter().collect()),
            file_browser_config: FileBrowserConfig {},
            path_changed: false,
            is_first_load: true,
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
}

impl eframe::App for TemplateApp {
    /// Called by the framework to save state before shutdown.
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
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
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
                                self.path_changed = true;
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
                        self.path_changed = true;
                    }
                }

                if ui.button(format!("{}", egui_phosphor::regular::FOLDER_OPEN)).clicked() {
                    if let Some(path) = rfd::FileDialog::new().set_directory(&self.directory_path).pick_folder() {
                        self.directory_path = path.display().to_string();
                        self.working_path = self.directory_path.clone();
                        self.path_changed = true;
                    } else {
                        eprintln!("Error selecting a directory");
                        // Handle the error, e.g., display an error message to the user
                    }
                }
            });
            ui.separator();

            self.file_browser_table.set_select_full_row(true);

            self.file_browser_table.show_ui(ui, |builder| {
                let mut table = builder
                    .striped(true)
                    .resizable(true)
                    .cell_layout(Layout::left_to_right(Align::LEFT))
                    .drag_to_scroll(false)
                    .auto_shrink([false; 2]);

                for fb_column in FileBrowserColumns::iter() {
                    let mut column = Column::initial(150.0);
                    if fb_column == FileBrowserColumns::PathType {
                        column = column.at_least(25.0);
                        column = column.at_most(25.0);
                    }
                    table = table.column(column);
                }
                table
            });

            if self.path_changed || self.is_first_load {
                self.is_first_load = false;

                let paths = fs::read_dir(self.directory_path.as_str()).unwrap();
                self.file_browser_table.clear_all_rows();
                for path in paths {
                    if let Ok(path) = path {
                        self.file_browser_table.add_modify_row(|_| {
                            let mut new_row = FileBrowserRow {
                                name: "".to_string(),
                                new_name: "".to_string(),
                                size_ui: "--".to_string(),
                                date_modified: "".to_string(),
                                date_created: "".to_string(),
                                kind: "".to_string(),
                                path_type: "*".to_string(),
                                size: 0,
                            };
                            if let Ok(name) = path.file_name().into_string() {
                                new_row.name = name.clone();
                                new_row.new_name = name.clone();
                            }
                            if let Ok(metadata) = path.metadata() {
                                if let Ok(date_created) = metadata.created() {
                                    new_row.date_created = format_system_time(date_created);
                                }
                                if let Ok(date_modified) = metadata.modified() {
                                    new_row.date_modified = format_system_time(date_modified);
                                }
                                new_row.size_ui = format_size(metadata.len());
                                new_row.size = metadata.len();

                                if metadata.is_dir() {
                                    new_row.path_type = format!("{}", egui_phosphor::regular::FOLDER);
                                    new_row.kind = "Folder".to_string();
                                } else if metadata.is_file() {
                                    new_row.path_type = format!("{}", egui_phosphor::regular::FILE);
                                    new_row.kind = format_file_type(&path.path());
                                } else if metadata.is_symlink() {
                                    new_row.path_type = format!("{}", egui_phosphor::regular::LINK_SIMPLE_HORIZONTAL);
                                    new_row.kind = "symlink".to_string();
                                }
                            }
                            Some(new_row)
                        });
                    } else {
                        eprintln!("Error getting path: {}", path.unwrap_err());
                    }
                }
                self.file_browser_table.recreate_rows();
                self.file_browser_table.set_auto_reload(None);

                self.path_changed = false;
            }

            ctx.request_repaint();
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

// File browser config

#[derive(Default, Clone, Copy)]
pub struct FileBrowserConfig {}

#[derive(Clone, Default)]
struct FileBrowserRow {
    name: String,
    new_name: String,
    size_ui: String,
    size: u64,
    date_modified: String,
    date_created: String,
    kind: String,
    path_type: String,
}
#[derive(Eq, PartialEq, Debug, Ord, PartialOrd, Clone, Copy, Hash, Default, EnumIter)]
enum FileBrowserColumns {
    #[default]
    PathType,
    Name,
    NewName,
    Size,
    DateModified,
    DateCreated,
    Kind,
}

fn format_file_type(path: &Path) -> String {
    if let Some(file_extension) = path.extension().and_then(|ext| ext.to_str()) {
        let mime_type = lookup(file_extension);
        match mime_type {
            Some(mime_type) => {
                mime_type.to_string()
            }
            None => "Unknown".to_string(),
        }
    } else {
        "Unknown".to_string()
    }
}

fn format_size(size: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut i = 0;
    let mut size_f = size as f64;

    while size_f >= 1024.0 && i < units.len() - 1 {
        size_f /= 1024.0;
        i += 1;
    }

    format!("{:.1}{}", size_f, units[i])
}

fn format_system_time(system_time: SystemTime) -> String {
    let datetime: DateTime<Utc> = system_time.into();
    let local_datetime = datetime.with_timezone(&chrono::Local);
    let formatted_time = local_datetime.format("%d %b %Y at %I:%M %p");
    formatted_time.to_string()
}

impl ColumnOperations<FileBrowserRow, FileBrowserColumns, FileBrowserConfig> for FileBrowserColumns {
    fn create_header(&self, ui: &mut Ui, sort_order: Option<SortOrder>, _table: &mut SelectableTable<FileBrowserRow, FileBrowserColumns, FileBrowserConfig>) -> Option<Response> {
        let mut text = match self {
            FileBrowserColumns::PathType => "",
            FileBrowserColumns::Name => "Name",
            FileBrowserColumns::NewName => "New Name",
            FileBrowserColumns::Size => "Size",
            FileBrowserColumns::DateModified => "Date Modified",
            FileBrowserColumns::DateCreated => "Date Created",
            FileBrowserColumns::Kind => "Kind",
        }.to_string();
        let selected = if let Some(sort) = sort_order {
            text = match sort {
                SortOrder::Ascending => format!("{} {}", text, egui_phosphor::regular::SORT_DESCENDING),
                SortOrder::Descending => format!("{} {}", text, egui_phosphor::regular::SORT_ASCENDING),
            }.to_string();
            true
        } else {
            false
        };

        let label_text = RichText::new(text).strong();
        let response = ui.add_sized(ui.available_size(), SelectableLabel::new(selected, label_text));
        Some(response)
    }

    fn create_table_row(&self, ui: &mut Ui, row: &SelectableRow<FileBrowserRow, FileBrowserColumns>, column_selected: bool, table: &mut SelectableTable<FileBrowserRow, FileBrowserColumns, FileBrowserConfig>) -> Response {
        let row_data = &row.row_data;
        let row_text = self.assign_row_column(row_data);

        let response = match self {
            FileBrowserColumns::PathType => {
                // center aligned content
                ui.add_sized(
                    ui.available_size(),
                    SelectableLabel::new(column_selected, &row_text),
                )
            }
            // left aligned content
            _ => ui.add(SelectableLabel::new(column_selected, row_text)),
        };
        response.context_menu(|ui| {
            if ui.button("Select all items").clicked() {
                table.select_all();
                ui.close_menu();
            }
        });

        response
    }

    fn column_text(&self, row: &FileBrowserRow) -> String {
        self.assign_row_column(row)
    }
}

impl FileBrowserColumns {
    fn assign_row_column(&self, row: &FileBrowserRow) -> String {
        match self {
            FileBrowserColumns::PathType => row.path_type.to_string(),
            FileBrowserColumns::Name => row.name.to_string(),
            FileBrowserColumns::NewName => row.new_name.to_string(),
            FileBrowserColumns::Size => row.size_ui.to_string(),
            FileBrowserColumns::DateModified => row.date_modified.to_string(),
            FileBrowserColumns::DateCreated => row.date_created.to_string(),
            FileBrowserColumns::Kind => row.kind.to_string(),
        }
    }
}

impl ColumnOrdering<FileBrowserRow> for FileBrowserColumns {
    fn order_by(&self, row_1: &FileBrowserRow, row_2: &FileBrowserRow) -> Ordering {
        match self {
            FileBrowserColumns::PathType => row_1.path_type.cmp(&row_2.path_type),
            FileBrowserColumns::Name => row_1.name.cmp(&row_2.name),
            FileBrowserColumns::NewName => row_1.new_name.cmp(&row_2.new_name),
            FileBrowserColumns::Size => row_1.size.cmp(&row_2.size),
            FileBrowserColumns::DateModified => row_1.date_modified.cmp(&row_2.date_modified),
            FileBrowserColumns::DateCreated => row_1.date_created.cmp(&row_2.date_created),
            FileBrowserColumns::Kind => row_1.kind.cmp(&row_2.kind),
        }
    }
}