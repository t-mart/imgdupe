use std::path::{Path, PathBuf};

use walkdir::WalkDir;

pub fn walk_files<P: AsRef<Path>>(paths: Vec<P>) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for path in paths {
        if path.as_ref().is_dir() {
            files.extend(walk_files_in_dir(path));
        } else {
            files.push(path.as_ref().to_path_buf());
        }
    }
    files
}

fn walk_files_in_dir<P: AsRef<Path>>(root_dir: P) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().is_dir())
    {
        let path = entry.path();
        files.push(path.to_path_buf())
    }
    files
}
