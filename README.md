# Code Scanner Toolkit

Code Scanner Toolkit provides ready-to-run scripts (Bash and Python) that walk through software projects, extract source files and relevant configuration, and produce concise text bundles you can share or audit. The scripts intentionally ignore hefty build artifacts and secrets, keeping the output focused on the code that matters.

## Repository Layout
- `scripts/scan_project.sh` – main Bash scanner for comprehensive project analysis
- `scripts/generate_project_summary.py` – Python scanner optimized for web projects (JavaScript/TypeScript/React)
- `scripts/generate_project_summary_build.py` – Python scanner for build/package projects with binary detection
- `scripts/generate_project_summary_django.py` – Python scanner for Django/Python backend projects
- `input/` – drop the projects or folders you want to scan; kept in Git via `.gitkeep`
- `output/` – generated reports (ignored by Git); each project becomes `<name>_project_code.txt` or `<name>_*_summary.txt`

## Requirements
### For Bash Scanner
- macOS or Linux with Bash 4+ (default on most systems)
- POSIX utilities already used by the script (`find`, `sed`, `awk`, `stat`, `file`, `tr`, `nl`, `grep`)

### For Python Scanners
- Python 3.6+ (no external dependencies required)

## Quick Start

### Using the Bash Scanner (Comprehensive)
```bash
git clone <this-repo>
cd Code-Scanner
# Add the project(s) you want to review into the input/ directory
./scripts/scan_project.sh
```

### Using Python Scanners (Specialized)
```bash
# For web projects (JavaScript/TypeScript/React)
python3 scripts/generate_project_summary.py

# For build/package projects
python3 scripts/generate_project_summary_build.py

# For Django/Python projects
python3 scripts/generate_project_summary_django.py
```

On first run, the scripts ensure `input/` and `output/` exist. If `input/` is empty, the scripts will notify you to add projects before running again.

## Customising the Scan

### Bash Scanner (`scan_project.sh`)
The Bash scanner is entirely driven by environment variables, so you can tailor it without editing the source:

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

### Python Scanners
Python scanners support these environment variables:

| Variable | Purpose | Example |
| --- | --- | --- |
| `INPUT_DIR` | Directory to scan | `INPUT_DIR=./my-samples python3 scripts/generate_project_summary.py` |
| `OUTPUT_DIR` | Where to write reports | `OUTPUT_DIR=./reports python3 scripts/generate_project_summary.py` |
| `TARGET_SUBDIRS` | Subdirs to scan deeply (web only) | `TARGET_SUBDIRS=src,docs,lib python3 scripts/generate_project_summary.py` |
| `TARGET_SUBDIR` | Subdir to scan (build/django) | `TARGET_SUBDIR=backend python3 scripts/generate_project_summary_django.py` |

### Scanner Types and Use Cases

**`scan_project.sh` (Bash)** - Most comprehensive scanner
- Supports all major programming languages and frameworks
- Includes extensive configuration options
- Binary file detection
- Best for: General-purpose project analysis

**`generate_project_summary.py` (Python)** - Web Projects
- Optimized for JavaScript/TypeScript/React projects
- Scans multiple target directories (default: `src`, `docs`)
- Includes web-specific file types (JSX, TSX, SCSS, etc.)
- Best for: Frontend and full-stack web applications

**`generate_project_summary_build.py` (Python)** - Build Projects
- Binary file detection and handling
- Single target directory focus (default: `package`)
- Configuration and script files
- Best for: Build outputs, deployment packages, QPKG packages

**`generate_project_summary_django.py` (Python)** - Django/Python
- Optimized for Django/Python backend projects
- Handles Python-specific files (Pipfile, requirements.txt)
- Filters `__pycache__` and `.pyc` files
- Single target directory (default: `back`)
- Best for: Django applications and Python backends

### Output Format
Output files for each project contain:
1. A project overview
2. A tree view of directories and files that were included
3. Summaries for each included file with size metadata
4. Full contents of text-based files (with line numbers for Bash scanner)

## Tips
- **Choose the right scanner**: Use specialized Python scanners for faster, more focused results on specific project types
- Use `IGNORE_FILES_EXTRA` and `IGNORE_DIRS_EXTRA` (Bash) to keep noisy artefacts out of reports
- Large binary files are automatically skipped by all scanners
- If you only need a single project, place it directly in `input/` or point `TARGET_DIR`/`INPUT_DIR` to it
- Python scanners generate output files with type suffixes: `*_web_summary.txt`, `*_build_summary.txt`, `*_django_summary.txt`
- All scanners automatically create `input/` and `output/` directories if they don't exist

## Licensing
This project is released under the MIT License; see `LICENSE` for full details.
