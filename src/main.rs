//
// main.rs
// Code-Scanner-rs
//
// Entry point that parses CLI flags, prepares output directories, prints a banner, and dispatches project scanning either as a single root or by iterating subdirectories.
//
// Thales Matheus Mendonça Santos - November 2025
//

use code_scanner::cli::Args;
use code_scanner::project::is_single_project_root;
use code_scanner::scanner::{collect_loc_stats, process_project, LocStats};
use code_scanner::utils::format_size;
use anyhow::{bail, Context, Result};
use clap::Parser;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    // Parse CLI input and hydrate the Args struct with defaults and user flags.
    let args = Args::parse();

    if let Some(loc_path) = &args.loc {
        return run_loc_mode(loc_path, &args);
    }

    // Fail fast if the input directory does not exist to avoid silent no-ops.
    if !args.input_dir.exists() {
        if is_default_input_dir(&args.input_dir) {
            fs::create_dir_all(&args.input_dir).with_context(|| {
                format!(
                    "Failed to create input directory: {}",
                    args.input_dir.display()
                )
            })?;
            println!(
                "ℹ️  The default input directory was created at: {}",
                args.input_dir.display()
            );
            println!(
                "   Add the projects you want to analyze inside this directory and run again."
            );
            return Ok(());
        }
        bail!("Input directory not found: {:?}", args.input_dir);
    }

    // Ensure the output directory exists so report writes do not panic later.
    fs::create_dir_all(&args.output_dir).context("Failed to create output directory")?;

    // Show a quick banner so users know what is being processed.
    print_banner(&args);

    // Treat the root as a single project when it looks like a repo root, otherwise
    // iterate over subfolders and process them individually.
    if is_single_project_root(&args.input_dir) {
        process_project(&args.input_dir, &args.output_dir, &args)?;
    } else {
        process_subdirectories(&args)?;
    }

    println!("\n✨ Completed!");
    Ok(())
}

fn process_subdirectories(args: &Args) -> Result<()> {
    let mut projects_found = 0;

    // Walk over the first level of the input directory and process each folder
    // as a standalone project.
    for entry in fs::read_dir(&args.input_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            process_project(&path, &args.output_dir, args)?;
            projects_found += 1;
        }
    }

    // If nothing was found, fall back to treating the root as a single project.
    if projects_found == 0 {
        println!("ℹ️  No subdirectories found. Processing root as a single project.");
        process_project(&args.input_dir, &args.output_dir, args)?;
    }

    Ok(())
}

fn is_default_input_dir(path: &std::path::Path) -> bool {
    path == std::path::Path::new("./input") || path == std::path::Path::new("input")
}

fn print_banner(args: &Args) {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║               RUST CODE SCANNER (UNIFIED)                     ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!("📍 Input: {:?}", args.input_dir);
    println!("📍 Output: {:?}", args.output_dir);
    println!();
}

fn run_loc_mode(loc_path: &Path, args: &Args) -> Result<()> {
    if !loc_path.exists() {
        bail!("Directory for LOC not found: {}", loc_path.display());
    }

    println!("📏 LOC mode");
    println!("📍 Target: {}", loc_path.display());
    if args.no_gitignore {
        println!("⚠️  .gitignore disabled (--no-gitignore)");
    }
    let stats = collect_loc_stats(loc_path, args)?;
    print_loc_summary(loc_path, &stats);
    Ok(())
}

fn print_loc_summary(loc_path: &Path, stats: &LocStats) {
    println!();
    println!("📊 LOC SUMMARY");
    println!("  📁 Target: {}", loc_path.display());
    println!("  ✅ Files processed: {}", stats.processed_files);
    println!("  ⏭️  Files skipped: {}", stats.skipped_files);
    println!("  🧮 Total lines: {}", stats.total_lines);
    println!("  🔤 Total characters: {}", stats.total_chars);
    println!("  🤖 Estimated tokens: {}", stats.estimated_tokens);
    println!();
    println!("📈 TOP 10 FILES WITH MOST LINES");
    if stats.largest_files.is_empty() {
        println!("  (no files counted)");
        return;
    }
    for (index, entry) in stats.largest_files.iter().enumerate() {
        println!(
            "  {:>2}. {} ({} lines, {})",
            index + 1,
            entry.path.display(),
            entry.lines,
            format_size(entry.size)
        );
    }
}
