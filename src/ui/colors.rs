/// ============================================================
/// Design Token - Bảng màu thống nhất toàn ứng dụng
/// ============================================================
/// Mọi màu sắc trong UI phải tham chiếu từ file này.
/// Không được dùng Color32::from_rgb(...) trực tiếp ở nơi khác.
/// ============================================================
use eframe::egui::Color32;

// ─── ACCENT ─────────────────────────────────────────────────
/// Màu nhấn chính - xanh dương sáng (dùng cho active, highlight, link)
pub const ACCENT: Color32 = Color32::from_rgb(79, 195, 247);
/// Nền mờ của accent (dùng cho card info, frame highlight)
pub const ACCENT_SUBTLE: Color32 = Color32::from_rgb(32, 45, 55);

// ─── TEXT ────────────────────────────────────────────────────
/// Văn bản chính - trắng tinh (tiêu đề, label nổi bật)
pub const TEXT_PRIMARY: Color32 = Color32::WHITE;
/// Văn bản phụ - xám sáng (label thông thường, metadata)
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(176, 190, 197);
/// Văn bản mờ - xám trung (placeholder, disabled)
pub const TEXT_MUTED: Color32 = Color32::from_rgb(144, 164, 174);
/// Văn bản trên nền sidebar mặc định
pub const TEXT_SIDEBAR: Color32 = Color32::from_rgb(180, 195, 210);
/// Văn bản trên nền sidebar khi hover
pub const TEXT_SIDEBAR_HOVER: Color32 = Color32::from_rgb(220, 230, 240);

// ─── STATUS ──────────────────────────────────────────────────
/// Màu thành công - xanh lá (selected count, success message)
pub const STATUS_SUCCESS: Color32 = Color32::from_rgb(102, 187, 106);
/// Màu cảnh báo - vàng (status message, warning)
pub const STATUS_WARNING: Color32 = Color32::from_rgb(255, 238, 88);
/// Màu nguy hiểm - cam đỏ (nút xóa, lỗi)
pub const STATUS_DANGER: Color32 = Color32::from_rgb(255, 112, 67);
/// Màu lỗi nền - đỏ đậm (nền nút xác nhận xóa)
pub const STATUS_ERROR_BG: Color32 = Color32::from_rgb(211, 47, 47);
/// Màu nền thành công - xanh lá đậm (nền nút xác nhận OK)
pub const STATUS_SUCCESS_BG: Color32 = Color32::from_rgb(56, 142, 60);

// ─── FILE / FOLDER ───────────────────────────────────────────
/// Màu tên file được chọn (selected highlight)
pub const FILE_SELECTED: Color32 = Color32::from_rgb(255, 238, 88);
/// Màu tên file bình thường
pub const FILE_NORMAL: Color32 = Color32::from_rgb(176, 190, 197);

// ─── SIDEBAR ─────────────────────────────────────────────────
/// Nền sidebar item đang active
pub const SIDEBAR_ACTIVE_BG: Color32 = Color32::from_rgb(28, 58, 80);
/// Nền sidebar item khi hover
pub const SIDEBAR_HOVER_BG: Color32 = Color32::from_rgb(45, 55, 65);
// Viền trái accent → dùng ACCENT

// ─── HEADER / COLUMN ─────────────────────────────────────────
// Màu chữ header cột → dùng ACCENT khi có sort
