use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn run_loc(cwd: &Path, target: &Path, extra_args: &[&str]) -> String {
    let bin = env!("CARGO_BIN_EXE_code_scanner");
    let mut cmd = Command::new(bin);
    cmd.current_dir(cwd)
        .env("CODE_SCANNER_LANG", "en-US")
        .arg("--loc")
        .arg(target)
        .args(extra_args);

    let output = cmd.output().expect("run code_scanner");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn loc_mode_outputs_summary_and_top_list() {
    let dir = tempdir().expect("temp dir");
    let root = dir.path();

    fs::write(root.join("a.rs"), "one\ntwo\nthree\n").expect("write a.rs");
    fs::write(root.join("b.txt"), "alpha\nbeta\n").expect("write b.txt");
    fs::write(root.join("yarn.lock"), "lock").expect("write lock");
    fs::write(root.join("image.png"), [137u8, 80, 78, 71]).expect("write png");
    fs::write(root.join("bin.rs"), [0u8, 1, 2]).expect("write binary");

    fs::create_dir_all(root.join("node_modules")).expect("create node_modules");
    fs::write(root.join("node_modules").join("ignored.js"), "ignored\n")
        .expect("write ignored");

    let stdout = run_loc(root, root, &["--no-gitignore"]);

    assert!(stdout.contains("LOC SUMMARY"));
    assert!(stdout.contains("Files processed: 2"));
    assert!(stdout.contains("Files skipped: 3"));
    assert!(stdout.contains("Total lines: 5"));
    assert!(stdout.contains("Total characters: 25"));
    assert!(stdout.contains("Estimated tokens: 7"));
    assert!(stdout.contains("TOP 10 FILES WITH MOST LINES"));
    assert!(stdout.contains("a.rs (3 lines"));
    assert!(stdout.contains("b.txt (2 lines"));

    assert!(
        !root.join("output").exists(),
        "LOC mode should not create output directory"
    );
}

#[test]
fn loc_mode_respects_gitignore_by_default() {
    let dir = tempdir().expect("temp dir");
    let root = dir.path();

    fs::write(root.join("main.rs"), "one\ntwo\nthree\n").expect("write main");
    fs::write(root.join("ignored.txt"), "a\nb\nc\nd\n").expect("write ignored");
    fs::write(root.join(".gitignore"), "ignored.txt\n").expect("write gitignore");

    let stdout = run_loc(root, root, &[]);

    assert!(stdout.contains("Files processed: 1"));
    assert!(stdout.contains("Total lines: 3"));
    assert!(!stdout.contains("ignored.txt"));
}
