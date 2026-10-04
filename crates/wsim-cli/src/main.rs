//! Command line for WSim. Developer tool for headless runs, data validation and logs.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use clap::{Parser, Subcommand, ValueEnum};
use wsim_core::calendar::{Date, RoundLength};
use wsim_core::catalog::Catalog;
use wsim_core::game::Game;
use wsim_core::ledger::Account;
use wsim_core::money::Money;
use wsim_core::save;
use wsim_core::state::{AiSettings, GameSettings, StartForm};
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
    /// Schreibt Beispielsichten (JSON) für die Browser-Vorschau der Oberfläche.
    Beispielsichten {
        /// Zieldatei, z. B. ui/src/kern/beispiel.json
        datei: PathBuf,
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
    /// Zahl der KI-Firmen (0 = ohne Gegner, Märkte in realer Größe)
    #[arg(long, default_value_t = 0)]
    ki: u32,
    /// Schwierigkeit der KI (Schlüssel aus kimodell.yaml, z. B. leicht, mittel, schwer)
    #[arg(long)]
    schwierigkeit: Option<String>,
    /// Am Ende einen Weltbericht ausgeben (Firmen, Produktion, Preise)
    #[arg(long)]
    welt: bool,
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
        Command::Beispielsichten { datei, daten } => match example_views(&daten, &datei) {
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
                ai: ai_settings(&catalog, args)?,
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
    if args.welt {
        print_world(texts, &game);
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

/// Views of a real game for the UI preview: options, a new game in 1914, the report of
/// its seventh month (July 1914) and the overview after it.
fn example_views(data: &Path, out: &Path) -> Result<(), String> {
    use wsim_session::{NewGameRequest, Session};
    let saves = std::env::temp_dir().join("wsim-beispielsichten");
    let mut session = Session::open(data, saves)?;
    let options = session.options();
    let request = NewGameRequest {
        seed: 1,
        start_year: 1914,
        country: "DEU".into(),
        capital_usd: 100_000.0,
        start_form: "werkstatt".into(),
        company_name: "Neue Firma".into(),
        companies: options.companies.default,
        difficulty: options.default_difficulty.clone(),
        research_factor: 1.0,
    };
    let message = |m: wsim_core::views::MessageView| m.key;
    let start = session.new_game(&request).map_err(message)?;
    // The July round shows the start of the First World War and competitors' moves.
    for _ in 0..6 {
        session.end_round("monat", |_| {}).map_err(message)?;
    }
    let report = session.end_round("monat", |_| {}).map_err(message)?;
    let after = session.overview().map_err(message)?;
    let map = session.world_map().map_err(message)?;
    let countries: serde_json::Map<String, serde_json::Value> = ["DEU", "GBR", "USA"]
        .iter()
        .map(|&k| {
            let detail = session.country(k).map_err(message)?;
            let value = serde_json::to_value(detail).map_err(|e| e.to_string())?;
            Ok((k.to_owned(), value))
        })
        .collect::<Result<_, String>>()?;
    let json = serde_json::json!({
        "optionen": options,
        "uebersicht_start": start,
        "bericht": report,
        "uebersicht": after,
        "weltkarte": map,
        "laender": countries,
        "produktion": session.production().map_err(message)?,
        "markt": session.market("DEU").map_err(message)?,
        "forschung": session.research().map_err(message)?,
        "finanzen": session.finance().map_err(message)?,
    });
    let text = serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?;
    fs::write(out, text + "\n").map_err(|e| format!("{}: {e}", out.display()))?;
    println!("Geschrieben: {}", out.display());
    Ok(())
}

fn ai_settings(catalog: &Catalog, args: &RunArgs) -> Result<AiSettings, String> {
    let model = &catalog.ai_model;
    let difficulty = match &args.schwierigkeit {
        Some(key) => model
            .difficulties
            .iter()
            .find(|d| &d.key == key)
            .ok_or_else(|| format!("Schwierigkeit „{key}“ gibt es nicht."))?,
        None => model
            .difficulties
            .get(model.default_difficulty)
            .ok_or("Die Spieldaten nennen keine Schwierigkeit.")?,
    };
    Ok(AiSettings {
        companies: args.ki,
        competence: difficulty.competence,
        aggressiveness: difficulty.aggressiveness,
    })
}

/// Companies, output and prices of the simulated world.
fn print_world(texts: &wsim_data::Texts, game: &Game) {
    use wsim_data::format_number;
    let state = game.state();
    let catalog = game.catalog();
    let ai: Vec<_> = state.companies.iter().filter(|c| c.ai.is_some()).collect();
    let bankrupt = ai.iter().filter(|c| c.bankrupt).count();
    println!(
        "Welt am {}: {} KI-Firmen, davon {} pleite, {} Standorte, Marktmaßstab {}",
        format_date(state.date),
        ai.len(),
        bankrupt,
        state.sites.len(),
        format_number(state.settings.market_scale, 3)
    );
    let mut by_equity: Vec<_> = state
        .companies
        .iter()
        .map(|c| {
            (
                c,
                c.ledger.total_assets() - c.ledger.balance(Account::Loans),
            )
        })
        .collect();
    by_equity.sort_by_key(|b| std::cmp::Reverse(b.1));
    println!("Größte Firmen (Eigenkapital):");
    for (c, equity) in by_equity.iter().take(15) {
        println!(
            "  {:<45} {:>18}  {}",
            c.name,
            format_money(*equity),
            texts
                .get(&format!("land.{}", catalog.countries.key(c.headquarters)))
                .unwrap_or_default()
        );
    }
    println!("Produkte (Welt, letzter Monat):");
    for (product, _) in catalog.products.iter() {
        let mut sold = 0.0;
        let mut demand = 0.0;
        let mut revenue = Money::ZERO;
        for (_, m) in state.markets.get(product).iter() {
            sold += m.last_month.sold;
            demand += m.last_month.demand;
            revenue += m.last_month.revenue;
        }
        let capacity: f64 = state
            .sites
            .iter()
            .filter(|s| !state.companies[s.owner.index()].bankrupt)
            .flat_map(|s| s.slots.iter())
            .filter_map(|sl| {
                let r = catalog.recipes.get(sl.recipe?);
                (r.product == product).then(|| {
                    catalog.facilities.get(sl.facility).runs_per_day
                        * f64::from(sl.count)
                        * r.output
                        * 30.0
                })
            })
            .sum();
        if sold <= 0.0 && demand <= 0.0 && capacity <= 0.0 {
            continue;
        }
        let produced: f64 = state
            .sites
            .iter()
            .filter(|s| !state.companies[s.owner.index()].bankrupt)
            .flat_map(|s| s.slots.iter())
            .filter_map(|sl| {
                let r = catalog.recipes.get(sl.recipe?);
                (r.product == product).then_some(sl.last_runs * r.output * 30.0)
            })
            .sum();
        let stock: f64 = state
            .sites
            .iter()
            .filter_map(|s| s.inventory.get(&product))
            .map(|st| st.quantity)
            .sum();
        let price = if sold > 0.0 {
            format_money(revenue.scale(1.0 / sold))
        } else {
            "–".into()
        };
        println!(
            "  {:<16} Nachfrage {:>12}  verkauft {:>12}  Kapazität {:>12}  Produktion {:>12}  Lager {:>12}  Preis {:>12}",
            texts
                .get(&format!("produkt.{}", catalog.products.key(product)))
                .unwrap_or_default(),
            format_number(demand, 0),
            format_number(sold, 0),
            format_number(capacity, 0),
            format_number(produced, 0),
            format_number(stock, 0),
            price
        );
    }
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
