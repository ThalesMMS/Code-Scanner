#!/usr/bin/env python3
"""
Project Summary Generator for Web Projects

Scans web projects (JavaScript/TypeScript/React) from the input directory
and generates comprehensive text summaries in the output directory.
Focuses on source code and documentation folders.
"""

import os
import sys
from pathlib import Path

# --- Configuration ---
SCRIPT_DIR = Path(__file__).parent.resolve()
REPO_ROOT = SCRIPT_DIR.parent
DEFAULT_INPUT_DIR = REPO_ROOT / "input"
DEFAULT_OUTPUT_DIR = REPO_ROOT / "output"

# Default target subdirectories to scan deeply
DEFAULT_TARGET_SUBDIRS = {'src', 'docs'}

# File extensions to include content from
CODE_EXTENSIONS = {
    '.js', '.jsx', '.ts', '.tsx',  # JavaScript/TypeScript & Frameworks (React, etc.)
    '.html', '.css', '.scss',      # Web frontend basics
    '.json',                       # Data/configuration files (except ignored ones below)
    '.md',                         # Markdown
    '.config.js',                  # JS configuration files
    '.yaml', '.yml',               # Configuration
    '.sh', '.bash',                # Shell scripts
    '.mjs',                        # ES Modules
    '.puml',                       # PlantUML
    '.mermaid',                    # Mermaid diagrams
}

# Specific root files to include content from (if not in ignore list)
INCLUDE_ROOT_FILES_BY_NAME = {
     'eslint.config.js', 'vite.config.js', 'index.html',
     'package.json', 'README.md', 'citation.cff',
     'nest-cli.json', 'tsconfig.json', 'tsconfig.build.json',
}

# Files whose CONTENT will be ignored, even if extension/location matches
IGNORE_CONTENT_FILES = {
    'package-lock.json',
    '.gitignore',
}

# Extensions whose CONTENT will be ignored (useful for assets like SVG, images, etc.)
IGNORE_CONTENT_EXTENSIONS = {
    '.svg', '.png', '.jpg', '.jpeg', '.gif', '.webp', '.ico',  # Common images
    '.woff', '.woff2', '.ttf', '.otf',  # Fonts
}

INDENT_STRING = "  "

# Directories to ignore at root level
IGNORE_DIRS_ROOT = {
    'node_modules', 'dist', 'build', '.git', '.vscode', 
    '__pycache__', '.idea', 'input', 'output'
}
# --- End Configuration ---


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

    # 2. Include if it's a specifically named file in root (and not ignored above)
    if is_in_root and filename in include_root_files:
        return True

    # 3. Include if extension is in allowed list (and not ignored above)
    # Handle double extensions like .config.js
    effective_ext = file_ext
    if file_ext in {'.js', '.mjs'}:
        double_ext_check = os.path.splitext(os.path.splitext(filename)[0])[1].lower() + file_ext
        if double_ext_check in code_extensions:
            effective_ext = double_ext_check

    if effective_ext in code_extensions:
        return True

    return False


def write_project_structure(start_path, root_dir, target_subdirs, output_file, 
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
            # If at root AND this is one of target directories (e.g., 'src' or 'docs')
            if is_root_level and item in target_subdirs:
                output_file.write(f"{prefix}- {item}/\n")
                write_project_structure(item_path, root_dir, target_subdirs, output_file, 
                                      code_files_list, indent_level + 1, process_children=True)
            # If at root, but NOT a target directory
            elif is_root_level:
                output_file.write(f"{prefix}- {item}/ [...ignored]\n")
            # If already processing children (inside 'src' or 'docs')
            elif process_children:
                output_file.write(f"{prefix}- {item}/\n")
                write_project_structure(item_path, root_dir, target_subdirs, output_file, 
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


def process_project(project_dir, output_filename, target_subdirs=None):
    """Process a single project directory."""
    if target_subdirs is None:
        target_subdirs = DEFAULT_TARGET_SUBDIRS

    project_name = os.path.basename(project_dir)
    code_files_to_read = []

    print(f"Processing: {project_name}")
    print(f"  Output file: {output_filename}")
    print(f"  Target subdirectories for deep analysis: {', '.join(target_subdirs)}")
    print(f"  Extensions with content included: {', '.join(CODE_EXTENSIONS)}")
    print(f"  Root files with content included (by name): {', '.join(INCLUDE_ROOT_FILES_BY_NAME)}")
    print(f"  Files with content ignored (by name): {', '.join(IGNORE_CONTENT_FILES)}")
    print(f"  Extensions with content ignored: {', '.join(IGNORE_CONTENT_EXTENSIONS)}")
    print(f"  Directories ignored at root: {', '.join(IGNORE_DIRS_ROOT)}")

    try:
        with open(output_filename, 'w', encoding='utf-8') as outfile:
            # --- Part 1: File Structure ---
            outfile.write("=" * 30 + "\n")
            outfile.write(" Project Structure\n")
            outfile.write("=" * 30 + "\n")
            outfile.write(f"(Content from subfolders except {', '.join(target_subdirs)} was ignored)\n")
            outfile.write(f"(Content from files like {', '.join(IGNORE_CONTENT_FILES)} ")
            outfile.write(f"and extensions {', '.join(IGNORE_CONTENT_EXTENSIONS)} ignored)\n\n")
            outfile.write(f"- {project_name}/ (root)\n")

            write_project_structure(project_dir, project_dir, target_subdirs, outfile, 
                                  code_files_to_read, indent_level=1, process_children=False)

            # --- Part 2: File Contents ---
            outfile.write("\n\n" + "="*30 + "\n")
            outfile.write(" Relevant File Contents\n")
            outfile.write("="*30 + "\n\n")

            code_files_to_read.sort()

            for file_path in code_files_to_read:
                relative_path = os.path.relpath(file_path, project_dir).replace('\\', '/')
                try:
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
    
    # Get target subdirectories from environment or use defaults
    target_subdirs_env = os.environ.get('TARGET_SUBDIRS', '')
    if target_subdirs_env:
        target_subdirs = set(target_subdirs_env.split(','))
    else:
        target_subdirs = DEFAULT_TARGET_SUBDIRS

    print("=" * 60)
    print("PROJECT SUMMARY GENERATOR - WEB PROJECTS")
    print("=" * 60)
    print(f"Input directory: {input_dir}")
    print(f"Output directory: {output_dir}")
    print(f"Target subdirectories: {', '.join(target_subdirs)}")
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
        output_filename = output_dir / f"{project_name}_web_summary.txt"
        
        print(f"\n[Project: {project_name}]")
        if process_project(str(project_path), str(output_filename), target_subdirs):
            success_count += 1
        print()

    print("=" * 60)
    print(f"COMPLETED! Processed {success_count}/{len(projects)} projects")
    print(f"Output files in: {output_dir}")
    print("=" * 60)
    
    return 0 if success_count > 0 else 1


if __name__ == "__main__":
    sys.exit(main())
