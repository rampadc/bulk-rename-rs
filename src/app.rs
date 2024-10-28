use chrono::{DateTime, Utc};
use egui::{Align, Grid, Label, Layout, Response, RichText, SelectableLabel, Ui};
use egui_extras::Column;
use egui_selectable_table::{ColumnOperations, ColumnOrdering, SelectableRow, SelectableTable, SortOrder};
use mime_db::lookup;
use resolve_path::PathResolveExt;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::fs;
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};
use std::time::SystemTime;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

struct FileBrowserPath {
    directory_path: String,
}

impl FileBrowserPath {
    fn navigate_to(directory_path: &str, from: &str) -> Self {
        let new_path = format!("{}/{}", from, directory_path);
        match fs::canonicalize(new_path) {
            Ok(path) => {
                if path.exists() {
                    FileBrowserPath {
                        directory_path: path.to_str().unwrap().to_string(),
                    }
                } else {
                    FileBrowserPath {
                        directory_path: "".to_string(),
                    }
                }
            }
            Err(_) => {
                FileBrowserPath {
                    directory_path: "".to_string(),
                }
            }
        }
    }

    fn get_path(&self) -> String {
        self.directory_path.clone()
    }
}
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
    #[serde(skip)]
    tx: Sender<String>,
    #[serde(skip)]
    rx: Receiver<String>,

    regex_match: String,
    regex_substitution: String,
    regex_including_extension: bool,

    replace_match: String,
    replace_with: String,
    replace_case_sensitive: bool,
    replace_first_only: bool,

    remove_first_n: String,
    remove_last_n: String,
    remove_from: String,
    remove_to: String,
    remove_chars: String,
    remove_words: String,
    remove_trim: bool,
    remove_digits: bool,
    remove_accents: bool,

    add_prefix: String,
    add_insert: String,
    add_at_pos: String,
    add_suffix: String,
    add_word_space: bool,

    auto_date_type: String,
    auto_date_format: String,

    numbering_mode: String,
    numbering_at: String,
    numbering_start: String,
    numbering_increment: String,
    numbering_pad: String,
    numbering_break: String,
    numbering_base: String,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            directory_path: "~".resolve().to_str().unwrap().to_string(),
            working_path: "~".resolve().to_str().unwrap().to_string(),
            file_browser_table: SelectableTable::new(FileBrowserColumns::iter().collect()),
            file_browser_config: FileBrowserConfig {},
            path_changed: false,
            is_first_load: true,
            tx,
            rx,
            regex_match: "".to_string(),
            regex_substitution: "".to_string(),
            regex_including_extension: false,
            replace_match: "".to_string(),
            replace_with: "".to_string(),
            replace_case_sensitive: false,
            replace_first_only: false,
            remove_first_n: "".to_string(),
            remove_last_n: "".to_string(),
            remove_from: "".to_string(),
            remove_to: "".to_string(),
            remove_chars: "".to_string(),
            remove_words: "".to_string(),
            remove_trim: false,
            remove_digits: false,
            remove_accents: false,
            add_prefix: "".to_string(),
            add_insert: "".to_string(),
            add_at_pos: "".to_string(),
            add_suffix: "".to_string(),
            add_word_space: false,
            auto_date_type: "".to_string(),
            auto_date_format: "".to_string(),
            numbering_mode: "".to_string(),
            numbering_at: "".to_string(),
            numbering_start: "".to_string(),
            numbering_increment: "".to_string(),
            numbering_pad: "".to_string(),
            numbering_break: "".to_string(),
            numbering_base: "".to_string(),
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
                ui.label("[Use Ctrl to multi-select, or select & drag]");
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.group(|ui| {
                Grid::new("regex")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        // Regex
                        ui.add(Label::new(RichText::new("Regex").strong()));
                        ui.end_row();

                        ui.add(Label::new("Match"));
                        ui.text_edit_singleline(&mut self.regex_match);
                        ui.end_row();

                        ui.add(Label::new("Replace"));
                        ui.text_edit_singleline(&mut self.regex_substitution);
                        ui.end_row();

                        ui.checkbox(&mut self.regex_including_extension, "Include extension");
                        ui.end_row();
                    });
            });
            ui.group(|ui| {
                Grid::new("replace")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        // Regex
                        ui.add(Label::new(RichText::new("Replace").strong()));
                        ui.end_row();

                        ui.add(Label::new("Replace"));
                        ui.text_edit_singleline(&mut self.replace_match);
                        ui.end_row();

                        ui.add(Label::new("With"));
                        ui.text_edit_singleline(&mut self.replace_with);
                        ui.end_row();

                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.replace_case_sensitive, "Match case");
                            ui.checkbox(&mut self.replace_first_only, "First");
                        });
                        ui.end_row();
                    });
            });
            ui.group(|ui| {
                Grid::new("remove")
                    .num_columns(4)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        // Regex
                        ui.add(Label::new(RichText::new("Remove").strong()));
                        ui.end_row();

                        ui.add(Label::new("First n"));
                        ui.text_edit_singleline(&mut self.remove_first_n);
                        ui.add(Label::new("Last n"));
                        ui.text_edit_singleline(&mut self.remove_last_n);
                        ui.end_row();

                        ui.add(Label::new("From"));
                        ui.text_edit_singleline(&mut self.remove_from);
                        ui.add(Label::new("To"));
                        ui.text_edit_singleline(&mut self.remove_to);
                        ui.end_row();

                        ui.add(Label::new("Chars"));
                        ui.text_edit_singleline(&mut self.remove_chars);
                        ui.add(Label::new("Words"));
                        ui.text_edit_singleline(&mut self.remove_words);
                        ui.end_row();

                        ui.checkbox(&mut self.remove_digits, "Digits");
                        ui.checkbox(&mut self.remove_accents, "Accents");
                        ui.checkbox(&mut self.remove_trim, "Trim");
                        ui.end_row();
                    });
            });
            ui.add_space(8.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Ok(selected_new_path) = self.rx.try_recv() {
                self.directory_path = selected_new_path.clone();
                self.working_path = selected_new_path.clone();
                self.path_changed = true;
            }
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
                    .cell_layout(Layout::top_down_justified(Align::LEFT))
                    .drag_to_scroll(true)
                    .auto_shrink([false; 2]);

                for fb_column in FileBrowserColumns::iter() {
                    let mut column = Column::initial(250.0);
                    match fb_column {
                        FileBrowserColumns::PathType => {
                            column = column.at_least(25.0);
                            column = column.at_most(25.0);
                        }
                        FileBrowserColumns::Size => {
                            column = column.at_most(80.0);
                        }
                        _ => {}
                    }
                    table = table.column(column);
                }
                table
            });

            // Ensure that browser is filled properly
            let paths = fs::read_dir(self.directory_path.as_str()).unwrap();
            if paths.count() > 0 && self.file_browser_table.total_rows() == 0 {
                self.is_first_load = true;
            }

            if self.path_changed || self.is_first_load {
                self.file_browser_table.clear_all_rows();
                let paths = fs::read_dir(self.directory_path.as_str()).unwrap();
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
                                absolute_path: self.directory_path.clone(),
                                tx: self.tx.clone(),
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
                self.is_first_load = false;
            }

            ctx.request_repaint();
        });
    }
}

// File browser config

#[derive(Default, Clone, Copy)]
pub struct FileBrowserConfig {}

#[derive(Clone)]
struct FileBrowserRow {
    name: String,
    new_name: String,
    size_ui: String,
    size: u64,
    date_modified: String,
    date_created: String,
    kind: String,
    path_type: String,
    tx: Sender<String>,
    absolute_path: String,
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
        if response.double_clicked() {
            if row_data.kind == "Folder" {
                let new_path = FileBrowserPath::navigate_to(
                    &row_data.name,
                    &row_data.absolute_path,
                );
                if new_path.get_path().len() > 0 {
                    let _ = row_data.tx.send(new_path.get_path());
                }
            }
        }


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