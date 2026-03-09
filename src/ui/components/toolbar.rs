use crate::ui::colors;
use eframe::egui;

/// Khoảng thời gian chọn file cũ
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OldFilePeriod {
    OneMonth,
    TwoMonths,
    ThreeMonths,
    SixMonths,
    OneYear,
}

impl OldFilePeriod {
    pub fn label(&self) -> &str {
        match self {
            OldFilePeriod::OneMonth => "1 tháng",
            OldFilePeriod::TwoMonths => "2 tháng",
            OldFilePeriod::ThreeMonths => "3 tháng",
            OldFilePeriod::SixMonths => "6 tháng",
            OldFilePeriod::OneYear => "1 năm",
        }
    }

    pub fn days(&self) -> u64 {
        match self {
            OldFilePeriod::OneMonth => 30,
            OldFilePeriod::TwoMonths => 60,
            OldFilePeriod::ThreeMonths => 90,
            OldFilePeriod::SixMonths => 180,
            OldFilePeriod::OneYear => 365,
        }
    }

    pub const ALL: &[OldFilePeriod] = &[
        OldFilePeriod::OneMonth,
        OldFilePeriod::TwoMonths,
        OldFilePeriod::ThreeMonths,
        OldFilePeriod::SixMonths,
        OldFilePeriod::OneYear,
    ];
}

/// Enum hành động toolbar trả về
#[derive(Debug, Clone, PartialEq)]
pub enum ToolbarAction {
    None,
    Rescan,
    Sort,
    SelectOld(u64), // số ngày
    Delete,
    DeselectAll,
}

/// Render toolbar và trả về hành động người dùng đã chọn
pub fn render_toolbar(
    ui: &mut egui::Ui,
    selected_period: &mut OldFilePeriod,
    selected_count: usize,
    sort_count: usize,
    show_period_selector: &mut bool,
) -> ToolbarAction {
    let mut action = ToolbarAction::None;
    let has_selection = selected_count > 0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;

        // Nút Quét lại
        let rescan_btn = egui::Button::new("🔄 Quét lại").min_size(egui::vec2(100.0, 32.0));
        if ui.add(rescan_btn).clicked() {
            action = ToolbarAction::Rescan;
        }

        ui.separator();

        // Nút Sắp xếp
        let sort_label = if sort_count > 0 {
            format!("📂 Sắp xếp ({})", sort_count)
        } else {
            "📂 Sắp xếp".to_string()
        };
        let sort_btn = egui::Button::new(sort_label).min_size(egui::vec2(100.0, 32.0));
        if ui.add(sort_btn).clicked() {
            action = ToolbarAction::Sort;
        }

        ui.separator();

        if *show_period_selector {
            egui::ComboBox::from_id_salt("old_file_period")
                .selected_text("🕐 Chọn thời gian...")
                .width(150.0)
                .show_ui(ui, |ui| {
                    for period in OldFilePeriod::ALL {
                        if ui
                            .selectable_value(selected_period, *period, period.label())
                            .clicked()
                        {
                            action = ToolbarAction::SelectOld(period.days());
                            *show_period_selector = false;
                        }
                    }
                });

            if ui.button("❌").on_hover_text("Hủy chọn").clicked() {
                *show_period_selector = false;
            }
        } else {
            let select_old_btn =
                egui::Button::new("🕐 Chọn file cũ").min_size(egui::vec2(120.0, 32.0));
            if ui.add(select_old_btn).clicked() {
                *show_period_selector = true;
            }
        }

        ui.separator();

        // Nút Bỏ chọn tất cả
        if has_selection {
            let deselect_btn = egui::Button::new("⬜ Bỏ chọn").min_size(egui::vec2(100.0, 32.0));
            if ui.add(deselect_btn).clicked() {
                action = ToolbarAction::DeselectAll;
            }

            ui.separator();
        }

        // Nút Xóa
        if has_selection {
            let delete_label = format!("🗑 Xóa đã chọn ({})", selected_count);
            let delete_btn =
                egui::Button::new(egui::RichText::new(delete_label).color(colors::STATUS_DANGER))
                    .min_size(egui::vec2(120.0, 32.0));

            if ui.add(delete_btn).clicked() {
                action = ToolbarAction::Delete;
            }
        }
    });

    action
}
