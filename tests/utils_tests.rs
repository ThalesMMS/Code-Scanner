use code_scanner::utils::{format_size, is_binary};
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
