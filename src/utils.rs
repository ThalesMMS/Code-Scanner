//
// utils.rs
// Code-Scanner-rs
//
// Provides helper utilities for binary detection and human-readable file sizing used throughout the scanning pipeline.
//
// Thales Matheus Mendonça Santos - November 2025
//

use std::fs::File;
use std::io::Read;
use std::path::Path;

// Naively detect binary files by scanning for null bytes in the first 1KB.
pub fn is_binary(path: &Path) -> bool {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return true, // Assume binary if the file cannot be opened
    };

    let mut buffer = [0; 1024];
    let n = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return true,
    };

    buffer[..n].contains(&0)
}

// Present human-readable file sizes (e.g., 1.2 MB).
pub fn format_size(size: u64) -> String {
    humansize::format_size(size, humansize::DECIMAL)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn is_binary_detects_null_bytes() {
        let dir = tempdir().expect("temp dir");
        let file_path = dir.path().join("binary.bin");
        fs::write(&file_path, [0u8, 1, 2]).expect("write binary");
        assert!(is_binary(&file_path));
    }

    #[test]
    fn is_binary_false_for_text() {
        let dir = tempdir().expect("temp dir");
        let file_path = dir.path().join("text.txt");
        fs::write(&file_path, "hello\nworld\n").expect("write text");
        assert!(!is_binary(&file_path));
    }

    #[test]
    fn format_size_formats_zero() {
        assert_eq!(format_size(0), "0 B");
    }
}
