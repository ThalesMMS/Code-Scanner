//
// scanner.rs
// Code-Scanner-rs
//
// Implements the scanning pipeline: builds directory walkers, filters files, writes tree structure and contents to reports, and aggregates scan statistics.
//
// Thales Matheus Mendonça Santos - November 2025
//

use crate::cli::Args;
use crate::config::{load_config, ProjectConfig};
use crate::project::detect_project_type;
use crate::utils::{format_size, is_binary};
use anyhow::{Context, Result};
use chrono::Local;
use ignore::{Walk, WalkBuilder};
use pathdiff::diff_paths;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

// Orchestrates a full scan for a single project and writes the report.
pub fn process_project(project_path: &Path, output_dir: &Path, args: &Args) -> Result<()> {
    let project_name = project_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let output_file_path = output_dir.join(format!("{}_project_code.txt", project_name));
    let project_type = detect_project_type(project_path);
    let config = load_config(project_path);

    // Visible progress helps when scanning multiple folders.
    println!("📦 Processing: {} ({})", project_name, project_type);

    let mut output_file = File::create(&output_file_path).with_context(|| {
        format!(
            "Failed to create output file: {}",
            output_file_path.display()
        )
    })?;

    write_header(&mut output_file, &project_name, &project_type)?;
    writeln!(output_file, "\nFOLDER STRUCTURE")?;
    writeln!(
        output_file
    )?;

    // Walk the file system with the configured filters and collect files to dump.
    let walker = build_walker(project_path, args, &config);
    let (mut valid_files, mut stats) =
        collect_files(project_path, &config, args, walker, &mut output_file)?;
    valid_files.sort();

    writeln!(output_file, "\n📄 FILE CONTENTS")?;
    writeln!(
        output_file,
        "═══════════════════════════════════════════════════════════════"
    )?;

    write_file_contents(
        project_path,
        &valid_files,
        &mut output_file,
        &mut stats,
        args,
    )?;
    if args.verbose {
        write_summary(&mut output_file, &stats, valid_files.len())?;
    }

    println!("  ✅ Saved to: {}", output_file_path.display());
    Ok(())
}

fn build_walker(project_path: &Path, args: &Args, config: &ProjectConfig) -> Walk {
    let ignore_dirs = config.ignore_dirs.clone();
    // Build a walker that respects .gitignore unless the user disabled it.
    WalkBuilder::new(project_path)
        .git_ignore(!args.no_gitignore)
        .require_git(false)
        .hidden(false)
        .filter_entry(move |entry| {
            if entry.depth() == 0 {
                return true;
            }
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                return !ignore_dirs.contains(&name);
            }
            true
        })
        .build()
}

// Collect totals for LOC/token estimation without writing any report files.
pub fn collect_loc_stats(project_path: &Path, args: &Args) -> Result<LocStats> {
    let config = load_config(project_path);
    let walker = build_walker(project_path, args, &config);
    let mut stats = LocStats::default();

    for result in walker {
        match result {
            Ok(entry) => {
                let path = entry.path();

                if path == project_path {
                    continue;
                }

                let file_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_lowercase();

                if config.ignore_dirs.contains(&file_name) {
                    continue;
                }

                if path.is_dir() {
                    continue;
                }

                if config.ignore_files.contains(&file_name) {
                    stats.skipped_files += 1;
                    continue;
                }

                if file_name.starts_with('.') {
                    let trimmed = file_name.trim_start_matches('.');
                    let whitelisted = config.code_extensions.contains(&file_name)
                        || (!trimmed.is_empty() && config.code_extensions.contains(trimmed));
                    if !whitelisted {
                        stats.skipped_files += 1;
                        continue;
                    }
                }

                let ext = path
                    .extension()
                    .map(|e| e.to_string_lossy().to_string().to_lowercase())
                    .unwrap_or_default();

                if config.ignore_extensions.contains(&ext) {
                    stats.skipped_files += 1;
                    continue;
                }

                if !ext.is_empty() && !config.code_extensions.contains(&ext) {
                    if !config.code_extensions.contains(&file_name) {
                        stats.skipped_files += 1;
                        continue;
                    }
                }

                let metadata = match path.metadata() {
                    Ok(m) => m,
                    Err(_) => {
                        stats.skipped_files += 1;
                        continue;
                    }
                };
                let size = metadata.len();

                if size > config.max_file_size {
                    if args.verbose {
                        println!("Ignoring {} (excessive size)", path.display());
                    }
                    stats.skipped_files += 1;
                    continue;
                }

                if is_binary(path) {
                    stats.skipped_files += 1;
                    continue;
                }

                match fs::read_to_string(path) {
                    Ok(content) => {
                        let relative_path = diff_paths(path, project_path)
                            .unwrap_or_else(|| path.to_path_buf());
                        let line_count = content.lines().count() as u64;
                        let char_count = content.chars().count() as u64;
                        stats.processed_files += 1;
                        stats.total_lines += line_count;
                        stats.total_chars += char_count;
                        record_top_loc_file(&mut stats, relative_path, line_count, size);
                    }
                    Err(_) => {
                        stats.skipped_files += 1;
                        continue;
                    }
                }
            }
            Err(err) => {
                if args.verbose {
                    eprintln!("Error reading input: {}", err);
                }
            }
        }
    }

    stats.estimated_tokens = estimate_tokens(stats.total_chars);
    Ok(stats)
}

fn estimate_tokens(total_chars: u64) -> u64 {
    (total_chars + 3) / 4
}

fn record_top_loc_file(stats: &mut LocStats, path: PathBuf, lines: u64, size: u64) {
    stats
        .largest_files
        .push(LocFileEntry { path, lines, size });
    stats
        .largest_files
        .sort_by(|a, b| b.lines.cmp(&a.lines));
    if stats.largest_files.len() > 10 {
        stats.largest_files.truncate(10);
    }
}

fn write_header(output_file: &mut File, project_name: &str, _project_type: &str) -> Result<()> {
    // Simple header for the human-friendly report.
    writeln!(output_file)?;
    writeln!(output_file, "{}", project_name)?;
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    writeln!(output_file, "{}", now)?;
    writeln!(output_file)?;
    Ok(())
}

fn collect_files(
    project_path: &Path,
    config: &ProjectConfig,
    args: &Args,
    walker: Walk,
    output_file: &mut File,
) -> Result<(Vec<PathBuf>, ScanStats)> {
    let mut valid_files: Vec<PathBuf> = Vec::new();
    let mut stats = ScanStats::default();

    for result in walker {
        match result {
            // ignore::Walk yields entries that can error; handle them gently.
            Ok(entry) => {
                let path = entry.path();

                if path == project_path {
                    // Skip the root path itself; we only care about its children.
                    continue;
                }

                let relative_path =
                    diff_paths(path, project_path).unwrap_or_else(|| path.to_path_buf());
                let file_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_lowercase();

                if config.ignore_dirs.contains(&file_name) {
                    // Prune entire directories early to avoid unnecessary work.
                    continue;
                }

                if path.is_dir() {
                    // Log directory structure in the output file with indentation.
                    let depth = relative_path.components().count();
                    let indent = "  ".repeat(depth.saturating_sub(1));
                    writeln!(
                        output_file,
                        "{}├── {}/",
                        indent,
                        relative_path.file_name().unwrap().to_string_lossy()
                    )?;
                    continue;
                }

                if config.ignore_files.contains(&file_name) {
                    // Skip noisy files but still count them as skipped for the summary.
                    stats.skipped += 1;
                    continue;
                }

                let ext = path
                    .extension()
                    .map(|e| e.to_string_lossy().to_string().to_lowercase())
                    .unwrap_or_default();

                if config.ignore_extensions.contains(&ext) {
                    // Common binary or heavy files we do not want to dump.
                    stats.skipped += 1;
                    continue;
                }

                // If an extension exists and is not whitelisted, drop it unless the
                // whole filename is explicitly whitelisted (Dockerfile, Makefile, etc).
                if !ext.is_empty() && !config.code_extensions.contains(&ext) {
                    if !config.code_extensions.contains(&file_name) {
                        stats.skipped += 1;
                        continue;
                    }
                }

                let metadata = match path.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                // Enforce max file size to keep output manageable.
                if metadata.len() > config.max_file_size {
                    if args.verbose {
                        println!("Ignoring {} (excessive size)", relative_path.display());
                    }
                    stats.skipped += 1;
                    continue;
                }

                valid_files.push(path.to_path_buf());

                // Record the file in the tree view with indentation to reflect depth.
                let depth = relative_path.components().count();
                let indent = "  ".repeat(depth.saturating_sub(1));
                writeln!(
                    output_file,
                    "{}└── {}",
                    indent,
                    relative_path.file_name().unwrap().to_string_lossy()
                )?;
            }
            Err(err) => {
                if args.verbose {
                    eprintln!("Error reading input: {}", err);
                }
            }
        }
    }

    Ok((valid_files, stats))
}

fn write_file_contents(
    project_path: &Path,
    files: &[PathBuf],
    output_file: &mut File,
    stats: &mut ScanStats,
    args: &Args,
) -> Result<()> {
    for path in files {
        let relative_path = diff_paths(path, project_path).unwrap_or_else(|| path.to_path_buf());
        let relative_str = relative_path.to_string_lossy();
        let size = path
            .metadata()
            .with_context(|| format!("Failed to read metadata of {}", relative_path.display()))?
            .len();

        if args.verbose {
            // Section header for each individual file.
            writeln!(
                output_file,
                "┌─────────────────────────────────────────────────────────────"
            )?;
            writeln!(output_file, "│ 📄 {}", relative_str)?;
            writeln!(output_file, "│ 📊 Size: {}", format_size(size))?;
            writeln!(
                output_file,
                "├─────────────────────────────────────────────────────────────"
            )?;
        } else {
            writeln!(output_file, "📄 {}", relative_str)?;
        }

        // Avoid dumping binary content which would clutter the report.
        if is_binary(path) {
            if args.verbose {
                writeln!(
                    output_file,
                    "│ [Binary file or unsupported encoding - content omitted]"
                )?;
            } else {
                writeln!(
                    output_file,
                    "[Binary file or unsupported encoding - content omitted]"
                )?;
            }
        } else {
            match fs::read_to_string(path) {
                Ok(content) => {
                    if args.verbose {
                        // Include line numbers to make the output easy to reference.
                        for (i, line) in content.lines().enumerate() {
                            writeln!(output_file, "{:>4} │ {}", i + 1, line)?;
                        }
                    } else {
                        for line in content.lines() {
                            writeln!(output_file, "{}", line)?;
                        }
                    }
                }
                Err(_) => {
                    if args.verbose {
                        writeln!(output_file, "│ [Error reading file as UTF-8 text]")?;
                    } else {
                        writeln!(output_file, "[Error reading file as UTF-8 text]")?;
                    }
                }
            }
        }

        if args.verbose {
            writeln!(
                output_file,
                "└─────────────────────────────────────────────────────────────\n"
            )?;
        } else {
            writeln!(output_file)?;
        }
        stats.total_size += size;
    }

    Ok(())
}

fn write_summary(output_file: &mut File, stats: &ScanStats, processed_count: usize) -> Result<()> {
    // Final footer with a lightweight count of what happened.
    writeln!(
        output_file,
        "\n═══════════════════════════════════════════════════════════════"
    )?;
    writeln!(output_file, "📊 SUMMARY")?;
    writeln!(
        output_file,
        "  ✅ Files processed: {}",
        processed_count
    )?;
    writeln!(
        output_file,
        "  ⏭️  Files skipped (estimated): {}",
        stats.skipped
    )?;
    writeln!(
        output_file,
        "  💾 Total content size: {}",
        format_size(stats.total_size)
    )?;
    writeln!(
        output_file,
        "═══════════════════════════════════════════════════════════════"
    )?;
    Ok(())
}

#[derive(Default)]
// Lightweight counters gathered during a scan.
struct ScanStats {
    total_size: u64,
    skipped: u64,
}

#[derive(Default)]
pub struct LocStats {
    pub processed_files: u64,
    pub skipped_files: u64,
    pub total_lines: u64,
    pub total_chars: u64,
    pub estimated_tokens: u64,
    pub largest_files: Vec<LocFileEntry>,
}

pub struct LocFileEntry {
    pub path: PathBuf,
    pub lines: u64,
    pub size: u64,
}
