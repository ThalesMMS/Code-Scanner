# Code Scanner Toolkit

Code Scanner Toolkit bundles a ready-to-run Bash script that walks through software projects, extracts source files and relevant configuration, and produces concise text bundles you can share or audit. The script intentionally ignores hefty build artifacts and secrets, keeping the output focused on the code that matters.

## Repository Layout
- `scripts/scan_project.sh` – main script; run it from the repository root.
- `input/` – drop the projects or folders you want to scan; kept in Git via `.gitkeep`.
- `output/` – generated reports (ignored by Git); each project becomes `<name>_project_code.txt`.

## Requirements
- macOS or Linux with Bash 4+ (default on most systems).
- POSIX utilities already used by the script (`find`, `sed`, `awk`, `stat`, `file`, `tr`, `nl`, `grep`).

## Quick Start
```bash
git clone <this-repo>
cd Code-Scanner
# Add the project(s) you want to review into the input/ directory
./scripts/scan_project.sh
```

On first run the script ensures `input/` and `output/` exist. If `input/` is empty, the script exits after guiding you to place projects inside it.

## Customising the Scan
The script is entirely driven by environment variables, so you can tailor it without editing the source:

| Variable | Purpose | Example |
| --- | --- | --- |
| `TARGET_DIR` | Directory to scan | `TARGET_DIR=./my-samples ./scripts/scan_project.sh` |
| `OUTPUT_DIR` | Where to write reports | `OUTPUT_DIR=./reports ./scripts/scan_project.sh` |
| `OUTPUT_FILE_SUFFIX` | Change filename suffix | `OUTPUT_FILE_SUFFIX=_audit.txt ./scripts/scan_project.sh` |
| `MAX_SIZE_BYTES` | Limit per file (default 2MB) | `MAX_SIZE_BYTES=$((1024*1024)) ./scripts/scan_project.sh` |
| `IGNORE_FILES_EXTRA` | Extra file patterns to skip | `IGNORE_FILES_EXTRA='*.snap|*.bin' ./scripts/scan_project.sh` |
| `IGNORE_DIRS_EXTRA` | Extra directories to skip | `IGNORE_DIRS_EXTRA='docs|examples' ./scripts/scan_project.sh` |
| `IGNORE_PATHS` | Relative paths inside a project | `IGNORE_PATHS='vendor/cache|data/generated' ./scripts/scan_project.sh` |
| `IGNORE_ABSOLUTE_PATHS` | Absolute directories to skip | `IGNORE_ABSOLUTE_PATHS="$PWD/input/big-lib" ./scripts/scan_project.sh` |

Output files for each project contain:
1. A project overview.
2. A tree view of directories and files that were included.
3. Summaries for each included file with size metadata.
4. Full contents of text-based files labelled with line numbers.

## Tips
- Use `IGNORE_FILES_EXTRA` and `IGNORE_DIRS_EXTRA` to keep noisy artefacts out of reports.
- Large binary files are automatically skipped, but they still count toward the per-project summary.
- If you only need a single project, place it directly in `input/` or point `TARGET_DIR` to it.

## Licensing
This project is released under the MIT License; see `LICENSE` for full details.
