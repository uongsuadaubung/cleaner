use crate::actions::{cleaner, sorter};
use crate::file_info::{FileEntry, SortCriteria, SortDirection, SortState};
use crate::scanner;
use crate::ui::colors;
use crate::ui::components::dialogs::{DialogResult, DialogState};
use crate::ui::components::toolbar::{self, OldFilePeriod, ToolbarAction};
use crate::ui::components::tree_view;
use crate::utils::format_size;
use eframe::egui;
use std::path::{Path, PathBuf};

pub struct CleanupState {
    pub entries: Vec<FileEntry>,
    pub dialog_state: DialogState,
    pub selected_period: OldFilePeriod,
    pub status_message: Option<String>,
    pub progress_rx: Option<std::sync::mpsc::Receiver<(usize, usize, String)>>,
    pub clean_result_rx: Option<std::sync::mpsc::Receiver<cleaner::CleanResult>>,
    pub show_period_selector: bool,
    pub sort_state: Option<SortState>,
}

impl Default for CleanupState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            dialog_state: DialogState::None,
            selected_period: OldFilePeriod::OneMonth,
            status_message: None,
            progress_rx: None,
            clean_result_rx: None,
            show_period_selector: false,
            sort_state: None,
        }
    }
}

impl CleanupState {
    pub fn new(entries: Vec<FileEntry>) -> Self {
        Self {
            entries,
            ..Default::default()
        }
    }

    pub fn rescan(&mut self, path: &Path) {
        self.entries = scanner::scan_directory(path);
        self.apply_sorting();
        self.status_message = Some("Đã quét lại thư mục".to_string());
    }

    pub fn apply_sorting(&mut self) {
        for entry in &mut self.entries {
            entry.sort_recursive(self.sort_state);
        }

        self.entries.sort_by(|a, b| {
            let dir_cmp = b.is_dir.cmp(&a.is_dir);
            if dir_cmp != std::cmp::Ordering::Equal {
                return dir_cmp;
            }

            if let Some(state) = self.sort_state {
                let order = match state.criteria {
                    SortCriteria::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    SortCriteria::Created => a.created.cmp(&b.created),
                    SortCriteria::Modified => a.modified.cmp(&b.modified),
                    SortCriteria::Size => {
                        let size_a = if a.is_dir { a.total_size() } else { a.size };
                        let size_b = if b.is_dir { b.total_size() } else { b.size };
                        size_a.cmp(&size_b)
                    }
                };

                if state.direction == SortDirection::Desc {
                    order.reverse()
                } else {
                    order
                }
            } else {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
        });
    }

    pub fn toggle_sort(&mut self, criteria: SortCriteria) {
        match self.sort_state {
            Some(state) if state.criteria == criteria => match state.direction {
                SortDirection::Asc => {
                    self.sort_state = Some(SortState {
                        criteria,
                        direction: SortDirection::Desc,
                    });
                }
                SortDirection::Desc => {
                    self.sort_state = None;
                }
            },
            _ => {
                self.sort_state = Some(SortState {
                    criteria,
                    direction: SortDirection::Asc,
                });
            }
        }
        self.apply_sorting();
    }

    pub fn handle_toolbar_action(&mut self, action: ToolbarAction, scan_path: &Path) {
        match action {
            ToolbarAction::None => {}
            ToolbarAction::Rescan => {
                self.rescan(scan_path);
            }
            ToolbarAction::Sort => {
                self.dialog_state = DialogState::ConfirmSort;
            }
            ToolbarAction::SelectOld(days) => {
                cleaner::select_old_files(&mut self.entries, days);
                let count = count_selected_in_entries(&self.entries);
                let size = selected_size_in_entries(&self.entries);
                self.status_message = Some(format!(
                    "Đã chọn {} file cũ hơn {} ngày ({})",
                    count,
                    days,
                    format_size(size)
                ));
            }
            ToolbarAction::Delete => {
                let count = count_selected_in_entries(&self.entries);
                let size = selected_size_in_entries(&self.entries);
                if count > 0 {
                    self.dialog_state = DialogState::ConfirmDelete {
                        file_count: count,
                        total_size: size,
                    };
                }
            }
            ToolbarAction::DeselectAll => {
                cleaner::deselect_all(&mut self.entries);
                self.status_message = Some("Đã bỏ chọn tất cả".to_string());
            }
        }
    }

    pub fn handle_dialog_result(&mut self, result: DialogResult, scan_path: &Path) {
        match result {
            DialogResult::None => {}
            DialogResult::Confirmed => match &self.dialog_state {
                DialogState::ConfirmDelete { .. } => {
                    let (progress_tx, progress_rx) = std::sync::mpsc::channel();
                    let (result_tx, result_rx) = std::sync::mpsc::channel();

                    let entries_clone = self.entries.clone();

                    self.dialog_state = DialogState::Processing {
                        title: "Đang xóa...".to_string(),
                        message: "Đang bắt đầu...".to_string(),
                        current: 0,
                        total: count_selected_in_entries(&entries_clone),
                    };

                    self.progress_rx = Some(progress_rx);
                    self.clean_result_rx = Some(result_rx);

                    std::thread::spawn(move || {
                        let result =
                            cleaner::delete_selected_files(&entries_clone, Some(progress_tx));
                        let _ = result_tx.send(result);
                    });
                }
                DialogState::ConfirmSort => {
                    let has_selection = count_selected_in_entries(&self.entries) > 0;

                    let entries_to_sort: Vec<FileEntry> = if has_selection {
                        self.entries
                            .iter()
                            .filter(|e| !e.is_dir && e.selected)
                            .cloned()
                            .collect()
                    } else {
                        self.entries.iter().filter(|e| !e.is_dir).cloned().collect()
                    };

                    let sort_result = sorter::sort_files(&entries_to_sort, scan_path);
                    let message = if sort_result.failed.is_empty() {
                        format!("Đã sắp xếp {} file thành công!", sort_result.moved)
                    } else {
                        format!(
                            "Đã sắp xếp {} file. {} file thất bại:\n{}",
                            sort_result.moved,
                            sort_result.failed.len(),
                            sort_result
                                .failed
                                .iter()
                                .map(|(name, reason)| format!("  • {}: {}", name, reason))
                                .collect::<Vec<_>>()
                                .join("\n")
                        )
                    };
                    self.dialog_state = DialogState::ResultMessage {
                        title: "Kết quả sắp xếp".to_string(),
                        message,
                        is_error: !sort_result.failed.is_empty(),
                    };
                    self.rescan(scan_path);
                }
                _ => {
                    self.dialog_state = DialogState::None;
                }
            },
            DialogResult::Cancelled | DialogResult::Closed => {
                self.dialog_state = DialogState::None;
            }
        }
    }

    pub fn check_background_tasks(&mut self, ctx: &egui::Context, scan_path: &Path) {
        if let Some(ref rx) = self.progress_rx {
            let mut got_update = false;
            while let Ok((current, total, file_name)) = rx.try_recv() {
                got_update = true;
                if let DialogState::Processing { title, .. } = &self.dialog_state {
                    self.dialog_state = DialogState::Processing {
                        title: title.clone(),
                        message: format!("Đang xử lý: {}", file_name),
                        current,
                        total,
                    };
                }
            }
            if got_update {
                ctx.request_repaint();
            }
        }

        if let Some(ref rx) = self.clean_result_rx
            && let Ok(clean_result) = rx.try_recv()
        {
            let message = if clean_result.failed.is_empty() {
                format!("Đã xóa {} file thành công!", clean_result.deleted)
            } else {
                format!(
                    "Đã xóa {} file. {} file thất bại:\n{}",
                    clean_result.deleted,
                    clean_result.failed.len(),
                    clean_result
                        .failed
                        .iter()
                        .map(|(name, reason)| format!("  • {}: {}", name, reason))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            };

            self.dialog_state = DialogState::ResultMessage {
                title: "Kết quả xóa".to_string(),
                message,
                is_error: !clean_result.failed.is_empty(),
            };

            self.progress_rx = None;
            self.clean_result_rx = None;
            self.rescan(scan_path);
        }
    }
}

pub fn render_cleanup(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    state: &mut CleanupState,
    scan_path: &mut PathBuf,
) {
    state.check_background_tasks(ctx, scan_path);

    // ---- HEADER TITLE ----
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("🧹 Công Cụ Dọn Dẹp")
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

        if ui
            .add(egui::Button::new("📂 Thay đổi...").small())
            .clicked()
            && let Some(path) = rfd::FileDialog::new()
                .set_directory(&*scan_path)
                .pick_folder()
        {
            *scan_path = path;
            state.rescan(scan_path);
        }
    });

    ui.add_space(8.0);

    // ---- TOOLBAR BUTTONS ----
    let total_selected = count_selected_in_entries(&state.entries);
    let sort_selected_count = state
        .entries
        .iter()
        .filter(|e| !e.is_dir && e.selected)
        .count();

    ui.horizontal(|ui| {
        let tb_action = toolbar::render_toolbar(
            ui,
            &mut state.selected_period,
            total_selected,
            sort_selected_count,
            &mut state.show_period_selector,
        );
        state.handle_toolbar_action(tb_action, scan_path);
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    // ---- DANH SÁCH FILE & STATUS BAR ----
    egui::TopBottomPanel::bottom("cleanup_status_panel")
        .frame(egui::Frame::NONE)
        .show_inside(ui, |ui| {
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let selected_count = total_selected;
                let selected_size = selected_size_in_entries(&state.entries);
                let total_files = total_files_in_entries(&state.entries);
                let total_size = total_size_in_entries(&state.entries);

                ui.label(
                    egui::RichText::new(format!(
                        "✅ Đã chọn: {} file ({})",
                        selected_count,
                        format_size(selected_size)
                    ))
                    .color(if selected_count > 0 {
                        colors::STATUS_SUCCESS
                    } else {
                        colors::TEXT_SECONDARY
                    }),
                );

                ui.separator();

                ui.label(
                    egui::RichText::new(format!(
                        "📊 Tổng: {} file ({})",
                        total_files,
                        format_size(total_size)
                    ))
                    .color(colors::TEXT_SECONDARY),
                );

                if let Some(msg) = &state.status_message {
                    ui.separator();
                    ui.label(
                        egui::RichText::new(msg)
                            .color(colors::STATUS_WARNING)
                            .small(),
                    );
                }
            });
        });

    if state.entries.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.label(
                egui::RichText::new("📭 Thư mục trống")
                    .size(24.0)
                    .color(colors::TEXT_SECONDARY),
            );
            ui.add_space(10.0);
            ui.label("Không có file nào trong thư mục đã chọn");
        });
    } else {
        if let Some(sort_criteria) =
            tree_view::render_tree_view(ui, &mut state.entries, state.sort_state)
        {
            state.toggle_sort(sort_criteria);
        }
    }

    if state.dialog_state != DialogState::None {
        let dialog_result = crate::ui::components::dialogs::render_dialog(ctx, &state.dialog_state);
        state.handle_dialog_result(dialog_result, scan_path);
    }
}

// --- Helper functions ---

fn count_selected_in_entries(entries: &[FileEntry]) -> usize {
    entries.iter().map(|e| e.count_selected()).sum()
}

fn selected_size_in_entries(entries: &[FileEntry]) -> u64 {
    entries.iter().map(|e| e.selected_size()).sum()
}

fn total_files_in_entries(entries: &[FileEntry]) -> usize {
    entries.iter().map(|e| e.total_files()).sum()
}

fn total_size_in_entries(entries: &[FileEntry]) -> u64 {
    entries.iter().map(|e| e.total_size()).sum()
}
