use crate::lang::{Lang, Language};
use crate::ui::colors;
use crate::ui::theme;
use eframe::egui;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeSetting {
    System,
    Light,
    Dark,
}

impl ThemeSetting {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Light" => ThemeSetting::Light,
            "Dark" => ThemeSetting::Dark,
            _ => ThemeSetting::System,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeSetting::System => "System",
            ThemeSetting::Light => "Light",
            ThemeSetting::Dark => "Dark",
        }
    }
}

pub struct SettingsState {
    pub theme: ThemeSetting,
    pub language: Language,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            theme: ThemeSetting::System,
            language: Language::Vietnamese,
        }
    }
}

impl SettingsState {
    fn get_settings_path() -> PathBuf {
        crate::cache::settings_path()
    }

    pub fn load() -> Self {
        let mut state = Self::default();
        let path = Self::get_settings_path();
        if let Ok(mut file) = std::fs::File::open(&path) {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                for line in content.lines() {
                    let parts: Vec<&str> = line.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim();
                        let value = parts[1].trim();
                        match key {
                            "theme" => state.theme = ThemeSetting::from_str(value),
                            "language" => state.language = Language::from_str(value),
                            _ => {}
                        }
                    }
                }
            }
        }
        state
    }

    pub fn save(&self) {
        let path = Self::get_settings_path();
        // Đảm bảo thư mục tồn tại
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if let Ok(mut file) = std::fs::File::create(&path) {
            let content = format!(
                "theme={}\nlanguage={}\n",
                self.theme.as_str(),
                self.language.as_str()
            );
            let _ = file.write_all(content.as_bytes());
        }
    }
}

pub fn render_settings(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    state: &mut SettingsState,
    lang: &Lang,
) {
    let t = &theme::DEFAULT;

    ui.heading(
        egui::RichText::new(lang.settings_title)
            .size(t.font_heading)
            .color(colors::text_primary(ui.visuals().dark_mode)),
    );
    ui.add_space(t.space_xl);

    let mut changed = false;

    ui.group(|ui| {
        ui.label(
            egui::RichText::new(lang.settings_appearance)
                .strong()
                .size(t.font_md),
        );
        ui.add_space(t.space_md);

        ui.horizontal(|ui| {
            if ui
                .radio_value(
                    &mut state.theme,
                    ThemeSetting::System,
                    lang.settings_theme_system,
                )
                .clicked()
            {
                ctx.options_mut(|o| o.theme_preference = egui::ThemePreference::System);
                changed = true;
            }
            if ui
                .radio_value(
                    &mut state.theme,
                    ThemeSetting::Dark,
                    lang.settings_theme_dark,
                )
                .clicked()
            {
                ctx.options_mut(|o| o.theme_preference = egui::ThemePreference::Dark);
                changed = true;
            }
            if ui
                .radio_value(
                    &mut state.theme,
                    ThemeSetting::Light,
                    lang.settings_theme_light,
                )
                .clicked()
            {
                ctx.options_mut(|o| o.theme_preference = egui::ThemePreference::Light);
                changed = true;
            }
        });
    });

    ui.add_space(t.space_xl);

    ui.group(|ui| {
        ui.label(
            egui::RichText::new(lang.settings_language)
                .strong()
                .size(t.font_md),
        );
        ui.add_space(t.space_md);

        ui.horizontal(|ui| {
            let previous_language = state.language;
            egui::ComboBox::from_id_salt("language_combobox")
                .selected_text(if state.language == Language::Vietnamese {
                    "Tiếng Việt"
                } else {
                    "English"
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.language, Language::Vietnamese, "Tiếng Việt");
                    ui.selectable_value(&mut state.language, Language::English, "English");
                });
            if state.language != previous_language {
                changed = true;
            }
        });
    });

    if changed {
        state.save();
    }
}
