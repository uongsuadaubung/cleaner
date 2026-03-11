/// Enum đại diện cho ngôn ngữ được chọn
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    Vietnamese,
    English,
}

impl Default for Language {
    fn default() -> Self {
        Language::Vietnamese
    }
}

impl Language {
    pub fn from_str(s: &str) -> Self {
        match s {
            "English" => Language::English,
            _ => Language::Vietnamese,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Vietnamese => "Vietnamese",
            Language::English => "English",
        }
    }

    /// Trả về bộ chuỗi UI tương ứng ngôn ngữ
    pub fn strings(&self) -> Lang {
        match self {
            Language::Vietnamese => VI,
            Language::English => EN,
        }
    }
}

/// Struct chứa toàn bộ chuỗi UI
#[derive(Clone, Copy)]
pub struct Lang {
    // ── Sidebar ─────────────────────────────────────────────
    pub nav_cleanup: &'static str,
    pub nav_cleanup_tooltip: &'static str,
    pub nav_duplicates: &'static str,
    pub nav_duplicates_tooltip: &'static str,
    pub nav_settings: &'static str,
    pub nav_settings_tooltip: &'static str,

    // ── Chung ────────────────────────────────────────────────
    pub path_label: &'static str,
    pub btn_change: &'static str,

    // ── Cleanup page ─────────────────────────────────────────
    pub cleanup_title: &'static str,
    pub status_selected: &'static str, // "✅ Đã chọn: {n} file ({size})"
    pub status_total: &'static str,    // "📊 Tổng: {n} file ({size})"
    pub empty_folder: &'static str,
    pub empty_folder_desc: &'static str,

    // ── Toolbar ──────────────────────────────────────────────
    pub btn_rescan: &'static str,
    pub btn_sort: &'static str,
    pub btn_select_old: &'static str,
    pub btn_deselect: &'static str,
    pub period_select_time: &'static str,
    pub period_cancel_tooltip: &'static str,
    pub period_1m: &'static str,
    pub period_2m: &'static str,
    pub period_3m: &'static str,
    pub period_6m: &'static str,
    pub period_1y: &'static str,
    pub period_scope_current: &'static str,
    pub period_scope_recursive: &'static str,
    pub period_scope_label: &'static str,
    pub period_btn_confirm: &'static str,
    pub period_select_label: &'static str, // "🕐 Chọn thời gian:"
    pub msg_old_file_found: &'static str,  // "{n} file ({size}) | {days} ngày | {scope}"
    pub msg_old_file_not_found: &'static str, // "Không tìm thấy... {days} ngày ({scope})"

    // ── Tree view headers ────────────────────────────────────
    pub col_name: &'static str,
    pub col_created: &'static str,
    pub col_modified: &'static str,
    pub col_size: &'static str,

    // ── Duplicate finder ─────────────────────────────────────
    pub dup_title: &'static str,
    pub btn_scan_duplicates: &'static str,
    pub dup_idle_desc: &'static str,
    pub dup_idle_hint: &'static str,
    pub dup_scanning: &'static str,
    pub dup_scanned_items: &'static str, // "Đã duyệt {} mục" hoặc "Scanned {} items"
    pub dup_hashing: &'static str,
    pub dup_deleting: &'static str,
    pub dup_no_duplicates: &'static str,
    pub dup_found_groups: &'static str, // "Tìm thấy {} nhóm trùng lặp"
    pub dup_delete_btn: &'static str,   // "🗑 Xóa {n} file ({size})"
    pub dup_deselect_all: &'static str,
    pub dup_quick_select: &'static str,
    pub dup_group_label: &'static str, // "📦 Nhóm {i} ({size} / {unit})"
    pub dup_each_file: &'static str,   // "mỗi file" hoặc "each"
    pub dup_hash_label: &'static str,  // "Hash:"
    pub dup_moved_to_trash: &'static str, // "Đã chuyển {n} file vào thùng rác."
    pub dup_delete_error: &'static str, // "Đã xảy ra lỗi xóa {n} file."
    pub dup_starting: &'static str,
    pub dup_analyzing: &'static str,
    pub dup_found_files: &'static str, // "Đã tìm thấy {n} file..."
    pub dup_checking_content: &'static str,
    pub dup_reading_content: &'static str, // "Đang đọc nội dung... ({i}/{total})"
    pub dup_moving_to_trash: &'static str,

    // ── Dialogs ──────────────────────────────────────────────
    pub dialog_please_wait: &'static str,
    pub dialog_confirm_delete_title: &'static str,
    pub dialog_confirm_delete_msg: &'static str,
    pub dialog_confirm_permanent_delete_msg: &'static str,
    pub dialog_file_count: &'static str,
    pub dialog_total_size: &'static str,
    pub dialog_recycle_note: &'static str,
    pub dialog_permanent_note: &'static str,
    pub dialog_btn_delete: &'static str,
    pub dialog_btn_delete_permanent: &'static str,
    pub dialog_btn_cancel: &'static str,
    pub dialog_confirm_sort_title: &'static str,
    pub dialog_confirm_sort_msg: &'static str,
    pub dialog_folders_label: &'static str,
    pub dialog_btn_sort: &'static str,

    // ── Settings ─────────────────────────────────────────────
    pub settings_title: &'static str,
    pub settings_appearance: &'static str,
    pub settings_theme_system: &'static str,
    pub settings_theme_dark: &'static str,
    pub settings_theme_light: &'static str,
    pub settings_language: &'static str,
    pub settings_exclude_list: &'static str,
    pub settings_exclude_hint: &'static str,

    // ── Status messages ──────────────────────────────────────
    pub msg_rescanned: &'static str,
    pub msg_deselected_all: &'static str,
    pub msg_processing: &'static str, // "Đang xử lý: {name}"
    pub msg_deleting_title: &'static str,
    pub msg_deleting_starting: &'static str,
    pub msg_selected_old: &'static str, // "Đã chọn {n} file cũ hơn {days} ngày ({size})"
}

// ═══════════════════════════════════════════════════════════
//  TIẾNG VIỆT
// ═══════════════════════════════════════════════════════════
pub const VI: Lang = Lang {
    nav_cleanup: "Dọn dẹp",
    nav_cleanup_tooltip: "Dọn dẹp thư mục: xóa file cũ, sắp xếp file",
    nav_duplicates: "Trùng lặp",
    nav_duplicates_tooltip: "Tìm và xóa các file trùng lặp",
    nav_settings: "Cài đặt",
    nav_settings_tooltip: "Giao diện và ngôn ngữ",

    path_label: "📁 Đường dẫn:",
    btn_change: "📂 Thay đổi...",

    cleanup_title: "🧹 Công Cụ Dọn Dẹp",
    status_selected: "✅ Đã chọn:",
    status_total: "📊 Tổng:",
    empty_folder: "📭 Thư mục trống",
    empty_folder_desc: "Không có file nào trong thư mục đã chọn",

    btn_rescan: "🔄 Quét lại",
    btn_sort: "📂 Sắp xếp",
    btn_select_old: "🕐 Chọn file cũ",
    btn_deselect: "⬜ Bỏ chọn",
    period_select_time: "🕐 Chọn thời gian...",
    period_cancel_tooltip: "Hủy chọn",
    period_1m: "1 tháng",
    period_2m: "2 tháng",
    period_3m: "3 tháng",
    period_6m: "6 tháng",
    period_1y: "1 năm",
    period_scope_current: "📄 Chỉ thư mục hiện tại",
    period_scope_recursive: "📂 Bao gồm thư mục con",
    period_scope_label: "Phạm vi tìm kiếm:",
    period_btn_confirm: "Xác nhận",
    period_select_label: "🕐 Chọn thời gian:",
    msg_old_file_found: "{} file ({}) | {} ngày | {}",
    msg_old_file_not_found: "Không tìm thấy file nào cũ hơn {} ngày ({})",

    col_name: "Tên file / Thư mục",
    col_created: "Ngày tạo",
    col_modified: "Ngày sửa",
    col_size: "Dung lượng",

    dup_title: "🔍 Trình Tìm File Trùng Lặp",
    btn_scan_duplicates: "▶ Quét file trùng lặp",
    dup_idle_desc: "Tính năng tìm file giống nhau qua nội dung (hash) trong thư mục gốc được chọn.",
    dup_idle_hint: "Nhấn 'Quét file trùng lặp' để bắt đầu!",
    dup_scanning: "Đang quét thư mục...",
    dup_scanned_items: "Đã duyệt {} mục",
    dup_hashing: "Đang đối chiếu file...",
    dup_deleting: "Đang xóa...",
    dup_no_duplicates: "Không tìm thấy file trùng lặp nào! 🎉",
    dup_found_groups: "Tìm thấy {} nhóm trùng lặp",
    dup_delete_btn: "🗑 Xóa",
    dup_deselect_all: "Bỏ chọn tất cả",
    dup_quick_select: "⚡ Chọn nhanh file bản sao",
    dup_group_label: "📦 Nhóm {i} ({size} / {unit})",
    dup_each_file: "mỗi file",
    dup_hash_label: "Hash:",
    dup_moved_to_trash: "Đã chuyển {} file vào thùng rác.",
    dup_delete_error: "Đã xảy ra lỗi xóa {} file.",
    dup_starting: "Bắt đầu...",
    dup_analyzing: "Đang phân tích cấu trúc thư mục...",
    dup_found_files: "Đã tìm thấy",
    dup_checking_content: "Đang kiểm tra nội dung file...",
    dup_reading_content: "Đang đọc nội dung...",
    dup_moving_to_trash: "Đang chuyển file vào Recycle Bin...",

    dialog_please_wait: "⏳ Vui lòng chờ...",
    dialog_confirm_delete_title: "⚠ Xác nhận xóa",
    dialog_confirm_delete_msg: "Bạn có chắc muốn xóa các file đã chọn?",
    dialog_confirm_permanent_delete_msg: "Bạn có chắc muốn XÓA VĨNH VIỄN các file đã chọn?",
    dialog_file_count: "Số file:",
    dialog_total_size: "Tổng dung lượng:",
    dialog_recycle_note: "📋 File sẽ được chuyển vào Recycle Bin",
    dialog_permanent_note: "⚠ File bị xóa sẽ KHÔNG THỂ LẤY LẠI",
    dialog_btn_delete: "🗑 Xóa vào thùng rác",
    dialog_btn_delete_permanent: "🔥 Xóa Vĩnh Viễn",
    dialog_btn_cancel: "Hủy",
    dialog_confirm_sort_title: "📂 Xác nhận sắp xếp",
    dialog_confirm_sort_msg: "Sắp xếp file vào thư mục theo loại?",
    dialog_folders_label: "Các thư mục sẽ được tạo:",
    dialog_btn_sort: "📂 Sắp xếp",

    settings_title: "⚙ Cài đặt",
    settings_appearance: "Giao diện",
    settings_theme_system: "Theo hệ thống",
    settings_theme_dark: "Tối",
    settings_theme_light: "Sáng",
    settings_language: "Ngôn ngữ",
    settings_exclude_list: "Danh sách bỏ qua",
    settings_exclude_hint: "Các file, thư mục, hoặc phần mở rộng sẽ không bị xóa (mỗi mục 1 dòng hoặc cách nhau dấu phẩy)",

    msg_rescanned: "Đã quét lại thư mục",
    msg_deselected_all: "Đã bỏ chọn tất cả",
    msg_processing: "Đang xử lý:",
    msg_deleting_title: "Đang xóa...",
    msg_deleting_starting: "Đang bắt đầu...",
    msg_selected_old: "Đã chọn {} file cũ hơn {} ngày ({})",
};

// ═══════════════════════════════════════════════════════════
//  ENGLISH
// ═══════════════════════════════════════════════════════════
pub const EN: Lang = Lang {
    nav_cleanup: "Cleanup",
    nav_cleanup_tooltip: "Clean up folder: delete old files, sort files",
    nav_duplicates: "Duplicates",
    nav_duplicates_tooltip: "Find and remove duplicate files",
    nav_settings: "Settings",
    nav_settings_tooltip: "Appearance & language",

    path_label: "📁 Path:",
    btn_change: "📂 Change...",

    cleanup_title: "🧹 Cleanup Tool",
    status_selected: "✅ Selected:",
    status_total: "📊 Total:",
    empty_folder: "📭 Empty folder",
    empty_folder_desc: "No files found in the selected folder",

    btn_rescan: "🔄 Rescan",
    btn_sort: "📂 Sort",
    btn_select_old: "🕐 Select old files",
    btn_deselect: "⬜ Deselect",
    period_select_time: "🕐 Select period...",
    period_cancel_tooltip: "Cancel",
    period_1m: "1 month",
    period_2m: "2 months",
    period_3m: "3 months",
    period_6m: "6 months",
    period_1y: "1 year",
    period_scope_current: "📄 Current folder only",
    period_scope_recursive: "📂 Include subfolders",
    period_scope_label: "Search scope:",
    period_btn_confirm: "Confirm",
    period_select_label: "🕐 Select period:",
    msg_old_file_found: "{} files ({}) | {} days | {}",
    msg_old_file_not_found: "No files older than {} days ({})",

    col_name: "File / Folder name",
    col_created: "Created",
    col_modified: "Modified",
    col_size: "Size",

    dup_title: "🔍 Duplicate File Finder",
    btn_scan_duplicates: "▶ Scan for duplicates",
    dup_idle_desc: "Find identical files by content (hash) within the selected root folder.",
    dup_idle_hint: "Click 'Scan for duplicates' to begin!",
    dup_scanning: "Scanning folder...",
    dup_scanned_items: "Scanned {} items",
    dup_hashing: "Comparing files...",
    dup_deleting: "Deleting...",
    dup_no_duplicates: "No duplicate files found! 🎉",
    dup_found_groups: "Found {} duplicate group(s)",
    dup_delete_btn: "🗑 Delete",
    dup_deselect_all: "Deselect all",
    dup_quick_select: "⚡ Auto-select copies",
    dup_group_label: "📦 Group {i} ({size} / {unit})",
    dup_each_file: "each",
    dup_hash_label: "Hash:",
    dup_moved_to_trash: "Moved {} file(s) to Recycle Bin.",
    dup_delete_error: "Failed to delete {} file(s).",
    dup_starting: "Starting...",
    dup_analyzing: "Analyzing folder structure...",
    dup_found_files: "Found",
    dup_checking_content: "Checking file contents...",
    dup_reading_content: "Reading content...",
    dup_moving_to_trash: "Moving files to Recycle Bin...",

    dialog_please_wait: "⏳ Please wait...",
    dialog_confirm_delete_title: "⚠ Confirm delete",
    dialog_confirm_delete_msg: "Are you sure you want to delete the selected files?",
    dialog_confirm_permanent_delete_msg: "Are you sure you want to PERMANENTLY DELETE the selected files?",
    dialog_file_count: "File count:",
    dialog_total_size: "Total size:",
    dialog_recycle_note: "📋 Files will be moved to Recycle Bin",
    dialog_permanent_note: "⚠ Deleted files CANNOT BE RECOVERED",
    dialog_btn_delete: "🗑 Move to Trash",
    dialog_btn_delete_permanent: "🔥 Delete Permanently",
    dialog_btn_cancel: "Cancel",
    dialog_confirm_sort_title: "📂 Confirm sort",
    dialog_confirm_sort_msg: "Sort files into folders by type?",
    dialog_folders_label: "Folders that will be created:",
    dialog_btn_sort: "📂 Sort",

    settings_title: "⚙ Settings",
    settings_appearance: "Appearance",
    settings_theme_system: "System",
    settings_theme_dark: "Dark",
    settings_theme_light: "Light",
    settings_language: "Language",
    settings_exclude_list: "Exclude list",
    settings_exclude_hint: "Files, folders, or extensions that will never be deleted (one per line or comma-separated)",

    msg_rescanned: "Folder rescanned",
    msg_deselected_all: "All deselected",
    msg_processing: "Processing:",
    msg_deleting_title: "Deleting...",
    msg_deleting_starting: "Starting...",
    msg_selected_old: "Selected {} files older than {} days ({})",
};
