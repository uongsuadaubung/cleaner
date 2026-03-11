/// ============================================================
/// Design Token - Bảng màu thống nhất toàn ứng dụng
/// ============================================================
/// Mọi màu sắc trong UI phải tham chiếu từ file này.
/// Không được dùng Color32::from_rgb(...) trực tiếp ở nơi khác.
/// ============================================================
use eframe::egui::Color32;

// ─── ACCENT ─────────────────────────────────────────────────
pub fn accent(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(79, 195, 247) } else { Color32::from_rgb(13, 71, 161) } // Darker Blue for light theme
}
pub fn accent_subtle(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(32, 45, 55) } else { Color32::from_rgb(215, 235, 250) } // Slightly more contrast background
}

// ─── TEXT ────────────────────────────────────────────────────
pub fn text_primary(is_dark: bool) -> Color32 {
    if is_dark { Color32::WHITE } else { Color32::BLACK }
}
pub fn text_secondary(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(176, 190, 197) } else { Color32::from_rgb(55, 71, 79) } // Tối hơn nhiều
}
pub fn text_muted(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(144, 164, 174) } else { Color32::from_rgb(96, 125, 139) } // Tối hơn
}
pub fn text_sidebar(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(180, 195, 210) } else { Color32::from_rgb(55, 71, 79) }
}
pub fn text_sidebar_hover(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(220, 230, 240) } else { Color32::BLACK }
}
pub fn text_on_accent(_is_dark: bool) -> Color32 {
    Color32::WHITE
}

// ─── STATUS ──────────────────────────────────────────────────
pub fn status_success(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(102, 187, 106) } else { Color32::from_rgb(27, 94, 32) } // Xanh đậm hơn
}
pub fn status_warning(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(255, 238, 88) } else { Color32::from_rgb(216, 67, 21) } // Cam sậm thay vì vàng dễ lẫn
}
pub fn status_danger(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(255, 112, 67) } else { Color32::from_rgb(183, 28, 28) } // Đỏ sậm hơn
}
pub fn status_error_bg(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(211, 47, 47) } else { Color32::from_rgb(229, 115, 115) }
}
pub fn status_success_bg(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(56, 142, 60) } else { Color32::from_rgb(129, 199, 132) }
}

// ─── FILE / FOLDER ───────────────────────────────────────────
pub fn file_selected(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(255, 238, 88) } else { Color32::from_rgb(216, 67, 21) } // Cam sậm
}
pub fn file_normal(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(176, 190, 197) } else { Color32::from_rgb(38, 50, 56) }
}

// ─── SIDEBAR ─────────────────────────────────────────────────
pub fn sidebar_active_bg(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(28, 58, 80) } else { Color32::from_rgb(227, 242, 253) }
}
pub fn sidebar_hover_bg(is_dark: bool) -> Color32 {
    if is_dark { Color32::from_rgb(45, 55, 65) } else { Color32::from_rgb(245, 245, 245) }
}

// ── MISC ─────────────────────────────────────────────────────
pub fn transparent() -> Color32 {
    Color32::TRANSPARENT
}
pub fn hover_overlay(_is_dark: bool) -> Color32 {
    Color32::from_white_alpha(15)
}

