use eframe::egui;

/// Toàn bộ design tokens của app — spacing, sizing, radius, font, v.v.
/// Chỉnh sửa tại đây để thay đổi đồng nhất trên toàn bộ UI.
pub struct Theme {
    // ── Spacing ──────────────────────────────────────────────
    /// 2.0 — khoảng cách rất nhỏ (giữa các row trong tree)
    pub space_xs: f32,
    /// 4.0 — khoảng cách nhỏ (status bar separator, item padding)
    pub space_sm: f32,
    /// 8.0 — khoảng cách chuẩn giữa các section trong trang
    pub space_md: f32,
    /// 16.0 — khoảng cách sau spinner
    pub space_lg: f32,
    /// 20.0 — khoảng cách giữa các group trong Settings
    pub space_xl: f32,
    /// 40.0 — khoảng trống phía trên các empty state / loading state
    pub space_xxl: f32,
    /// 100.0 — khoảng trống phía trên empty folder indicator
    pub space_empty_top: f32,

    // ── Button sizing ─────────────────────────────────────────
    /// Chiều cao chuẩn của nút toolbar (32.0)
    pub btn_height: f32,
    /// Chiều rộng tối thiểu nút nhỏ — OK (80.0)
    pub btn_width_sm: f32,
    /// Chiều rộng tối thiểu nút chuẩn (100.0)
    pub btn_width_md: f32,
    /// Chiều rộng tối thiểu nút rộng — Delete, Sort (120.0)
    pub btn_width_lg: f32,
    /// Chiều rộng tối thiểu nút lớn nhất — Scan (160.0)
    pub btn_width_xl: f32,
    /// Chiều cao nút trong dialog (35.0)
    pub dialog_btn_height: f32,
    /// Chiều rộng tối thiểu nút dialog — Confirm/Cancel (100.0)
    pub dialog_btn_width: f32,

    // ── Border radius ─────────────────────────────────────────
    /// 6 — widget, card, frame chuẩn
    pub radius_sm: u8,
    /// 10 — window / dialog
    pub radius_md: u8,
    /// 12 — highlight card (idle state)
    pub radius_lg: u8,

    // ── Card / Frame padding ──────────────────────────────────
    /// 8 — padding chuẩn bên trong card/frame
    pub card_padding_sm: i8,
    /// 20 — padding lớn (highlight card)
    pub card_padding_lg: i8,

    // ── Font sizes ────────────────────────────────────────────
    /// 11.0 — sidebar label
    pub font_sidebar_label: f32,
    /// 14.0 — kết quả dialog
    pub font_sm: f32,
    /// 16.0 — dialog message, section heading
    pub font_md: f32,
    /// 18.0 — empty state result text
    pub font_lg: f32,
    /// 20.0 — tiêu đề trang (page title)
    pub font_page_title: f32,
    /// 22.0 — sidebar icon
    pub font_sidebar_icon: f32,
    /// 24.0 — settings heading
    pub font_heading: f32,

    // ── Sidebar ───────────────────────────────────────────────
    /// Chiều rộng sidebar (72.0)
    pub sidebar_width: f32,
    /// Chiều cao mỗi item trong sidebar (60.0)
    pub sidebar_item_height: f32,

    // ── Tree view ─────────────────────────────────────────────
    /// Chiều cao mỗi row (28.0)
    pub tree_row_height: f32,
    /// Chiều rộng cột checkbox (30.0)
    pub tree_checkbox_width: f32,
    /// Chiều rộng cột dung lượng (90.0)
    pub tree_size_col: f32,
    /// Chiều rộng cột ngày (140.0)
    pub tree_date_col: f32,
    /// Khoảng cách giữa các cột (12.0)
    pub tree_col_spacing: f32,
    /// Độ thụt lề mỗi cấp thư mục con (18.0)
    pub tree_indent_per_level: f32,

    // ── Dialog window sizing ──────────────────────────────────
    /// Chiều rộng dialog Processing (300.0)
    pub dialog_processing_width: f32,
    /// Chiều cao dialog Processing (120.0)
    pub dialog_processing_height: f32,
    /// Chiều rộng tối thiểu dialog xóa (350.0)
    pub dialog_delete_min_width: f32,
    /// Chiều rộng tối thiểu dialog sắp xếp (400.0)
    pub dialog_sort_min_width: f32,
    /// Khoảng cách giữa các nút trong dialog (20.0)
    pub dialog_btn_spacing: f32,

    // ── Global egui style ─────────────────────────────────────
    /// item_spacing.x (8.0)
    pub item_spacing_x: f32,
    /// item_spacing.y (6.0)
    pub item_spacing_y: f32,
}

/// Bộ giá trị mặc định — toàn bộ UI sử dụng bộ này.
/// Thay đổi tại đây để ảnh hưởng đồng nhất lên toàn bộ giao diện.
pub const DEFAULT: Theme = Theme {
    // Spacing
    space_xs: 2.0,
    space_sm: 4.0,
    space_md: 8.0,
    space_lg: 16.0,
    space_xl: 20.0,
    space_xxl: 40.0,
    space_empty_top: 100.0,

    // Button sizing
    btn_height: 32.0,
    btn_width_sm: 80.0,
    btn_width_md: 100.0,
    btn_width_lg: 120.0,
    btn_width_xl: 160.0,
    dialog_btn_height: 35.0,
    dialog_btn_width: 100.0,

    // Border radius
    radius_sm: 6,
    radius_md: 10,
    radius_lg: 12,

    // Card padding
    card_padding_sm: 8,
    card_padding_lg: 20,

    // Font sizes
    font_sidebar_label: 11.0,
    font_sm: 14.0,
    font_md: 16.0,
    font_lg: 18.0,
    font_page_title: 20.0,
    font_sidebar_icon: 22.0,
    font_heading: 24.0,

    // Sidebar
    sidebar_width: 72.0,
    sidebar_item_height: 60.0,

    // Tree view
    tree_row_height: 28.0,
    tree_checkbox_width: 30.0,
    tree_size_col: 90.0,
    tree_date_col: 140.0,
    tree_col_spacing: 12.0,
    tree_indent_per_level: 18.0,

    // Dialog sizing
    dialog_processing_width: 300.0,
    dialog_processing_height: 120.0,
    dialog_delete_min_width: 350.0,
    dialog_sort_min_width: 400.0,
    dialog_btn_spacing: 20.0,

    // Global style
    item_spacing_x: 8.0,
    item_spacing_y: 6.0,
};

// ── Helper functions ──────────────────────────────────────────────────────────

/// Tạo CornerRadius đồng nhất từ giá trị u8
#[inline]
pub fn corner_radius(r: u8) -> egui::CornerRadius {
    egui::CornerRadius::same(r)
}

/// Tạo Margin đồng nhất từ giá trị i8 (kiểu mà egui::Margin::same yêu cầu)
#[inline]
pub fn padding(v: i8) -> egui::Margin {
    egui::Margin::same(v)
}

/// Tạo Vec2 (width, height) với height từ theme button height
#[inline]
pub fn btn_size(width: f32) -> egui::Vec2 {
    egui::vec2(width, DEFAULT.btn_height)
}

/// Tạo Vec2 với dialog button height
#[inline]
pub fn dialog_btn_size(width: f32) -> egui::Vec2 {
    egui::vec2(width, DEFAULT.dialog_btn_height)
}
