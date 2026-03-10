use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::lang::Lang;
use crate::ui::colors;
use crate::ui::components::bread_crumb;
use crate::ui::theme;
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

/// Helper block: Calculate hash for a file
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

/// Lấy tất cả file recursively
fn collect_all_files(
    dir: &Path,
    files: &mut Vec<(PathBuf, u64, String)>,
    tx: &Option<ScanSenderType>,
    found_label: &str,
) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(meta) = fs::metadata(&path) {
                if meta.is_dir() {
                    collect_all_files(&path, files, tx, found_label);
                } else if meta.is_file() {
                    let size = meta.len();
                    if size > 0 {
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
                                message: format!("{} {} file...", found_label, files.len()),
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

/// Thread quét trùng lặp (với chuỗi ngôn ngữ truyền vào)
pub fn scan_duplicates_task(
    root_path: PathBuf,
    tx: ScanSenderType,
    analyzing_msg: String,
    found_label: String,
    checking_msg: String,
    reading_msg: String,
) {
    // 1. Collect all files
    let _ = tx.send((
        ScanStatus::Scanning {
            message: analyzing_msg,
            current: 0,
            total: 0,
        },
        None,
    ));

    let mut all_files = Vec::new();
    collect_all_files(&root_path, &mut all_files, &Some(tx.clone()), &found_label);

    // 2. Nhóm theo size
    let mut size_groups: HashMap<u64, Vec<(PathBuf, String)>> = HashMap::new();
    for (path, size, name) in all_files {
        size_groups.entry(size).or_default().push((path, name));
    }

    // Only keep sizes with > 1 file
    let mut potential_duplicates = Vec::new();
    for (size, files) in size_groups {
        if files.len() > 1 {
            for (path, name) in files {
                potential_duplicates.push((path, size, name));
            }
        }
    }

    // 3. Tính hash
    let total_to_hash = potential_duplicates.len();
    let _ = tx.send((
        ScanStatus::Hashing {
            message: checking_msg,
            current: 0,
            total: total_to_hash,
        },
        None,
    ));

    let mut hash_groups: HashMap<String, DuplicateGroup> = HashMap::new();
    for (i, (path, size, name)) in potential_duplicates.into_iter().enumerate() {
        if i % 10 == 0 || size > 10 * 1024 * 1024 {
            let _ = tx.send((
                ScanStatus::Hashing {
                    message: format!("{} ({}/{})", reading_msg, i, total_to_hash),
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

    // 4. Lọc nhóm trùng lặp
    let mut final_groups: Vec<DuplicateGroup> = hash_groups
        .into_values()
        .filter(|g| g.files.len() > 1)
        .collect();

    final_groups.sort_by(|a, b| b.size.cmp(&a.size));

    let _ = tx.send((ScanStatus::Done, Some(final_groups)));
}

/// Render trang tìm file trùng lặp
pub fn render_duplicate_finder(
    ui: &mut egui::Ui,
    state: &mut DuplicateFinderState,
    scan_path: &mut std::path::PathBuf,
    ctx: &egui::Context,
    lang: &Lang,
) {
    let t = &theme::DEFAULT;
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
                for g in &mut state.groups {
                    g.files.retain(|f| !f.selected);
                }
                state.groups.retain(|g| g.files.len() > 1);

                let err_msg = if clean_res.failed.is_empty() {
                    (
                        lang.dup_moved_to_trash
                            .replace("{}", &clean_res.deleted.to_string()),
                        false,
                    )
                } else {
                    (
                        lang.dup_delete_error
                            .replace("{}", &clean_res.failed.len().to_string()),
                        true,
                    )
                };
                state.result_message = Some(err_msg);
            }
            ctx.request_repaint();
        }
    }

    // ---- HEADER ----
    ui.add_space(t.space_md);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(lang.dup_title)
                .size(t.font_page_title)
                .strong()
                .color(colors::text_primary(ui.visuals().dark_mode)),
        );
    });

    ui.add_space(t.space_md);

    // ---- CHỌN ĐƯỜNG DẪN (breadcrumb) ----
    let path_enabled = matches!(state.status, ScanStatus::Idle | ScanStatus::Done);
    if bread_crumb::render_bread_crumb(ui, scan_path, lang, path_enabled) {
        state.groups.clear();
        state.status = ScanStatus::Idle;
        state.result_message = None;
    }

    ui.add_space(t.space_md);

    // ---- TOOLBAR QUÉT ----
    ui.horizontal(|ui| {
        if matches!(state.status, ScanStatus::Idle | ScanStatus::Done)
            && ui
                .add(
                    egui::Button::new(lang.btn_scan_duplicates)
                        .min_size(theme::btn_size(t.btn_width_xl)),
                )
                .clicked()
        {
            let (tx, rx) = channel();
            state.scan_rx = Some(rx);
            state.status = ScanStatus::Scanning {
                message: lang.dup_starting.to_string(),
                current: 0,
                total: 0,
            };
            state.groups.clear();
            state.result_message = None;

            let path_clone = scan_path.to_path_buf();
            let analyzing_msg = lang.dup_analyzing.to_string();
            let found_label = lang.dup_found_files.to_string();
            let checking_msg = lang.dup_checking_content.to_string();
            let reading_msg = lang.dup_reading_content.to_string();
            std::thread::spawn(move || {
                scan_duplicates_task(
                    path_clone,
                    tx,
                    analyzing_msg,
                    found_label,
                    checking_msg,
                    reading_msg,
                );
            });
        }
    });

    ui.add_space(t.space_md);
    ui.separator();

    // ---- KẾT QUẢ / LỖI ----
    if let Some((msg, is_error)) = &state.result_message {
        ui.add_space(t.space_md);
        let color = if *is_error {
            colors::status_danger(ui.visuals().dark_mode)
        } else {
            colors::status_success(ui.visuals().dark_mode)
        };
        ui.label(egui::RichText::new(msg).color(color).strong());
        ui.add_space(t.space_sm);
    }

    match &state.status {
        ScanStatus::Idle => {
            ui.add_space(t.space_xxl);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(lang.dup_idle_desc)
                        .color(colors::text_muted(ui.visuals().dark_mode)),
                );
                ui.add_space(t.space_lg);
                egui::Frame::new()
                    .fill(colors::accent_subtle(ui.visuals().dark_mode))
                    .stroke(egui::Stroke::new(
                        1.0,
                        colors::accent(ui.visuals().dark_mode).gamma_multiply(0.15),
                    ))
                    .corner_radius(theme::corner_radius(t.radius_lg))
                    .inner_margin(theme::padding(t.card_padding_lg))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(lang.dup_idle_hint)
                                .color(colors::accent(ui.visuals().dark_mode)),
                        );
                    });
            });
            return;
        }
        ScanStatus::Scanning {
            message, current, ..
        } => {
            ui.add_space(t.space_xxl);
            ui.vertical_centered(|ui| {
                ui.spinner();
                ui.add_space(t.space_lg);
                ui.label(egui::RichText::new(lang.dup_scanning).strong());
                ui.label(
                    egui::RichText::new(message)
                        .color(colors::text_muted(ui.visuals().dark_mode))
                        .small(),
                );
                ui.label(
                    egui::RichText::new(format!("{} {} items", lang.dup_scanned_items, current))
                        .color(colors::text_secondary(ui.visuals().dark_mode)),
                );
            });
            return;
        }
        ScanStatus::Hashing {
            message,
            current,
            total,
        } => {
            ui.add_space(t.space_xxl);
            ui.vertical_centered(|ui| {
                ui.spinner();
                ui.add_space(t.space_lg);
                ui.label(egui::RichText::new(lang.dup_hashing).strong());
                let progress = if *total > 0 {
                    *current as f32 / *total as f32
                } else {
                    0.0
                };
                ui.add(egui::ProgressBar::new(progress).show_percentage());
                ui.label(
                    egui::RichText::new(message)
                        .color(colors::text_muted(ui.visuals().dark_mode))
                        .small(),
                );
            });
            return;
        }
        ScanStatus::Deleting { message } => {
            ui.add_space(t.space_xxl);
            ui.vertical_centered(|ui| {
                ui.spinner();
                ui.add_space(t.space_lg);
                ui.label(
                    egui::RichText::new(lang.dup_deleting)
                        .strong()
                        .color(colors::status_danger(ui.visuals().dark_mode)),
                );
                ui.label(
                    egui::RichText::new(message)
                        .color(colors::text_muted(ui.visuals().dark_mode))
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
        ui.add_space(t.space_xxl);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new(lang.dup_no_duplicates)
                    .size(t.font_lg)
                    .color(colors::status_success(ui.visuals().dark_mode)),
            );
        });
        return;
    }

    // ToolBar kết quả
    ui.add_space(t.space_md);
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
            egui::RichText::new(format!(
                "{} {} {}",
                lang.dup_found_groups,
                groups_len,
                if groups_len == 1 { "group" } else { "groups" }
            ))
            .strong()
            .color(colors::accent(ui.visuals().dark_mode)),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if total_selected > 0 {
                if ui
                    .button(
                        egui::RichText::new(format!(
                            "{} {} ({})",
                            lang.dup_delete_btn,
                            total_selected,
                            format_size(selected_size)
                        ))
                        .color(colors::status_error_bg(ui.visuals().dark_mode)),
                    )
                    .clicked()
                {
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
                            message: lang.dup_moving_to_trash.to_string(),
                        };
                        std::thread::spawn(move || {
                            let mut result = crate::actions::cleaner::CleanResult {
                                deleted: 0,
                                failed: Vec::new(),
                            };
                            match trash::delete_all(&to_delete) {
                                Ok(()) => result.deleted = to_delete.len(),
                                Err(e) => result.failed.push(("Delete".to_string(), e.to_string())),
                            }
                            let _ = tx.send((ScanStatus::Done, Some(result)));
                        });
                    }
                }

                if ui.button(lang.dup_deselect_all).clicked() {
                    for g in &mut state.groups {
                        for f in &mut g.files {
                            f.selected = false;
                        }
                    }
                }
            }
            if ui.button(lang.dup_quick_select).clicked() {
                for g in &mut state.groups {
                    let mut first = true;
                    g.files.sort_by(|a, b| a.name.len().cmp(&b.name.len()));
                    for f in &mut g.files {
                        if first {
                            f.selected = false;
                            first = false;
                        } else {
                            f.selected = true;
                        }
                    }
                }
            }
        });
    });
    ui.add_space(t.space_md);

    // ---- DANH SÁCH THEO NHÓM ----
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for (i, group) in state.groups.iter_mut().enumerate() {
                ui.add_space(t.space_lg);
                egui::Frame::new()
                    .fill(colors::accent_subtle(ui.visuals().dark_mode).linear_multiply(0.2))
                    .corner_radius(theme::corner_radius(t.radius_sm))
                    .inner_margin(theme::padding(t.card_padding_sm))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let short_hash = if group.hash.len() >= 8 {
                                &group.hash[0..8]
                            } else {
                                &group.hash
                            };
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} {} ({} / {})",
                                    lang.dup_group_label,
                                    i + 1,
                                    format_size(group.size),
                                    "each"
                                ))
                                .color(colors::text_primary(ui.visuals().dark_mode))
                                .strong(),
                            );
                            ui.label(
                                egui::RichText::new(format!("Hash: {}", short_hash))
                                    .color(colors::text_muted(ui.visuals().dark_mode))
                                    .small(),
                            );
                        });

                        ui.add_space(t.space_sm);

                        for file in &mut group.files {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut file.selected, "");

                                let text_color = if file.selected {
                                    colors::status_danger(ui.visuals().dark_mode)
                                } else {
                                    colors::file_normal(ui.visuals().dark_mode)
                                };

                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new(&file.name).color(text_color).strong(),
                                    );
                                    ui.label(
                                        egui::RichText::new(file.path.display().to_string())
                                            .color(colors::text_muted(ui.visuals().dark_mode))
                                            .small(),
                                    );
                                });
                            });
                            ui.add_space(t.space_xs);
                        }
                    });
            }
        });
}
