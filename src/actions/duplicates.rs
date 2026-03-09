use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use sha2::{Sha256, Digest};
use crate::file_info::{DuplicateFile, DuplicateGroup};

/// Tìm kiếm các file trùng lặp trong thư mục
pub fn find_duplicates(root: &Path) -> Vec<DuplicateGroup> {
    let mut size_map: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    
    // Bước 1: Thu thập tất cả file và nhóm theo kích thước
    collect_files_by_size(root, &mut size_map);
    
    // Bước 2: Chỉ lấy các nhóm có nhiều hơn 1 file
    let potential_duplicates: Vec<(u64, Vec<PathBuf>)> = size_map
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .collect();
        
    let mut duplicates_by_hash: HashMap<String, Vec<DuplicateFile>> = HashMap::new();
    
    // Bước 3: Tính hash cho các file có cùng kích thước
    for (size, paths) in potential_duplicates {
        for path in paths {
            if let Ok(hash) = hash_file(&path) {
                let metadata = fs::metadata(&path).ok();
                let file = DuplicateFile {
                    name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                    path: path.clone(),
                    size,
                    modified: metadata.and_then(|m| m.modified().ok()),
                    selected: false,
                };
                duplicates_by_hash.entry(hash).or_default().push(file);
            }
        }
    }
    
    // Bước 4: Chuyển đổi thành Vec<DuplicateGroup>
    duplicates_by_hash
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(hash, files)| {
            let size = files[0].size;
            DuplicateGroup { hash, size, files }
        })
        .collect()
}

fn collect_files_by_size(path: &Path, size_map: &mut HashMap<u64, Vec<PathBuf>>) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files_by_size(&path, size_map);
            } else if path.is_file() {
                if let Ok(metadata) = entry.metadata() {
                    size_map.entry(metadata.len()).or_default().push(path);
                }
            }
        }
    }
}

fn hash_file(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 { break; }
        hasher.update(&buffer[..count]);
    }
    
    Ok(hex::encode(hasher.finalize()))
}
