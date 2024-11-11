use std::fmt;
use egui::{ComboBox, Grid, RichText, Ui};
use crate::utilities::mutation_pipeline::Mutation;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CaseMutation {
    pub enabled: bool,
    pub case_type: CaseType,
}

#[derive(Debug, PartialEq, Default, Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum CaseType {
    #[default]
    None,
    LowerCamelCase,
    UpperCamelCase,
    ShoutyKebabCase,
    ShoutySnakeCase,
    SnakeCase,
    TitleCase,
    KebabCase,
    UpperCase,
    LowerCase,
}

impl fmt::Display for CaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CaseType::None => write!(f, "None"),
            CaseType::LowerCamelCase => write!(f, "lower_camel_case"),
            CaseType::UpperCamelCase => write!(f, "Upper_Camel_Case"),
            CaseType::ShoutyKebabCase => write!(f, "SHOUTY-KEBAB-CASE"),
            CaseType::ShoutySnakeCase => write!(f, "SHOUTY_SNAKE_CASE"),
            CaseType::SnakeCase => write!(f, "snake_case"),
            CaseType::TitleCase => write!(f, "Title Case"),
            CaseType::KebabCase => write!(f, "kebab-case"),
            CaseType::UpperCase => write!(f, "UPPER CASE"),
            CaseType::LowerCase => write!(f, "lower case"),
        }
    }
}

impl Default for CaseMutation {
    fn default() -> Self {
        Self {
            case_type: CaseType::None,
            enabled: false
        }
    }
}

impl CaseMutation {
    pub fn render(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            Grid::new("case")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.checkbox(&mut self.enabled, RichText::new("Case").strong());
                    ui.end_row();

                    ComboBox::from_label("")
                        .selected_text(format!("{:?}", self.case_type))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.case_type, CaseType::None, "None");
                            ui.selectable_value(&mut self.case_type, CaseType::LowerCamelCase, "Lower Camel Case");
                            ui.selectable_value(&mut self.case_type, CaseType::UpperCamelCase, "Upper Camel Case");
                            ui.selectable_value(&mut self.case_type, CaseType::ShoutyKebabCase, "SHOUTY-KEBAB-CASE");
                            ui.selectable_value(&mut self.case_type, CaseType::ShoutySnakeCase, "SHOUTY_SNAKE_CASE");
                            ui.selectable_value(&mut self.case_type, CaseType::SnakeCase, "snake_case");
                            ui.selectable_value(&mut self.case_type, CaseType::TitleCase, "Title Case");
                            ui.selectable_value(&mut self.case_type, CaseType::KebabCase, "kebab-case");
                        });
                    ui.end_row();
                });
        });
    }

    fn extract_base_name(filename: &str) -> String {
        if filename.starts_with('.') {
            filename.to_string()
        } else {
            filename.split('.').next().unwrap().to_string()
        }
    }

}

impl Mutation for CaseMutation {
    fn mutate(&self, input: &str) -> String {
        if self.enabled {
            match &self.case_type {
                CaseType::None => input.to_string(),
                CaseType::LowerCamelCase => heck::AsLowerCamelCase(input).to_string(),
                CaseType::UpperCamelCase => heck::AsUpperCamelCase(input).to_string(),
                CaseType::ShoutyKebabCase => heck::AsShoutyKebabCase(input).to_string(),
                CaseType::ShoutySnakeCase => heck::AsShoutySnakeCase(input).to_string(),
                CaseType::SnakeCase => heck::AsSnakeCase(input).to_string(),
                CaseType::TitleCase => heck::AsTitleCase(input).to_string(),
                CaseType::KebabCase => heck::AsTrainCase(input).to_string(),
                CaseType::UpperCase => input.to_uppercase().to_string(),
                CaseType::LowerCase => input.to_lowercase().to_string(),
            }
        } else {
            input.to_string()
        }
    }
}