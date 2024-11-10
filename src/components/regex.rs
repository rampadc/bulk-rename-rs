
use egui::{Grid, Label, RichText, Ui};
use regex;
use crate::utilities::mutation_pipeline::Mutation;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RegexMutation {
    pub enabled: bool,
    pub pattern: String,
    pub substitution: String,
    pub including_extension: bool,
}

impl Default for RegexMutation {
    fn default() -> Self {
        Self {
            enabled: true,
            pattern: "".to_string(),
            substitution: "".to_string(),
            including_extension: false,
        }
    }
}
impl RegexMutation {
    pub fn render(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            Grid::new("regex")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.checkbox(&mut self.enabled, RichText::new("Regex").strong());
                    ui.end_row();

                    ui.add(Label::new("Match"));
                    ui.text_edit_singleline(&mut self.pattern);
                    ui.end_row();

                    ui.add(Label::new("Replace"));
                    ui.text_edit_singleline(&mut self.substitution);
                    ui.end_row();

                    ui.checkbox(&mut self.including_extension, "Include extension");
                    ui.end_row();
                });
        });
    }

    fn split_filename(filename: &str) -> (String, String) {
        let is_hidden = filename.starts_with('.');
        let filename = if is_hidden { &filename[1..] } else { filename };

        if let Some(last_dot_index) = filename.rfind('.') {
            let (name, ext) = filename.split_at(last_dot_index);
            (name.to_string(), ext[1..].to_string())
        } else {
            (filename.to_string(), "".to_string())
        }
    }

}

impl Mutation for RegexMutation {
    fn mutate(&self, input: &str) -> String {
        if self.enabled && self.pattern.len() > 0 {
            let (filename, extension) = Self::split_filename(input);
            if let Ok(regex) = regex::Regex::new(&self.pattern) {
                let regex_input = if self.including_extension { input } else { &filename };
                let replaced_filename = match regex.replace_all(regex_input, &self.substitution).parse() {
                    Ok(replaced) => replaced,
                    Err(err) => {
                        eprintln!("Error replacing all for regex: {:#?}", err);
                        input.to_string()
                    },
                };
                if !self.including_extension {
                    format!("{}.{}", replaced_filename, extension)
                } else {
                    replaced_filename
                }
            } else {
                input.to_string()
            }
        } else {
            eprintln!("Regex not enabled, pattern length = 0. Enabled: {}, Len: {}.", self.enabled, self.pattern.len());
            input.to_string()
        }
    }
}