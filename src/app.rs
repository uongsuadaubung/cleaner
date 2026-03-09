use std::path::PathBuf;

use eframe::egui;

use crate::scanner;
use crate::ui::components::sidebar::{self, ActivePage};

/// State chính của ứng dụng
pub struct FolderCleanerApp {
    /// Trang đang được hiển thị
    active_page: ActivePage,
    /// Đường dẫn thư mục chung
    scan_path: PathBuf,

    /// Trạng thái riêng của trang Dọn Dẹp
    cleanup_state: crate::ui::pages::cleanup::CleanupState,

    /// Trạng thái riêng của trang Tìm file trùng lặp
    duplicate_finder_state: crate::ui::pages::duplicate_finder::DuplicateFinderState,
}

impl FolderCleanerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::setup_fonts(&cc.egui_ctx);
        Self::setup_styles(&cc.egui_ctx);

        let scan_path = dirs::download_dir().unwrap_or_else(|| {
            eprintln!("Không tìm thấy thư mục Downloads, sử dụng thư mục hiện tại");
            PathBuf::from(".")
        });

        let entries = scanner::scan_directory(&scan_path);

        Self {
            active_page: ActivePage::Cleanup,
            scan_path,
            cleanup_state: crate::ui::pages::cleanup::CleanupState::new(entries),
            duplicate_finder_state: Default::default(),
        }
    }

    /// Cấu hình font hỗ trợ tiếng Việt
    fn setup_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        let font_path = std::path::Path::new("C:\\Windows\\Fonts\\segoeui.ttf");
        if let Ok(font_data) = std::fs::read(font_path) {
            fonts.font_data.insert(
                "segoe_ui".to_string(),
                egui::FontData::from_owned(font_data).into(),
            );

            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "segoe_ui".to_string());

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
}

impl eframe::App for FolderCleanerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Left panel - Sidebar navigation
        egui::SidePanel::left("sidebar_panel")
            .exact_width(72.0)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(ctx.style().visuals.panel_fill)
                    .stroke(egui::Stroke::NONE),
            )
            .show(ctx, |ui| {
                sidebar::render_sidebar(ui, &mut self.active_page);
            });

        // Central panel - nội dung trang
        egui::CentralPanel::default().show(ctx, |ui| match self.active_page {
            ActivePage::Cleanup => {
                crate::ui::pages::cleanup::render_cleanup(
                    ui,
                    ctx,
                    &mut self.cleanup_state,
                    &mut self.scan_path,
                );
            }
            ActivePage::DuplicateFinder => {
                crate::ui::pages::duplicate_finder::render_duplicate_finder(
                    ui,
                    &mut self.duplicate_finder_state,
                    &mut self.scan_path,
                    ctx,
                );
            }
        });
    }
}
