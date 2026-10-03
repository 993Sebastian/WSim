//! Command line for WSim. Developer tool for headless runs, data validation and logs.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use clap::{Parser, Subcommand, ValueEnum};
use wsim_core::calendar::{Date, RoundLength};
use wsim_core::game::Game;
use wsim_core::money::Money;
use wsim_core::save;
use wsim_core::state::{GameSettings, StartForm};
use wsim_data::{GameData, format_date};

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
    /// Lässt eine Partie ohne Oberfläche laufen.
    Run(RunArgs),
}

#[derive(clap::Args)]
struct RunArgs {
    /// Datenverzeichnis
    #[arg(long, default_value = "data")]
    daten: PathBuf,
    /// Spielstand laden statt neu zu beginnen
    #[arg(long)]
    laden: Option<PathBuf>,
    /// Spielstand am Ende speichern
    #[arg(long)]
    speichern: Option<PathBuf>,
    /// Zufallswert für reproduzierbare Partien
    #[arg(long, default_value_t = 1)]
    seed: u64,
    #[arg(long, default_value_t = 1900)]
    startjahr: i32,
    /// Startland bzw. Firmensitz (ISO-Code)
    #[arg(long, default_value = "DEU")]
    land: String,
    /// Startkapital in USD
    #[arg(long, default_value_t = 100_000.0)]
    kapital: f64,
    #[arg(long, value_enum, default_value_t = Startform::Werkstatt)]
    startform: Startform,
    #[arg(long, default_value = "Neue Firma")]
    name: String,
    /// Länge jeder Runde
    #[arg(long, value_enum, default_value_t = Runde::Monat)]
    runde: Runde,
    /// Anzahl der Runden (Standard: 1, oder bis zum Datum aus --bis)
    #[arg(long)]
    runden: Option<u32>,
    /// Laufen bis zu diesem Tag (JJJJ-MM-TT, ausschließlich)
    #[arg(long, value_parser = parse_date)]
    bis: Option<Date>,
    /// Keine Ausgabe je Runde
    #[arg(long)]
    leise: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum Runde {
    Tag,
    Woche,
    Monat,
    Quartal,
}

#[derive(Clone, Copy, ValueEnum)]
enum Startform {
    Werkstatt,
    Handel,
}

fn parse_date(text: &str) -> Result<Date, String> {
    let parts: Vec<&str> = text.split('-').collect();
    let invalid = || format!("„{text}“ ist kein gültiges Datum (JJJJ-MM-TT)");
    let [y, m, d] = parts.as_slice() else {
        return Err(invalid());
    };
    let (Ok(y), Ok(m), Ok(d)) = (y.parse(), m.parse(), d.parse()) else {
        return Err(invalid());
    };
    Date::new(y, m, d).ok_or_else(invalid)
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
        Command::Run(args) => match run(&args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(message) => {
                eprintln!("{message}");
                ExitCode::FAILURE
            }
        },
    }
}

fn validate(directory: &Path) -> ExitCode {
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

fn load_data(directory: &Path) -> Result<GameData, String> {
    let outcome = wsim_data::load_dir(directory);
    outcome.data.ok_or_else(|| {
        let findings: Vec<String> = outcome.report.errors().map(ToString::to_string).collect();
        format!(
            "Die Spieldaten enthalten Fehler:\n\n{}",
            findings.join("\n\n")
        )
    })
}

fn run(args: &RunArgs) -> Result<(), String> {
    let data = load_data(&args.daten)?;
    let texts = &data.texts;
    let catalog = Arc::new(data.catalog.clone());

    let mut game = match &args.laden {
        Some(path) => {
            let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
            let loaded = save::decode(&bytes, catalog).map_err(|e| texts.render(&e.message()))?;
            if loaded.data_changed {
                println!("Hinweis: Die Spieldaten haben sich seit dem Speichern geändert.");
            }
            loaded.game
        }
        None => {
            let start_country = catalog
                .countries
                .id(&args.land)
                .ok_or_else(|| format!("Land „{}“ gibt es in den Spieldaten nicht.", args.land))?;
            let settings = GameSettings {
                seed: args.seed,
                start_year: args.startjahr,
                start_country,
                start_capital: Money::from_usd(args.kapital).unwrap_or(Money::ZERO),
                start_form: match args.startform {
                    Startform::Werkstatt => StartForm::Workshop,
                    Startform::Handel => StartForm::Trading,
                },
                company_name: args.name.clone(),
            };
            Game::new(catalog, settings).map_err(|e| texts.render(&e.message()))?
        }
    };

    let length = match args.runde {
        Runde::Tag => RoundLength::Day,
        Runde::Woche => RoundLength::Week,
        Runde::Monat => RoundLength::Month,
        Runde::Quartal => RoundLength::Quarter,
    };
    let rounds = args
        .runden
        .unwrap_or(if args.bis.is_some() { u32::MAX } else { 1 });
    for _ in 0..rounds {
        if game.is_over() || args.bis.is_some_and(|end| game.date() >= end) {
            break;
        }
        let report = game.advance(length, |_| {});
        if !args.leise {
            let last_day = report.to.add_days(-1);
            println!(
                "{} – {} ({} Tage)",
                format_date(report.from),
                format_date(last_day),
                report.days
            );
        }
        for message in &report.messages {
            println!("  {}", texts.render(message));
        }
    }

    if let Some(path) = &args.speichern {
        fs::write(path, save::encode(&game)).map_err(|e| format!("{}: {e}", path.display()))?;
        println!("Gespeichert: {}", path.display());
    }
    println!(
        "Stand: {} · Zustands-Hash {}",
        format_date(game.date()),
        game.state_hash()
    );
    Ok(())
}
