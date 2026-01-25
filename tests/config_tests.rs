use code_scanner::config::ProjectConfig;
use std::collections::HashSet;
use std::fs;
use tempfile::tempdir;

#[test]
fn apply_overrides_replaces_non_empty_fields() {
    let mut base = ProjectConfig::default();
    let overrides = ProjectConfig {
        code_extensions: HashSet::from(["rs".to_string()]),
        ignore_dirs: HashSet::new(),
        ignore_files: HashSet::from(["custom.lock".to_string()]),
        ignore_extensions: HashSet::new(),
        max_file_size: 123,
    };

    base.apply_overrides(overrides);

    assert_eq!(base.code_extensions, HashSet::from(["rs".to_string()]));
    assert_eq!(base.ignore_files, HashSet::from(["custom.lock".to_string()]));
    assert_eq!(base.max_file_size, 123);
    assert!(base.ignore_dirs.contains("node_modules"));
}

#[test]
fn load_config_merges_json_overrides() {
    let dir = tempdir().expect("temp dir");
    let config_path = dir.path().join(".scanner-config.json");
    let config_json = r#"{
        "code_extensions": ["rs"],
        "ignore_dirs": ["vendor"],
        "max_file_size": 2048
    }"#;
    fs::write(&config_path, config_json).expect("write config");

    let config = code_scanner::config::load_config(dir.path());

    assert!(config.code_extensions.contains("rs"));
    assert!(config.ignore_dirs.contains("vendor"));
    assert_eq!(config.max_file_size, 2048);
    assert!(config.ignore_files.contains("package-lock.json"));
}
