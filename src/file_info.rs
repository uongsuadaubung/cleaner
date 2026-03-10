use std::path::PathBuf;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SortCriteria {
    Name,
    Created,
    Modified,
    Size,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SortState {
    pub criteria: SortCriteria,
    pub direction: SortDirection,
}

/// Phân loại file theo extension
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FileCategory {
    Document,
    Image,
    Video,
    Audio,
    Archive,
    Executable,
    Code,
    Other,
    Folder,
}

impl FileCategory {
    /// Lấy tên thư mục đích khi sắp xếp
    pub fn folder_name(&self) -> &str {
        match self {
            FileCategory::Document => "Documents",
            FileCategory::Image => "Images",
            FileCategory::Video => "Videos",
            FileCategory::Audio => "Music",
            FileCategory::Archive => "Archives",
            FileCategory::Executable => "Programs",
            FileCategory::Code => "Code",
            FileCategory::Other => "Others",
            FileCategory::Folder => "",
        }
    }

    /// Lấy icon emoji cho category
    pub fn icon(&self) -> &str {
        match self {
            FileCategory::Document => "📄",
            FileCategory::Image => "🖼",
            FileCategory::Video => "🎬",
            FileCategory::Audio => "🎵",
            FileCategory::Archive => "📦",
            FileCategory::Executable => "⚙",
            FileCategory::Code => "💻",
            FileCategory::Other => "📎",
            FileCategory::Folder => "📁",
        }
    }
}

/// Phân loại file dựa trên extension
pub fn get_category_from_extension(ext: &str) -> FileCategory {
    match ext.to_lowercase().as_str() {
        // Documents
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "csv" | "rtf"
        | "odt" | "ods" | "odp" => FileCategory::Document,

        // Images
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" | "tiff" | "tif"
        | "raw" | "psd" | "ai" => FileCategory::Image,

        // Videos
        "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "3gp" => {
            FileCategory::Video
        }

        // Audio
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" | "opus" => FileCategory::Audio,

        // Archives
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" | "cab" => FileCategory::Archive,

        // Executables
        "exe" | "msi" | "bat" | "cmd" | "ps1" | "com" | "scr" => FileCategory::Executable,

        // Code
        "rs" | "py" | "js" | "ts" | "html" | "css" | "java" | "cpp" | "c" | "h" | "hpp" | "go"
        | "rb" | "php" | "swift" | "kt" | "scala" | "lua" | "sh" | "json" | "xml" | "yaml"
        | "yml" | "toml" | "md" | "sql" => FileCategory::Code,

        _ => FileCategory::Other,
    }
}

/// Cấu trúc lưu thông tin một file hoặc thư mục
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
    pub accessed: Option<SystemTime>,
    pub selected: bool,
    pub expanded: bool,
    pub children: Vec<FileEntry>,
    pub category: FileCategory,
}

pub struct FileEntryParams {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
    pub accessed: Option<SystemTime>,
    pub extension: Option<String>,
}

impl FileEntry {
    pub fn new(params: FileEntryParams) -> Self {
        let category = if params.is_dir {
            FileCategory::Folder
        } else {
            params
                .extension
                .as_deref()
                .map(get_category_from_extension)
                .unwrap_or(FileCategory::Other)
        };

        Self {
            name: params.name,
            path: params.path,
            is_dir: params.is_dir,
            size: params.size,
            created: params.created,
            modified: params.modified,
            accessed: params.accessed,
            selected: false,
            expanded: false,
            children: Vec::new(),
            category,
        }
    }

    /// Tính tổng dung lượng (bao gồm cả thư mục con)
    pub fn total_size(&self) -> u64 {
        if self.is_dir {
            self.children.iter().map(|c| c.total_size()).sum()
        } else {
            self.size
        }
    }

    /// Đếm tổng số file (bao gồm cả thư mục con)
    pub fn total_files(&self) -> usize {
        if self.is_dir {
            self.children.iter().map(|c| c.total_files()).sum()
        } else {
            1
        }
    }

    /// Chọn/bỏ chọn tất cả con
    pub fn set_selected_recursive(&mut self, selected: bool) {
        self.selected = selected;
        for child in &mut self.children {
            child.set_selected_recursive(selected);
        }
    }

    /// Đếm số file/thư mục đã chọn
    pub fn count_selected(&self) -> usize {
        // Nếu là thư mục rỗng và được chọn, đếm là 1.
        // Nếu là file và được chọn, đếm là 1.
        // Nếu là thư mục không rỗng, ta không đếm bản thân thư mục để tránh đếm trùng với các file con bên trong,
        // trừ khi muốn đếm theo kiểu "số lượng item". Ở đây ta giữ logic đếm file, nhưng bổ sung cho thư mục rỗng.
        let mut count = if self.selected && (!self.is_dir || self.children.is_empty()) {
            1
        } else {
            0
        };
        for child in &self.children {
            count += child.count_selected();
        }
        count
    }

    /// Tính dung lượng file đã chọn
    pub fn selected_size(&self) -> u64 {
        let mut size = if self.selected && !self.is_dir {
            self.size
        } else {
            0
        };
        for child in &self.children {
            size += child.selected_size();
        }
        size
    }

    #[allow(dead_code)]
    /// Thu thập các file cần sắp xếp.
    /// Nếu only_selected = true, tìm tất cả file được đánh dấu selected trong cây.
    /// Nếu only_selected = false, chỉ lấy chính nó nếu là file.
    pub fn collect_for_sorting(&self, only_selected: bool) -> Vec<FileEntry> {
        let mut files = Vec::new();
        if self.is_dir {
            if only_selected {
                for child in &self.children {
                    files.extend(child.collect_for_sorting(true));
                }
            }
        } else {
            if !only_selected || self.selected {
                // Tạo một bản sao nông (không có con vì là file)
                files.push(self.clone());
            }
        }
        files
    }

    /// Sắp xếp đệ quy các mục con
    pub fn sort_recursive(&mut self, sort_state: Option<SortState>) {
        // Trước tiên sắp xếp các con của từng thư mục con
        for child in &mut self.children {
            if child.is_dir {
                child.sort_recursive(sort_state);
            }
        }

        // Sau đó sắp xếp danh sách con hiện tại
        self.children.sort_by(|a, b| {
            // Quy tắc 1: Luôn giữ thư mục lên trước (hoặc tùy biến nếu muốn sort cả thư mục)
            // Ở đây ta giữ truyền thống: Thư mục trước, File sau.
            let dir_cmp = b.is_dir.cmp(&a.is_dir);
            if dir_cmp != std::cmp::Ordering::Equal {
                return dir_cmp;
            }

            // Quy tắc 2: Nếu cùng loại (cùng là file hoặc cùng là folder), áp dụng sort_state
            if let Some(state) = sort_state {
                let order = match state.criteria {
                    SortCriteria::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    SortCriteria::Created => a.created.cmp(&b.created),
                    SortCriteria::Modified => a.modified.cmp(&b.modified),
                    SortCriteria::Size => {
                        let size_a = if a.is_dir { a.total_size() } else { a.size };
                        let size_b = if b.is_dir { b.total_size() } else { b.size };
                        size_a.cmp(&size_b)
                    }
                };

                if state.direction == SortDirection::Desc {
                    order.reverse()
                } else {
                    order
                }
            } else {
                // Mặc định: Tên A-Z
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
        });
    }
}
