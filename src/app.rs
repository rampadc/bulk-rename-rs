use std::collections::HashMap;
use crate::components::file_browser::{FileAbsolutePath, FileBrowser, FileNewName};
use crate::components::regex::RegexMutation;
use egui::{Grid, Label, RichText};
use crate::components::case::{CaseMutation, CaseType};
use crate::utilities::mutation_pipeline::MutationPipeline;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    file_browser: FileBrowser,
    regex_mutation: RegexMutation,
    case_mutation: CaseMutation,

    replace_match: String,
    replace_with: String,
    replace_case_sensitive: bool,
    replace_first_only: bool,
    replace_enabled: bool,

    remove_first_n: String,
    remove_last_n: String,
    remove_from: String,
    remove_to: String,
    remove_chars: String,
    remove_words: String,
    remove_trim: bool,
    remove_digits: bool,
    remove_accents: bool,
    remove_enabled: bool,

    add_prefix: String,
    add_insert: String,
    add_at_pos: String,
    add_suffix: String,
    add_word_space: bool,
    add_enabled: bool,

    auto_date_type: String,
    auto_date_format: String,
    auto_date_enabled: bool,

    numbering_mode: String,
    numbering_at: String,
    numbering_start: String,
    numbering_increment: String,
    numbering_separator: String,
    numbering_pad: String,
    numbering_break: String,
    numbering_base: String,
    numbering_base_case: String,
    numbering_enabled: bool,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            file_browser: FileBrowser::new(),
            regex_mutation: RegexMutation::default(),
            case_mutation: CaseMutation::default(),
            replace_match: "".to_string(),
            replace_with: "".to_string(),
            replace_case_sensitive: false,
            replace_first_only: false,
            replace_enabled: false,
            remove_first_n: "".to_string(),
            remove_last_n: "".to_string(),
            remove_from: "".to_string(),
            remove_to: "".to_string(),
            remove_chars: "".to_string(),
            remove_words: "".to_string(),
            remove_trim: false,
            remove_digits: false,
            remove_accents: false,
            remove_enabled: false,
            add_prefix: "".to_string(),
            add_insert: "".to_string(),
            add_at_pos: "".to_string(),
            add_suffix: "".to_string(),
            add_word_space: false,
            add_enabled: false,
            auto_date_type: "".to_string(),
            auto_date_format: "".to_string(),
            auto_date_enabled: false,
            numbering_mode: "".to_string(),
            numbering_at: "".to_string(),
            numbering_start: "".to_string(),
            numbering_separator: "".to_string(),
            numbering_increment: "".to_string(),
            numbering_pad: "".to_string(),
            numbering_break: "".to_string(),
            numbering_base: "".to_string(),
            numbering_base_case: "".to_string(),
            numbering_enabled: false,
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
        let mut pipeline = MutationPipeline::new();
        pipeline.add_mutation(Box::new(RegexMutation {
            pattern: self.regex_mutation.pattern.clone(),
            substitution: self.regex_mutation.substitution.clone(),
            enabled: self.regex_mutation.enabled,
        }));
        pipeline.add_mutation(Box::new(CaseMutation {
            enabled: self.case_mutation.enabled,
            case_type: self.case_mutation.case_type.clone(),
        }));

        let mut new_names:HashMap<FileAbsolutePath, FileNewName> = HashMap::new();
        if let Ok(changing_files) = self.file_browser.selected_files_rx.try_recv() {
            // do stuff here
            for (k, v) in changing_files {
                let new_name = pipeline.apply_mutation(v.as_str());
                new_names.insert(k, new_name);
            }
            self.file_browser.selected_files_new_name_tx.try_send(new_names).expect("Cannot send new names to file browser");
        }

        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_theme_preference_buttons(ui);
                ui.label("[Use Ctrl to multi-select, or select & drag]");
            });
        });

        egui::SidePanel::right("editor_panel").show(ctx, |ui| {
            ui.add_space(8.0);
            self.regex_mutation.render(ui);
            ui.add_space(4.0);
            self.case_mutation.render(ui);
            ui.add_space(4.0);
            ui.group(|ui| {
                Grid::new("replace")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.checkbox(&mut self.replace_enabled, RichText::new("Replace").strong());
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
            ui.add_space(4.0);
            ui.group(|ui| {
                Grid::new("remove")
                    .num_columns(4)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.checkbox(&mut self.remove_enabled, RichText::new("Remove").strong());
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
            ui.add_space(4.0);
            ui.group(|ui| {
                Grid::new("add")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.checkbox(&mut self.add_enabled, RichText::new("Add").strong());
                        ui.end_row();

                        ui.add(Label::new("Prefix"));
                        ui.text_edit_singleline(&mut self.add_prefix);
                        ui.end_row();

                        ui.add(Label::new("Insert"));
                        ui.text_edit_singleline(&mut self.add_insert);
                        ui.end_row();

                        ui.add(Label::new("at pos"));
                        ui.text_edit_singleline(&mut self.add_at_pos);
                        ui.end_row();

                        ui.add(Label::new("Suffix"));
                        ui.text_edit_singleline(&mut self.add_suffix);
                        ui.end_row();

                        ui.checkbox(&mut self.add_word_space, "Word space");
                        ui.end_row();
                    });
            });
            ui.add_space(4.0);
            ui.group(|ui| {
                Grid::new("auto_date")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.checkbox(
                            &mut self.auto_date_enabled,
                            RichText::new("Auto Date").strong(),
                        );
                        ui.end_row();

                        ui.add(Label::new("Date type"));
                        ui.text_edit_singleline(&mut self.auto_date_type);
                        ui.end_row();

                        ui.add(Label::new("Format"));
                        ui.text_edit_singleline(&mut self.auto_date_format);
                        ui.end_row();
                    });
            });
            ui.add_space(4.0);
            ui.group(|ui| {
                Grid::new("numbering")
                    .num_columns(4)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.checkbox(
                            &mut self.numbering_enabled,
                            RichText::new("Numbering").strong(),
                        );
                        ui.end_row();

                        ui.add(Label::new("Mode"));
                        ui.text_edit_singleline(&mut self.numbering_mode);
                        ui.add(Label::new("at"));
                        ui.text_edit_singleline(&mut self.numbering_at);
                        ui.end_row();

                        ui.add(Label::new("Start"));
                        ui.text_edit_singleline(&mut self.numbering_start);
                        ui.add(Label::new("Incr."));
                        ui.text_edit_singleline(&mut self.numbering_increment);
                        ui.end_row();

                        ui.add(Label::new("Pad"));
                        ui.text_edit_singleline(&mut self.numbering_pad);
                        ui.add(Label::new("Separator"));
                        ui.text_edit_singleline(&mut self.numbering_separator);
                        ui.end_row();

                        ui.add(Label::new("Break"));
                        ui.text_edit_singleline(&mut self.numbering_break);
                        ui.end_row();

                        ui.add(Label::new(RichText::new("Base").strong()));
                        ui.end_row();

                        ui.add(Label::new("Base"));
                        ui.text_edit_singleline(&mut self.numbering_base);
                        ui.add(Label::new("Case"));
                        ui.text_edit_singleline(&mut self.numbering_base_case);
                        ui.end_row();
                    });
            });

            ui.add_space(8.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.file_browser.render(ui);
            ctx.request_repaint();
        });
    }
}
