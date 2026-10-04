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
use wsim_data::{GameData, format_date, format_money};

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
    /// Zeigt die berechneten Werte eines Landes (Ländermodell).
    Land {
        /// ISO-Code, z. B. DEU
        land: String,
        #[arg(long, default_value_t = 1900)]
        jahr: i32,
        #[arg(long, default_value = "data")]
        daten: PathBuf,
    },
    /// Zeigt den günstigsten Transportweg zwischen zwei Ländern je Transportklasse.
    Route {
        /// ISO-Code des Abgangslands, z. B. GBR
        von: String,
        /// ISO-Code des Ziellands, z. B. USA
        nach: String,
        #[arg(long, default_value_t = 1900)]
        jahr: i32,
        #[arg(long, default_value = "data")]
        daten: PathBuf,
    },
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
    /// Am Ende GuV, Bilanz und Kapitalfluss der eigenen Firma ausgeben
    #[arg(long)]
    bericht: bool,
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
        Command::Land { land, jahr, daten } => match show_country(&daten, &land, jahr) {
            Ok(()) => ExitCode::SUCCESS,
            Err(message) => {
                eprintln!("{message}");
                ExitCode::FAILURE
            }
        },
        Command::Route {
            von,
            nach,
            jahr,
            daten,
        } => match show_route(&daten, &von, &nach, jahr) {
            Ok(()) => ExitCode::SUCCESS,
            Err(message) => {
                eprintln!("{message}");
                ExitCode::FAILURE
            }
        },
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
            "Daten geprüft: {} Länder, {} Produkte, {} Rezepte, {} Anlagen, {} Technologien, {} Lagerstätten, {} Verkehrsmittel, {} Texte.",
            c.countries.len(),
            c.products.len(),
            c.recipes.len(),
            c.facilities.len(),
            c.technologies.len(),
            c.deposits.len(),
            c.vehicles.len(),
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
                research_ahead_factor: 1.0,
                market_scale: 1.0,
                ai: Default::default(),
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
    if args.bericht
        && let Some(company) = game.state().company(game.player())
    {
        print_report(texts, &company.ledger);
    }
    if let Some(company) = game.state().company(game.player()) {
        println!(
            "{}: Kasse {}, Ergebnis laufendes Jahr {}",
            company.name,
            format_money(company.ledger.cash()),
            format_money(company.ledger.year.total())
        );
    }
    println!(
        "Stand: {} · Zustands-Hash {}",
        format_date(game.date()),
        game.state_hash()
    );
    Ok(())
}

fn show_country(directory: &Path, key: &str, year: i32) -> Result<(), String> {
    use wsim_core::ids::Id;
    use wsim_data::format_number;

    let data = load_data(directory)?;
    let c = &data.catalog;
    let id = c
        .countries
        .id(key)
        .ok_or_else(|| format!("Land „{key}“ gibt es nicht."))?;
    let s = wsim_core::country_model::compute(c, id, Date::first_of_year(year));
    let name = data.texts.get(&format!("land.{key}")).unwrap_or(key);
    let n = |v: f64| format_number(v, 0);
    let p = |v: f64| format!("{} %", format_number(v * 100.0, 1));
    println!("{name} am 01.01.{year}");
    println!("  Bevölkerung            {}", n(s.population));
    println!("  BIP je Kopf (KKP)      {} USD", n(s.gdp_per_capita_usd));
    println!(
        "  Preisniveau            {}",
        format_number(s.price_level, 2)
    );
    println!("  Gini                   {}", format_number(s.gini, 3));
    let quintiles: Vec<String> = s.income_quintiles_usd.iter().map(|&v| n(v)).collect();
    println!("  Einkommen je Fünftel   {} USD", quintiles.join(" / "));
    println!("  Erwerbspersonen        {}", n(s.labor_force));
    for (group, _) in c.labor_groups.iter() {
        let g = group.index();
        println!(
            "    {:<26} {:>12} Personen  {:>8} USD/h",
            c.labor_groups.key(group),
            n(s.labor_pool[g]),
            format_number(s.hourly_wage_usd[g], 2)
        );
    }
    println!(
        "  Strompreis             {} USD/MWh, Netz {}",
        n(s.electricity_price_usd_mwh),
        p(s.grid_share)
    );
    println!(
        "  Steuern                Gewinn {}, Dividende {}",
        p(s.corporate_tax),
        p(s.dividend_tax)
    );
    let i = &s.infrastructure;
    println!(
        "  Infrastruktur          Schiene {}, Straße {}, Hafen {}, Luft {}",
        p(i.rail),
        p(i.road),
        p(i.port),
        p(i.air)
    );
    println!("  Entwicklung            {}", p(s.development));
    println!("  Stabilität             {}", p(s.stability));
    println!("  Automatisierung        {}", p(s.automation_affinity));
    let research: Vec<String> = c
        .specializations
        .iter()
        .map(|(id, _)| {
            format!(
                "{} {}",
                c.specializations.key(id),
                format_number(s.research_efficiency[id.index()], 2)
            )
        })
        .collect();
    println!("  Forschung              {}", research.join(", "));
    Ok(())
}

fn show_route(directory: &Path, from: &str, to: &str, year: i32) -> Result<(), String> {
    use wsim_data::format_number;

    let data = load_data(directory)?;
    let c = &data.catalog;
    let country = |key: &str| {
        c.countries
            .id(key)
            .ok_or_else(|| format!("Land „{key}“ gibt es nicht."))
    };
    let (a, b) = (country(from)?, country(to)?);
    let routes = wsim_core::transport::Routes::new(c, year, None);
    let name = |key: String| data.texts.get(&key).unwrap_or(&key).to_owned();
    println!(
        "{} → {} im Jahr {year}",
        name(format!("land.{from}")),
        name(format!("land.{to}"))
    );
    for (class, _) in c.transport_classes.iter() {
        let label = name(format!(
            "transportklasse.{}",
            c.transport_classes.key(class)
        ));
        match routes.get(class, a, b) {
            Some(r) => println!(
                "  {label:<14} {:>10} USD/t  {:>6} Tage  {}",
                format_number(r.cost_per_t, 2),
                format_number(r.days, 1),
                if r.by_sea { "über See" } else { "über Land" }
            ),
            None => println!("  {label:<14} kein Transportweg"),
        }
    }
    Ok(())
}

fn print_report(texts: &wsim_data::Texts, ledger: &wsim_core::ledger::Ledger) {
    use wsim_core::reports::{balance_sheet, income_statement};
    let name = |key: &str| texts.get(key).unwrap_or(key).to_owned();
    let line = |label: &str, value: Money| println!("  {label:<32} {:>20}", format_money(value));
    let periods = ledger
        .years
        .last()
        .into_iter()
        .map(|p| ("Letztes Jahr", p))
        .chain([("Laufendes Jahr", &ledger.year)]);
    for (title, period) in periods {
        let statement = income_statement(period);
        println!("\nGewinn- und Verlustrechnung – {title}");
        for (cost, amount) in &statement.lines {
            line(&name(cost.text_key()), *amount);
        }
        line("Ergebnis", statement.result);
        println!("  Kapitalfluss");
        line("    operativ", statement.cash_flow.operating);
        line("    Investitionen", statement.cash_flow.investing);
        line("    Finanzierung", statement.cash_flow.financing);
    }
    let sheet = balance_sheet(ledger);
    println!("\nBilanz");
    for (account, amount) in sheet.assets.iter().chain(&sheet.claims) {
        line(&name(account.text_key()), *amount);
    }
    line("Bilanzsumme", sheet.total);
    println!();
}
