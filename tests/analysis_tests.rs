use code_scanner::cli::Args;
use code_scanner::scanner::analyze_project;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn args(no_gitignore: bool) -> Args {
    Args {
        input_dir: PathBuf::from("."),
        output_dir: PathBuf::from("."),
        loc: None,
        no_gitignore,
        verbose: false,
        ignore: vec![],
        max_output_lines: None,
    }
}

#[test]
fn analysis_aggregates_extension_stats() {
    let dir = tempdir().expect("temp dir");
    let src = dir.path().join("src");
    fs::create_dir_all(&src).expect("create src");
    fs::write(src.join("lib.rs"), "one\ntwo\n").expect("write rust");
    fs::write(src.join("app.ts"), "const x = 1;\n").expect("write ts");
    fs::write(dir.path().join("README.md"), "# Title\nbody\n").expect("write readme");

    let analysis = analyze_project(dir.path(), &args(true)).expect("analysis");

    assert_eq!(analysis.processed_files, 3);
    assert_eq!(analysis.total_lines, 5);
    assert_eq!(analysis.tree.directories, 1);
    assert_eq!(analysis.tree.files, 3);
    assert_eq!(analysis.tree.max_depth, 2);

    let by_ext: HashMap<_, _> = analysis
        .extension_breakdown
        .iter()
        .map(|entry| (entry.extension.as_str(), entry))
        .collect();
    assert_eq!(by_ext["rs"].files, 1);
    assert_eq!(by_ext["rs"].lines, 2);
    assert_eq!(by_ext["ts"].language, "TypeScript");
    assert_eq!(by_ext["md"].lines, 2);
    assert_eq!(analysis.largest_by_lines[0].lines, 2);
}

#[test]
fn analysis_counts_skipped_files_by_reason() {
    let dir = tempdir().expect("temp dir");
    fs::write(dir.path().join("yarn.lock"), "lock").expect("write lock");
    fs::write(dir.path().join("image.png"), [137u8, 80, 78, 71]).expect("write image");
    fs::write(dir.path().join("binary.rs"), [0u8, 1, 2]).expect("write binary");
    fs::write(dir.path().join("scratch.tmp"), "tmp").expect("write tmp");
    fs::write(dir.path().join(".env"), "SECRET=1").expect("write env");

    let analysis = analyze_project(dir.path(), &args(true)).expect("analysis");
    let reasons: HashMap<_, _> = analysis
        .skipped_reasons
        .iter()
        .map(|entry| (entry.reason.as_str(), entry.files))
        .collect();

    assert_eq!(analysis.processed_files, 0);
    assert_eq!(analysis.skipped_files, 5);
    assert_eq!(reasons["ignored file"], 1);
    assert_eq!(reasons["ignored extension"], 1);
    assert_eq!(reasons["binary file"], 1);
    assert_eq!(reasons["unsupported extension"], 1);
    assert_eq!(reasons["hidden file"], 1);
}

#[test]
fn analysis_respects_gitignore_by_default() {
    let dir = tempdir().expect("temp dir");
    fs::write(dir.path().join(".gitignore"), "ignored.rs\n").expect("write gitignore");
    fs::write(dir.path().join("ignored.rs"), "ignored\n").expect("write ignored");
    fs::write(dir.path().join("kept.rs"), "kept\n").expect("write kept");

    let analysis = analyze_project(dir.path(), &args(false)).expect("analysis");

    assert!(analysis.config.gitignore_enabled);
    assert_eq!(analysis.processed_files, 1);
    assert_eq!(analysis.largest_by_lines[0].path, PathBuf::from("kept.rs"));
}

#[test]
fn analysis_can_disable_gitignore() {
    let dir = tempdir().expect("temp dir");
    fs::write(dir.path().join(".gitignore"), "ignored.rs\n").expect("write gitignore");
    fs::write(dir.path().join("ignored.rs"), "ignored\n").expect("write ignored");
    fs::write(dir.path().join("kept.rs"), "kept\n").expect("write kept");

    let analysis = analyze_project(dir.path(), &args(true)).expect("analysis");

    assert!(!analysis.config.gitignore_enabled);
    assert_eq!(analysis.processed_files, 2);
}

#[test]
fn analysis_applies_scanner_config() {
    let dir = tempdir().expect("temp dir");
    fs::write(
        dir.path().join(".scanner-config.json"),
        r#"{
            "code_extensions": ["rs"],
            "ignore_extensions": ["txt"],
            "max_file_size": 4
        }"#,
    )
    .expect("write config");
    fs::write(dir.path().join("small.rs"), "ok\n").expect("write small");
    fs::write(dir.path().join("big.rs"), "12345").expect("write big");
    fs::write(dir.path().join("app.ts"), "const x = 1;\n").expect("write ts");
    fs::write(dir.path().join("notes.txt"), "notes\n").expect("write txt");

    let analysis = analyze_project(dir.path(), &args(true)).expect("analysis");
    let reasons: HashMap<_, _> = analysis
        .skipped_reasons
        .iter()
        .map(|entry| (entry.reason.as_str(), entry.files))
        .collect();

    assert!(analysis.config.config_present);
    assert_eq!(analysis.config.max_file_size, 4);
    assert_eq!(analysis.processed_files, 1);
    assert_eq!(analysis.largest_by_size[0].path, PathBuf::from("small.rs"));
    assert_eq!(reasons["over max file size"], 1);
    assert_eq!(reasons["unsupported extension"], 1);
    assert_eq!(reasons["ignored extension"], 1);
}

#[test]
fn analysis_handles_empty_projects() {
    let dir = tempdir().expect("temp dir");

    let analysis = analyze_project(dir.path(), &args(true)).expect("analysis");

    assert_eq!(analysis.processed_files, 0);
    assert_eq!(analysis.skipped_files, 0);
    assert_eq!(analysis.total_lines, 0);
    assert_eq!(analysis.extension_breakdown.len(), 0);
}
