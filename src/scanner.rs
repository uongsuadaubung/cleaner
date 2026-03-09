use std::fs;
use std::path::Path;

use crate::file_info::{FileEntry, FileEntryParams};

/// Quét thư mục và trả về danh sách FileEntry
pub fn scan_directory(path: &Path) -> Vec<FileEntry> {
    let mut entries = Vec::new();

    let read_dir = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(e) => {
            eprintln!("Không thể đọc thư mục {}: {}", path.display(), e);
            return entries;
        }
    };

    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let is_dir = metadata.is_dir();
        let size = if is_dir { 0 } else { metadata.len() };
        let created = metadata.created().ok();
        let modified = metadata.modified().ok();
        let accessed = metadata.accessed().ok();
        let extension = if is_dir {
            None
        } else {
            path.extension().map(|e| e.to_string_lossy().to_string())
        };

        let mut file_entry = FileEntry::new(FileEntryParams {
            name,
            path: path.clone(),
            is_dir,
            size,
            created,
            modified,
            accessed,
            extension,
        });

        // Quét đệ quy cho thư mục con
        if is_dir {
            file_entry.children = scan_directory(&path);
        }

        entries.push(file_entry);
    }

    // Sắp xếp: thư mục trước, sau đó theo tên
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    entries
}
