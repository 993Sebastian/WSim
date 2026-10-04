//! Balance protocol of a world run (Architektur §4, M15): world markets month by month,
//! summed up per year with products and companies, and the anomalies that matter for
//! balancing. Writes `produkte.csv`, `firmen.csv` and `auswertung.md`.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use wsim_core::game::Game;
use wsim_core::ids::{Id, ProductId};
use wsim_core::ledger::Account;
use wsim_core::market;
use wsim_data::Texts;

/// Thresholds of the anomalies (only for the protocol, not for the game).
const EXPENSIVE: f64 = 1.5;
const CHEAP: f64 = 0.6;
const SHORTAGE: f64 = 0.85;
const OVERCAPACITY: f64 = 2.0;
const RICH_MARGIN: f64 = 0.4;
const LOSS_MARGIN: f64 = -0.1;

#[derive(Default)]
struct Month {
    /// Consumer markets.
    demand: f64,
    sold: f64,
    imported: f64,
    /// Inputs the running facilities need (planned production).
    input_need: f64,
    /// Output of all facilities (from the last day, times the days of the month).
    produced: f64,
    /// Sales of companies (to consumers, traders and other companies).
    company_sold: f64,
    company_revenue_usd: f64,
    /// Reference price weighted with the companies' sales.
    reference_usd: f64,
}

struct ProductYear {
    year: i32,
    product: ProductId,
    /// Consumer demand and input need of facilities.
    demand: f64,
    consumer_demand: f64,
    produced: f64,
    sold: f64,
    price_usd: Option<f64>,
    reference_usd: Option<f64>,
    capacity: f64,
    producers: usize,
    stock: f64,
    profit_usd: f64,
    imported: f64,
    flags: Vec<&'static str>,
}

struct CompanyYear {
    year: i32,
    active: usize,
    bankrupt: usize,
    founded: usize,
    profitable: usize,
    median_equity_usd: f64,
    player_equity_usd: f64,
    player_cash_usd: f64,
    player_result_usd: f64,
}

#[derive(Default)]
pub struct Protocol {
    month: Option<(i32, u32)>,
    year: BTreeMap<ProductId, Month>,
    companies_before: usize,
    products: Vec<ProductYear>,
    companies: Vec<CompanyYear>,
}

fn usd(m: wsim_core::money::Money) -> f64 {
    m.to_usd()
}

impl Protocol {
    pub fn new(game: &Game) -> Self {
        Self {
            month: Some((game.date().year(), game.date().month())),
            companies_before: game.state().companies.len(),
            ..Self::default()
        }
    }

    /// Call after every round; rounds must not be longer than a month.
    pub fn after_round(&mut self, game: &Game) {
        let date = game.date();
        let now = (date.year(), date.month());
        if self.month == Some(now) {
            return;
        }
        self.month = Some(now);
        self.add_month(game);
        if date.month() == 1 {
            self.close_year(game, date.year() - 1);
        }
    }

    fn add_month(&mut self, game: &Game) {
        let state = game.state();
        let catalog = game.catalog();
        let closed = state.date.add_days(-1);
        let days = f64::from(wsim_core::calendar::days_in_month(
            closed.year(),
            closed.month(),
        ));
        for (product, p) in catalog.products.iter() {
            let acc = self.year.entry(product).or_default();
            for (_, m) in state.markets.get(product).iter() {
                let t = &m.last_month;
                if outside_demand(p) {
                    acc.demand += t.demand;
                }
                acc.sold += t.sold;
                acc.imported += t.imported;
            }
        }
        for s in &state.sites {
            if state.companies[s.owner.index()].bankrupt {
                continue;
            }
            for sl in &s.slots {
                let Some(r) = sl.recipe.map(|r| catalog.recipes.get(r)) else {
                    continue;
                };
                if sl.ready > state.date {
                    continue;
                }
                let planned = catalog.facilities.get(sl.facility).runs_per_day
                    * f64::from(sl.count)
                    * sl.utilization;
                for &(p, q) in &r.inputs {
                    self.year.entry(p).or_default().input_need += q * planned * days;
                }
                self.year.entry(r.product).or_default().produced += sl.last_runs * r.output * days;
                for &(p, q) in &r.by_products {
                    self.year.entry(p).or_default().produced += sl.last_runs * q * days;
                }
            }
            for (&p, o) in &s.offers {
                let acc = self.year.entry(p).or_default();
                acc.company_sold += o.sold_last_month;
                acc.company_revenue_usd += o.sold_last_month * usd(o.price);
                acc.reference_usd +=
                    o.sold_last_month * usd(market::local_reference(catalog, state, s.country, p));
            }
        }
    }

    fn close_year(&mut self, game: &Game, year: i32) {
        let state = game.state();
        let catalog = game.catalog();
        let alive = |owner: usize| !state.companies[owner].bankrupt;
        let months = std::mem::take(&mut self.year);
        for (product, m) in months {
            let mut capacity = 0.0;
            let mut producers = std::collections::BTreeSet::new();
            for s in state.sites.iter().filter(|s| alive(s.owner.index())) {
                for sl in &s.slots {
                    let Some(r) = sl.recipe.map(|r| catalog.recipes.get(r)) else {
                        continue;
                    };
                    if r.product == product && sl.ready <= state.date {
                        capacity += catalog.facilities.get(sl.facility).runs_per_day
                            * f64::from(sl.count)
                            * r.output
                            * 365.0;
                        producers.insert(s.owner);
                    }
                }
            }
            let stock: f64 = state
                .sites
                .iter()
                .filter_map(|s| s.inventory.get(&product))
                .map(|st| st.quantity)
                .sum();
            let profit_usd: f64 = state
                .companies
                .iter()
                .filter_map(|c| c.ledger.years.last())
                .filter_map(|y| y.by_product.get(&product))
                .map(|&v| usd(v))
                .sum();
            let demand = m.demand + m.input_need;
            if demand <= 0.0 && m.produced <= 0.0 && capacity <= 0.0 {
                continue;
            }
            let price_usd = (m.company_sold > 0.0).then(|| m.company_revenue_usd / m.company_sold);
            let reference_usd = (m.company_sold > 0.0).then(|| m.reference_usd / m.company_sold);
            let mut flags = Vec::new();
            if let (Some(p), Some(r)) = (price_usd, reference_usd) {
                if p > EXPENSIVE * r {
                    flags.push("teuer");
                } else if p < CHEAP * r {
                    flags.push("billig");
                }
            }
            // Consumer goods: demand the markets could not serve; inputs: need the
            // facilities could not cover from the world's production.
            let short = if m.demand > 0.0 {
                m.sold < SHORTAGE * m.demand
            } else {
                m.input_need > 0.0 && m.produced < SHORTAGE * m.input_need
            };
            // Goods without a recipe come from the state market (no shortage).
            let made = catalog
                .recipes
                .values()
                .any(|r| r.product == product || r.by_products.iter().any(|&(p, _)| p == product));
            if short && made {
                flags.push("Mangel");
            }
            if demand > 0.0 && capacity > OVERCAPACITY * demand {
                flags.push("Überkapazität");
            }
            if m.company_revenue_usd > 0.0 {
                let margin = profit_usd / m.company_revenue_usd;
                if margin > RICH_MARGIN {
                    flags.push("sehr profitabel");
                } else if margin < LOSS_MARGIN {
                    flags.push("Verlust");
                }
            }
            self.products.push(ProductYear {
                year,
                product,
                demand,
                consumer_demand: m.demand,
                produced: m.produced,
                sold: m.company_sold,
                price_usd,
                reference_usd,
                capacity,
                producers: producers.len(),
                stock,
                profit_usd,
                imported: m.imported,
                flags,
            });
        }

        let ai: Vec<_> = state.companies.iter().filter(|c| c.ai.is_some()).collect();
        let mut equity: Vec<f64> = ai
            .iter()
            .filter(|c| !c.bankrupt)
            .map(|c| usd(c.ledger.total_assets() - c.ledger.balance(Account::Loans)))
            .collect();
        equity.sort_by(f64::total_cmp);
        let player = &state.companies[game.player().index()];
        self.companies.push(CompanyYear {
            year,
            active: ai.iter().filter(|c| !c.bankrupt).count(),
            bankrupt: ai.iter().filter(|c| c.bankrupt).count(),
            founded: state.companies.len() - self.companies_before,
            profitable: ai
                .iter()
                .filter(|c| !c.bankrupt)
                .filter(|c| {
                    c.ledger
                        .years
                        .last()
                        .is_some_and(|y| y.total() > wsim_core::money::Money::ZERO)
                })
                .count(),
            median_equity_usd: equity.get(equity.len() / 2).copied().unwrap_or(0.0),
            player_equity_usd: usd(
                player.ledger.total_assets() - player.ledger.balance(Account::Loans)
            ),
            player_cash_usd: usd(player.ledger.cash()),
            player_result_usd: player.ledger.years.last().map_or(0.0, |y| usd(y.total())),
        });
        self.companies_before = state.companies.len();
    }

    pub fn write(&self, game: &Game, texts: &Texts, dir: &Path) -> Result<(), String> {
        let catalog = game.catalog();
        let name = |p: ProductId| {
            texts
                .get(&format!("produkt.{}", catalog.products.key(p)))
                .unwrap_or(catalog.products.key(p))
                .to_owned()
        };
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let opt = |v: Option<f64>| v.map_or(String::new(), |v| format!("{v:.2}"));

        let mut csv = String::from(
            "jahr;produkt;bedarf;konsumnachfrage;produktion;absatz_firmen;preis_usd;richtpreis_usd;kapazitaet;hersteller;lager;ergebnis_usd;einfuhr;auffaellig\n",
        );
        for p in &self.products {
            let _ = writeln!(
                csv,
                "{};{};{:.0};{:.0};{:.0};{:.0};{};{};{:.0};{};{:.0};{:.0};{:.0};{}",
                p.year,
                catalog.products.key(p.product),
                p.demand,
                p.consumer_demand,
                p.produced,
                p.sold,
                opt(p.price_usd),
                opt(p.reference_usd),
                p.capacity,
                p.producers,
                p.stock,
                p.profit_usd,
                p.imported,
                p.flags.join(",")
            );
        }
        write(dir, "produkte.csv", &csv)?;

        let mut csv = String::from(
            "jahr;ki_aktiv;ki_pleite;gegruendet;mit_gewinn;median_eigenkapital_usd;spieler_eigenkapital_usd;spieler_kasse_usd;spieler_ergebnis_usd\n",
        );
        for c in &self.companies {
            let _ = writeln!(
                csv,
                "{};{};{};{};{};{:.0};{:.0};{:.0};{:.0}",
                c.year,
                c.active,
                c.bankrupt,
                c.founded,
                c.profitable,
                c.median_equity_usd,
                c.player_equity_usd,
                c.player_cash_usd,
                c.player_result_usd
            );
        }
        write(dir, "firmen.csv", &csv)?;

        let mut md = String::from("# Balance-Protokoll\n\n");
        let s = &game.state().settings;
        let _ = writeln!(
            md,
            "Seed {}, Start {}, {} KI-Firmen, Marktmaßstab {:.3}, Stand {}.\n",
            s.seed,
            s.start_year,
            s.ai.companies,
            s.market_scale,
            wsim_data::format_date(game.date())
        );
        md.push_str("## Firmen je Jahr\n\n| Jahr | KI aktiv | pleite (gesamt) | gegründet | mit Gewinn | Median Eigenkapital | Spieler Eigenkapital | Spieler Ergebnis |\n| --- | --- | --- | --- | --- | --- | --- | --- |\n");
        for c in &self.companies {
            let _ = writeln!(
                md,
                "| {} | {} | {} | {} | {} | {} | {} | {} |",
                c.year,
                c.active,
                c.bankrupt,
                c.founded,
                c.profitable,
                money(c.median_equity_usd),
                money(c.player_equity_usd),
                money(c.player_result_usd)
            );
        }
        md.push_str("\n## Auffälligkeiten je Produkt\n\nJahre mit Auffälligkeit (teuer > 1,5 × Richtpreis, billig < 0,6 ×, Mangel < 85 % der Nachfrage bzw. des Vorproduktbedarfs gedeckt, Überkapazität > 2 × Bedarf, sehr profitabel > 40 % Marge, Verlust < −10 %).\n\n| Produkt | Jahre | teuer | billig | Mangel | Überkapazität | sehr profitabel | Verlust |\n| --- | --- | --- | --- | --- | --- | --- | --- |\n");
        let mut by_product: BTreeMap<ProductId, (usize, BTreeMap<&str, usize>)> = BTreeMap::new();
        for p in &self.products {
            let e = by_product.entry(p.product).or_default();
            e.0 += 1;
            for f in &p.flags {
                *e.1.entry(f).or_default() += 1;
            }
        }
        for (product, (years, flags)) in &by_product {
            if flags.is_empty() {
                continue;
            }
            let n = |k: &str| flags.get(k).map_or(String::new(), ToString::to_string);
            let _ = writeln!(
                md,
                "| {} | {} | {} | {} | {} | {} | {} | {} |",
                name(*product),
                years,
                n("teuer"),
                n("billig"),
                n("Mangel"),
                n("Überkapazität"),
                n("sehr profitabel"),
                n("Verlust")
            );
        }
        if let Some(last) = self.products.last().map(|p| p.year) {
            let _ = writeln!(
                md,
                "\n## Produkte {last}\n\n| Produkt | Bedarf | Produktion | Absatz Firmen | Preis/Richtpreis | Kapazität/Bedarf | Hersteller | Marge | auffällig |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- |"
            );
            for p in self.products.iter().filter(|p| p.year == last) {
                let ratio = match (p.price_usd, p.reference_usd) {
                    (Some(a), Some(b)) if b > 0.0 => format!("{:.2}", a / b),
                    _ => "–".into(),
                };
                let cap = if p.demand > 0.0 {
                    format!("{:.2}", p.capacity / p.demand)
                } else {
                    "–".into()
                };
                let revenue = p.price_usd.unwrap_or(0.0) * p.sold;
                let margin = if revenue > 0.0 {
                    format!("{:.0} %", 100.0 * p.profit_usd / revenue)
                } else {
                    "–".into()
                };
                let _ = writeln!(
                    md,
                    "| {} | {:.0} | {:.0} | {:.0} | {} | {} | {} | {} | {} |",
                    name(p.product),
                    p.demand,
                    p.produced,
                    p.sold,
                    ratio,
                    cap,
                    p.producers,
                    margin,
                    p.flags.join(", ")
                );
            }
        }
        write(dir, "auswertung.md", &md)?;
        write(dir, "laender.csv", &by_country(game))?;
        let countries: Vec<_> = ["DEU", "GBR", "USA"]
            .iter()
            .filter_map(|k| catalog.countries.id(k))
            .collect();
        write(
            dir,
            "rezepte.md",
            &recipe_margins(catalog, texts, s.start_year, &countries),
        )?;
        println!("Protokoll geschrieben: {}", dir.display());
        Ok(())
    }
}

fn money(v: f64) -> String {
    wsim_data::format_number(v, 0) + " USD"
}

fn write(dir: &Path, name: &str, text: &str) -> Result<(), String> {
    let path = dir.join(name);
    fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Unit costs of every recipe at reference prices of the inputs and the wages and
/// electricity prices of the given countries, against the product's reference price
/// (Markdown). A margin far above or below the typical range points at a reference
/// price or recipe that needs balancing.
pub fn recipe_margins(
    catalog: &wsim_core::catalog::Catalog,
    texts: &Texts,
    year: i32,
    countries: &[wsim_core::ids::CountryId],
) -> String {
    use wsim_core::calendar::Date;
    let values: Vec<_> = countries
        .iter()
        .map(|&c| wsim_core::country_model::compute(catalog, c, Date::first_of_year(year)))
        .collect();
    let text = |key: String| texts.get(&key).map_or(key.clone(), str::to_owned);
    let mut md = format!(
        "# Rezeptmargen {year}\n\nStückkosten: Vorprodukte zu Richtpreisen, Arbeitsstunden zu den Löhnen des Landes, Strom zum Landespreis, Anlage über die Lebensdauer mit Instandhaltung; Nebenprodukte zu Richtpreisen abgezogen. Marge = 1 − Kosten/Richtpreis.\n\n| Rezept | Produkt | erfunden | Richtpreis |"
    );
    for c in countries {
        let _ = write!(md, " Kosten {} | Marge |", catalog.countries.key(*c));
    }
    md.push_str(" Anteil Arbeit |\n| --- | --- | --- | --- |");
    for _ in countries {
        md.push_str(" --- | --- |");
    }
    md.push_str(" --- |\n");
    for (id, r) in catalog.recipes.iter() {
        let f = catalog.facilities.get(r.facility);
        let invented = [r.technology, f.technology]
            .into_iter()
            .flatten()
            .map(|t| catalog.technologies.get(t).invention_year)
            .max();
        let price = catalog.products.get(r.product).reference_price.to_usd();
        let inputs: f64 = r
            .inputs
            .iter()
            .map(|&(p, q)| q * catalog.products.get(p).reference_price.to_usd())
            .sum();
        let by_products: f64 = r
            .by_products
            .iter()
            .map(|&(p, q)| q * catalog.products.get(p).reference_price.to_usd())
            .sum();
        let capital = f.investment.to_usd()
            * (1.0 / f64::from(f.lifetime_years.max(1)) + f.maintenance_share)
            / (365.0 * f.runs_per_day.max(1e-9));
        let _ = write!(
            md,
            "| {} | {} | {} | {} |",
            text(format!("rezept.{}", catalog.recipes.key(id))),
            text(format!("produkt.{}", catalog.products.key(r.product))),
            invented.map_or("–".into(), |y| y.to_string()),
            money(price)
        );
        let mut labor_share = 0.0;
        for v in &values {
            let labor: f64 = r
                .labor_hours
                .iter()
                .map(|&(g, h)| h * v.hourly_wage_usd[g.index()])
                .sum();
            let energy = r.energy_mwh * v.electricity_price_usd_mwh;
            let cost = (inputs + labor + energy + capital - by_products) / r.output.max(1e-9);
            labor_share = labor / (inputs + labor + energy + capital).max(1e-9);
            let margin = if price > 0.0 { 1.0 - cost / price } else { 0.0 };
            let mark = if !(-0.05..=0.4).contains(&margin) {
                " ⚠"
            } else {
                ""
            };
            let _ = write!(md, " {} | {:.0} %{} |", money(cost), margin * 100.0, mark);
        }
        let _ = writeln!(md, " {:.0} % |", labor_share * 100.0);
    }
    md
}

/// Production, need, consumer demand, sales and prices of the last month per product
/// and country (where something happens), to find trade problems between countries.
fn by_country(game: &Game) -> String {
    let state = game.state();
    let catalog = game.catalog();
    #[derive(Default)]
    struct Row {
        produced: f64,
        need: f64,
        sold: f64,
        revenue: f64,
        stock: f64,
        sellers: usize,
    }
    let mut rows: BTreeMap<(ProductId, wsim_core::ids::CountryId), Row> = BTreeMap::new();
    for s in &state.sites {
        if state.companies[s.owner.index()].bankrupt {
            continue;
        }
        for sl in &s.slots {
            let Some(r) = sl.recipe.map(|r| catalog.recipes.get(r)) else {
                continue;
            };
            if sl.ready > state.date {
                continue;
            }
            let planned = catalog.facilities.get(sl.facility).runs_per_day
                * f64::from(sl.count)
                * sl.utilization;
            for &(p, q) in &r.inputs {
                rows.entry((p, s.country)).or_default().need += q * planned * 30.0;
            }
            rows.entry((r.product, s.country)).or_default().produced +=
                sl.last_runs * r.output * 30.0;
        }
        for (&p, o) in &s.offers {
            let row = rows.entry((p, s.country)).or_default();
            row.sold += o.sold_last_month;
            row.revenue += o.sold_last_month * usd(o.price);
            row.sellers += 1;
        }
        for (&p, st) in &s.inventory {
            rows.entry((p, s.country)).or_default().stock += st.quantity;
        }
    }
    let mut csv = String::from(
        "produkt;land;produktion_monat;bedarf_vorprodukt_monat;konsumnachfrage_monat;absatz_monat;preis_usd;richtpreis_usd;anbieter;lager;einfuhr;ausfuhr\n",
    );
    for ((p, c), r) in &rows {
        let market = state.markets.get(*p).get(*c);
        let t = &market.last_month;
        let demand = if outside_demand(catalog.products.get(*p)) {
            t.demand
        } else {
            0.0
        };
        if r.produced + r.need + r.sold + demand <= 1e-6 {
            continue;
        }
        let price = if r.sold > 0.0 {
            r.revenue / r.sold
        } else {
            0.0
        };
        let _ = writeln!(
            csv,
            "{};{};{:.0};{:.0};{:.0};{:.0};{:.2};{:.2};{};{:.0};{:.0};{:.0}",
            catalog.products.key(*p),
            catalog.countries.key(*c),
            r.produced,
            r.need,
            demand,
            r.sold,
            price,
            usd(market::local_reference(catalog, state, *c, *p)),
            r.sellers,
            r.stock,
            t.imported,
            t.exported
        );
    }
    csv
}

/// Whether consumers or governments buy the product; the markets' demand of other goods
/// is the purchase orders of companies, counted as input need instead.
fn outside_demand(p: &wsim_core::catalog::Product) -> bool {
    p.consumer_demand.is_some() || p.state_demand.is_some()
}
