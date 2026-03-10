use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::file_info::FileEntry;

const APP_DIR_NAME: &str = "folder_cleaner";

/// Thư mục gốc chứa mọi dữ liệu tạm (cache, settings) của app
pub fn app_data_dir() -> PathBuf {
    std::env::temp_dir().join(APP_DIR_NAME)
}

/// Đường dẫn file settings.ini
pub fn settings_path() -> PathBuf {
    app_data_dir().join("settings.ini")
}

/// Tính đường dẫn file cache cho một scan path cụ thể.
/// Dùng SHA256 của path string làm tên file để tránh ký tự đặc biệt.
pub fn cache_path_for(scan_path: &Path) -> PathBuf {
    let hash = {
        let mut h = Sha256::new();
        h.update(scan_path.to_string_lossy().as_bytes());
        format!("{:.16x}", h.finalize()) // 16 hex chars là đủ
    };
    app_data_dir().join(format!("{}.bin", hash))
}

/// Tải cache từ disk. Trả về `None` nếu không có hoặc đọc lỗi.
pub fn load(scan_path: &Path) -> Option<Vec<FileEntry>> {
    let path = cache_path_for(scan_path);
    let bytes = std::fs::read(&path).ok()?;
    bincode::deserialize(&bytes).ok()
}

/// Ghi cache ra disk (tạo thư mục nếu chưa có).
pub fn save(scan_path: &Path, entries: &[FileEntry]) {
    let dir = app_data_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("Không thể tạo cache dir: {}", e);
        return;
    }
    let path = cache_path_for(scan_path);
    match bincode::serialize(entries) {
        Ok(bytes) => {
            if let Err(e) = std::fs::write(&path, bytes) {
                eprintln!("Không thể ghi cache: {}", e);
            }
        }
        Err(e) => eprintln!("Không thể serialize cache: {}", e),
    }
}

/// "Fingerprint" nhanh để so sánh hai kết quả scan.
/// Dùng (số entry, tổng size) — đủ để phát hiện 99% thay đổi.
pub fn fingerprint(entries: &[FileEntry]) -> (usize, u64) {
    let count = count_recursive(entries);
    let size = size_recursive(entries);
    (count, size)
}

fn count_recursive(entries: &[FileEntry]) -> usize {
    entries
        .iter()
        .map(|e| 1 + count_recursive(&e.children))
        .sum()
}

fn size_recursive(entries: &[FileEntry]) -> u64 {
    entries
        .iter()
        .map(|e| e.size + size_recursive(&e.children))
        .sum()
}
