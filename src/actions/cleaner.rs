use std::path::PathBuf;

use crate::file_info::FileEntry;
use crate::utils::is_older_than_days;

/// Kết quả xóa file
pub struct CleanResult {
    pub deleted: usize,
    pub failed: Vec<(String, String)>, // (tên file, lý do)
}

/// Chọn tất cả file cũ hơn N ngày (đánh dấu selected) - đệ quy toàn bộ cây
pub fn select_old_files(entries: &mut [FileEntry], days: u64) {
    for entry in entries.iter_mut() {
        if entry.is_dir {
            select_old_files(&mut entry.children, days);
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
pub fn select_old_files_shallow(entries: &mut [FileEntry], days: u64) {
    for entry in entries.iter_mut() {
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

/// Xóa các file đã chọn (chuyển vào Recycle Bin)
pub fn delete_selected_files(
    entries: &[FileEntry],
    progress_tx: Option<std::sync::mpsc::Sender<(usize, usize, String)>>,
) -> CleanResult {
    let mut result = CleanResult {
        deleted: 0,
        failed: Vec::new(),
    };

    let selected_paths = collect_selected_paths(entries);
    let total = selected_paths.len();

    // Chia nhỏ thành các batch (ví dụ 50 file một lần) để cân bằng giữa tốc độ (Windows Shell) và tiến độ UI
    let chunk_size = 50;
    for (chunk_idx, chunk) in selected_paths.chunks(chunk_size).enumerate() {
        let current_count = (chunk_idx * chunk_size + chunk.len()).min(total);

        let file_name = chunk[0]
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| chunk[0].display().to_string());

        // Gửi progress trước khi xóa batch
        if let Some(ref tx) = progress_tx {
            let _ = tx.send((
                current_count,
                total,
                format!("Đang xóa nhóm chứa: {}", file_name),
            ));
        }

        match trash::delete_all(chunk) {
            Ok(()) => result.deleted += chunk.len(),
            Err(e) => {
                result.failed.push((
                    format!("Nhóm số {}", chunk_idx + 1),
                    format!("Lỗi xóa hàng loạt: {}", e),
                ));
            }
        }
    }

    result
}

/// Thu thập đường dẫn các item đã chọn (file hoặc thư mục)
fn collect_selected_paths(entries: &[FileEntry]) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for entry in entries {
        if entry.selected {
            // Nếu item được chọn (có thể là file hoặc thư mục), ta lấy chính nó.
            // Với thư mục, việc xóa nó sẽ bao gồm cả việc xóa các con bên trong.
            paths.push(entry.path.clone());
        } else if entry.is_dir {
            // Nếu thư mục không được chọn, ta duyệt tiếp các con của nó.
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
