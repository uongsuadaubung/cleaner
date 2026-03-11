use std::path::PathBuf;

use eframe::egui;

use crate::ui::components::sidebar::{self, ActivePage};
use crate::ui::theme;

/// State chính của ứng dụng
pub struct FolderCleanerApp {
    active_page: ActivePage,
    scan_path: PathBuf,
    cleanup_state: crate::ui::pages::cleanup::CleanupState,
    duplicate_finder_state: crate::ui::pages::duplicate_finder::DuplicateFinderState,
    settings_state: crate::ui::pages::settings::SettingsState,
}

impl FolderCleanerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::setup_fonts(&cc.egui_ctx);
        Self::setup_styles(&cc.egui_ctx);

        let scan_path = dirs::download_dir().unwrap_or_else(|| {
            eprintln!("Cannot find Downloads folder, using current directory");
            PathBuf::from(".")
        });

        let settings_state = crate::ui::pages::settings::SettingsState::load();
        let theme_preference = match settings_state.theme {
            crate::ui::pages::settings::ThemeSetting::System => egui::ThemePreference::System,
            crate::ui::pages::settings::ThemeSetting::Dark => egui::ThemePreference::Dark,
            crate::ui::pages::settings::ThemeSetting::Light => egui::ThemePreference::Light,
        };
        cc.egui_ctx
            .options_mut(|o| o.theme_preference = theme_preference);

        // Khởi tạo cleanup state và bắt đầu scan bất đồng bộ
        let mut cleanup_state = crate::ui::pages::cleanup::CleanupState::default();
        let lang = settings_state.language.strings();
        cleanup_state.rescan(&scan_path, &lang);

        Self {
            active_page: ActivePage::Cleanup,
            scan_path,
            cleanup_state,
            duplicate_finder_state: Default::default(),
            settings_state,
        }
    }

    /// Cấu hình font hỗ trợ tiếng Việt
    fn setup_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        let font_path = std::path::Path::new("C:\\Windows\\Fonts\\segoeui.ttf");
        if let Ok(font_data) = std::fs::read(font_path) {
            fonts.font_data.insert(
                "segoe_ui".to_string(),
                egui::FontData::from_owned(font_data).into(),
            );

            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "segoe_ui".to_string());

            fonts
                .families
                .get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .insert(0, "segoe_ui".to_string());
        } else {
            eprintln!("Segoe UI font not found, using default font");
        }

        // Load Segoe UI Emoji as fallback for modern emoji (🧹🔍⚙…)
        let emoji_font_path = std::path::Path::new("C:\\Windows\\Fonts\\seguiemj.ttf");
        if let Ok(font_data) = std::fs::read(emoji_font_path) {
            fonts.font_data.insert(
                "segoe_ui_emoji".to_string(),
                egui::FontData::from_owned(font_data).into(),
            );
            // Append as fallback so Segoe UI is tried first, emoji font fills the gaps
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                family.push("segoe_ui_emoji".to_string());
            }
        } else {
            eprintln!("Segoe UI Emoji font not found, some icons may not display");
        }

        ctx.set_fonts(fonts);
    }

    fn setup_styles(ctx: &egui::Context) {
        let t = &theme::DEFAULT;
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(t.item_spacing_x, t.item_spacing_y);
        let r_sm = theme::corner_radius(t.radius_sm);
        style.visuals.widgets.noninteractive.corner_radius = r_sm;
        style.visuals.widgets.inactive.corner_radius = r_sm;
        style.visuals.widgets.active.corner_radius = r_sm;
        style.visuals.widgets.hovered.corner_radius = r_sm;
        style.visuals.window_corner_radius = theme::corner_radius(t.radius_md);
        ctx.set_style(style);
    }
}

impl eframe::App for FolderCleanerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Resolve current language strings once per frame
        let lang = self.settings_state.language.strings();

        // Left panel - Sidebar navigation
        egui::SidePanel::left("sidebar_panel")
            .exact_width(theme::DEFAULT.sidebar_width)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(ctx.style().visuals.panel_fill)
                    .stroke(egui::Stroke::NONE),
            )
            .show(ctx, |ui| {
                sidebar::render_sidebar(ui, &mut self.active_page, &lang);
            });

        // Central panel
        egui::CentralPanel::default().show(ctx, |ui| match self.active_page {
            ActivePage::Cleanup => {
                crate::ui::pages::cleanup::render_cleanup(
                    ui,
                    ctx,
                    &mut self.cleanup_state,
                    &mut self.scan_path,
                    &self.settings_state.exclude_list,
                    &lang,
                );
            }
            ActivePage::DuplicateFinder => {
                crate::ui::pages::duplicate_finder::render_duplicate_finder(
                    ui,
                    &mut self.duplicate_finder_state,
                    &mut self.scan_path,
                    ctx,
                    &lang,
                );
            }
            ActivePage::Settings => {
                crate::ui::pages::settings::render_settings(
                    ui,
                    ctx,
                    &mut self.settings_state,
                    &lang,
                );
            }
        });
    }
}
