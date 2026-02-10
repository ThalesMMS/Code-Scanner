# Code Scanner

Toolkit with two independent scanners that produce text bundles of your code: a Rust CLI and a Bash script. Pick the version you want by running the corresponding binary or script—no wrapper CLI required.

## What's inside
- `src/` – Rust CLI that walks projects, respects `.gitignore`, and writes combined reports.
- `bash/scan_project.sh` – Bash scanner with project-type detection, `.gitignore` support, and verbose/debug modes.
- `input/` – Drop projects to scan (kept by `.gitkeep`).
- `output/` – Generated reports (ignored except for `.gitkeep`).
- `.scanner-config.example.json` – Example configuration shared by the scanners.

## Requirements
- Rust CLI: Rust 1.70+ with Cargo.
- Bash scanner: Bash 4+ on macOS/Linux with standard POSIX tools (`find`, `sed`, `awk`, `stat`, `nl`, `grep`, etc.).

## Quick start
```bash
git clone https://github.com/ThalesMMS/Code-Scanner.git
cd Code-Scanner
# Add the project(s) you want to inspect inside input/
```

Run whichever scanner you prefer:
- Rust: `cargo run` (defaults to `./input` and `./output`)
  - LOC summary (repo alias): `cargo loc <path>` (same as `cargo run -- --loc <path>`)
- Bash: `./bash/scan_project.sh`

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

## LOC Mode (Rust)
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
