use crate::actions::{cleaner, sorter};
use crate::cache;
use crate::file_info::{FileEntry, SortCriteria, SortDirection, SortState};
use crate::lang::Lang;
use crate::scanner::{self, ScanEvent};
use crate::ui::colors;
use crate::ui::components::bread_crumb;
use crate::ui::components::dialogs::{DialogResult, DialogState};
use crate::ui::components::toolbar::{self, OldFilePeriod, OldFileScope, ToolbarAction};
use crate::ui::components::tree_view::{self, TreeViewAction};
use crate::ui::theme;
use crate::utils::format_size;
use eframe::egui;
use std::path::{Path, PathBuf};

pub struct CleanupState {
    pub entries: Vec<FileEntry>,
    pub dialog_state: DialogState,
    pub selected_period: OldFilePeriod,
    pub selected_scope: OldFileScope,
    pub status_message: Option<String>,
    pub progress_rx: Option<std::sync::mpsc::Receiver<(usize, usize, String)>>,
    pub clean_result_rx: Option<std::sync::mpsc::Receiver<cleaner::CleanResult>>,
    pub show_period_selector: bool,
    pub sort_state: Option<SortState>,
    /// Receiver nhận kết quả scan bất đồng bộ
    pub scan_rx: Option<std::sync::mpsc::Receiver<ScanEvent>>,
    /// Đang scan (chưa có cache) — hiển thị spinner
    pub is_scanning: bool,
    /// Đang scan âm thầm (có cache rồi) — không spinner
    pub is_silent_scanning: bool,
    /// Progress khi đang scan: (số entry đã đọc, đường dẫn hiện tại)
    pub scan_progress: Option<(usize, String)>,
    /// Lưu path đang scan để ghi cache khi xong
    current_scan_path: PathBuf,
    /// Cờ để cuộn lên đầu danh sách (ví dụ sau khi chuyển thư mục)
    pub should_scroll_to_top: bool,
}

impl Default for CleanupState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            dialog_state: DialogState::None,
            selected_period: OldFilePeriod::OneMonth,
            selected_scope: OldFileScope::CurrentOnly,
            status_message: None,
            progress_rx: None,
            clean_result_rx: None,
            show_period_selector: false,
            sort_state: None,
            scan_rx: None,
            is_scanning: false,
            is_silent_scanning: false,
            scan_progress: None,
            current_scan_path: PathBuf::new(),
            should_scroll_to_top: false,
        }
    }
}

impl CleanupState {
    /// Bắt đầu scan bất đồng bộ.
    /// - Nếu có cache: load ngay lập tức, scan âm thầm để kiểm tra thay đổi.
    /// - Nếu chưa có cache: hiển thị spinner cho đến khi xong.
    pub fn rescan(&mut self, path: &Path, _lang: &Lang) {
        self.current_scan_path = path.to_path_buf();
        let (tx, rx) = std::sync::mpsc::channel();
        self.scan_rx = Some(rx);
        self.scan_progress = Some((0, path.display().to_string()));

        if let Some(cached) = cache::load(path) {
            // Có cache: load ngay, scan âm thầm
            self.entries = cached;
            self.is_scanning = false;
            self.is_silent_scanning = true;
        } else {
            // Chưa có cache: cần spinner
            self.is_scanning = true;
            self.is_silent_scanning = false;
        }

        self.should_scroll_to_top = true;
        scanner::scan_directory_async(path.to_path_buf(), tx);
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

    pub fn handle_toolbar_action(&mut self, action: ToolbarAction, scan_path: &Path, exclude_list: &[String], lang: &Lang) {
        match action {
            ToolbarAction::None => {}
            ToolbarAction::Rescan => {
                self.rescan(scan_path, lang);
            }
            ToolbarAction::Sort => {
                self.dialog_state = DialogState::ConfirmSort;
            }
            ToolbarAction::SelectOld { days, scope } => {
                match scope {
                    OldFileScope::CurrentOnly => {
                        cleaner::select_old_files_shallow(&mut self.entries, days, exclude_list);
                    }
                    OldFileScope::Recursive => {
                        cleaner::select_old_files(&mut self.entries, days, exclude_list);
                    }
                }
                let count = count_selected_in_entries(&self.entries);
                let size = selected_size_in_entries(&self.entries);
                let scope_label = match scope {
                    OldFileScope::CurrentOnly => lang.period_scope_current,
                    OldFileScope::Recursive => lang.period_scope_recursive,
                };
                if count > 0 {
                    self.status_message = Some(
                        lang.msg_old_file_found
                            .replacen("{}", &count.to_string(), 1)
                            .replacen("{}", &format_size(size), 1)
                            .replacen("{}", &days.to_string(), 1)
                            .replacen("{}", scope_label, 1),
                    );
                } else {
                    self.status_message = Some(
                        lang.msg_old_file_not_found
                            .replacen("{}", &days.to_string(), 1)
                            .replacen("{}", scope_label, 1),
                    );
                }
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
                self.status_message = Some(lang.msg_deselected_all.to_string());
            }
        }
    }

    pub fn handle_dialog_result(&mut self, result: DialogResult, scan_path: &Path, lang: &Lang) {
        match result {
            DialogResult::None => {}
            DialogResult::Confirmed => match &self.dialog_state {
                DialogState::ConfirmDelete { .. } => {
                    let (progress_tx, progress_rx) = std::sync::mpsc::channel();
                    let (result_tx, result_rx) = std::sync::mpsc::channel();

                    let entries_clone = self.entries.clone();

                    self.dialog_state = DialogState::Processing {
                        title: lang.msg_deleting_title.to_string(),
                        message: lang.msg_deleting_starting.to_string(),
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
                        format!("{} {} file!", lang.dialog_btn_sort, sort_result.moved)
                    } else {
                        format!(
                            "{} {} file. {} file:\n{}",
                            lang.dialog_btn_sort,
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
                    let title = lang
                        .dialog_confirm_sort_title
                        .trim_start_matches(['📂', ' '])
                        .to_string();
                    self.dialog_state = DialogState::ResultMessage {
                        title,
                        message,
                        is_error: !sort_result.failed.is_empty(),
                    };
                    self.rescan(scan_path, lang);
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

    pub fn check_background_tasks(&mut self, ctx: &egui::Context, _scan_path: &Path, lang: &Lang) {
        // Poll scan bất đồng bộ (cả spinner lẫn silent)
        if self.is_scanning || self.is_silent_scanning {
            if let Some(ref rx) = self.scan_rx {
                let mut got_update = false;
                loop {
                    match rx.try_recv() {
                        Ok(ScanEvent::Progress {
                            scanned,
                            current_path,
                        }) => {
                            self.scan_progress =
                                Some((scanned, current_path.display().to_string()));
                            got_update = true;
                        }
                        Ok(ScanEvent::Done(mut new_entries)) => {
                            // Áp dụng sort
                            for e in &mut new_entries {
                                e.sort_recursive(self.sort_state);
                            }
                            new_entries.sort_by(|a, b| {
                                b.is_dir
                                    .cmp(&a.is_dir)
                                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                            });

                            // So sánh fingerprint với entries hiện tại
                            let new_fp = cache::fingerprint(&new_entries);
                            let old_fp = cache::fingerprint(&self.entries);
                            let changed = new_fp != old_fp;

                            // Luôn ghi cache mới
                            cache::save(&self.current_scan_path, &new_entries);

                            if changed || self.is_scanning {
                                // Khác cache hoặc lần đuầu (không có cache trước) — cập nhật UI
                                self.entries = new_entries;
                                self.status_message = Some(lang.msg_rescanned.to_string());
                            }

                            self.is_scanning = false;
                            self.is_silent_scanning = false;
                            self.scan_rx = None;
                            self.scan_progress = None;
                            got_update = true;
                            break;
                        }
                        Ok(ScanEvent::Error(e)) => {
                            self.is_scanning = false;
                            self.is_silent_scanning = false;
                            self.scan_rx = None;
                            self.scan_progress = None;
                            self.status_message = Some(format!("Lỗi scan: {}", e));
                            got_update = true;
                            break;
                        }
                        Err(_) => break,
                    }
                }
                if got_update {
                    ctx.request_repaint();
                }
            }
        }

        // Poll delete progress
        if let Some(ref rx) = self.progress_rx {
            let mut got_update = false;
            while let Ok((current, total, file_name)) = rx.try_recv() {
                got_update = true;
                if let DialogState::Processing { title, .. } = &self.dialog_state {
                    self.dialog_state = DialogState::Processing {
                        title: title.clone(),
                        message: format!("{} {}", lang.msg_processing, file_name),
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
                format!("{} {} file!", lang.dup_delete_btn, clean_result.deleted)
            } else {
                format!(
                    "{} {} file. {} file:\n{}",
                    lang.dup_delete_btn,
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

            let title = lang
                .dialog_confirm_delete_title
                .trim_start_matches(['⚠', ' '])
                .to_string();
            self.dialog_state = DialogState::ResultMessage {
                title,
                message,
                is_error: !clean_result.failed.is_empty(),
            };

            self.progress_rx = None;
            self.clean_result_rx = None;
            // Cập nhật danh sách: lọc bỏ các entry đã bị xóa khỏi disk
            remove_deleted_entries(&mut self.entries);
        }
    }
}

pub fn render_cleanup(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    state: &mut CleanupState,
    scan_path: &mut PathBuf,
    exclude_list: &[String],
    lang: &Lang,
) {
    let t = &theme::DEFAULT;
    state.check_background_tasks(ctx, scan_path, lang);

    // ---- HEADER TITLE ----
    ui.add_space(t.space_md);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(lang.cleanup_title)
                .size(t.font_page_title)
                .strong()
                .color(colors::text_primary(ui.visuals().dark_mode)),
        );
    });

    ui.add_space(t.space_md);

    // ---- CHỌN ĐƯỜNG DẪN (breadcrumb) ----
    if bread_crumb::render_bread_crumb(ui, scan_path, lang, true) {
        state.rescan(scan_path, lang);
    }

    ui.add_space(t.space_md);

    // ---- TOOLBAR BUTTONS ----
    let total_selected = count_selected_in_entries(&state.entries);
    let sort_selected_count = state
        .entries
        .iter()
        .filter(|e| !e.is_dir && e.selected)
        .count();

    ui.horizontal(|ui| {
        ui.add_enabled_ui(!state.is_scanning, |ui| {
            let tb_action = toolbar::render_toolbar(
                ui,
                &mut state.selected_period,
                &mut state.selected_scope,
                total_selected,
                sort_selected_count,
                &mut state.show_period_selector,
                lang,
            );
            state.handle_toolbar_action(tb_action, scan_path, exclude_list, lang);
        });
    });

    ui.add_space(t.space_md);
    ui.separator();
    ui.add_space(t.space_md);

    // ---- DANH SÁCH FILE & STATUS BAR ----
    egui::TopBottomPanel::bottom("cleanup_status_panel")
        .frame(egui::Frame::NONE)
        .show_inside(ui, |ui| {
            ui.add_space(t.space_sm);
            ui.separator();
            ui.add_space(t.space_sm);
            ui.horizontal(|ui| {
                let selected_count = total_selected;
                let selected_size = selected_size_in_entries(&state.entries);
                let total_files = total_files_in_entries(&state.entries);
                let total_size = total_size_in_entries(&state.entries);

                ui.label(
                    egui::RichText::new(format!(
                        "{} {} file ({})",
                        lang.status_selected,
                        selected_count,
                        format_size(selected_size)
                    ))
                    .color(if selected_count > 0 {
                        colors::status_success(ui.visuals().dark_mode)
                    } else {
                        colors::text_secondary(ui.visuals().dark_mode)
                    }),
                );

                ui.separator();

                ui.label(
                    egui::RichText::new(format!(
                        "{} {} file ({})",
                        lang.status_total,
                        total_files,
                        format_size(total_size)
                    ))
                    .color(colors::text_secondary(ui.visuals().dark_mode)),
                );

                if let Some(msg) = &state.status_message {
                    ui.separator();
                    ui.label(
                        egui::RichText::new(msg)
                            .color(colors::status_warning(ui.visuals().dark_mode))
                            .small(),
                    );
                }

                // Silent scan indicator — hiển thị nhỏ, không gây phiền
                if state.is_silent_scanning {
                    ui.separator();
                    ui.add(egui::Spinner::new().size(10.0));
                    ui.label(
                        egui::RichText::new("Đang cập nhật...")
                            .small()
                            .color(colors::text_muted(ui.visuals().dark_mode)),
                    );
                    ctx.request_repaint();
                }
            });
        });

    if state.is_scanning {
        // Hiển thị spinner khi đang scan
        ui.vertical_centered(|ui| {
            ui.add_space(t.space_empty_top);
            ui.add(egui::Spinner::new().size(32.0));
            ui.add_space(t.space_md);
            if let Some((count, path)) = &state.scan_progress {
                if *count > 0 {
                    ui.label(
                        egui::RichText::new(format!("Đã quét {} mục...", count))
                            .color(colors::text_secondary(ui.visuals().dark_mode)),
                    );
                    ui.add_space(4.0);
                }
                // Truncate path nếu quá dài
                let display_path = if path.len() > 60 {
                    format!("...{}", &path[path.len() - 57..])
                } else {
                    path.clone()
                };
                ui.label(
                    egui::RichText::new(display_path)
                        .small()
                        .color(colors::text_secondary(ui.visuals().dark_mode)),
                );
            }
            // Yêu cầu repaint liên tục để spinner quay
            ctx.request_repaint();
        });
    } else if state.entries.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(t.space_empty_top);
            ui.label(
                egui::RichText::new(lang.empty_folder)
                    .size(t.font_heading)
                    .color(colors::text_secondary(ui.visuals().dark_mode)),
            );
            ui.add_space(t.space_md + 2.0);
            ui.label(lang.empty_folder_desc);
        });
    } else if let Some(action) = tree_view::render_tree_view(
        ui,
        &mut state.entries,
        state.sort_state,
        lang,
        &mut state.should_scroll_to_top,
    ) {
        match action {
            TreeViewAction::Sort(criteria) => state.toggle_sort(criteria),
            TreeViewAction::NavigateTo(path) => {
                *scan_path = path;
                state.rescan(scan_path, lang);
            }
        }
    }

    if state.dialog_state != DialogState::None {
        let dialog_result =
            crate::ui::components::dialogs::render_dialog(ctx, &state.dialog_state, lang);
        state.handle_dialog_result(dialog_result, scan_path, lang);
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

/// Lọc đệ quy các entry đã bị xóa khỏi disk.
/// Chỉ check path.exists() — nhanh hơn rescan nhiều.
fn remove_deleted_entries(entries: &mut Vec<FileEntry>) {
    entries.retain_mut(|e| {
        if !e.path.exists() {
            return false;
        }
        if !e.children.is_empty() {
            remove_deleted_entries(&mut e.children);
        }
        true
    });
}
