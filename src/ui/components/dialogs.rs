use eframe::egui;

use crate::lang::Lang;
use crate::ui::colors;
use crate::ui::theme;
use crate::utils::format_size;

/// Trạng thái dialog
#[derive(Debug, Clone, PartialEq)]
pub enum DialogState {
    None,
    ConfirmDelete {
        file_count: usize,
        total_size: u64,
        permanent: bool,
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
    let t = &theme::DEFAULT;
    match state {
        DialogState::None => DialogResult::None,
        DialogState::ConfirmDelete { file_count, total_size, permanent } => {
            render_confirm_delete(ctx, *file_count, *total_size, *permanent, lang, t)
        }
        DialogState::ConfirmSort => render_confirm_sort(ctx, lang, t),
        DialogState::ResultMessage { title, message, is_error } => {
            render_result_message(ctx, title, message, *is_error, t)
        }
        DialogState::Processing { title, message, current, total } => {
            render_processing(ctx, title, message, *current, *total, lang, t)
        }
    }
}

fn render_processing(
    ctx: &egui::Context,
    title: &str,
    message: &str,
    current: usize,
    total: usize,
    lang: &Lang,
    t: &theme::Theme,
) -> DialogResult {
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .fixed_size(egui::vec2(t.dialog_processing_width, t.dialog_processing_height))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(t.space_md + 2.0);
                ui.label(egui::RichText::new(lang.dialog_please_wait).size(t.font_md));
                ui.add_space(t.space_md + 2.0);

                let progress = if total > 0 { current as f32 / total as f32 } else { 0.0 };
                ui.add(egui::ProgressBar::new(progress).text(format!("{} / {}", current, total)));

                ui.add_space(t.space_md);
                ui.label(
                    egui::RichText::new(message)
                        .small()
                        .color(colors::text_secondary(ui.visuals().dark_mode)),
                );
                ui.add_space(t.space_md + 2.0);
            });
        });

    DialogResult::None
}

fn render_confirm_delete(
    ctx: &egui::Context,
    file_count: usize,
    total_size: u64,
    permanent: bool,
    lang: &Lang,
    t: &theme::Theme,
) -> DialogResult {
    let mut result = DialogResult::None;

    let window_title = if permanent {
        format!("‼ {}", lang.dialog_confirm_delete_title)
    } else {
        lang.dialog_confirm_delete_title.to_string()
    };

    egui::Window::new(window_title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .min_width(t.dialog_delete_min_width)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(t.space_md + 2.0);
                let msg = if permanent {
                    lang.dialog_confirm_permanent_delete_msg
                } else {
                    lang.dialog_confirm_delete_msg
                };
                ui.label(egui::RichText::new(msg).size(t.font_md));
                ui.add_space(t.space_md);
                ui.label(format!("{} {}", lang.dialog_file_count, file_count));
                ui.label(format!("{} {}", lang.dialog_total_size, format_size(total_size)));
                ui.add_space(t.space_sm + 1.0);
                
                if permanent {
                    ui.label(
                        egui::RichText::new(lang.dialog_permanent_note)
                            .color(colors::status_danger(ui.visuals().dark_mode))
                            .strong()
                            .small(),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(lang.dialog_recycle_note)
                            .color(colors::status_success(ui.visuals().dark_mode))
                            .small(),
                    );
                }
                ui.add_space(t.space_xl - 5.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = t.dialog_btn_spacing;
                    let available = ui.available_width();
                    ui.add_space((available - (t.dialog_btn_width * 2.0 + t.dialog_btn_spacing)) / 2.0);

                    let btn_label = if permanent {
                        lang.dialog_btn_delete_permanent
                    } else {
                        lang.dialog_btn_delete
                    };

                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(btn_label).color(colors::text_on_accent(ui.visuals().dark_mode)),
                            )
                            .fill(colors::status_error_bg(ui.visuals().dark_mode))
                            .min_size(theme::dialog_btn_size(t.dialog_btn_width)),
                        )
                        .clicked()
                    {
                        result = DialogResult::Confirmed;
                    }

                    if ui
                        .add(
                            egui::Button::new(lang.dialog_btn_cancel)
                                .min_size(theme::dialog_btn_size(t.dialog_btn_width)),
                        )
                        .clicked()
                    {
                        result = DialogResult::Cancelled;
                    }
                });
                ui.add_space(t.space_md + 2.0);
            });
        });

    result
}

fn render_confirm_sort(ctx: &egui::Context, lang: &Lang, t: &theme::Theme) -> DialogResult {
    let mut result = DialogResult::None;

    egui::Window::new(lang.dialog_confirm_sort_title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .min_width(t.dialog_sort_min_width)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(t.space_md + 2.0);
                ui.label(egui::RichText::new(lang.dialog_confirm_sort_msg).size(t.font_md));
                ui.add_space(t.space_md);
                ui.label(lang.dialog_folders_label);
                ui.add_space(t.space_sm);

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

                ui.add_space(t.space_xl - 5.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = t.dialog_btn_spacing;
                    let available = ui.available_width();
                    ui.add_space((available - (t.dialog_btn_width * 2.0 + t.dialog_btn_spacing)) / 2.0);

                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(lang.dialog_btn_sort).color(colors::text_on_accent(ui.visuals().dark_mode)),
                            )
                            .fill(colors::status_success_bg(ui.visuals().dark_mode))
                            .min_size(theme::dialog_btn_size(t.dialog_btn_width)),
                        )
                        .clicked()
                    {
                        result = DialogResult::Confirmed;
                    }

                    if ui
                        .add(
                            egui::Button::new(lang.dialog_btn_cancel)
                                .min_size(theme::dialog_btn_size(t.dialog_btn_width)),
                        )
                        .clicked()
                    {
                        result = DialogResult::Cancelled;
                    }
                });
                ui.add_space(t.space_md + 2.0);
            });
        });

    result
}

fn render_result_message(
    ctx: &egui::Context,
    title: &str,
    message: &str,
    is_error: bool,
    t: &theme::Theme,
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
        .min_width(t.dialog_processing_width)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(t.space_md + 2.0);

                let color = if is_error {
                    colors::status_danger(ui.visuals().dark_mode)
                } else {
                    colors::status_success(ui.visuals().dark_mode)
                };

                ui.label(egui::RichText::new(message).size(t.font_sm).color(color));

                ui.add_space(t.space_xl - 5.0);

                if ui
                    .add(egui::Button::new("OK").min_size(theme::btn_size(t.btn_width_sm)))
                    .clicked()
                {
                    result = DialogResult::Closed;
                }

                ui.add_space(t.space_md + 2.0);
            });
        });

    result
}
