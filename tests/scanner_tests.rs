use code_scanner::cli::Args;
use code_scanner::scanner::collect_loc_stats;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn collect_loc_stats_counts_lines_and_skips_ignored() {
    let dir = tempdir().expect("temp dir");
    let root = dir.path();

    fs::write(root.join("main.rs"), "line1\nline2\nline3\n").expect("write main");
    fs::write(root.join("notes.txt"), "a\nb\n").expect("write notes");
    fs::write(root.join("yarn.lock"), "lock").expect("write lock");
    fs::write(root.join("image.png"), [137u8, 80, 78, 71]).expect("write png");
    fs::write(root.join("binary.rs"), [0u8, 1, 2]).expect("write binary");

    let ignored_dir = root.join("node_modules");
    fs::create_dir_all(&ignored_dir).expect("create node_modules");
    fs::write(ignored_dir.join("ignored.js"), "ignored\n").expect("write ignored");

    let args = Args {
        input_dir: PathBuf::from("."),
        output_dir: PathBuf::from("."),
        loc: None,
        no_gitignore: true,
        verbose: false,
        ignore: vec![],
    };

    let stats = collect_loc_stats(root, &args).expect("loc stats");

    assert_eq!(stats.processed_files, 2);
    assert_eq!(stats.skipped_files, 3);
    assert_eq!(stats.total_lines, 5);
    assert_eq!(stats.total_chars, 22);
    assert_eq!(stats.estimated_tokens, 6);
    assert_eq!(stats.largest_files[0].lines, 3);
    assert_eq!(stats.largest_files[1].lines, 2);
}
