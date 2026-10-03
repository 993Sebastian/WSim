//! Command line for WSim. Developer tool for headless runs, data validation and logs.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "wsim",
    version,
    about = "WSim – Wirtschaftssimulation 1900–2100 (Kommandozeile)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Zeigt Informationen zum Simulationskern.
    Info,
    /// Prüft die Spieldaten und meldet Fehler und Warnungen.
    Validate {
        /// Datenverzeichnis
        #[arg(default_value = "data")]
        verzeichnis: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Info => {
            let info = wsim_core::core_info();
            println!("WSim-Simulationskern {}", info.version);
            ExitCode::SUCCESS
        }
        Command::Validate { verzeichnis } => validate(&verzeichnis),
    }
}

fn validate(directory: &std::path::Path) -> ExitCode {
    let outcome = wsim_data::load_dir(directory);
    for finding in outcome.report.findings() {
        println!("{finding}\n");
    }
    let errors = outcome.report.errors().count();
    let warnings = outcome.report.warnings().count();
    if let Some(data) = &outcome.data {
        let c = &data.catalog;
        println!(
            "Daten geprüft: {} Länder, {} Produkte, {} Rezepte, {} Anlagen, {} Technologien, {} Lagerstätten, {} Texte.",
            c.countries.len(),
            c.products.len(),
            c.recipes.len(),
            c.facilities.len(),
            c.technologies.len(),
            c.deposits.len(),
            data.texts.len(),
        );
    }
    println!("{errors} Fehler, {warnings} Warnungen.");
    if errors > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
