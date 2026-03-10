use eframe::egui;

use crate::lang::Lang;
use crate::ui::colors;
use crate::utils::format_size;

/// Trạng thái dialog
#[derive(Debug, Clone, PartialEq)]
pub enum DialogState {
    None,
    ConfirmDelete {
        file_count: usize,
        total_size: u64,
    },
    ConfirmSort,
    ResultMessage {
        title: String,
        message: String,
        is_error: bool,
    },
    Processing {
        title: String,
        message: String,
        current: usize,
        total: usize,
    },
}

/// Kết quả xử lý dialog
#[derive(Debug, Clone, PartialEq)]
pub enum DialogResult {
    None,
    Confirmed,
    Cancelled,
    Closed,
}

/// Render dialog và trả về kết quả
pub fn render_dialog(ctx: &egui::Context, state: &DialogState, lang: &Lang) -> DialogResult {
    match state {
        DialogState::None => DialogResult::None,
        DialogState::ConfirmDelete {
            file_count,
            total_size,
        } => render_confirm_delete(ctx, *file_count, *total_size, lang),
        DialogState::ConfirmSort => render_confirm_sort(ctx, lang),
        DialogState::ResultMessage {
            title,
            message,
            is_error,
        } => render_result_message(ctx, title, message, *is_error),
        DialogState::Processing {
            title,
            message,
            current,
            total,
        } => render_processing(ctx, title, message, *current, *total, lang),
    }
}

/// Dialog đang xử lý (Loading/Progress)
fn render_processing(
    ctx: &egui::Context,
    title: &str,
    message: &str,
    current: usize,
    total: usize,
    lang: &Lang,
) -> DialogResult {
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .fixed_size(egui::vec2(300.0, 120.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.label(egui::RichText::new(lang.dialog_please_wait).size(16.0));
                ui.add_space(10.0);

                let progress = if total > 0 {
                    current as f32 / total as f32
                } else {
                    0.0
                };
                ui.add(egui::ProgressBar::new(progress).text(format!("{} / {}", current, total)));

                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(message)
                        .small()
                        .color(colors::text_secondary(ui.visuals().dark_mode)),
                );
                ui.add_space(10.0);
            });
        });

    DialogResult::None
}

/// Dialog xác nhận xóa
fn render_confirm_delete(
    ctx: &egui::Context,
    file_count: usize,
    total_size: u64,
    lang: &Lang,
) -> DialogResult {
    let mut result = DialogResult::None;

    egui::Window::new(lang.dialog_confirm_delete_title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .min_width(350.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new(lang.dialog_confirm_delete_msg).size(16.0),
                );
                ui.add_space(8.0);
                ui.label(format!("{} {}", lang.dialog_file_count, file_count));
                ui.label(format!(
                    "{} {}",
                    lang.dialog_total_size,
                    format_size(total_size)
                ));
                ui.add_space(5.0);
                ui.label(
                    egui::RichText::new(lang.dialog_recycle_note)
                        .color(colors::status_success(ui.visuals().dark_mode))
                        .small(),
                );
                ui.add_space(15.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 20.0;
                    let available = ui.available_width();
                    ui.add_space((available - 240.0) / 2.0);

                    let delete_btn = egui::Button::new(
                        egui::RichText::new(lang.dialog_btn_delete).color(egui::Color32::WHITE),
                    )
                    .fill(colors::status_error_bg(ui.visuals().dark_mode))
                    .min_size(egui::vec2(100.0, 35.0));

                    if ui.add(delete_btn).clicked() {
                        result = DialogResult::Confirmed;
                    }

                    let cancel_btn =
                        egui::Button::new(lang.dialog_btn_cancel).min_size(egui::vec2(100.0, 35.0));
                    if ui.add(cancel_btn).clicked() {
                        result = DialogResult::Cancelled;
                    }
                });
                ui.add_space(10.0);
            });
        });

    result
}

/// Dialog xác nhận sắp xếp
fn render_confirm_sort(ctx: &egui::Context, lang: &Lang) -> DialogResult {
    let mut result = DialogResult::None;

    egui::Window::new(lang.dialog_confirm_sort_title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .min_width(400.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new(lang.dialog_confirm_sort_msg).size(16.0),
                );
                ui.add_space(8.0);
                ui.label(lang.dialog_folders_label);
                ui.add_space(4.0);

                let folders = [
                    "📄 Documents/",
                    "🖼 Images/",
                    "🎬 Videos/",
                    "🎵 Music/",
                    "📦 Archives/",
                    "⚙ Programs/",
                    "💻 Code/",
                    "📎 Others/",
                ];
                for folder in &folders {
                    ui.label(
                        egui::RichText::new(*folder)
                            .color(colors::status_warning(ui.visuals().dark_mode))
                            .small(),
                    );
                }

                ui.add_space(15.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 20.0;
                    let available = ui.available_width();
                    ui.add_space((available - 240.0) / 2.0);

                    let confirm_btn = egui::Button::new(
                        egui::RichText::new(lang.dialog_btn_sort).color(egui::Color32::WHITE),
                    )
                    .fill(colors::status_success_bg(ui.visuals().dark_mode))
                    .min_size(egui::vec2(100.0, 35.0));

                    if ui.add(confirm_btn).clicked() {
                        result = DialogResult::Confirmed;
                    }

                    let cancel_btn =
                        egui::Button::new(lang.dialog_btn_cancel).min_size(egui::vec2(100.0, 35.0));
                    if ui.add(cancel_btn).clicked() {
                        result = DialogResult::Cancelled;
                    }
                });
                ui.add_space(10.0);
            });
        });

    result
}

/// Dialog thông báo kết quả
fn render_result_message(
    ctx: &egui::Context,
    title: &str,
    message: &str,
    is_error: bool,
) -> DialogResult {
    let mut result = DialogResult::None;

    let window_title = if is_error {
        format!("❌ {}", title)
    } else {
        format!("✅ {}", title)
    };

    egui::Window::new(window_title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .min_width(300.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);

                let color = if is_error {
                    colors::status_danger(ui.visuals().dark_mode)
                } else {
                    colors::status_success(ui.visuals().dark_mode)
                };

                ui.label(egui::RichText::new(message).size(14.0).color(color));

                ui.add_space(15.0);

                let ok_btn = egui::Button::new("OK").min_size(egui::vec2(80.0, 32.0));
                if ui.add(ok_btn).clicked() {
                    result = DialogResult::Closed;
                }

                ui.add_space(10.0);
            });
        });

    result
}
