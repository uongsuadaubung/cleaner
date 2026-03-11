use crate::lang::Lang;
use crate::ui::colors;
use crate::ui::theme;
use eframe::egui;

/// Các trang chức năng của ứng dụng
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivePage {
    Cleanup,
    DuplicateFinder,
    Settings,
}

/// Render sidebar bên trái
pub fn render_sidebar(ui: &mut egui::Ui, current_page: &mut ActivePage, lang: &Lang) {
    let t = &theme::DEFAULT;
    let sidebar_width = t.sidebar_width;

    let items = [
        (ActivePage::Cleanup, "🧹", lang.nav_cleanup, lang.nav_cleanup_tooltip),
        (ActivePage::DuplicateFinder, "🔍", lang.nav_duplicates, lang.nav_duplicates_tooltip),
        (ActivePage::Settings, "⚙", lang.nav_settings, lang.nav_settings_tooltip),
    ];

    ui.allocate_ui_with_layout(
        egui::vec2(sidebar_width, ui.available_height()),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.set_min_width(sidebar_width);
            ui.set_max_width(sidebar_width);

            ui.add_space(t.space_md + 4.0); // 12px top

            for (page, icon, label, tooltip) in &items {
                let is_active = *current_page == *page;

                let desired_size = egui::vec2(sidebar_width - 8.0, t.sidebar_item_height);
                let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

                let response = response.on_hover_text(*tooltip);

                let bg_color = if is_active {
                    colors::sidebar_active_bg(ui.visuals().dark_mode)
                } else if response.hovered() {
                    colors::sidebar_hover_bg(ui.visuals().dark_mode)
                } else {
                    colors::transparent()
                };

                if bg_color != colors::transparent() {
                    ui.painter().rect_filled(rect, theme::corner_radius(t.radius_sm), bg_color);
                }

                if is_active {
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(rect.left_top(), egui::vec2(3.0, rect.height())),
                        0.0,
                        colors::accent(ui.visuals().dark_mode),
                    );
                }

                let icon_color = if is_active {
                    colors::text_primary(ui.visuals().dark_mode)
                } else if response.hovered() {
                    colors::text_sidebar_hover(ui.visuals().dark_mode)
                } else {
                    colors::text_sidebar(ui.visuals().dark_mode)
                };

                ui.painter().text(
                    egui::pos2(rect.center().x, rect.top() + 10.0),
                    egui::Align2::CENTER_TOP,
                    *icon,
                    egui::FontId::proportional(t.font_sidebar_icon),
                    icon_color,
                );

                let label_color = if is_active {
                    colors::accent(ui.visuals().dark_mode)
                } else if response.hovered() {
                    colors::text_sidebar_hover(ui.visuals().dark_mode)
                } else {
                    colors::text_sidebar(ui.visuals().dark_mode)
                };

                ui.painter().text(
                    egui::pos2(rect.center().x, rect.top() + 38.0),
                    egui::Align2::CENTER_TOP,
                    *label,
                    egui::FontId::proportional(t.font_sidebar_label),
                    label_color,
                );

                if response.clicked() {
                    *current_page = *page;
                }

                ui.add_space(t.space_md - 2.0); // 6px gap between items
            }
        },
    );
}
