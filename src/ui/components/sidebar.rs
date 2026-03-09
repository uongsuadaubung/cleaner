use crate::ui::colors;
use eframe::egui;

/// Các trang chức năng của ứng dụng
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivePage {
    Cleanup,
    DuplicateFinder,
}

/// Cấu hình một mục menu trong sidebar
struct MenuItem {
    page: ActivePage,
    icon: &'static str,
    label: &'static str,
    tooltip: &'static str,
}

const MENU_ITEMS: &[MenuItem] = &[
    MenuItem {
        page: ActivePage::Cleanup,
        icon: "🧹",
        label: "Dọn dẹp",
        tooltip: "Dọn dẹp thư mục: xóa file cũ, sắp xếp file",
    },
    MenuItem {
        page: ActivePage::DuplicateFinder,
        icon: "🔍",
        label: "Trùng lặp",
        tooltip: "Tìm và xóa các file trùng lặp",
    },
];

/// Render sidebar bên trái, trả về page mới nếu người dùng chuyển trang
pub fn render_sidebar(ui: &mut egui::Ui, current_page: &mut ActivePage) {
    let sidebar_width = 72.0;

    ui.allocate_ui_with_layout(
        egui::vec2(sidebar_width, ui.available_height()),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.set_min_width(sidebar_width);
            ui.set_max_width(sidebar_width);

            ui.add_space(12.0);

            for item in MENU_ITEMS {
                let is_active = *current_page == item.page;

                let desired_size = egui::vec2(sidebar_width - 8.0, 60.0);
                let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

                let response = response.on_hover_text(item.tooltip);

                // --- Nền nút ---
                let bg_color = if is_active {
                    colors::SIDEBAR_ACTIVE_BG
                } else if response.hovered() {
                    colors::SIDEBAR_HOVER_BG
                } else {
                    egui::Color32::TRANSPARENT
                };

                if bg_color != egui::Color32::TRANSPARENT {
                    ui.painter().rect_filled(rect, 8.0, bg_color);
                }

                // --- Viền trái accent khi active ---
                if is_active {
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(rect.left_top(), egui::vec2(3.0, rect.height())),
                        0.0,
                        colors::ACCENT,
                    );
                }

                // --- Màu icon ---
                let icon_color = if is_active {
                    colors::TEXT_PRIMARY
                } else if response.hovered() {
                    colors::TEXT_SIDEBAR_HOVER
                } else {
                    colors::TEXT_SIDEBAR
                };

                ui.painter().text(
                    egui::pos2(rect.center().x, rect.top() + 10.0),
                    egui::Align2::CENTER_TOP,
                    item.icon,
                    egui::FontId::proportional(22.0),
                    icon_color,
                );

                // --- Màu label ---
                let label_color = if is_active {
                    colors::ACCENT
                } else if response.hovered() {
                    colors::TEXT_SIDEBAR_HOVER
                } else {
                    colors::TEXT_SIDEBAR
                };

                ui.painter().text(
                    egui::pos2(rect.center().x, rect.top() + 38.0),
                    egui::Align2::CENTER_TOP,
                    item.label,
                    egui::FontId::proportional(11.0),
                    label_color,
                );

                if response.clicked() {
                    *current_page = item.page;
                }

                ui.add_space(6.0);
            }
        },
    );
}
