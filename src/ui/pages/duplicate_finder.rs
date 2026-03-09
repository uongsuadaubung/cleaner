use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::ui::colors;
use crate::utils::format_size;
use eframe::egui;
use sha2::{Digest, Sha256};

pub type ScanSenderType = Sender<(ScanStatus, Option<Vec<DuplicateGroup>>)>;

#[derive(Clone, PartialEq, Default)]
pub enum ScanStatus {
    #[default]
    Idle,
    Scanning {
        message: String,
        current: usize,
        total: usize,
    },
    Hashing {
        message: String,
        current: usize,
        total: usize,
    },
    Deleting {
        message: String,
    },
    Done,
}

#[derive(Clone)]
pub struct DuplicateFile {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub selected: bool,
}

#[derive(Clone)]
pub struct DuplicateGroup {
    pub hash: String,
    pub size: u64,
    pub files: Vec<DuplicateFile>,
}

#[derive(Default)]
pub struct DuplicateFinderState {
    pub groups: Vec<DuplicateGroup>,
    pub status: ScanStatus,
    pub result_message: Option<(String, bool)>, // (message, is_error)
    pub scan_rx: Option<Receiver<(ScanStatus, Option<Vec<DuplicateGroup>>)>>,
    pub delete_rx: Option<Receiver<(ScanStatus, Option<crate::actions::cleaner::CleanResult>)>>,
}

/// Helper block: Calculate hash for a file (reads up to a reasonable limit, or chunks)
fn calculate_hash(path: &Path) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 65536]; // 64KB chunks
    loop {
        let n = file.read(&mut buffer).ok()?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Some(hex::encode(hasher.finalize()))
}

/// Lấy tất cả file recursively (return path, size, name)
fn collect_all_files(
    dir: &Path,
    files: &mut Vec<(PathBuf, u64, String)>,
    tx: &Option<ScanSenderType>,
) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(meta) = fs::metadata(&path) {
                if meta.is_dir() {
                    collect_all_files(&path, files, tx);
                } else if meta.is_file() {
                    let size = meta.len();
                    if size > 0 {
                        // Ignore 0 byte files or it will match everywhere
                        files.push((
                            path.clone(),
                            size,
                            entry.file_name().to_string_lossy().to_string(),
                        ));
                    }
                    if files.len().is_multiple_of(500)
                        && let Some(sender) = tx
                    {
                        let _ = sender.send((
                            ScanStatus::Scanning {
                                message: format!("Đã tìm thấy {} file...", files.len()),
                                current: files.len(),
                                total: 0,
                            },
                            None,
                        ));
                    }
                }
            }
        }
    }
}

/// Thread quét trùng lặp
pub fn scan_duplicates_task(root_path: PathBuf, tx: ScanSenderType) {
    // 1. Collect all files
    let _ = tx.send((
        ScanStatus::Scanning {
            message: "Đang phân tích cấu trúc thư mục...".to_string(),
            current: 0,
            total: 0,
        },
        None,
    ));

    let mut all_files = Vec::new();
    collect_all_files(&root_path, &mut all_files, &Some(tx.clone()));

    // 2. Nhóm theo size
    let mut size_groups: HashMap<u64, Vec<(PathBuf, String)>> = HashMap::new();
    for (path, size, name) in all_files {
        size_groups.entry(size).or_default().push((path, name));
    }

    // Only keep sizes that have > 1 file
    let mut potential_duplicates = Vec::new();
    for (size, files) in size_groups {
        if files.len() > 1 {
            for (path, name) in files {
                potential_duplicates.push((path, size, name));
            }
        }
    }

    // 3. Tính hash cho mọi file potential file
    let total_to_hash = potential_duplicates.len();
    let _ = tx.send((
        ScanStatus::Hashing {
            message: "Đang kiểm tra nội dung file...".to_string(),
            current: 0,
            total: total_to_hash,
        },
        None,
    ));

    let mut hash_groups: HashMap<String, DuplicateGroup> = HashMap::new();
    for (i, (path, size, name)) in potential_duplicates.into_iter().enumerate() {
        if i % 10 == 0 || size > 10 * 1024 * 1024 {
            // Cap update rate
            let _ = tx.send((
                ScanStatus::Hashing {
                    message: format!("Đang đọc nội dung... ({}/{})", i, total_to_hash),
                    current: i,
                    total: total_to_hash,
                },
                None,
            ));
        }

        if let Some(hash) = calculate_hash(&path) {
            let entry = hash_groups
                .entry(hash.clone())
                .or_insert_with(|| DuplicateGroup {
                    hash,
                    size,
                    files: Vec::new(),
                });
            entry.files.push(DuplicateFile {
                path,
                name,
                size,
                selected: false,
            });
        }
    }

    // 4. Lọc các nhóm trùng lặp (size >= 2)
    let mut final_groups: Vec<DuplicateGroup> = hash_groups
        .into_values()
        .filter(|g| g.files.len() > 1)
        .collect();

    // Sort groups by size desc
    final_groups.sort_by(|a, b| b.size.cmp(&a.size));

    let _ = tx.send((ScanStatus::Done, Some(final_groups)));
}

/// Render trang tìm file trùng lặp
pub fn render_duplicate_finder(
    ui: &mut egui::Ui,
    state: &mut DuplicateFinderState,
    scan_path: &mut std::path::PathBuf,
    ctx: &egui::Context,
) {
    // ---- BACKGROUND TASKS LISTENER ----
    if let Some(ref rx) = state.scan_rx {
        while let Ok((new_status, result)) = rx.try_recv() {
            state.status = new_status;
            if let Some(groups) = result {
                state.groups = groups;
            }
            ctx.request_repaint();
        }
    }

    if let Some(ref rx) = state.delete_rx {
        while let Ok((new_status, res)) = rx.try_recv() {
            state.status = new_status;
            if let Some(clean_res) = res {
                // Xóa các file đã xóa thành công khỏi groups
                for g in &mut state.groups {
                    g.files.retain(|f| {
                        // Tối ưu: Nếu file được trash thành công, ta không nên retain nó nữa
                        // Nhưng trash::delete_all không trả về danh sách file xóa thành công chi tiết,
                        // ta cứ giả định là những file `selected` đã bị xóa.
                        !f.selected
                    });
                }
                state.groups.retain(|g| g.files.len() > 1); // Remove empty groups or singletons

                let err_msg = if clean_res.failed.is_empty() {
                    (
                        format!("Đã chuyển {} file vào thùng rác.", clean_res.deleted),
                        false,
                    )
                } else {
                    (
                        format!("Đã xảy ra lỗi xóa {} file.", clean_res.failed.len()),
                        true,
                    )
                };
                state.result_message = Some(err_msg);
            }
            ctx.request_repaint();
        }
    }

    // ---- HEADER THÔNG TIN  ----
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("🔍 Trình Tìm File Trùng Lặp")
                .size(20.0)
                .strong()
                .color(colors::TEXT_PRIMARY),
        );
    });

    ui.add_space(8.0);

    // ---- CHỌN ĐƯỜNG DẪN ----
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("📁 Đường dẫn:").color(colors::TEXT_SECONDARY));
        ui.label(
            egui::RichText::new(scan_path.display().to_string())
                .color(colors::ACCENT)
                .strong(),
        );

        if matches!(state.status, ScanStatus::Idle | ScanStatus::Done)
            && ui
                .add(egui::Button::new("📂 Thay đổi...").small())
                .clicked()
            && let Some(path) = rfd::FileDialog::new()
                .set_directory(&*scan_path)
                .pick_folder()
        {
            *scan_path = path;
            // Reset kết quả quét cũ khi đổi thư mục
            state.groups.clear();
            state.status = ScanStatus::Idle;
            state.result_message = None;
        }
    });

    ui.add_space(8.0);

    // ---- TOOLBAR QUÉT ----
    ui.horizontal(|ui| {
        if matches!(state.status, ScanStatus::Idle | ScanStatus::Done)
            && ui
                .add(egui::Button::new("▶ Quét file trùng lặp").min_size(egui::vec2(160.0, 32.0)))
                .clicked()
        {
            let (tx, rx) = channel();
            state.scan_rx = Some(rx);
            state.status = ScanStatus::Scanning {
                message: "Bắt đầu...".into(),
                current: 0,
                total: 0,
            };
            state.groups.clear();
            state.result_message = None;

            let path_clone = scan_path.to_path_buf();
            std::thread::spawn(move || {
                scan_duplicates_task(path_clone, tx);
            });
        }
    });

    ui.add_space(8.0);
    ui.separator();

    // ---- TRẠNG THÁI & LỖI ----
    if let Some((msg, is_error)) = &state.result_message {
        ui.add_space(8.0);
        let color = if *is_error {
            colors::STATUS_DANGER
        } else {
            colors::STATUS_SUCCESS
        };
        ui.label(egui::RichText::new(msg).color(color).strong());
        ui.add_space(4.0);
    }

    match &state.status {
        ScanStatus::Idle => {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("Tính năng tìm file giống nhau qua nội dung (hash) trong thư mục gốc được chọn.").color(colors::TEXT_MUTED));
                ui.add_space(16.0);
                egui::Frame::new()
                    .fill(colors::ACCENT_SUBTLE)
                    .stroke(egui::Stroke::new(1.0, colors::ACCENT.gamma_multiply(0.15)))
                    .corner_radius(egui::CornerRadius::same(12))
                    .inner_margin(egui::Margin::same(20))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Nhấn 'Quét file trùng lặp' để bắt đầu!").color(colors::ACCENT));
                    });
            });
            return;
        }
        ScanStatus::Scanning {
            message, current, ..
        } => {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.spinner();
                ui.add_space(16.0);
                ui.label(egui::RichText::new("Đang quét thư mục...").strong());
                ui.label(
                    egui::RichText::new(message)
                        .color(colors::TEXT_MUTED)
                        .small(),
                );
                ui.label(
                    egui::RichText::new(format!("Đã duyệt {} mục", current))
                        .color(colors::TEXT_SECONDARY),
                );
            });
            return;
        }
        ScanStatus::Hashing {
            message,
            current,
            total,
        } => {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.spinner();
                ui.add_space(16.0);
                ui.label(egui::RichText::new("Đang đối chiếu file...").strong());
                let progress = if *total > 0 {
                    *current as f32 / *total as f32
                } else {
                    0.0
                };
                ui.add(egui::ProgressBar::new(progress).show_percentage());
                ui.label(
                    egui::RichText::new(message)
                        .color(colors::TEXT_MUTED)
                        .small(),
                );
            });
            return;
        }
        ScanStatus::Deleting { message } => {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.spinner();
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("Đang xóa...")
                        .strong()
                        .color(colors::STATUS_DANGER),
                );
                ui.label(
                    egui::RichText::new(message)
                        .color(colors::TEXT_MUTED)
                        .small(),
                );
            });
            return;
        }
        ScanStatus::Done => {
            // Hiển thị kết quả bên dưới
        }
    }

    // ---- KẾT QUẢ ----
    if state.groups.is_empty() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("Không tìm thấy file trùng lặp nào! 🎉")
                    .size(18.0)
                    .color(colors::STATUS_SUCCESS),
            );
        });
        return;
    }

    // ToolBar của kết quả
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let mut total_selected = 0;
        let mut selected_size = 0;
        for g in &state.groups {
            for f in &g.files {
                if f.selected {
                    total_selected += 1;
                    selected_size += f.size;
                }
            }
        }

        let groups_len = state.groups.len();
        ui.label(
            egui::RichText::new(format!("Tìm thấy {} nhóm trùng lặp", groups_len))
                .strong()
                .color(colors::ACCENT),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if total_selected > 0 {
                if ui
                    .button(
                        egui::RichText::new(format!(
                            "🗑 Xóa {} file ({})",
                            total_selected,
                            format_size(selected_size)
                        ))
                        .color(colors::STATUS_ERROR_BG),
                    )
                    .clicked()
                {
                    // Thu thập danh sách file được chọn để xóa
                    let mut to_delete = Vec::new();
                    for g in &state.groups {
                        for f in &g.files {
                            if f.selected {
                                to_delete.push(f.path.clone());
                            }
                        }
                    }
                    if !to_delete.is_empty() {
                        let (tx, rx) = channel();
                        state.delete_rx = Some(rx);
                        state.status = ScanStatus::Deleting {
                            message: "Đang chuyển file vào Recycle Bin...".into(),
                        };
                        std::thread::spawn(move || {
                            let mut result = crate::actions::cleaner::CleanResult {
                                deleted: 0,
                                failed: Vec::new(),
                            };
                            match trash::delete_all(&to_delete) {
                                Ok(()) => result.deleted = to_delete.len(),
                                Err(e) => result.failed.push(("Xóa".to_string(), e.to_string())),
                            }
                            let _ = tx.send((ScanStatus::Done, Some(result)));
                        });
                    }
                }

                if ui.button("Bỏ chọn tất cả").clicked() {
                    for g in &mut state.groups {
                        for f in &mut g.files {
                            f.selected = false;
                        }
                    }
                }
            }
            if ui.button("⚡ Chọn nhanh file bản sao").clicked() {
                // Giữ lại 1 bản gốc (unselected), chọn (selected) các bản còn lại trong nhóm
                for g in &mut state.groups {
                    let mut first = true;
                    // Ta sort file theo độ dài tên để ưu tiên giữ file tên ngắn (thường là bản gốc)
                    g.files.sort_by(|a, b| a.name.len().cmp(&b.name.len()));

                    for f in &mut g.files {
                        if first {
                            f.selected = false; // Gốc
                            first = false;
                        } else {
                            f.selected = true; // Bản sao
                        }
                    }
                }
            }
        });
    });
    ui.add_space(8.0);

    // ---- DANH SÁCH THEO NHÓM ----
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for (i, group) in state.groups.iter_mut().enumerate() {
                ui.add_space(16.0);
                egui::Frame::new()
                    .fill(colors::ACCENT_SUBTLE.linear_multiply(0.2)) // Dùng màu nhạt hơn
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let short_hash = if group.hash.len() >= 8 {
                                &group.hash[0..8]
                            } else {
                                &group.hash
                            };
                            ui.label(
                                egui::RichText::new(format!(
                                    "📦 Nhóm {} ({} / mỗi file)",
                                    i + 1,
                                    format_size(group.size)
                                ))
                                .color(colors::TEXT_PRIMARY)
                                .strong(),
                            );
                            ui.label(
                                egui::RichText::new(format!("Hash: {}", short_hash))
                                    .color(colors::TEXT_MUTED)
                                    .small(),
                            );
                        });

                        ui.add_space(4.0);

                        for file in &mut group.files {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut file.selected, "");

                                let text_color = if file.selected {
                                    colors::STATUS_DANGER
                                } else {
                                    colors::FILE_NORMAL
                                };

                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new(&file.name).color(text_color).strong(),
                                    );
                                    // Rút gọn đường dẫn nếu cần hoặc để nguyên
                                    ui.label(
                                        egui::RichText::new(file.path.display().to_string())
                                            .color(colors::TEXT_MUTED)
                                            .small(),
                                    );
                                });
                            });
                            ui.add_space(2.0);
                        }
                    });
            }
        });
}
