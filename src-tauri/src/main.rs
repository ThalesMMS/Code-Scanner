#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use code_scanner::cli::Args;
use code_scanner::scanner::{
    analyze_project as analyze_project_data, collect_loc_stats, process_project,
};
use code_scanner::utils::format_size;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{Manager, PhysicalPosition, PhysicalSize};

const WINDOW_EDGE_MARGIN: i32 = 24;

/// Options shared by both the full-scan and LOC-only commands, mirroring the CLI flags.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScanOptions {
    no_gitignore: bool,
    verbose: bool,
    ignore: Vec<String>,
    max_output_lines: Option<u64>,
}

impl ScanOptions {
    fn into_args(self, input_dir: PathBuf, output_dir: PathBuf, loc: Option<PathBuf>) -> Args {
        Args {
            input_dir,
            output_dir,
            loc,
            no_gitignore: self.no_gitignore,
            verbose: self.verbose,
            ignore: self.ignore,
            max_output_lines: self.max_output_lines,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanOutcomeDto {
    project_name: String,
    project_type: String,
    output_paths: Vec<String>,
    processed_files: usize,
    skipped_files: u64,
    total_size_human: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LocFileEntryDto {
    path: String,
    lines: u64,
    size_human: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LocStatsDto {
    processed_files: u64,
    skipped_files: u64,
    total_lines: u64,
    total_chars: u64,
    estimated_tokens: u64,
    largest_files: Vec<LocFileEntryDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectAnalysisDto {
    project_name: String,
    project_type: String,
    root_path: String,
    processed_files: u64,
    skipped_files: u64,
    total_lines: u64,
    total_chars: u64,
    estimated_tokens: u64,
    total_size: u64,
    total_size_human: String,
    extension_breakdown: Vec<ExtensionStatsDto>,
    largest_by_lines: Vec<AnalysisFileEntryDto>,
    largest_by_tokens: Vec<AnalysisFileEntryDto>,
    largest_by_size: Vec<AnalysisFileEntryDto>,
    skipped_reasons: Vec<SkippedReasonStatsDto>,
    tree: TreeStatsDto,
    config: AnalysisConfigStateDto,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExtensionStatsDto {
    extension: String,
    language: String,
    files: u64,
    lines: u64,
    chars: u64,
    estimated_tokens: u64,
    size: u64,
    size_human: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisFileEntryDto {
    path: String,
    extension: String,
    lines: u64,
    chars: u64,
    estimated_tokens: u64,
    size: u64,
    size_human: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SkippedReasonStatsDto {
    reason: String,
    files: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TreeStatsDto {
    directories: u64,
    files: u64,
    max_depth: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisConfigStateDto {
    config_present: bool,
    gitignore_enabled: bool,
    max_file_size: u64,
    max_file_size_human: String,
}

#[tauri::command]
fn run_scan(
    input_dir: String,
    output_dir: String,
    options: ScanOptions,
) -> Result<ScanOutcomeDto, String> {
    let input = PathBuf::from(&input_dir);
    let output = PathBuf::from(&output_dir);

    if !input.is_dir() {
        return Err(format!("Input folder does not exist: {input_dir}"));
    }
    std::fs::create_dir_all(&output).map_err(|e| e.to_string())?;

    let args = options.into_args(input.clone(), output.clone(), None);
    let outcome = process_project(&input, &output, &args).map_err(|e| e.to_string())?;

    Ok(ScanOutcomeDto {
        project_name: outcome.project_name,
        project_type: outcome.project_type,
        output_paths: outcome
            .output_paths
            .iter()
            .map(|p| p.display().to_string())
            .collect(),
        processed_files: outcome.processed_files,
        skipped_files: outcome.skipped_files,
        total_size_human: format_size(outcome.total_size),
    })
}

#[tauri::command]
fn analyze_project(path: String, options: ScanOptions) -> Result<ProjectAnalysisDto, String> {
    let target = PathBuf::from(&path);
    if !target.is_dir() {
        return Err(format!("Folder does not exist: {path}"));
    }

    let args = options.into_args(target.clone(), target.clone(), None);
    let analysis = analyze_project_data(&target, &args).map_err(|e| e.to_string())?;

    Ok(ProjectAnalysisDto {
        project_name: analysis.project_name,
        project_type: analysis.project_type,
        root_path: analysis.root_path.display().to_string(),
        processed_files: analysis.processed_files,
        skipped_files: analysis.skipped_files,
        total_lines: analysis.total_lines,
        total_chars: analysis.total_chars,
        estimated_tokens: analysis.estimated_tokens,
        total_size: analysis.total_size,
        total_size_human: format_size(analysis.total_size),
        extension_breakdown: analysis
            .extension_breakdown
            .into_iter()
            .map(|entry| ExtensionStatsDto {
                extension: entry.extension,
                language: entry.language,
                files: entry.files,
                lines: entry.lines,
                chars: entry.chars,
                estimated_tokens: entry.estimated_tokens,
                size: entry.size,
                size_human: format_size(entry.size),
            })
            .collect(),
        largest_by_lines: analysis
            .largest_by_lines
            .into_iter()
            .map(|entry| AnalysisFileEntryDto {
                path: entry.path.display().to_string(),
                extension: entry.extension,
                lines: entry.lines,
                chars: entry.chars,
                estimated_tokens: entry.estimated_tokens,
                size: entry.size,
                size_human: format_size(entry.size),
            })
            .collect(),
        largest_by_tokens: analysis
            .largest_by_tokens
            .into_iter()
            .map(|entry| AnalysisFileEntryDto {
                path: entry.path.display().to_string(),
                extension: entry.extension,
                lines: entry.lines,
                chars: entry.chars,
                estimated_tokens: entry.estimated_tokens,
                size: entry.size,
                size_human: format_size(entry.size),
            })
            .collect(),
        largest_by_size: analysis
            .largest_by_size
            .into_iter()
            .map(|entry| AnalysisFileEntryDto {
                path: entry.path.display().to_string(),
                extension: entry.extension,
                lines: entry.lines,
                chars: entry.chars,
                estimated_tokens: entry.estimated_tokens,
                size: entry.size,
                size_human: format_size(entry.size),
            })
            .collect(),
        skipped_reasons: analysis
            .skipped_reasons
            .into_iter()
            .map(|entry| SkippedReasonStatsDto {
                reason: entry.reason,
                files: entry.files,
            })
            .collect(),
        tree: TreeStatsDto {
            directories: analysis.tree.directories,
            files: analysis.tree.files,
            max_depth: analysis.tree.max_depth,
        },
        config: AnalysisConfigStateDto {
            config_present: analysis.config.config_present,
            gitignore_enabled: analysis.config.gitignore_enabled,
            max_file_size: analysis.config.max_file_size,
            max_file_size_human: format_size(analysis.config.max_file_size),
        },
    })
}

#[tauri::command]
fn run_loc(path: String, options: ScanOptions) -> Result<LocStatsDto, String> {
    let target = PathBuf::from(&path);
    if !target.is_dir() {
        return Err(format!("Folder does not exist: {path}"));
    }

    let args = options.into_args(target.clone(), target.clone(), Some(target.clone()));
    let stats = collect_loc_stats(&target, &args).map_err(|e| e.to_string())?;

    Ok(LocStatsDto {
        processed_files: stats.processed_files,
        skipped_files: stats.skipped_files,
        total_lines: stats.total_lines,
        total_chars: stats.total_chars,
        estimated_tokens: stats.estimated_tokens,
        largest_files: stats
            .largest_files
            .into_iter()
            .map(|f| LocFileEntryDto {
                path: f.path.display().to_string(),
                lines: f.lines,
                size_human: format_size(f.size),
            })
            .collect(),
    })
}

fn fit_main_window_to_work_area(app: &tauri::App) -> tauri::Result<()> {
    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };
    let Some(monitor) = window.current_monitor()?.or(window.primary_monitor()?) else {
        window.center()?;
        return Ok(());
    };

    let work_area = monitor.work_area();
    let margin = WINDOW_EDGE_MARGIN.max(0);
    let margin_twice = (margin * 2) as u32;
    let width = work_area.size.width.saturating_sub(margin_twice).max(900);
    let height = work_area.size.height.saturating_sub(margin_twice).max(620);
    let x = work_area.position.x + ((work_area.size.width.saturating_sub(width) / 2) as i32);
    let y = work_area.position.y + ((work_area.size.height.saturating_sub(height) / 2) as i32);

    window.set_size(PhysicalSize::new(width, height))?;
    window.set_position(PhysicalPosition::new(x, y))?;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            fit_main_window_to_work_area(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![run_scan, run_loc, analyze_project])
        .run(tauri::generate_context!())
        .expect("error while running the Code Scanner application");
}
