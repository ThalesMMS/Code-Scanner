#!/usr/bin/env python3
"""
Project Summary Generator for Build Projects

Scans build/package projects with binary detection and specific target directories.
Useful for analyzing build artifacts, packages, and deployment structures.
"""

import os
import sys
from pathlib import Path

# --- Configuration ---
SCRIPT_DIR = Path(__file__).parent.resolve()
REPO_ROOT = SCRIPT_DIR.parent
DEFAULT_INPUT_DIR = REPO_ROOT / "input"
DEFAULT_OUTPUT_DIR = REPO_ROOT / "output"

# Default target subdirectory to scan deeply
DEFAULT_TARGET_SUBDIR = "package"

# File extensions to include content from
CODE_EXTENSIONS = {
    '.json',                       # Data/configuration files (except ignored ones)
    '.md',                         # Markdown
    '.config.js',                  # JS configuration files
    '.yaml', '.yml',               # Configuration
    '.sh', '.bash',                # Shell scripts
    '.cfg',                        # Configuration files
}

# Specific root files to include content from (if not in ignore list)
INCLUDE_ROOT_FILES_BY_NAME = {
    'eslint.config.js', 'vite.config.js', 'index.html',
    'package.json', 'README.md',
}

# Files whose CONTENT will be ignored
IGNORE_CONTENT_FILES = {
    'package-lock.json',
    '.gitignore',
}

# Extensions whose CONTENT will be ignored
IGNORE_CONTENT_EXTENSIONS = {
    '.svg', '.png', '.jpg', '.jpeg', '.gif', '.webp', '.ico',  # Common images
    '.woff', '.woff2', '.ttf', '.otf',  # Fonts
    '.wasm', '.so',  # Binary files
}

# List of extensions considered binary for checking files without extension
BINARY_EXTENSIONS = {
    '.exe', '.dll', '.so', '.dylib', '.bin', '.dat', '.obj', '.o', '.a', '.lib',
    '.pyc', '.pyd', '.pyo', '.wasm', '.class', '.jar',
    '.png', '.jpg', '.jpeg', '.gif', '.webp', '.bmp', '.ico', '.tiff',
    '.mp3', '.mp4', '.avi', '.mov', '.mkv', '.flv', '.wav', '.wma',
    '.zip', '.tar', '.gz', '.7z', '.rar', '.bz2',
    '.ttf', '.woff', '.woff2', '.eot', '.otf',
    '.db', '.sqlite', '.mdb'
}

INDENT_STRING = "  "

# Directories to ignore at root level
IGNORE_DIRS_ROOT = {
    'node_modules', 'dist', 'build', '.git', '.vscode', 
    '__pycache__', 'input', 'output'
}
# --- End Configuration ---


def is_binary_file(file_path):
    """Check if a file is binary or text."""
    # Check extension first
    file_ext = os.path.splitext(file_path)[1].lower()
    if file_ext in BINARY_EXTENSIONS:
        return True
    
    # If no extension or not a known binary extension,
    # try reading first bytes to detect if it's binary
    try:
        with open(file_path, 'rb') as f:
            # Read first 1024 bytes
            chunk = f.read(1024)
            # Check for null bytes or control characters uncommon in text files
            return b'\x00' in chunk or sum(1 for b in chunk if b < 9 and b != 0) > 0
    except Exception:
        # In case of error, assume binary to avoid issues
        return True


def should_include_content(file_path, root_dir, code_extensions, include_root_files, 
                          ignore_content_files, ignore_content_extensions):
    """Check if file content should be included in output."""
    relative_path = os.path.relpath(file_path, root_dir)
    filename = os.path.basename(file_path)
    file_ext = os.path.splitext(filename)[1].lower()
    is_in_root = os.path.dirname(relative_path) == ''

    # 1. DO NOT include if NAME or EXTENSION are in ignore lists
    if filename in ignore_content_files:
        return False
    if file_ext in ignore_content_extensions:
        return False
        
    # 1.5. DO NOT include if file is binary (no extension or unknown extension)
    if file_ext == '' or file_ext not in code_extensions:
        if is_binary_file(file_path):
            return False

    # 2. Include if it's a specifically named file in root (and not ignored above)
    if is_in_root and filename in include_root_files:
        return True

    # 3. Include if extension is in allowed list (and not ignored above)
    # Handle double extensions like .config.js
    effective_ext = file_ext
    if file_ext == '.js':
        double_ext_check = os.path.splitext(os.path.splitext(filename)[0])[1].lower() + file_ext
        if double_ext_check in code_extensions:
            effective_ext = double_ext_check

    if effective_ext in code_extensions:
        return True
        
    # 4. Include files without extension if NOT binary (already checked above)
    if file_ext == '':
        return True

    return False


def write_project_structure(start_path, root_dir, target_subdir, output_file, 
                           code_files_list, indent_level=0, process_children=False):
    """Recursively write structure and collect code files."""
    try:
        listed_items = sorted(os.listdir(start_path))
    except OSError as e:
        print(f"Warning: Could not read directory {start_path}: {e}", file=sys.stderr)
        return

    is_root_level = (start_path == root_dir)
    items_to_process = []

    # Filter ignored items BEFORE processing
    for item in listed_items:
        item_path = os.path.join(start_path, item)
        is_dir = os.path.isdir(item_path)

        # Ignore directories defined in IGNORE_DIRS_ROOT at root level
        if is_root_level and is_dir and item in IGNORE_DIRS_ROOT:
            continue

        items_to_process.append(item)

    # Process remaining items
    for item in items_to_process:
        item_path = os.path.join(start_path, item)
        prefix = INDENT_STRING * indent_level
        is_dir = os.path.isdir(item_path)

        if is_dir:
            # If at root AND this is the target directory (e.g., 'package')
            if is_root_level and item == target_subdir:
                output_file.write(f"{prefix}- {item}/\n")
                write_project_structure(item_path, root_dir, target_subdir, output_file, 
                                      code_files_list, indent_level + 1, process_children=True)
            # If at root, but NOT the target directory
            elif is_root_level:
                output_file.write(f"{prefix}- {item}/ [...ignored]\n")
            # If already processing children (inside target directory)
            elif process_children:
                output_file.write(f"{prefix}- {item}/\n")
                write_project_structure(item_path, root_dir, target_subdir, output_file, 
                                      code_files_list, indent_level + 1, process_children=True)
        else:  # It's a file
            output_file.write(f"{prefix}- {item}\n")
            # If processing children (inside target directory) OR file is at root
            if process_children or is_root_level:
                # Check if this file's content should be included
                if should_include_content(item_path, root_dir, CODE_EXTENSIONS, 
                                        INCLUDE_ROOT_FILES_BY_NAME, IGNORE_CONTENT_FILES, 
                                        IGNORE_CONTENT_EXTENSIONS):
                    code_files_list.append(item_path)


def process_project(project_dir, output_filename, target_subdir=None):
    """Process a single project directory."""
    if target_subdir is None:
        target_subdir = DEFAULT_TARGET_SUBDIR

    project_name = os.path.basename(project_dir)
    code_files_to_read = []

    print(f"Processing: {project_name}")
    print(f"  Output file: {output_filename}")
    print(f"  Target subdirectory for deep analysis: {target_subdir}")
    print(f"  Extensions with content included: {', '.join(CODE_EXTENSIONS)}")
    print(f"  Root files with content included (by name): {', '.join(INCLUDE_ROOT_FILES_BY_NAME)}")
    print(f"  Files with content ignored (by name): {', '.join(IGNORE_CONTENT_FILES)}")
    print(f"  Extensions with content ignored: {', '.join(IGNORE_CONTENT_EXTENSIONS)}")
    print(f"  Directories ignored at root: {', '.join(IGNORE_DIRS_ROOT)}")
    print(f"  Files without extension will be checked for binary nature")

    try:
        with open(output_filename, 'w', encoding='utf-8') as outfile:
            # --- Part 1: File Structure ---
            outfile.write("=" * 30 + "\n")
            outfile.write(" Project Structure\n")
            outfile.write("=" * 30 + "\n")
            outfile.write(f"(Content from subfolders except '{target_subdir}' was ignored)\n")
            outfile.write(f"(Content from files like {', '.join(IGNORE_CONTENT_FILES)} ")
            outfile.write(f"and extensions {', '.join(IGNORE_CONTENT_EXTENSIONS)} ignored)\n")
            outfile.write(f"(Binary files are ignored)\n\n")
            outfile.write(f"- {project_name}/ (root)\n")

            write_project_structure(project_dir, project_dir, target_subdir, outfile, 
                                  code_files_to_read, indent_level=1, process_children=False)

            # --- Part 2: File Contents ---
            outfile.write("\n\n" + "="*30 + "\n")
            outfile.write(" Relevant File Contents\n")
            outfile.write("="*30 + "\n\n")

            code_files_to_read.sort()

            for file_path in code_files_to_read:
                relative_path = os.path.relpath(file_path, project_dir).replace('\\', '/')
                try:
                    if is_binary_file(file_path):
                        outfile.write(f"--- File: {relative_path} ---\n\n")
                        outfile.write("*** BINARY FILE - CONTENT NOT DISPLAYED ***\n")
                        outfile.write("\n\n" + "="*15 + f" End of {relative_path} " + "="*15 + "\n\n")
                        continue
                        
                    with open(file_path, 'r', encoding='utf-8', errors='ignore') as infile:
                        content = infile.read()
                        # Optional: Skip files that are empty after reading
                        if not content.strip() and os.path.getsize(file_path) == 0:
                            continue

                        outfile.write(f"--- File: {relative_path} ---\n\n")
                        outfile.write(content)
                        # Add clear separator between files
                        outfile.write("\n\n" + "="*15 + f" End of {relative_path} " + "="*15 + "\n\n")

                except Exception as e:
                    outfile.write(f"--- File: {relative_path} ---\n\n")
                    outfile.write(f"*** Error reading file: {e} ***\n")
                    outfile.write("\n\n" + "="*15 + f" End of {relative_path} (with error) " + "="*15 + "\n\n")

        print(f"  ✓ Successfully generated '{output_filename}'!")
        return True

    except IOError as e:
        print(f"\n  ✗ I/O error writing to file {output_filename}: {e}", file=sys.stderr)
        return False
    except Exception as e:
        print(f"\n  ✗ Unexpected error occurred: {e}", file=sys.stderr)
        return False


def main():
    """Main function to process all projects in input directory."""
    # Get directories from environment or use defaults
    input_dir = Path(os.environ.get('INPUT_DIR', DEFAULT_INPUT_DIR))
    output_dir = Path(os.environ.get('OUTPUT_DIR', DEFAULT_OUTPUT_DIR))
    
    # Get target subdirectory from environment or use default
    target_subdir = os.environ.get('TARGET_SUBDIR', DEFAULT_TARGET_SUBDIR)

    print("=" * 60)
    print("PROJECT SUMMARY GENERATOR - BUILD PROJECTS")
    print("=" * 60)
    print(f"Input directory: {input_dir}")
    print(f"Output directory: {output_dir}")
    print(f"Target subdirectory: {target_subdir}")
    print("=" * 60)
    print()

    # Ensure directories exist
    if not input_dir.exists():
        print(f"Error: Input directory not found: {input_dir}", file=sys.stderr)
        print("Please create it and add projects to scan.", file=sys.stderr)
        return 1

    output_dir.mkdir(parents=True, exist_ok=True)

    # Find all project directories in input
    projects = [d for d in input_dir.iterdir() if d.is_dir()]
    
    if not projects:
        print(f"Warning: No project directories found in {input_dir}", file=sys.stderr)
        print("Please add project directories to scan.", file=sys.stderr)
        return 1

    success_count = 0
    for project_path in sorted(projects):
        project_name = project_path.name
        output_filename = output_dir / f"{project_name}_build_summary.txt"
        
        print(f"\n[Project: {project_name}]")
        if process_project(str(project_path), str(output_filename), target_subdir):
            success_count += 1
        print()

    print("=" * 60)
    print(f"COMPLETED! Processed {success_count}/{len(projects)} projects")
    print(f"Output files in: {output_dir}")
    print("=" * 60)
    
    return 0 if success_count > 0 else 1


if __name__ == "__main__":
    sys.exit(main())
