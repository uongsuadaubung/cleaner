use crate::lang::Lang;
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
    pub fn label<'a>(&self, lang: &'a Lang) -> &'a str {
        match self {
            OldFilePeriod::OneMonth => lang.period_1m,
            OldFilePeriod::TwoMonths => lang.period_2m,
            OldFilePeriod::ThreeMonths => lang.period_3m,
            OldFilePeriod::SixMonths => lang.period_6m,
            OldFilePeriod::OneYear => lang.period_1y,
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
    SelectOld(u64),
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
    lang: &Lang,
) -> ToolbarAction {
    let mut action = ToolbarAction::None;
    let has_selection = selected_count > 0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;

        // Nút Quét lại
        let rescan_btn = egui::Button::new(lang.btn_rescan).min_size(egui::vec2(100.0, 32.0));
        if ui.add(rescan_btn).clicked() {
            action = ToolbarAction::Rescan;
        }

        ui.separator();

        // Nút Sắp xếp
        let sort_label = if sort_count > 0 {
            format!("{} ({})", lang.btn_sort, sort_count)
        } else {
            lang.btn_sort.to_string()
        };
        let sort_btn = egui::Button::new(sort_label).min_size(egui::vec2(100.0, 32.0));
        if ui.add(sort_btn).clicked() {
            action = ToolbarAction::Sort;
        }

        ui.separator();

        if *show_period_selector {
            egui::ComboBox::from_id_salt("old_file_period")
                .selected_text(lang.period_select_time)
                .width(150.0)
                .show_ui(ui, |ui| {
                    for period in OldFilePeriod::ALL {
                        if ui
                            .selectable_value(selected_period, *period, period.label(lang))
                            .clicked()
                        {
                            action = ToolbarAction::SelectOld(period.days());
                            *show_period_selector = false;
                        }
                    }
                });

            if ui.button("❌").on_hover_text(lang.period_cancel_tooltip).clicked() {
                *show_period_selector = false;
            }
        } else {
            let select_old_btn =
                egui::Button::new(lang.btn_select_old).min_size(egui::vec2(120.0, 32.0));
            if ui.add(select_old_btn).clicked() {
                *show_period_selector = true;
            }
        }

        ui.separator();

        // Nút Bỏ chọn tất cả
        if has_selection {
            let deselect_btn =
                egui::Button::new(lang.btn_deselect).min_size(egui::vec2(100.0, 32.0));
            if ui.add(deselect_btn).clicked() {
                action = ToolbarAction::DeselectAll;
            }

            ui.separator();
        }

        // Nút Xóa
        if has_selection {
            let delete_label = format!("{} ({})", lang.dialog_btn_delete, selected_count);
            let delete_btn = egui::Button::new(
                egui::RichText::new(delete_label)
                    .color(colors::status_danger(ui.visuals().dark_mode)),
            )
            .min_size(egui::vec2(120.0, 32.0));

            if ui.add(delete_btn).clicked() {
                action = ToolbarAction::Delete;
            }
        }
    });

    action
}
