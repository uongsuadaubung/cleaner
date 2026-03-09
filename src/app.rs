use std::path::PathBuf;

use eframe::egui;
use crate::utils::format_size;

use crate::actions::cleaner;
use crate::actions::sorter;
use crate::scanner;
use crate::ui::dialogs::{DialogResult, DialogState};
use crate::ui::toolbar::{self, OldFilePeriod, ToolbarAction};
use crate::ui::tree_view;
use crate::file_info::{FileEntry, SortCriteria, SortDirection, SortState};

/// State chính của ứng dụng
pub struct FolderCleanerApp {
    /// Đường dẫn thư mục đang quét
    scan_path: PathBuf,
    /// Danh sách file/folder đã quét
    entries: Vec<FileEntry>,
    /// Trạng thái dialog hiện tại
    dialog_state: DialogState,
    /// Khoảng thời gian chọn file cũ
    selected_period: OldFilePeriod,
    /// Thông báo trạng thái
    status_message: Option<String>,
    /// Receiver để nhận tiến độ từ thread khác
    progress_rx: Option<std::sync::mpsc::Receiver<(usize, usize, String)>>,
    /// Receiver để nhận kết quả cuối cùng từ thread xóa
    clean_result_rx: Option<std::sync::mpsc::Receiver<cleaner::CleanResult>>,
    /// Trạng thái hiển thị bộ chọn thời gian
    show_period_selector: bool,
    /// Trạng thái sắp xếp hiện tại
    sort_state: Option<SortState>,
}

impl FolderCleanerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load font hỗ trợ tiếng Việt
        Self::setup_fonts(&cc.egui_ctx);
        // Thiết lập style một lần duy nhất
        Self::setup_styles(&cc.egui_ctx);

        let scan_path = dirs::download_dir().unwrap_or_else(|| {
            eprintln!("Không tìm thấy thư mục Downloads, sử dụng thư mục hiện tại");
            PathBuf::from(".")
        });

        let entries = scanner::scan_directory(&scan_path);

        Self {
            scan_path,
            entries,
            dialog_state: DialogState::None,
            selected_period: OldFilePeriod::OneMonth,
            status_message: None,
            progress_rx: None,
            clean_result_rx: None,
            show_period_selector: false,
            sort_state: None,
        }
    }

    /// Cấu hình font hỗ trợ tiếng Việt
    fn setup_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        // Đọc font Segoe UI từ hệ thống Windows (hỗ trợ tiếng Việt)
        let font_path = std::path::Path::new("C:\\Windows\\Fonts\\segoeui.ttf");
        if let Ok(font_data) = std::fs::read(font_path) {
            fonts.font_data.insert(
                "segoe_ui".to_string(),
                egui::FontData::from_owned(font_data).into(),
            );

            // Đặt Segoe UI làm font ưu tiên cao nhất cho proportional
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "segoe_ui".to_string());

            // Cũng thêm vào monospace để đồng bộ
            fonts
                .families
                .get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .insert(0, "segoe_ui".to_string());
        } else {
            eprintln!("Không tìm thấy font Segoe UI, sử dụng font mặc định");
        }

        ctx.set_fonts(fonts);
    }

    /// Thiết lập style cho ứng dụng
    fn setup_styles(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(6);
        style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
        style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);
        style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
        style.visuals.window_corner_radius = egui::CornerRadius::same(10);
        ctx.set_style(style);
    }

    /// Quét lại thư mục
    fn rescan(&mut self) {
        self.entries = scanner::scan_directory(&self.scan_path);
        self.apply_sorting();
        self.status_message = Some("Đã quét lại thư mục".to_string());
    }

    /// Áp dụng sắp xếp cho danh sách hiện tại
    fn apply_sorting(&mut self) {
        for entry in &mut self.entries {
            entry.sort_recursive(self.sort_state);
        }
        
        // Sắp xếp danh sách cấp gốc
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

    /// Chuyển đổi trạng thái sắp xếp của một cột
    fn toggle_sort(&mut self, criteria: SortCriteria) {
        match self.sort_state {
            Some(state) if state.criteria == criteria => {
                match state.direction {
                    SortDirection::Asc => {
                        self.sort_state = Some(SortState {
                            criteria,
                            direction: SortDirection::Desc,
                        });
                    }
                    SortDirection::Desc => {
                        self.sort_state = None;
                    }
                }
            }
            _ => {
                self.sort_state = Some(SortState {
                    criteria,
                    direction: SortDirection::Asc,
                });
            }
        }
        self.apply_sorting();
    }

    /// Xử lý hành động từ toolbar
    fn handle_toolbar_action(&mut self, action: ToolbarAction) {
        match action {
            ToolbarAction::None => {}
            ToolbarAction::Rescan => {
                self.rescan();
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

    /// Xử lý kết quả dialog
    fn handle_dialog_result(&mut self, result: DialogResult) {
        match result {
            DialogResult::None => {}
            DialogResult::Confirmed => {
                match &self.dialog_state {
                    DialogState::ConfirmDelete { .. } => {
                        let (progress_tx, progress_rx) = std::sync::mpsc::channel();
                        let (result_tx, result_rx) = std::sync::mpsc::channel();
                        
                        let entries_clone = self.entries.clone();
                        
                        // Chuyển sang trạng thái Processing ngay lập tức
                        self.dialog_state = DialogState::Processing {
                            title: "Đang xóa...".to_string(),
                            message: "Đang bắt đầu...".to_string(),
                            current: 0,
                            total: count_selected_in_entries(&entries_clone),
                        };
                        
                        self.progress_rx = Some(progress_rx);
                        self.clean_result_rx = Some(result_rx);

                        // Spawn thread để xử lý xóa
                        std::thread::spawn(move || {
                            let result = cleaner::delete_selected_files(&entries_clone, Some(progress_tx));
                            let _ = result_tx.send(result);
                        });
                    }
                    DialogState::ConfirmSort => {
                        let has_selection = count_selected_in_entries(&self.entries) > 0;
                        
                        let entries_to_sort: Vec<FileEntry> = if has_selection {
                            // Nếu có chọn: chỉ lấy các file được chọn ở CẤP ĐỘ GỐC (bỏ qua folder và file trong folder con)
                            self.entries.iter()
                                .filter(|e| !e.is_dir && e.selected)
                                .cloned()
                                .collect()
                        } else {
                            // Nếu không chọn: sắp xếp tất cả các file ở CẤP ĐỘ GỐC (bỏ qua folder)
                            self.entries.iter()
                                .filter(|e| !e.is_dir)
                                .cloned()
                                .collect()
                        };

                        let sort_result = sorter::sort_files(&entries_to_sort, &self.scan_path);
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
                        self.rescan();
                    }
                    _ => {
                        self.dialog_state = DialogState::None;
                    }
                }
            }
            DialogResult::Cancelled => {
                self.dialog_state = DialogState::None;
            }
            DialogResult::Closed => {
                self.dialog_state = DialogState::None;
            }
        }
    }

    /// Kiểm tra các task chạy ngầm (tiến độ, kết quả)
    fn check_background_tasks(&mut self, ctx: &egui::Context) {
        // 1. Phản hồi tiến độ
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

        // 2. Kiểm tra kết quả xóa xong
        if let Some(ref rx) = self.clean_result_rx {
            if let Ok(clean_result) = rx.try_recv() {
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
                self.rescan();
            }
        }
    }
}

impl eframe::App for FolderCleanerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Kiểm tra task chạy ngầm
        self.check_background_tasks(ctx);

        // Top panel - Toolbar
        egui::TopBottomPanel::top("toolbar_panel")
            .min_height(50.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);

                // Đường dẫn thư mục
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("📁 Đường dẫn:")
                            .color(egui::Color32::from_rgb(176, 190, 197)),
                    );
                    ui.label(
                        egui::RichText::new(self.scan_path.display().to_string())
                            .color(egui::Color32::from_rgb(79, 195, 247))
                            .strong(),
                    );
                    
                    if ui.add(egui::Button::new("📂 Thay đổi...").small()).clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_directory(&self.scan_path)
                            .pick_folder() 
                        {
                            self.scan_path = path;
                            self.rescan();
                        }
                    }
                });

                ui.add_space(4.0);

                // Toolbar buttons
                let total_selected = count_selected_in_entries(&self.entries);
                let sort_selected_count = self.entries.iter()
                    .filter(|e| !e.is_dir && e.selected)
                    .count();
                
                let toolbar_action =
                    toolbar::render_toolbar(ui, &mut self.selected_period, total_selected, sort_selected_count, &mut self.show_period_selector);
                self.handle_toolbar_action(toolbar_action);

                ui.add_space(4.0);
            });

        // Bottom panel - Status bar
        egui::TopBottomPanel::bottom("status_panel")
            .min_height(30.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let selected_count = count_selected_in_entries(&self.entries);
                    let selected_size = selected_size_in_entries(&self.entries);
                    let total_files = total_files_in_entries(&self.entries);
                    let total_size = total_size_in_entries(&self.entries);

                    ui.label(
                        egui::RichText::new(format!(
                            "✅ Đã chọn: {} file ({})",
                            selected_count,
                            format_size(selected_size)
                        ))
                        .color(if selected_count > 0 {
                            egui::Color32::from_rgb(102, 187, 106)
                        } else {
                            egui::Color32::from_rgb(176, 190, 197)
                        }),
                    );

                    ui.separator();

                    ui.label(
                        egui::RichText::new(format!(
                            "📊 Tổng: {} file ({})",
                            total_files,
                            format_size(total_size)
                        ))
                        .color(egui::Color32::from_rgb(176, 190, 197)),
                    );

                    // Hiển thị status message
                    if let Some(msg) = &self.status_message {
                        ui.separator();
                        ui.label(
                            egui::RichText::new(msg)
                                .color(egui::Color32::from_rgb(255, 238, 88))
                                .small(),
                        );
                    }
                });
            });

        // Central panel - Tree view
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.entries.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.label(
                        egui::RichText::new("📭 Thư mục trống")
                            .size(24.0)
                            .color(egui::Color32::from_rgb(176, 190, 197)),
                    );
                    ui.add_space(10.0);
                    ui.label("Không có file nào trong thư mục Downloads");
                });
            } else {
                if let Some(criteria) = tree_view::render_tree_view(ui, &mut self.entries, self.sort_state) {
                    self.toggle_sort(criteria);
                }
            }
        });

        // Render dialog nếu có
        if self.dialog_state != DialogState::None {
            let dialog_result =
                crate::ui::dialogs::render_dialog(ctx, &self.dialog_state);
            self.handle_dialog_result(dialog_result);
        }
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
