use std::path::PathBuf;

use crate::file_info::FileEntry;
use crate::utils::is_older_than_days;

/// Kết quả xóa file
pub struct CleanResult {
    pub deleted: usize,
    pub failed: Vec<(String, String)>, // (tên file, lý do)
}

/// Kiểm tra xem tên có thuộc danh sách loại trừ không
pub fn is_excluded_by_name(name: &str, exclude_list: &[String]) -> bool {
    let lower_name = name.to_lowercase();
    for pattern in exclude_list {
        let p = pattern.to_lowercase();
        if lower_name == p {
            return true;
        }
        if p.starts_with('.') && lower_name.ends_with(&p) {
            return true;
        }
        if p.starts_with("*.") && lower_name.ends_with(&p[1..]) {
            return true;
        }
    }
    false
}

/// Chọn tất cả file cũ hơn N ngày (đánh dấu selected) - đệ quy toàn bộ cây
pub fn select_old_files(entries: &mut [FileEntry], days: u64, exclude_list: &[String]) {
    for entry in entries.iter_mut() {
        if is_excluded_by_name(&entry.name, exclude_list) {
            entry.set_selected_recursive(false);
            continue;
        }

        if entry.is_dir {
            select_old_files(&mut entry.children, days, exclude_list);
            // Nếu tất cả con đều được chọn thì chọn folder cha
            let all_selected =
                !entry.children.is_empty() && entry.children.iter().all(|c| c.selected);
            entry.selected = all_selected;
        } else {
            // Lấy thời gian gần nhất trong các mốc (Sửa, Truy cập, Tạo)
            let latest_time = [entry.modified, entry.accessed, entry.created]
                .iter()
                .filter_map(|&t| t)
                .max();

            entry.selected = is_older_than_days(latest_time, days);
        }
    }
}

/// Chọn file cũ hơn N ngày - chỉ trong thư mục gốc (không vào thư mục con)
pub fn select_old_files_shallow(entries: &mut [FileEntry], days: u64, exclude_list: &[String]) {
    for entry in entries.iter_mut() {
        if is_excluded_by_name(&entry.name, exclude_list) {
            entry.selected = false;
            continue;
        }

        if entry.is_dir {
            // Không đệ quy, bỏ qua thư mục con hoàn toàn
            entry.selected = false;
        } else {
            let latest_time = [entry.modified, entry.accessed, entry.created]
                .iter()
                .filter_map(|&t| t)
                .max();
            entry.selected = is_older_than_days(latest_time, days);
        }
    }
}

/// Xóa các file đã chọn (vào Recycle Bin hoặc xóa vĩnh viễn)
pub fn delete_selected_files(
    entries: &[FileEntry],
    permanent: bool,
    progress_tx: Option<std::sync::mpsc::Sender<(usize, usize, String)>>,
) -> CleanResult {
    let mut result = CleanResult {
        deleted: 0,
        failed: Vec::new(),
    };

    let selected_items = collect_selected_paths(entries);
    let total_files: usize = selected_items.iter().map(|(_, count)| count).sum();
    let mut current_files = 0;

    // Nếu xóa vĩnh viễn, ta xóa từng file/thư mục thay vì đưa vào thùng rác
    if permanent {
        for (path, count) in selected_items {
            let file_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string());

            if let Some(ref tx) = progress_tx {
                let _ = tx.send((
                    current_files,
                    total_files,
                    format!("Đang xóa vĩnh viễn: {}", file_name),
                ));
            }

            if path.is_dir() {
                if let Err(e) = std::fs::remove_dir_all(&path) {
                    result.failed.push((file_name, format!("Lỗi xóa thư mục: {}", e)));
                } else {
                    result.deleted += count;
                }
            } else {
                if let Err(e) = std::fs::remove_file(&path) {
                    result.failed.push((file_name, format!("Lỗi xóa file: {}", e)));
                } else {
                    result.deleted += count;
                }
            }
            
            current_files += count;
        }
    } else {
        // Chia nhỏ thành các batch (ví dụ 50 file một lần)
        let chunk_size = 50;
        for (chunk_idx, chunk) in selected_items.chunks(chunk_size).enumerate() {
            let paths: Vec<PathBuf> = chunk.iter().map(|(p, _)| p.clone()).collect();
            let chunk_files: usize = chunk.iter().map(|(_, c)| c).sum();

            let file_name = paths[0]
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| paths[0].display().to_string());

            // Gửi progress trước khi xóa batch
            if let Some(ref tx) = progress_tx {
                let _ = tx.send((
                    current_files,
                    total_files,
                    format!("Đang xóa nhóm chứa: {}", file_name),
                ));
            }

            match trash::delete_all(&paths) {
                Ok(()) => result.deleted += chunk_files,
                Err(e) => {
                    result.failed.push((
                        format!("Nhóm số {}", chunk_idx + 1),
                        format!("Lỗi xóa hàng loạt: {}", e),
                    ));
                }
            }
            
            current_files += chunk_files;
        }
    }
    
    // Đảm bảo gửi tiến trình cuối cùng là hoàn thành
    if let Some(ref tx) = progress_tx {
        let _ = tx.send((
            total_files,
            total_files,
            "Hoàn tất!".to_string(),
        ));
    }

    result
}

/// Thu thập đường dẫn các item đã chọn kèm theo số lượng file con của nó
fn collect_selected_paths(entries: &[FileEntry]) -> Vec<(PathBuf, usize)> {
    let mut paths = Vec::new();
    for entry in entries {
        if entry.selected {
            // Nếu item được chọn, lấy nó và đếm số lượng file thực tế của nó
            paths.push((entry.path.clone(), entry.count_selected()));
        } else if entry.is_dir {
            // Nếu không, tiếp tục duyệt nhánh con
            paths.extend(collect_selected_paths(&entry.children));
        }
    }
    paths
}

/// Bỏ chọn tất cả file
pub fn deselect_all(entries: &mut [FileEntry]) {
    for entry in entries.iter_mut() {
        entry.selected = false;
        if entry.is_dir {
            deselect_all(&mut entry.children);
        }
    }
}
