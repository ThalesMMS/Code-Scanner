//
// cli.rs
// Code-Scanner-rs
//
// Defines the command-line interface and arguments that control what gets scanned, where reports are written, and verbosity flags.
//
// Thales Matheus Mendonça Santos - November 2025
//

use clap::Parser;
use std::path::PathBuf;

/// Command-line interface definition for the code scanner.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Input directory (project to scan)
    #[arg(short, long, default_value = "./input")]
    pub input_dir: PathBuf,

    /// Output directory for reports
    #[arg(short, long, default_value = "./output")]
    pub output_dir: PathBuf,

    /// LOC mode: compute a lines/tokens summary for the target path
    #[arg(long, value_name = "PATH")]
    pub loc: Option<PathBuf>,

    /// Ignore the project's .gitignore file
    #[arg(long)]
    pub no_gitignore: bool,

    /// Verbose mode
    #[arg(short, long)]
    pub verbose: bool,

    /// File extensions to ignore (e.g., --ignore .ts .js .json)
    #[arg(long, value_name = "EXT", num_args = 1..)]
    pub ignore: Vec<String>,

    /// Split output into multiple files after this many lines (never breaks mid source file)
    #[arg(long, value_name = "LINES")]
    pub max_output_lines: Option<u64>,
}
