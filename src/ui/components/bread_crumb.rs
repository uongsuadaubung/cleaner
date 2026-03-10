use crate::lang::Lang;
use crate::ui::colors;
use crate::ui::theme;
use eframe::egui;
use std::path::PathBuf;

/// Render breadcrumb đường dẫn.
///
/// - Bấm vào bất kỳ phần nào trong path sẽ thay `scan_path` bằng đường dẫn tới phần đó.
/// - Bấm nút "Change" mở dialog chọn thư mục mới.
/// - Trả về `true` nếu `scan_path` đã thay đổi (caller cần rescan).
///
/// `enabled` — nếu false (vd đang scan) thì ẩn nút Change và không cho click từng phần.
pub fn render_bread_crumb(
    ui: &mut egui::Ui,
    scan_path: &mut PathBuf,
    lang: &Lang,
    enabled: bool,
) -> bool {
    let t = &theme::DEFAULT;
    let is_dark = ui.visuals().dark_mode;
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(lang.path_label)
                .size(t.font_sm)
                .color(colors::text_secondary(is_dark)),
        );

        // ---- Các thành phần đường dẫn (segments) ----
        let components: Vec<PathBuf> = {
            let mut acc = PathBuf::new();
            let mut segs: Vec<PathBuf> = Vec::new();
            for comp in scan_path.components() {
                acc.push(comp);
                segs.push(acc.clone());
            }
            segs
        };

        let total = components.len();
        for (i, seg_path) in components.into_iter().enumerate() {
            let is_last = i == total - 1;

            // Tên hiển thị của segment này
            let label_text = if i == 0 {
                // Root / drive (vd "C:\") — giữ nguyên
                seg_path.to_string_lossy().to_string()
            } else {
                seg_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default()
            };

            if is_last {
                // Segment cuối — highlight bằng accent, không phải link
                ui.label(
                    egui::RichText::new(&label_text)
                        .size(t.font_sm)
                        .color(colors::accent(is_dark))
                        .strong(),
                );
            } else if enabled {
                // Segment giữa — clickable
                let resp = ui.add(
                    egui::Button::new(
                        egui::RichText::new(&label_text)
                            .size(t.font_sm)
                            .color(colors::text_secondary(is_dark)),
                    )
                    .frame(false),
                );

                if resp
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    *scan_path = seg_path;
                    changed = true;
                    // không break — egui cần hoàn tất vòng lặp layout
                }

                // Dấu phân cách
                ui.label(
                    egui::RichText::new("›")
                        .size(t.font_sm)
                        .color(colors::text_muted(is_dark)),
                );
            } else {
                // Disabled — hiển thị nhưng không click được
                ui.label(
                    egui::RichText::new(&label_text)
                        .size(t.font_sm)
                        .color(colors::text_muted(is_dark)),
                );
                ui.label(
                    egui::RichText::new("›")
                        .size(t.font_sm)
                        .color(colors::text_muted(is_dark)),
                );
            }
        }

        // ---- Nút Change ----
        if enabled {
            ui.add_space(t.space_sm);
            if ui.add(egui::Button::new(lang.btn_change).small()).clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .set_directory(&*scan_path)
                    .pick_folder()
            {
                *scan_path = path;
                changed = true;
            }
        }
    });

    changed
}
