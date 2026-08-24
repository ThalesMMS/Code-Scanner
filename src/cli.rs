//
// cli.rs
// Code-Scanner-rs
//
// Defines the command-line interface and arguments that control what gets scanned, where reports are written, and verbosity flags.
//
// Thales Matheus Mendonça Santos - November 2025
//

use clap::{Command, CommandFactory, FromArgMatches, Parser};
use std::path::PathBuf;

use crate::t;

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

    /// Language for CLI messages
    #[arg(long, value_name = "LOCALE")]
    pub lang: Option<String>,
}

impl Args {
    pub fn parse_localized() -> Self {
        crate::i18n::init_from_cli();
        let matches = localize_command(Self::command()).get_matches();
        Self::from_arg_matches(&matches).expect("CLI matches")
    }
}

fn localize_command(mut command: Command) -> Command {
    // clap's help_heading only accepts &'static Str.
    let options: &'static str = Box::leak(
        t!("cli-help-options")
            .trim_end_matches([':', '：', '։', '՝'])
            .trim()
            .to_string()
            .into_boxed_str(),
    );
    let template = format!(
        "{{before-help}}{{about-with-newline}}\n{} {{usage}}\n\n{{all-args}}{{after-help}}",
        t!("cli-help-usage")
    );
    command = command
        .about(t!("app-about"))
        .help_template(template)
        .mut_args(|arg| arg.help_heading(options));
    for (id, key) in [
        ("input_dir", "cli-arg-input-help"),
        ("output_dir", "cli-arg-output-help"),
        ("loc", "cli-arg-loc-help"),
        ("no_gitignore", "cli-arg-no-gitignore-help"),
        ("verbose", "cli-arg-verbose-help"),
        ("ignore", "cli-arg-ignore-help"),
        ("max_output_lines", "cli-arg-max-output-lines-help"),
        ("lang", "cli-arg-lang-help"),
    ] {
        command = command.mut_arg(id, |arg| arg.help(t!(key)).help_heading(options));
    }
    // Built-in --help/--version exist only after clap materializes them.
    command.build();
    command = command
        .mut_arg("help", |arg| arg.help(t!("cli-help-print-help")).help_heading(options))
        .mut_arg("version", |arg| {
            arg.help(t!("cli-help-print-version")).help_heading(options)
        });
    command
}
