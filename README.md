# Code Scanner

Bundle a codebase into a single text report for audits, code reviews, and LLM prompts. This repository ships two independent scanners: a **Rust CLI** (compiled binary, LOC mode, CLI flags) and a **Bash script** (no Rust toolchain required). Pick the one that fits your environment—no wrapper CLI required.

## 30-second quickstart

```bash
git clone https://github.com/ThalesMMS/Code-Scanner.git
cd Code-Scanner
```

Scan a project with the Rust CLI:

```bash
cargo run -- --input-dir ../my-project --output-dir ./output
# quick size summary only:
cargo loc ../my-project
```

Or scan the same project with the Bash script:

```bash
TARGET_DIR=../my-project OUTPUT_DIR=./output ./bash/scan_project.sh
```

## What you get
- A plain-text bundle in `output/`, typically named `<project>_project_code.txt`
- A fast LOC-only summary (see [LOC mode](#loc-mode-rust))
- Optional verbose mode with file sizes, line numbers, and a final summary block
- Default support for `.gitignore`, common dependency folders, and large-file skipping

Example LOC output from this repository (`cargo loc .`):
Sample output only; values will vary by commit.

```text
📊 LOC SUMMARY
  ✅ Files processed: 15
  ⏭️  Files skipped: 6
  🧮 Total lines: 1790
  🔤 Total characters: 59879
  🤖 Estimated tokens: 14970
```

## What's inside
- `src/` – Rust CLI that walks projects, respects `.gitignore`, and writes combined reports.
- `bash/scan_project.sh` – Bash scanner with project-type detection, `.gitignore` support, and verbose/debug modes.
- `input/` – Default drop-in directory for projects to scan (kept by `.gitkeep`).
- `output/` – Generated reports (ignored except for `.gitkeep`).
- `.scanner-config.example.json` – Example configuration shared by the scanners.

## Requirements
- Rust CLI: Rust 1.70+ with Cargo.
- Bash scanner: Bash 4+ on macOS/Linux with standard POSIX tools (`find`, `sed`, `awk`, `stat`, `nl`, `grep`, etc.).

## Default workflow
If you prefer not to point at a project path directly, the repository also supports the original drop-in flow:

```bash
# add one or more projects inside ./input
cargo run
# or
./bash/scan_project.sh
```

## Configuration
- `.scanner-config.json` in the target project adjusts code extensions, ignore lists, and max file size (see `.scanner-config.example.json`).
- Bash scanner environment examples:
  - `USE_GITIGNORE=false ./bash/scan_project.sh`
  - `TARGET_DIR=./my-project OUTPUT_DIR=./reports ./bash/scan_project.sh`
- Rust CLI flags mirror the defaults used by the scripts:
  - `cargo run -- --no-gitignore --verbose`
  - Override defaults with `--input-dir` and `--output-dir` when needed.
  - LOC-only summary (no report files): `cargo loc ./path/to/project`

## Output
Each project yields a text report in `output/`, typically named `<project>_project_code.txt` or `<project>_*_summary.txt` depending on the scanner. Large binaries, dependency folders, IDE files, and `.gitignore`d paths are skipped by default.

Verbose mode (`--verbose` on the Rust CLI, `VERBOSE=true` in Bash) adds file headers with size, line numbers, and a final summary block. Without verbose, the report lists file paths followed by raw file contents.

## LOC mode (Rust)
Use LOC mode when you want a quick size summary without generating report files:

```bash
cargo loc ./path/to/project
```

Output includes total lines, total characters, a token estimate (chars/4), and the top 10 files by line count. Dotfiles are excluded from LOC counts unless you whitelist them by adding the filename (e.g., `.gitignore` or `gitignore`) to `code_extensions` in `.scanner-config.json`.

## Development (Rust)
- Format: `cargo fmt`
- Lint: `cargo clippy -- -D warnings`
- Tests: `cargo test`
- Coverage (install once with `cargo install cargo-llvm-cov`): `cargo coverage` (fails under 35% line coverage)