use code_scanner::config::ProjectConfig;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
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

#[test]
fn example_config_uses_supported_schema_and_extension_format() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(manifest_dir).join(".scanner-config.example.json");
    let content = fs::read_to_string(&path).expect("read example config");
    let value: Value = serde_json::from_str(&content).expect("parse example config json");
    let object = value
        .as_object()
        .expect("example config must be a JSON object");

    let expected_keys = [
        "code_extensions",
        "ignore_dirs",
        "ignore_files",
        "ignore_extensions",
        "max_file_size",
    ];
    let actual_keys: HashSet<_> = object.keys().map(String::as_str).collect();
    let expected_keys: HashSet<_> = expected_keys.into_iter().collect();
    assert_eq!(
        actual_keys, expected_keys,
        "example config keys must match the supported schema exactly"
    );

    for key in ["code_extensions", "ignore_extensions"] {
        let items = object[key]
            .as_array()
            .expect("extension lists must be arrays");
        assert!(
            items.iter().all(|item| {
                item.as_str()
                    .map(|s| !s.starts_with('.'))
                    .unwrap_or(false)
            }),
            "{key} entries must omit a leading dot to match the current parser"
        );
    }
}
