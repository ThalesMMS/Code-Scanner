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
use crate::i18n;
use crate::project::detect_project_type;
use crate::utils::{format_size, is_binary};
use crate::t;
use anyhow::{Context, Result};
use ignore::{Walk, WalkBuilder};
use pathdiff::diff_paths;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

struct OutputReport {
    output_dir: PathBuf,
    project_name: String,
    max_lines: Option<u64>,
    part: u32,
    file: File,
    line_count: u64,
    paths: Vec<PathBuf>,
}

impl OutputReport {
    fn create(
        output_dir: &Path,
        project_name: &str,
        project_type: &str,
        max_lines: Option<u64>,
    ) -> Result<Self> {
        let path = Self::path_for_part(output_dir, project_name, 1);
        let file = File::create(&path).with_context(|| {
            t!(
                "cli-error-create-report",
                "path" => path.display().to_string()
            )
        })?;
        let mut report = Self {
            output_dir: output_dir.to_path_buf(),
            project_name: project_name.to_owned(),
            max_lines,
            part: 1,
            file,
            line_count: 0,
            paths: vec![path],
        };
        report.write_initial_header(project_type)?;
        Ok(report)
    }

    fn path_for_part(output_dir: &Path, project_name: &str, part: u32) -> PathBuf {
        if part == 1 {
            output_dir.join(format!("{project_name}_project_code.txt"))
        } else {
            output_dir.join(format!("{project_name}_project_code_{part}.txt"))
        }
    }

    fn write_initial_header(&mut self, _project_type: &str) -> Result<()> {
        let project_name = self.project_name.clone();
        self.write_blank_line()?;
        self.write_line(&project_name)?;
        self.write_line(&i18n::format_timestamp(chrono::Local::now()))?;
        self.write_blank_line()?;
        self.write_line(&t!("report-folder-structure"))?;
        self.write_blank_line()?;
        Ok(())
    }

    fn write_continuation_header(&mut self) -> Result<()> {
        let project_name = self.project_name.clone();
        let part = self.part;
        self.write_blank_line()?;
        self.write_line(&project_name)?;
        self.write_line(&i18n::format_timestamp(chrono::Local::now()))?;
        self.write_line(&t!(
            "report-continued-part",
            "part" => i18n::format_number(part as i64)
        ))?;
        self.write_blank_line()?;
        self.write_file_contents_header()?;
        Ok(())
    }

    fn write_file_contents_header(&mut self) -> Result<()> {
        self.write_line(&t!("report-file-contents"))?;
        self.write_line(&t!("report-contents-rule"))?;
        Ok(())
    }

    fn reset_split_line_count(&mut self) {
        self.line_count = 0;
    }

    fn rotate_if_needed(&mut self) -> Result<()> {
        let Some(max) = self.max_lines else {
            return Ok(());
        };
        if self.line_count < max {
            return Ok(());
        }

        self.part += 1;
        let path = Self::path_for_part(&self.output_dir, &self.project_name, self.part);
        self.file = File::create(&path).with_context(|| {
            t!(
                "cli-error-create-report",
                "path" => path.display().to_string()
            )
        })?;
        self.paths.push(path);
        self.line_count = 0;
        self.write_continuation_header()?;
        Ok(())
    }

    fn write_line(&mut self, line: &str) -> Result<()> {
        writeln!(self.file, "{line}")?;
        self.line_count += 1;
        Ok(())
    }

    fn write_blank_line(&mut self) -> Result<()> {
        writeln!(self.file)?;
        self.line_count += 1;
        Ok(())
    }

    fn write_formatted(&mut self, args: std::fmt::Arguments<'_>) -> Result<()> {
        writeln!(self.file, "{args}")?;
        self.line_count += 1;
        Ok(())
    }

    fn print_saved_paths(&self) {
        for path in &self.paths {
            println!(
                "{}",
                t!("cli-saved-to", "path" => path.display().to_string())
            );
        }
    }
}

macro_rules! report_write {
    ($report:expr, $($arg:tt)*) => {
        $report.write_formatted(format_args!($($arg)*))
    };
}

// Orchestrates a full scan for a single project and writes the report.
pub fn process_project(
    project_path: &Path,
    output_dir: &Path,
    args: &Args,
) -> Result<ProcessOutcome> {
    let project_name = project_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let project_type = detect_project_type(project_path);
    let mut config = load_config(project_path);

    // Apply CLI ignore arguments
    if !args.ignore.is_empty() {
        config.ignore_extensions(&args.ignore);
    }

    // Visible progress helps when scanning multiple folders.
    println!(
        "{}",
        t!(
            "cli-processing",
            "project" => project_name.as_str(),
            "kind" => project_type.as_str()
        )
    );

    let mut report = OutputReport::create(
        output_dir,
        &project_name,
        &project_type,
        args.max_output_lines,
    )?;

    // Walk the file system with the configured filters and collect files to dump.
    let walker = build_walker(project_path, args, &config);
    let (mut valid_files, mut stats) =
        collect_files(project_path, &config, args, walker, &mut report)?;
    valid_files.sort();

    report.write_blank_line()?;
    report.write_file_contents_header()?;
    report.reset_split_line_count();

    write_file_contents(project_path, &valid_files, &mut report, &mut stats, args)?;
    if args.verbose {
        write_summary(&mut report, &stats, valid_files.len())?;
    }

    report.print_saved_paths();

    Ok(ProcessOutcome {
        project_name,
        project_type,
        output_paths: report.paths.clone(),
        processed_files: valid_files.len(),
        skipped_files: stats.skipped,
        total_size: stats.total_size,
    })
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
    let mut config = load_config(project_path);

    // Apply CLI ignore arguments
    if !args.ignore.is_empty() {
        config.ignore_extensions(&args.ignore);
    }

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
                        println!(
                            "{}",
                            t!(
                                "cli-ignoring-excessive-size",
                                "path" => path.display().to_string()
                            )
                        );
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
                        let relative_path =
                            diff_paths(path, project_path).unwrap_or_else(|| path.to_path_buf());
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
                    eprintln!(
                        "{}",
                        t!(
                            "cli-error-reading-input",
                            "error" => err.to_string()
                        )
                    );
                }
            }
        }
    }

    stats.estimated_tokens = estimate_tokens(stats.total_chars);
    Ok(stats)
}

// Collect structured project analytics for the desktop UI without writing report files.
pub fn analyze_project(project_path: &Path, args: &Args) -> Result<ProjectAnalysis> {
    let project_name = project_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let project_type = detect_project_type(project_path);
    let config_present = project_path.join(".scanner-config.json").exists();
    let mut config = load_config(project_path);

    if !args.ignore.is_empty() {
        config.ignore_extensions(&args.ignore);
    }

    let mut analysis = ProjectAnalysis {
        project_name,
        project_type,
        root_path: project_path.to_path_buf(),
        config: AnalysisConfigState {
            config_present,
            gitignore_enabled: !args.no_gitignore,
            max_file_size: config.max_file_size,
        },
        ..ProjectAnalysis::default()
    };
    let mut extension_stats: BTreeMap<String, ExtensionStats> = BTreeMap::new();
    let mut skipped_reasons: BTreeMap<String, u64> = BTreeMap::new();
    let walker = build_walker(project_path, args, &config);

    for result in walker {
        match result {
            Ok(entry) => {
                let path = entry.path();
                if path == project_path {
                    continue;
                }

                let relative_path =
                    diff_paths(path, project_path).unwrap_or_else(|| path.to_path_buf());
                let depth = relative_path.components().count() as u64;
                analysis.tree.max_depth = analysis.tree.max_depth.max(depth);

                let file_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_lowercase();

                if config.ignore_dirs.contains(&file_name) {
                    continue;
                }

                if path.is_dir() {
                    analysis.tree.directories += 1;
                    continue;
                }

                analysis.tree.files += 1;

                if config.ignore_files.contains(&file_name) {
                    record_skipped_reason(&mut skipped_reasons, "skip-reason-ignored-file");
                    continue;
                }

                if file_name.starts_with('.') {
                    let trimmed = file_name.trim_start_matches('.');
                    let whitelisted = config.code_extensions.contains(&file_name)
                        || (!trimmed.is_empty() && config.code_extensions.contains(trimmed));
                    if !whitelisted {
                        record_skipped_reason(&mut skipped_reasons, "skip-reason-hidden-file");
                        continue;
                    }
                }

                let ext = path
                    .extension()
                    .map(|e| e.to_string_lossy().to_string().to_lowercase())
                    .unwrap_or_default();

                if config.ignore_extensions.contains(&ext) {
                    record_skipped_reason(&mut skipped_reasons, "skip-reason-ignored-extension");
                    continue;
                }

                if !ext.is_empty() && !config.code_extensions.contains(&ext) {
                    if !config.code_extensions.contains(&file_name) {
                        record_skipped_reason(&mut skipped_reasons, "skip-reason-unsupported-extension");
                        continue;
                    }
                }

                let metadata = match path.metadata() {
                    Ok(m) => m,
                    Err(_) => {
                        record_skipped_reason(&mut skipped_reasons, "skip-reason-metadata-error");
                        continue;
                    }
                };
                let size = metadata.len();

                if size > config.max_file_size {
                    record_skipped_reason(&mut skipped_reasons, "skip-reason-over-max-file-size");
                    continue;
                }

                if is_binary(path) {
                    record_skipped_reason(&mut skipped_reasons, "skip-reason-binary-file");
                    continue;
                }

                match fs::read_to_string(path) {
                    Ok(content) => {
                        let line_count = content.lines().count() as u64;
                        let char_count = content.chars().count() as u64;
                        let estimated_tokens = estimate_tokens(char_count);
                        let extension = analysis_extension_label(path, &file_name);

                        analysis.processed_files += 1;
                        analysis.total_lines += line_count;
                        analysis.total_chars += char_count;
                        analysis.total_size += size;

                        record_extension_stats(
                            &mut extension_stats,
                            &extension,
                            line_count,
                            char_count,
                            estimated_tokens,
                            size,
                        );
                        record_analysis_file(
                            &mut analysis,
                            AnalysisFileEntry {
                                path: relative_path,
                                extension,
                                lines: line_count,
                                chars: char_count,
                                estimated_tokens,
                                size,
                            },
                        );
                    }
                    Err(_) => {
                        record_skipped_reason(&mut skipped_reasons, "skip-reason-read-error");
                    }
                }
            }
            Err(_) => {
                record_skipped_reason(&mut skipped_reasons, "skip-reason-walk-error");
            }
        }
    }

    analysis.estimated_tokens = estimate_tokens(analysis.total_chars);
    analysis.extension_breakdown = sorted_extension_stats(extension_stats);
    analysis.skipped_reasons = sorted_skipped_reasons(skipped_reasons);
    analysis.skipped_files = analysis.skipped_reasons.iter().map(|r| r.files).sum();

    Ok(analysis)
}

fn estimate_tokens(total_chars: u64) -> u64 {
    (total_chars + 3) / 4
}

fn analysis_extension_label(path: &Path, file_name: &str) -> String {
    path.extension()
        .map(|e| e.to_string_lossy().to_string().to_lowercase())
        .filter(|e| !e.is_empty())
        .unwrap_or_else(|| {
            if file_name.is_empty() {
                "no extension".to_string()
            } else {
                file_name.to_string()
            }
        })
}

fn record_extension_stats(
    stats: &mut BTreeMap<String, ExtensionStats>,
    extension: &str,
    lines: u64,
    chars: u64,
    estimated_tokens: u64,
    size: u64,
) {
    let entry = stats
        .entry(extension.to_string())
        .or_insert_with(|| ExtensionStats {
            extension: extension.to_string(),
            language: language_for_extension(extension).to_string(),
            ..ExtensionStats::default()
        });
    entry.files += 1;
    entry.lines += lines;
    entry.chars += chars;
    entry.estimated_tokens += estimated_tokens;
    entry.size += size;
}

fn record_analysis_file(analysis: &mut ProjectAnalysis, entry: AnalysisFileEntry) {
    analysis.largest_by_lines.push(entry.clone());
    analysis
        .largest_by_lines
        .sort_by(|a, b| b.lines.cmp(&a.lines));
    analysis.largest_by_lines.truncate(10);

    analysis.largest_by_tokens.push(entry.clone());
    analysis
        .largest_by_tokens
        .sort_by(|a, b| b.estimated_tokens.cmp(&a.estimated_tokens));
    analysis.largest_by_tokens.truncate(10);

    analysis.largest_by_size.push(entry);
    analysis.largest_by_size.sort_by(|a, b| b.size.cmp(&a.size));
    analysis.largest_by_size.truncate(10);
}

fn record_skipped_reason(reasons: &mut BTreeMap<String, u64>, reason: &str) {
    *reasons.entry(reason.to_string()).or_insert(0) += 1;
}

fn sorted_extension_stats(stats: BTreeMap<String, ExtensionStats>) -> Vec<ExtensionStats> {
    let mut values: Vec<_> = stats.into_values().collect();
    values.sort_by(|a, b| {
        b.lines
            .cmp(&a.lines)
            .then_with(|| b.files.cmp(&a.files))
            .then_with(|| a.extension.cmp(&b.extension))
    });
    values
}

fn sorted_skipped_reasons(reasons: BTreeMap<String, u64>) -> Vec<SkippedReasonStats> {
    let mut values: Vec<_> = reasons
        .into_iter()
        .map(|(reason, files)| SkippedReasonStats { reason, files })
        .collect();
    values.sort_by(|a, b| b.files.cmp(&a.files).then_with(|| a.reason.cmp(&b.reason)));
    values
}

fn language_for_extension(extension: &str) -> &str {
    match extension {
        "rs" => "Rust",
        "ts" | "tsx" => "TypeScript",
        "js" | "jsx" | "mjs" | "cjs" => "JavaScript",
        "html" => "HTML",
        "css" | "scss" => "Styles",
        "py" => "Python",
        "go" => "Go",
        "java" | "kt" => "Java/Kotlin",
        "swift" => "Swift",
        "dart" => "Dart",
        "rb" => "Ruby",
        "php" => "PHP",
        "cs" => "C#",
        "c" | "cpp" | "h" | "hpp" => "C/C++",
        "json" | "yaml" | "yml" | "toml" | "xml" => "Config",
        "md" | "txt" | "tex" => "Docs",
        "sh" | "bash" => "Shell",
        "sql" => "SQL",
        "csv" => "Data",
        "ipynb" => "Notebook",
        "dockerfile" => "Docker",
        "makefile" => "Make",
        "no extension" => "No extension",
        _ => "Other",
    }
}

fn record_top_loc_file(stats: &mut LocStats, path: PathBuf, lines: u64, size: u64) {
    stats.largest_files.push(LocFileEntry { path, lines, size });
    stats.largest_files.sort_by(|a, b| b.lines.cmp(&a.lines));
    if stats.largest_files.len() > 10 {
        stats.largest_files.truncate(10);
    }
}

fn collect_files(
    project_path: &Path,
    config: &ProjectConfig,
    args: &Args,
    walker: Walk,
    report: &mut OutputReport,
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
                    report_write!(
                        report,
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
                        println!(
                            "{}",
                            t!(
                                "cli-ignoring-excessive-size",
                                "path" => relative_path.display().to_string()
                            )
                        );
                    }
                    stats.skipped += 1;
                    continue;
                }

                valid_files.push(path.to_path_buf());

                // Record the file in the tree view with indentation to reflect depth.
                let depth = relative_path.components().count();
                let indent = "  ".repeat(depth.saturating_sub(1));
                report_write!(
                    report,
                    "{}└── {}",
                    indent,
                    relative_path.file_name().unwrap().to_string_lossy()
                )?;
            }
            Err(err) => {
                if args.verbose {
                    eprintln!(
                        "{}",
                        t!(
                            "cli-error-reading-input",
                            "error" => err.to_string()
                        )
                    );
                }
            }
        }
    }

    Ok((valid_files, stats))
}

fn write_file_contents(
    project_path: &Path,
    files: &[PathBuf],
    report: &mut OutputReport,
    stats: &mut ScanStats,
    args: &Args,
) -> Result<()> {
    for path in files {
        report.rotate_if_needed()?;

        let relative_path = diff_paths(path, project_path).unwrap_or_else(|| path.to_path_buf());
        let relative_str = relative_path.to_string_lossy();
        let size = path
            .metadata()
            .with_context(|| {
                t!(
                    "cli-error-metadata",
                    "path" => relative_path.display().to_string()
                )
            })?
            .len();

        if args.verbose {
            report.write_line(&t!("report-file-rule-top"))?;
            report.write_line(&t!(
                "report-file-name-verbose",
                "path" => relative_str.as_ref()
            ))?;
            report.write_line(&t!(
                "report-file-size-verbose",
                "size" => format_size(size)
            ))?;
            report.write_line(&t!("report-file-rule-mid"))?;
        } else {
            report.write_line(&t!("report-file-name", "path" => relative_str.as_ref()))?;
        }

        // Avoid dumping binary content which would clutter the report.
        if is_binary(path) {
            if args.verbose {
                report.write_line(&t!("report-binary-omitted-verbose"))?;
            } else {
                report.write_line(&t!("report-binary-omitted"))?;
            }
        } else {
            match fs::read_to_string(path) {
                Ok(content) => {
                    if args.verbose {
                        // Include line numbers to make the output easy to reference.
                        for (i, line) in content.lines().enumerate() {
                            report_write!(report, "{:>4} │ {}", i + 1, line)?;
                        }
                    } else {
                        for line in content.lines() {
                            report.write_line(line)?;
                        }
                    }
                }
                Err(_) => {
                    if args.verbose {
                        report.write_line(&t!("report-utf8-error-verbose"))?;
                    } else {
                        report.write_line(&t!("report-utf8-error"))?;
                    }
                }
            }
        }

        if args.verbose {
            report.write_line(&t!("report-file-rule-bottom"))?;
            report.write_blank_line()?;
        } else {
            report.write_blank_line()?;
        }
        stats.total_size += size;
    }

    Ok(())
}

fn write_summary(
    report: &mut OutputReport,
    stats: &ScanStats,
    processed_count: usize,
) -> Result<()> {
    // Final footer with a lightweight count of what happened.
    report.write_blank_line()?;
    report.write_line(&t!("report-contents-rule"))?;
    report.write_line(&t!("report-summary-title"))?;
    report.write_line(&t!(
        "cli-report-files-processed",
        "count" => i18n::format_number(processed_count as i64)
    ))?;
    report.write_line(&t!(
        "report-files-skipped",
        "count" => i18n::format_number(stats.skipped as i64)
    ))?;
    report.write_line(&t!(
        "report-total-content-size",
        "size" => format_size(stats.total_size)
    ))?;
    report.write_line(&t!("report-contents-rule"))?;
    Ok(())
}

#[derive(Default)]
// Lightweight counters gathered during a scan.
struct ScanStats {
    total_size: u64,
    skipped: u64,
}

// Summary handed back to callers (CLI banner, GUI results panel) once a scan finishes.
pub struct ProcessOutcome {
    pub project_name: String,
    pub project_type: String,
    pub output_paths: Vec<PathBuf>,
    pub processed_files: usize,
    pub skipped_files: u64,
    pub total_size: u64,
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

#[derive(Clone, Default)]
pub struct ProjectAnalysis {
    pub project_name: String,
    pub project_type: String,
    pub root_path: PathBuf,
    pub processed_files: u64,
    pub skipped_files: u64,
    pub total_lines: u64,
    pub total_chars: u64,
    pub estimated_tokens: u64,
    pub total_size: u64,
    pub extension_breakdown: Vec<ExtensionStats>,
    pub largest_by_lines: Vec<AnalysisFileEntry>,
    pub largest_by_tokens: Vec<AnalysisFileEntry>,
    pub largest_by_size: Vec<AnalysisFileEntry>,
    pub skipped_reasons: Vec<SkippedReasonStats>,
    pub tree: TreeStats,
    pub config: AnalysisConfigState,
}

#[derive(Clone, Default)]
pub struct ExtensionStats {
    pub extension: String,
    pub language: String,
    pub files: u64,
    pub lines: u64,
    pub chars: u64,
    pub estimated_tokens: u64,
    pub size: u64,
}

#[derive(Clone, Default)]
pub struct AnalysisFileEntry {
    pub path: PathBuf,
    pub extension: String,
    pub lines: u64,
    pub chars: u64,
    pub estimated_tokens: u64,
    pub size: u64,
}

#[derive(Clone, Default)]
pub struct SkippedReasonStats {
    pub reason: String,
    pub files: u64,
}

#[derive(Clone, Default)]
pub struct TreeStats {
    pub directories: u64,
    pub files: u64,
    pub max_depth: u64,
}

#[derive(Clone, Default)]
pub struct AnalysisConfigState {
    pub config_present: bool,
    pub gitignore_enabled: bool,
    pub max_file_size: u64,
}
