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
cargo run -- --loc ../my-project
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

Example LOC output from this repository (`cargo run -- --loc .`):
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
- `.scanner-config.example.json` – Example Rust CLI project config for `.scanner-config.json`.

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

The two scanners are configured differently.

### Rust CLI: `.scanner-config.json`

Place an optional `.scanner-config.json` file in the **target project root** to override the Rust CLI defaults.

Supported keys:
- `code_extensions`: file extensions or exact filenames to include
- `ignore_dirs`: directory names to skip
- `ignore_files`: exact filenames to skip
- `ignore_extensions`: file extensions to skip
- `max_file_size`: maximum file size in bytes

Important details verified in the current code:
- Write extensions **without a leading dot** (`"rs"`, `"png"`, not `".rs"`, `".png"`).
- `ignore_files` matches exact filenames such as `"package-lock.json"`.
- The Rust CLI looks only for `.scanner-config.json`; the example file is a template you copy and adapt.

Example:

```json
{
  "code_extensions": ["rs", "toml", "md", "sh"],
  "ignore_dirs": ["vendor", "generated"],
  "ignore_files": ["pnpm-lock.yaml", ".DS_Store"],
  "ignore_extensions": ["png", "jpg", "pdf"],
  "max_file_size": 1048576
}
```

Useful Rust CLI flags:
- `cargo run -- --no-gitignore --verbose`
- `cargo run -- --input-dir ./my-project --output-dir ./reports`
- `cargo run -- --input-dir ./my-project --output-dir ./reports --ignore ts js json` excludes matching file extensions from scanning
- LOC-only summary (no report files): `cargo run -- --loc ./path/to/project`

### Bash scanner: environment variables

The Bash scanner does **not** read `.scanner-config.json`. Configure it with environment variables instead.

Common examples:
- `USE_GITIGNORE=false ./bash/scan_project.sh`
- `TARGET_DIR=./my-project OUTPUT_DIR=./reports ./bash/scan_project.sh`
- `IGNORE_PATHS="src/vendor|tests/fixtures" VERBOSE=true ./bash/scan_project.sh`

Available Bash environment variables include:
- `TARGET_DIR`, `OUTPUT_DIR`, `OUTPUT_FILE_SUFFIX`
- `MAX_SIZE_BYTES`, `USE_GITIGNORE`, `VERBOSE`
- `IGNORE_FILES_EXTRA`, `IGNORE_DIRS_EXTRA`
- `IGNORE_PATHS`, `IGNORE_ABSOLUTE_PATHS`

## Output
Each project yields a text report in `output/`, typically named `<project>_project_code.txt` or `<project>_*_summary.txt` depending on the scanner. Large binaries, dependency folders, IDE files, and `.gitignore`d paths are skipped by default.

Verbose mode (`--verbose` on the Rust CLI, `VERBOSE=true` in Bash) adds file headers with size, line numbers, and a final summary block. Without verbose, the report lists file paths followed by raw file contents.

## LOC mode (Rust)
Use LOC mode when you want a quick size summary without generating report files:

```bash
cargo run -- --loc ./path/to/project
```

Output includes total lines, total characters, a token estimate (chars/4), and the top 10 files by line count. Dotfiles are excluded from LOC counts unless you whitelist them by adding the filename (e.g., `.gitignore` or `gitignore`) to `code_extensions` in `.scanner-config.json`.

## Development (Rust)
- Format: `cargo fmt`
- Lint: `cargo clippy -- -D warnings`
- Tests: `cargo test`
- Coverage (install once with `cargo install cargo-llvm-cov`): `cargo coverage` (fails under 35% line coverage)
