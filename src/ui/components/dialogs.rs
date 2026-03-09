use eframe::egui;

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
pub fn render_dialog(ctx: &egui::Context, state: &DialogState) -> DialogResult {
    match state {
        DialogState::None => DialogResult::None,
        DialogState::ConfirmDelete {
            file_count,
            total_size,
        } => render_confirm_delete(ctx, *file_count, *total_size),
        DialogState::ConfirmSort => render_confirm_sort(ctx),
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
        } => render_processing(ctx, title, message, *current, *total),
    }
}

/// Dialog đang xử lý (Loading/Progress)
fn render_processing(
    ctx: &egui::Context,
    title: &str,
    message: &str,
    current: usize,
    total: usize,
) -> DialogResult {
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .fixed_size(egui::vec2(300.0, 120.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("⏳ Vui lòng chờ...").size(16.0));
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
                        .color(colors::TEXT_SECONDARY),
                );
                ui.add_space(10.0);
            });
        });

    DialogResult::None
}

/// Dialog xác nhận xóa
fn render_confirm_delete(ctx: &egui::Context, file_count: usize, total_size: u64) -> DialogResult {
    let mut result = DialogResult::None;

    egui::Window::new("⚠ Xác nhận xóa")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .min_width(350.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Bạn có chắc muốn xóa các file đã chọn?").size(16.0));
                ui.add_space(8.0);
                ui.label(format!("Số file: {}", file_count));
                ui.label(format!("Tổng dung lượng: {}", format_size(total_size)));
                ui.add_space(5.0);
                ui.label(
                    egui::RichText::new("📋 File sẽ được chuyển vào Recycle Bin")
                        .color(colors::STATUS_SUCCESS)
                        .small(),
                );
                ui.add_space(15.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 20.0;
                    let available = ui.available_width();
                    ui.add_space((available - 240.0) / 2.0);

                    let delete_btn =
                        egui::Button::new(egui::RichText::new("🗑 Xóa").color(egui::Color32::WHITE))
                            .fill(colors::STATUS_ERROR_BG)
                            .min_size(egui::vec2(100.0, 35.0));

                    if ui.add(delete_btn).clicked() {
                        result = DialogResult::Confirmed;
                    }

                    let cancel_btn = egui::Button::new("Hủy").min_size(egui::vec2(100.0, 35.0));
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
fn render_confirm_sort(ctx: &egui::Context) -> DialogResult {
    let mut result = DialogResult::None;

    egui::Window::new("📂 Xác nhận sắp xếp")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .min_width(400.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Sắp xếp file vào thư mục theo loại?").size(16.0));
                ui.add_space(8.0);
                ui.label("Các thư mục sẽ được tạo:");
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
                            .color(colors::STATUS_WARNING)
                            .small(),
                    );
                }

                ui.add_space(15.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 20.0;
                    let available = ui.available_width();
                    ui.add_space((available - 240.0) / 2.0);

                    let confirm_btn = egui::Button::new(
                        egui::RichText::new("📂 Sắp xếp").color(egui::Color32::WHITE),
                    )
                    .fill(colors::STATUS_SUCCESS_BG)
                    .min_size(egui::vec2(100.0, 35.0));

                    if ui.add(confirm_btn).clicked() {
                        result = DialogResult::Confirmed;
                    }

                    let cancel_btn = egui::Button::new("Hủy").min_size(egui::vec2(100.0, 35.0));
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
                    colors::STATUS_DANGER
                } else {
                    colors::STATUS_SUCCESS
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
