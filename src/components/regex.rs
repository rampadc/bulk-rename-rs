
use egui::{Grid, Label, RichText, Ui};
use regex;
use crate::utilities::mutation_pipeline::Mutation;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RegexMutation {
    pub enabled: bool,
    pub pattern: String,
    pub substitution: String,
}

impl Default for RegexMutation {
    fn default() -> Self {
        Self {
            enabled: true,
            pattern: "".to_string(),
            substitution: "".to_string(),
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
                });
        });
    }
}

impl Mutation for RegexMutation {
    fn mutate(&self, input: &str) -> String {
        if self.enabled && self.pattern.len() > 0 {
            if let Ok(regex) = regex::Regex::new(&self.pattern) {
                match regex.replace_all(input, &self.substitution).parse() {
                    Ok(replaced) => replaced,
                    Err(err) => {
                        eprintln!("Error replacing all for regex: {:#?}", err);
                        input.to_string()
                    },
                }
            } else {
                input.to_string()
            }
        } else {
            // eprintln!("Regex not enabled, pattern length = 0. Enabled: {}, Len: {}.", self.enabled, self.pattern.len());
            input.to_string()
        }
    }
}