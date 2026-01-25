//
// main.rs
// Code-Scanner-rs
//
// Entry point that parses CLI flags, prepares output directories, prints a banner, and dispatches project scanning either as a single root or by iterating subdirectories.
//
// Thales Matheus Mendonça Santos - November 2025
//

mod cli;
mod config;
mod project;
mod scanner;
mod utils;

use crate::cli::Args;
use crate::project::is_single_project_root;
use crate::scanner::process_project;
use anyhow::{bail, Context, Result};
use clap::Parser;
use std::fs;

fn main() -> Result<()> {
    // Parse CLI input and hydrate the Args struct with defaults and user flags.
    let args = Args::parse();

    // Fail fast if the input directory does not exist to avoid silent no-ops.
    if !args.input_dir.exists() {
        if is_default_input_dir(&args.input_dir) {
            fs::create_dir_all(&args.input_dir).with_context(|| {
                format!(
                    "Falha ao criar diretório de entrada: {}",
                    args.input_dir.display()
                )
            })?;
            println!(
                "ℹ️  O diretório padrão de entrada foi criado em: {}",
                args.input_dir.display()
            );
            println!(
                "   Adicione os projetos que deseja analisar dentro desse diretório e execute novamente."
            );
            return Ok(());
        }
        bail!("Diretório de entrada não encontrado: {:?}", args.input_dir);
    }

    // Ensure the output directory exists so report writes do not panic later.
    fs::create_dir_all(&args.output_dir).context("Falha ao criar diretório de saída")?;

    // Show a quick banner so users know what is being processed.
    print_banner(&args);

    // Treat the root as a single project when it looks like a repo root, otherwise
    // iterate over subfolders and process them individually.
    if is_single_project_root(&args.input_dir) {
        process_project(&args.input_dir, &args.output_dir, &args)?;
    } else {
        process_subdirectories(&args)?;
    }

    println!("\n✨ Concluído!");
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
    println!("📍 Entrada: {:?}", args.input_dir);
    println!("📍 Saída:   {:?}", args.output_dir);
    println!();
}
