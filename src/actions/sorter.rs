use std::fs;
use std::path::{Path, PathBuf};

use crate::file_info::{FileCategory, FileEntry};

/// Kết quả sắp xếp
pub struct SortResult {
    pub moved: usize,
    pub failed: Vec<(String, String)>, // (tên file, lý do)
}

#[allow(dead_code)]
#[allow(dead_code)]
/// Danh sách các thư mục đích sẽ được tạo khi sắp xếp
const SORT_CATEGORIES: &[FileCategory] = &[
    FileCategory::Document,
    FileCategory::Image,
    FileCategory::Video,
    FileCategory::Audio,
    FileCategory::Archive,
    FileCategory::Executable,
    FileCategory::Code,
    FileCategory::Other,
];


/// Tạo tên file không trùng lặp (thêm suffix _1, _2...)
fn get_unique_path(target_dir: &Path, file_name: &str) -> PathBuf {
    let target_path = target_dir.join(file_name);
    if !target_path.exists() {
        return target_path;
    }

    let stem = Path::new(file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| file_name.to_string());
    let ext = Path::new(file_name)
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    let mut counter = 1;
    loop {
        let new_name = format!("{}_{}{}", stem, counter, ext);
        let new_path = target_dir.join(&new_name);
        if !new_path.exists() {
            return new_path;
        }
        counter += 1;
    }
}

/// Sắp xếp file vào thư mục theo loại
pub fn sort_files(entries: &[FileEntry], base_path: &Path) -> SortResult {
    let mut result = SortResult {
        moved: 0,
        failed: Vec::new(),
    };


    for entry in entries {
        // Bỏ qua thư mục và các thư mục phân loại đã tạo
        if entry.is_dir {
            continue;
        }

        let target_folder = entry.category.folder_name();
        if target_folder.is_empty() {
            continue;
        }

        let target_dir = base_path.join(target_folder);
        
        // Kiểm tra nếu file đã nằm đúng trong thư mục đích rồi thì bỏ qua
        if let Some(parent) = entry.path.parent() {
            if parent == target_dir {
                continue;
            }
        }

        // Tạo thư mục đích nếu chưa tồn tại (chỉ tạo khi có file thuộc loại này)
        if !target_dir.exists() {
            if let Err(e) = fs::create_dir_all(&target_dir) {
                result.failed.push((target_folder.to_string(), format!("Không thể tạo thư mục: {}", e)));
                continue;
            }
        }

        let target_path = get_unique_path(&target_dir, &entry.name);

        match fs::rename(&entry.path, &target_path) {
            Ok(()) => result.moved += 1,
            Err(e) => {
                // Thử copy + delete nếu rename thất bại (cross-device move)
                match fs::copy(&entry.path, &target_path) {
                    Ok(_) => {
                        if let Err(e) = fs::remove_file(&entry.path) {
                            result
                                .failed
                                .push((entry.name.clone(), format!("Không thể xóa gốc: {}", e)));
                        } else {
                            result.moved += 1;
                        }
                    }
                    Err(_) => {
                        result
                            .failed
                            .push((entry.name.clone(), format!("Không thể di chuyển: {}", e)));
                    }
                }
            }
        }
    }

    result
}

#[allow(dead_code)]
/// Lấy danh sách tên các thư mục sắp xếp (để loại trừ khi hiển thị)
pub fn get_sort_folder_names() -> Vec<&'static str> {
    SORT_CATEGORIES
        .iter()
        .map(|c| c.folder_name())
        .filter(|n| !n.is_empty())
        .collect()
}
