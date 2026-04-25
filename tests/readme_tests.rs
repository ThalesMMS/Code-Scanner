// Tests that verify the CLI behavior documented in the README is accurate.
//
// The README was updated in this PR to add:
// - A "30-second quickstart" section documenting --input-dir and --output-dir flags
// - A "What you get" section documenting the LOC summary format and output file naming
// - A "Which scanner should I use?" section referencing bash/scan_project.sh
// - A "Default workflow" section documenting default directories
// - A reference to .scanner-config.example.json
//
// Each test here validates that a specific claim in the README is accurate.

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_code_scanner")
}

// ---------------------------------------------------------------------------
// "30-second quickstart" section
// ---------------------------------------------------------------------------

/// README quickstart shows: `cargo run -- --input-dir ../my-project --output-dir ./output`
/// Verify the binary accepts both long-form flags.
#[test]
fn cli_accepts_input_dir_and_output_dir_flags() {
    let input = tempdir().expect("input temp dir");
    let output = tempdir().expect("output temp dir");

    // Write a minimal Rust file so the scanner has something to process.
    fs::write(input.path().join("main.rs"), "fn main() {}\n").expect("write main.rs");

    let status = Command::new(bin())
        .arg("--input-dir")
        .arg(input.path())
        .arg("--output-dir")
        .arg(output.path())
        .status()
        .expect("run binary");

    assert!(
        status.success(),
        "--input-dir / --output-dir flags must be accepted by the binary"
    );
}

/// README quickstart shows: `cargo run -- --input-dir ../my-project --output-dir ./output`
/// Verify that short forms -i / -o also work (they are defined on the same args).
#[test]
fn cli_accepts_short_input_output_flags() {
    let input = tempdir().expect("input temp dir");
    let output = tempdir().expect("output temp dir");

    fs::write(input.path().join("lib.rs"), "pub fn f() {}\n").expect("write lib.rs");

    let status = Command::new(bin())
        .arg("-i")
        .arg(input.path())
        .arg("-o")
        .arg(output.path())
        .status()
        .expect("run binary");

    assert!(
        status.success(),
        "-i / -o short flags must be accepted by the binary"
    );
}

/// README quickstart shows: `cargo loc ../my-project`
/// .cargo/config.toml maps `loc` to `run -- --loc`.
/// Verify that --loc <PATH> is a valid flag accepted by the binary.
#[test]
fn cli_accepts_loc_flag() {
    let dir = tempdir().expect("temp dir");
    fs::write(dir.path().join("hello.rs"), "fn main() {}\n").expect("write hello.rs");

    let output = Command::new(bin())
        .arg("--loc")
        .arg(dir.path())
        .arg("--no-gitignore")
        .output()
        .expect("run binary");

    assert!(
        output.status.success(),
        "--loc flag must be accepted by the binary; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// README documents: `cargo run -- --input-dir ./my-project --output-dir ./reports --ignore ts js json`
/// Verify one --ignore flag accepts multiple extension values.
#[test]
fn cli_accepts_multiple_ignore_extensions_after_one_flag() {
    let input = tempdir().expect("input temp dir");
    let output_dir = tempdir().expect("output temp dir");
    fs::write(input.path().join("main.rs"), "fn main() {}\n").expect("write main.rs");
    fs::write(input.path().join("app.ts"), "const app = true;\n").expect("write app.ts");
    fs::write(input.path().join("view.js"), "console.log('view');\n").expect("write view.js");
    fs::write(input.path().join("data.json"), "{\"ok\": true}\n").expect("write data.json");

    let status = Command::new(bin())
        .arg("--input-dir")
        .arg(input.path())
        .arg("--output-dir")
        .arg(output_dir.path())
        .arg("--ignore")
        .arg("ts")
        .arg("js")
        .arg("json")
        .status()
        .expect("run binary");

    assert!(
        status.success(),
        "--ignore must accept multiple values after one flag"
    );

    let report_path = fs::read_dir(output_dir.path())
        .expect("read output dir")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("txt"))
        .expect("scanner should write a report");
    let report = fs::read_to_string(report_path).expect("read report");

    assert!(
        report.contains("main.rs"),
        "non-ignored Rust file should still be included; got:\n{report}"
    );
    assert!(
        !report.contains("app.ts") && !report.contains("view.js") && !report.contains("data.json"),
        "ignored extensions must not appear in the report; got:\n{report}"
    );
}

// ---------------------------------------------------------------------------
// "What you get" — LOC summary labels
// ---------------------------------------------------------------------------

/// README documents the exact LOC summary output with these emoji labels:
///
///   📊 LOC SUMMARY
///   ✅ Files processed: 15
///   ⏭️  Files skipped: 6
///   🧮 Total lines: 1790
///   🔤 Total characters: 59879
///   🤖 Estimated tokens: 14970
///
/// Verify every label appears verbatim in the binary's LOC output.
#[test]
fn loc_summary_labels_match_readme_example() {
    let dir = tempdir().expect("temp dir");
    fs::write(dir.path().join("code.rs"), "fn a() {}\nfn b() {}\n").expect("write code.rs");

    let output = Command::new(bin())
        .arg("--loc")
        .arg(dir.path())
        .arg("--no-gitignore")
        .output()
        .expect("run binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("📊 LOC SUMMARY"),
        "LOC output must contain '📊 LOC SUMMARY'; got:\n{stdout}"
    );
    assert!(
        stdout.contains("✅ Files processed:"),
        "LOC output must contain '✅ Files processed:'; got:\n{stdout}"
    );
    assert!(
        stdout.contains("⏭️  Files skipped:"),
        "LOC output must contain '⏭️  Files skipped:'; got:\n{stdout}"
    );
    assert!(
        stdout.contains("🧮 Total lines:"),
        "LOC output must contain '🧮 Total lines:'; got:\n{stdout}"
    );
    assert!(
        stdout.contains("🔤 Total characters:"),
        "LOC output must contain '🔤 Total characters:'; got:\n{stdout}"
    );
    assert!(
        stdout.contains("🤖 Estimated tokens:"),
        "LOC output must contain '🤖 Estimated tokens:'; got:\n{stdout}"
    );
}

/// README says LOC mode also shows a "TOP 10 FILES WITH MOST LINES" list.
#[test]
fn loc_summary_includes_top_10_heading() {
    let dir = tempdir().expect("temp dir");
    fs::write(dir.path().join("main.rs"), "one\ntwo\nthree\n").expect("write main.rs");

    let output = Command::new(bin())
        .arg("--loc")
        .arg(dir.path())
        .arg("--no-gitignore")
        .output()
        .expect("run binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("TOP 10 FILES WITH MOST LINES"),
        "LOC output must contain 'TOP 10 FILES WITH MOST LINES'; got:\n{stdout}"
    );
}

/// README describes LOC mode as a "quick size summary without generating report files".
/// Verify --loc produces no output directory.
#[test]
fn loc_mode_does_not_create_output_files() {
    let dir = tempdir().expect("temp dir");
    fs::write(dir.path().join("mod.rs"), "pub mod m;\n").expect("write mod.rs");

    let output_dir = dir.path().join("output");
    assert!(!output_dir.exists(), "precondition: output dir must not exist");

    let status = Command::new(bin())
        .arg("--loc")
        .arg(dir.path())
        .arg("--no-gitignore")
        .current_dir(dir.path())
        .status()
        .expect("run binary");

    assert!(status.success());
    assert!(
        !output_dir.exists(),
        "LOC mode must not create any output directory"
    );
}

/// README says the target path is shown in the LOC summary ("📁 Target:").
#[test]
fn loc_summary_shows_target_path() {
    let dir = tempdir().expect("temp dir");
    fs::write(dir.path().join("app.rs"), "fn run() {}\n").expect("write app.rs");

    let output = Command::new(bin())
        .arg("--loc")
        .arg(dir.path())
        .arg("--no-gitignore")
        .output()
        .expect("run binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("📁 Target:"),
        "LOC output must contain '📁 Target:'; got:\n{stdout}"
    );
}

/// When there are no processable files LOC mode should report "(no files counted)" rather
/// than crashing — the README implies LOC always produces a summary.
#[test]
fn loc_summary_handles_empty_directory() {
    let dir = tempdir().expect("temp dir");
    // Write only a file that will be skipped (binary).
    fs::write(dir.path().join("img.png"), [137u8, 80, 78, 71]).expect("write png");

    let output = Command::new(bin())
        .arg("--loc")
        .arg(dir.path())
        .arg("--no-gitignore")
        .output()
        .expect("run binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("📊 LOC SUMMARY"),
        "LOC output must still contain summary header with no processable files; got:\n{stdout}"
    );
    assert!(
        stdout.contains("(no files counted)"),
        "LOC output must say '(no files counted)' when no files qualify; got:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// "What you get" — output file naming convention
// ---------------------------------------------------------------------------

/// README says: "A plain-text bundle in output/, typically named <project>_project_code.txt"
/// Verify the output file follows this naming pattern.
#[test]
fn scan_output_file_uses_project_name_suffix() {
    let input = tempdir().expect("input temp dir");
    let output = tempdir().expect("output temp dir");

    // Use a known directory name so we can predict the output filename.
    let project_dir = input.path().join("myproject");
    fs::create_dir_all(&project_dir).expect("create project dir");
    fs::write(project_dir.join("main.rs"), "fn main() {}\n").expect("write main.rs");

    let status = Command::new(bin())
        .arg("--input-dir")
        .arg(&project_dir)
        .arg("--output-dir")
        .arg(output.path())
        .status()
        .expect("run binary");

    assert!(status.success());

    let expected = output.path().join("myproject_project_code.txt");
    assert!(
        expected.exists(),
        "output file must be named <project>_project_code.txt; expected: {}",
        expected.display()
    );
}

// ---------------------------------------------------------------------------
// "Which scanner should I use?" — referenced files exist
// ---------------------------------------------------------------------------

/// README recommends `./bash/scan_project.sh` for users without Rust.
/// The file must actually exist in the repository.
#[test]
fn bash_scanner_script_exists_in_repo() {
    // Resolve relative to the workspace root using CARGO_MANIFEST_DIR.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let script = Path::new(manifest_dir).join("bash").join("scan_project.sh");
    assert!(
        script.exists(),
        "bash/scan_project.sh must exist (referenced in README); path checked: {}",
        script.display()
    );
}

/// README references `.scanner-config.example.json` as the shared example configuration.
/// The file must actually exist in the repository.
#[test]
fn scanner_config_example_exists_in_repo() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config = Path::new(manifest_dir).join(".scanner-config.example.json");
    assert!(
        config.exists(),
        ".scanner-config.example.json must exist (referenced in README); path checked: {}",
        config.display()
    );
}

// ---------------------------------------------------------------------------
// "Default workflow" section — default directories
// ---------------------------------------------------------------------------

/// README "Default workflow" says `cargo run` (no flags) uses ./input by default.
/// Verify the Args struct default for input_dir is "./input".
#[test]
fn default_input_dir_matches_readme() {
    // When cargo run is invoked with no --input-dir the binary uses "./input".
    // We verify this by running the binary without the flag in a temp working
    // directory that has no `input/` folder: it should create it and exit cleanly
    // (the documented behaviour for a missing default input dir).
    let cwd = tempdir().expect("cwd temp dir");

    let output = Command::new(bin())
        .current_dir(cwd.path())
        .output()
        .expect("run binary with no args");

    // The binary should exit successfully and print the info message about input/.
    assert!(
        output.status.success(),
        "binary must exit successfully when default ./input does not exist yet; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("input"),
        "binary must mention the default input directory; got:\n{stdout}"
    );

    // The default input directory must have been created.
    assert!(
        cwd.path().join("input").exists(),
        "binary must create the default ./input directory when it does not exist"
    );
}

/// README "Default workflow" says output goes to ./output by default.
/// The binary must create the output dir automatically.
#[test]
fn default_output_dir_is_created_automatically() {
    let input = tempdir().expect("input temp dir");
    let cwd = tempdir().expect("cwd");

    fs::write(input.path().join("src.rs"), "fn f() {}\n").expect("write src.rs");

    // Point to an explicit input but let output default to ./output in cwd.
    let output = Command::new(bin())
        .current_dir(cwd.path())
        .arg("--input-dir")
        .arg(input.path())
        .output()
        .expect("run binary");

    assert!(
        output.status.success(),
        "binary must exit successfully; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        cwd.path().join("output").exists(),
        "binary must create the default ./output directory automatically"
    );
}
