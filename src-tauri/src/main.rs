#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use code_scanner::cli::Args;
use code_scanner::scanner::{collect_loc_stats, process_project};
use code_scanner::utils::format_size;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

#[tauri::command]
fn run_scan(input_dir: String, output_dir: String, options: ScanOptions) -> Result<ScanOutcomeDto, String> {
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

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![run_scan, run_loc])
        .run(tauri::generate_context!())
        .expect("error while running the Code Scanner application");
}
