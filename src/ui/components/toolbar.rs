use crate::lang::Lang;
use crate::ui::{colors, theme};
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

/// Phạm vi tìm kiếm file cũ
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OldFileScope {
    /// Chỉ thư mục hiện tại (không vào thư mục con)
    CurrentOnly,
    /// Toàn bộ cây thư mục (đệ quy)
    Recursive,
}

/// Enum hành động toolbar trả về
#[derive(Debug, Clone, PartialEq)]
pub enum ToolbarAction {
    None,
    Rescan,
    Sort,
    SelectOld { days: u64, scope: OldFileScope },
    Delete,
    DeselectAll,
}

/// Render toolbar và trả về hành động người dùng đã chọn
pub fn render_toolbar(
    ui: &mut egui::Ui,
    selected_period: &mut OldFilePeriod,
    selected_scope: &mut OldFileScope,
    selected_count: usize,
    sort_count: usize,
    show_period_selector: &mut bool,
    lang: &Lang,
) -> ToolbarAction {
    let t = &theme::DEFAULT;
    let mut action = ToolbarAction::None;
    let has_selection = selected_count > 0;
    let is_dark = ui.visuals().dark_mode;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = t.space_md;

        // ── Nút Quét lại ──────────────────────────────────────
        if ui
            .add(egui::Button::new(lang.btn_rescan).min_size(theme::btn_size(t.btn_width_md)))
            .clicked()
        {
            action = ToolbarAction::Rescan;
        }

        ui.separator();

        // ── Nút Sắp xếp ───────────────────────────────────────
        let sort_label = if sort_count > 0 {
            format!("{} ({})", lang.btn_sort, sort_count)
        } else {
            lang.btn_sort.to_string()
        };
        if ui
            .add(egui::Button::new(sort_label).min_size(theme::btn_size(t.btn_width_md)))
            .clicked()
        {
            action = ToolbarAction::Sort;
        }

        ui.separator();

        // ── Nút mở popup chọn file cũ ─────────────────────────
        let btn_response = ui.add(
            egui::Button::new(lang.btn_select_old).min_size(theme::btn_size(t.btn_width_lg)),
        );

        if btn_response.clicked() {
            *show_period_selector = !*show_period_selector;
        }

        // Vị trí popup: ngay bên dưới nút
        if *show_period_selector {
            let popup_pos = btn_response.rect.left_bottom() + egui::vec2(0.0, t.space_xs);

            let area_resp = egui::Area::new(egui::Id::new("old_file_period_popup"))
                .order(egui::Order::Foreground)
                .fixed_pos(popup_pos)
                .show(ui.ctx(), |ui| {
                    let frame = egui::Frame::popup(ui.style())
                        .corner_radius(theme::corner_radius(t.radius_md))
                        .inner_margin(theme::padding(t.card_padding_sm));

                    frame.show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = t.space_sm;

                        // ── Chọn khoảng thời gian ────────────────
                        ui.label(
                            egui::RichText::new(lang.period_select_label)
                                .color(colors::text_secondary(is_dark))
                                .size(t.font_sm),
                        );

                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = t.space_sm;
                            for period in OldFilePeriod::ALL {
                                let is_sel = *selected_period == *period;
                                let btn = egui::Button::new(
                                    egui::RichText::new(period.label(lang))
                                        .color(if is_sel {
                                            colors::accent(is_dark)
                                        } else {
                                            colors::text_primary(is_dark)
                                        })
                                        .size(t.font_sm),
                                )
                                .fill(if is_sel {
                                    colors::accent_subtle(is_dark)
                                } else {
                                    colors::transparent()
                                })
                                .stroke(egui::Stroke::new(
                                    if is_sel { 1.5 } else { 0.0 },
                                    colors::accent(is_dark),
                                ))
                                .corner_radius(theme::corner_radius(t.radius_sm))
                                .min_size(egui::vec2(t.btn_width_sm, t.btn_height));

                                if ui.add(btn).clicked() {
                                    *selected_period = *period;
                                }
                            }
                        });

                        ui.add(egui::Separator::default().spacing(t.space_sm));

                        // ── Phạm vi tìm kiếm ──────────────────────
                        ui.label(
                            egui::RichText::new(lang.period_scope_label)
                                .color(colors::text_secondary(is_dark))
                                .size(t.font_sm),
                        );

                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = t.space_sm;
                            for (scope, label) in [
                                (OldFileScope::CurrentOnly, lang.period_scope_current),
                                (OldFileScope::Recursive, lang.period_scope_recursive),
                            ] {
                                let is_sel = *selected_scope == scope;
                                let btn = egui::Button::new(
                                    egui::RichText::new(label)
                                        .color(if is_sel {
                                            colors::accent(is_dark)
                                        } else {
                                            colors::text_primary(is_dark)
                                        })
                                        .size(t.font_sm),
                                )
                                .fill(if is_sel {
                                    colors::accent_subtle(is_dark)
                                } else {
                                    colors::transparent()
                                })
                                .stroke(egui::Stroke::new(
                                    if is_sel { 1.5 } else { 0.0 },
                                    colors::accent(is_dark),
                                ))
                                .corner_radius(theme::corner_radius(t.radius_sm))
                                .min_size(egui::vec2(t.btn_width_lg, t.btn_height));

                                if ui.add(btn).clicked() {
                                    *selected_scope = scope;
                                }
                            }
                        });

                        ui.add(egui::Separator::default().spacing(t.space_sm));

                        // ── Xác nhận / Hủy ────────────────────────
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = t.space_sm;

                            let confirm_btn = egui::Button::new(
                                egui::RichText::new(lang.period_btn_confirm)
                                    .color(colors::status_success(is_dark))
                                    .strong()
                                    .size(t.font_sm),
                            )
                            .fill(colors::accent_subtle(is_dark))
                            .stroke(egui::Stroke::new(1.0, colors::status_success(is_dark)))
                            .corner_radius(theme::corner_radius(t.radius_sm))
                            .min_size(theme::btn_size(t.btn_width_md));

                            if ui.add(confirm_btn).clicked() {
                                action = ToolbarAction::SelectOld {
                                    days: selected_period.days(),
                                    scope: *selected_scope,
                                };
                                *show_period_selector = false;
                            }

                            let cancel_btn = egui::Button::new(
                                egui::RichText::new(lang.dialog_btn_cancel)
                                    .color(colors::text_secondary(is_dark))
                                    .size(t.font_sm),
                            )
                            .min_size(theme::btn_size(t.btn_width_sm));

                            if ui.add(cancel_btn).clicked() {
                                *show_period_selector = false;
                            }
                        });
                    });
                });

            // Đóng popup khi click ra ngoài (dùng press_origin để kiểm tra tọa độ chính xác)
            if ui.input(|i| i.pointer.any_pressed()) {
                let popup_rect = area_resp.response.rect;
                let btn_rect = btn_response.rect;
                let clicked_outside = ui
                    .input(|i| i.pointer.press_origin())
                    .map(|pos| !popup_rect.contains(pos) && !btn_rect.contains(pos))
                    .unwrap_or(false);
                if clicked_outside {
                    *show_period_selector = false;
                }
            }
        }

        ui.separator();

        // ── Bỏ chọn & Xóa (chỉ hiện khi có file được chọn) ──
        if has_selection {
            if ui
                .add(egui::Button::new(lang.btn_deselect).min_size(theme::btn_size(t.btn_width_md)))
                .clicked()
            {
                action = ToolbarAction::DeselectAll;
            }

            ui.separator();

            let delete_label = format!("{} ({})", lang.dialog_btn_delete, selected_count);
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(delete_label)
                            .color(colors::status_danger(is_dark)),
                    )
                    .min_size(theme::btn_size(t.btn_width_lg)),
                )
                .clicked()
            {
                action = ToolbarAction::Delete;
            }
        }
    });

    action
}
