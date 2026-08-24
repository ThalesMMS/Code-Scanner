//
// main.rs
// Code-Scanner-rs
//
// Entry point that parses CLI flags, prepares output directories, prints a banner, and dispatches project scanning either as a single root or by iterating subdirectories.
//
// Thales Matheus Mendonça Santos - November 2025
//

use code_scanner::cli::Args;
use code_scanner::i18n;
use code_scanner::project::is_single_project_root;
use code_scanner::scanner::{collect_loc_stats, process_project, LocStats};
use code_scanner::utils::format_size;
use code_scanner::t;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let args = Args::parse_localized();

    if let Some(loc_path) = &args.loc {
        return run_loc_mode(loc_path, &args);
    }

    if !args.input_dir.exists() {
        if is_default_input_dir(&args.input_dir) {
            fs::create_dir_all(&args.input_dir).with_context(|| {
                t!(
                    "cli-error-create-input",
                    "path" => args.input_dir.display().to_string()
                )
            })?;
            println!(
                "{}",
                t!(
                    "cli-default-dir-created",
                    "path" => args.input_dir.display().to_string()
                )
            );
            return Ok(());
        }
        bail!(
            "{}",
            t!(
                "cli-error-input-not-found",
                "path" => format!("{:?}", args.input_dir)
            )
        );
    }

    fs::create_dir_all(&args.output_dir).context(t!("cli-error-create-output"))?;

    print_banner(&args);

    if is_single_project_root(&args.input_dir) {
        process_project(&args.input_dir, &args.output_dir, &args)?;
    } else {
        process_subdirectories(&args)?;
    }

    println!("\n{}", t!("cli-completed"));
    Ok(())
}

fn process_subdirectories(args: &Args) -> Result<()> {
    let mut projects_found = 0;

    for entry in fs::read_dir(&args.input_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            process_project(&path, &args.output_dir, args)?;
            projects_found += 1;
        }
    }

    if projects_found == 0 {
        println!("{}", t!("cli-no-subdirectories"));
        process_project(&args.input_dir, &args.output_dir, args)?;
    }

    Ok(())
}

fn is_default_input_dir(path: &std::path::Path) -> bool {
    path == std::path::Path::new("./input") || path == std::path::Path::new("input")
}

fn print_banner(args: &Args) {
    println!("{}", t!("app-banner"));
    println!(
        "{}",
        t!(
            "cli-label-input",
            "path" => format!("{:?}", args.input_dir)
        )
    );
    println!(
        "{}",
        t!(
            "cli-label-output",
            "path" => format!("{:?}", args.output_dir)
        )
    );
    if let Some(max_lines) = args.max_output_lines {
        println!(
            "{}",
            t!("cli-max-output-lines", "count" => i18n::format_number(max_lines as i64))
        );
    }
    println!();
}

fn run_loc_mode(loc_path: &Path, args: &Args) -> Result<()> {
    if !loc_path.exists() {
        bail!(
            "{}",
            t!(
                "cli-loc-dir-not-found",
                "path" => loc_path.display().to_string()
            )
        );
    }

    println!("{}", t!("cli-loc-mode"));
    println!(
        "{}",
        t!(
            "cli-loc-target",
            "path" => loc_path.display().to_string()
        )
    );
    if args.no_gitignore {
        println!("{}", t!("cli-gitignore-disabled"));
    }
    let stats = collect_loc_stats(loc_path, args)?;
    print_loc_summary(loc_path, &stats);
    Ok(())
}

fn print_loc_summary(loc_path: &Path, stats: &LocStats) {
    println!();
    println!("{}", t!("cli-loc-summary-title"));
    println!(
        "  {}",
        t!(
            "cli-loc-summary-target",
            "path" => loc_path.display().to_string()
        )
    );
    println!(
        "  {}",
        t!("cli-loc-files-processed", "count" => i18n::format_number(stats.processed_files as i64))
    );
    println!(
        "  {}",
        t!("cli-loc-files-skipped", "count" => i18n::format_number(stats.skipped_files as i64))
    );
    println!(
        "  {}",
        t!("cli-loc-total-lines", "count" => i18n::format_number(stats.total_lines as i64))
    );
    println!(
        "  {}",
        t!("cli-loc-total-chars", "count" => i18n::format_number(stats.total_chars as i64))
    );
    println!(
        "  {}",
        t!(
            "cli-loc-estimated-tokens",
            "count" => i18n::format_number(stats.estimated_tokens as i64)
        )
    );
    println!();
    println!("{}", t!("cli-loc-top-files"));
    if stats.largest_files.is_empty() {
        println!("{}", t!("cli-loc-no-files"));
        return;
    }
    for (index, entry) in stats.largest_files.iter().enumerate() {
        println!(
            "{}",
            t!(
                "cli-loc-top-file",
                "index" => format!("{:>2}", index + 1),
                "path" => entry.path.display().to_string(),
                "lines" => i18n::format_number(entry.lines as i64),
                "size" => format_size(entry.size)
            )
        );
    }
}
