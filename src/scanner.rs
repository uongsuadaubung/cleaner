use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use rayon::prelude::*;

use crate::cache;
use crate::file_info::{FileEntry, FileEntryParams};

/// Độ sâu tối đa khi scan thư mục (tính từ root = 1).
/// Tăng lên nếu muốn scan sâu hơn, giảm xuống nếu muốn nhanh hơn.
pub const MAX_SCAN_DEPTH: usize = 6;

/// Kiểm tra xem một path có nên bị bỏ qua khi scan không.
/// Hiện tại bỏ qua thư mục cache của app để tránh đệ quy vào chính mình.
#[inline]
fn is_excluded(path: &Path) -> bool {
    path == cache::app_data_dir()
}

/// Event gửi từ background scan thread về UI thread
pub enum ScanEvent {
    /// Đang scan, trả về số entry đã đọc và đường dẫn hiện tại
    Progress {
        scanned: usize,
        current_path: PathBuf,
    },
    /// Scan hoàn tất
    Done(Vec<FileEntry>),
    /// Lỗi không thể scan
    #[allow(dead_code)]
    Error(String),
}

/// Scan đệ quy tuần tự (dùng cho các cấp con để tránh stack overflow trên rayon threads).
fn scan_sequential(path: &Path, depth: usize) -> Vec<FileEntry> {
    let read_dir = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(_) => return Vec::new(),
    };

    let mut entries: Vec<FileEntry> = read_dir
        .filter_map(|e| e.ok())
        .filter(|entry| !is_excluded(&entry.path())) // bỏ qua thư mục bị loại trừ
        .filter_map(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let metadata = fs::metadata(&path).ok()?;
            let is_dir = metadata.is_dir();
            let size = if is_dir { 0 } else { metadata.len() };

            let mut file_entry = FileEntry::new(FileEntryParams {
                name,
                path: path.clone(),
                is_dir,
                size,
                created: metadata.created().ok(),
                modified: metadata.modified().ok(),
                accessed: metadata.accessed().ok(),
                extension: if is_dir {
                    None
                } else {
                    path.extension().map(|e| e.to_string_lossy().to_string())
                },
            });

            if is_dir && depth < MAX_SCAN_DEPTH {
                file_entry.children = scan_sequential(&path, depth + 1);
            }

            Some(file_entry)
        })
        .collect();

    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    entries
}

/// Scan song song: chỉ parallelize cấp đầu tiên (top-level entries),
/// các cấp sâu hơn dùng sequential để tránh stack overflow trên rayon threads.
pub fn scan_directory_parallel(path: &Path) -> Vec<FileEntry> {
    let read_dir = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(e) => {
            eprintln!("Không thể đọc thư mục {}: {}", path.display(), e);
            return Vec::new();
        }
    };

    // Thu thập raw entries (read_dir không Send, phải collect trước khi par_iter)
    let raw: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();

    // Parallelize chỉ ở cấp 1: mỗi top-level entry được xử lý trên một rayon thread
    // Children bên trong gọi scan_sequential — không đệ quy trên rayon → hết stack overflow
    let mut entries: Vec<FileEntry> = raw
        .into_par_iter()
        .filter(|entry| !is_excluded(&entry.path())) // bỏ qua thư mục bị loại trừ
        .filter_map(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            let metadata = fs::metadata(&path).ok()?;
            let is_dir = metadata.is_dir();
            let size = if is_dir { 0 } else { metadata.len() };

            let mut file_entry = FileEntry::new(FileEntryParams {
                name,
                path: path.clone(),
                is_dir,
                size,
                created: metadata.created().ok(),
                modified: metadata.modified().ok(),
                accessed: metadata.accessed().ok(),
                extension: if is_dir {
                    None
                } else {
                    path.extension().map(|e| e.to_string_lossy().to_string())
                },
            });

            if is_dir && 1 < MAX_SCAN_DEPTH {
                file_entry.children = scan_sequential(&path, 2);
            }

            Some(file_entry)
        })
        .collect();

    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    entries
}

/// Khởi động scan bất đồng bộ trên background thread.
/// Gửi các `ScanEvent` qua `sender`.
pub fn scan_directory_async(path: PathBuf, sender: mpsc::Sender<ScanEvent>) {
    // Spawn với stack lớn hơn (8MB) để đề phòng thư mục cực sâu
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            let _ = sender.send(ScanEvent::Progress {
                scanned: 0,
                current_path: path.clone(),
            });

            let result = scan_directory_parallel(&path);
            let count = count_total(&result);

            let _ = sender.send(ScanEvent::Progress {
                scanned: count,
                current_path: path.clone(),
            });

            let _ = sender.send(ScanEvent::Done(result));
        })
        .expect("Failed to spawn scan thread");
}

fn count_total(entries: &[FileEntry]) -> usize {
    entries.iter().map(|e| 1 + count_total(&e.children)).sum()
}

/// API cũ giữ lại để tương thích
#[allow(dead_code)]
pub fn scan_directory(path: &Path) -> Vec<FileEntry> {
    scan_directory_parallel(path)
}
