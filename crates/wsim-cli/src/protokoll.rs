//! Balance protocol of a world run (Architektur §4, M15): world markets month by month,
//! summed up per year with products and companies, and the anomalies that matter for
//! balancing. Writes `produkte.csv`, `firmen.csv` and `auswertung.md`.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use wsim_core::game::Game;
use wsim_core::ids::{CountryId, ProductId};
use wsim_core::ledger::Account;
use wsim_core::market;
use wsim_core::state::{CompanyId, ManagerId, Role, Unit};
use wsim_data::Texts;

/// Thresholds of the anomalies (only for the protocol, not for the game).
const EXPENSIVE: f64 = 1.5;
const CHEAP: f64 = 0.6;
const SHORTAGE: f64 = 0.85;
const OVERCAPACITY: f64 = 2.0;
const RICH_MARGIN: f64 = 0.4;
const LOSS_MARGIN: f64 = -0.1;

/// Plausibility limits (M16): what a believable world keeps to, for every product and
/// country alike.
const COVERAGE_MIN: f64 = 0.9;
/// Countries count from this share of a product's world demand on.
const COUNTRY_SHARE_MIN: f64 = 0.005;
const COUNTRY_COVERAGE_MIN: f64 = 0.75;
const INPUT_SHORTAGE_MAX: f64 = 0.1;
const PRICE_BAND: (f64, f64) = (0.5, 2.0);
const MARGIN_BAND: (f64, f64) = (-0.2, 0.5);
/// The player's workshop runs without any decision: its yearly result against its
/// starting equity.
const PLAYER_RETURN_MAX: f64 = 0.5;
const BANKRUPT_PER_YEAR_MAX: f64 = 0.05;
const AI_ACTIVE_MIN: f64 = 0.8;

/// A violated plausibility limit: subject (product, country), year and value.
type Finding = (String, i32, String);

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
    /// Demand of consumers and governments and what they got, worldwide and per country.
    outside_demand: f64,
    outside_sold: f64,
    countries: BTreeMap<CountryId, (f64, f64)>,
    /// Planned output, the part held back by missing inputs, and the full cost of the
    /// facilities at their planned utilization.
    planned: f64,
    input_limited: f64,
    labor_limited: f64,
    cost_usd: f64,
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
    /// Share of the consumers' and governments' demand that was served.
    coverage: Option<f64>,
    /// Relevant countries served below `COUNTRY_COVERAGE_MIN`, with their coverage.
    undersupplied: Vec<(CountryId, f64)>,
    /// Share of the planned output held back by missing inputs and by missing workers.
    input_shortage: f64,
    labor_shortage: f64,
    /// Margin of the selling price over the full unit cost (inputs at market prices,
    /// wages, electricity, depreciation and maintenance).
    margin: Option<f64>,
    flags: Vec<&'static str>,
}

struct CompanyYear {
    year: i32,
    active: usize,
    bankrupt: usize,
    founded: usize,
    /// Sites of insolvent companies sold in their auction and given up after it (M38).
    auction_sold: usize,
    auction_given_up: usize,
    profitable: usize,
    median_equity_usd: f64,
    player_equity_usd: f64,
    player_cash_usd: f64,
    player_result_usd: f64,
}

/// The market of managers at a year end (MA6).
struct ManagerYear {
    year: i32,
    /// AI companies with a CEO, and heads of AI sites.
    ai_ceos: usize,
    ai_heads: usize,
    /// Mean strength of the AI CEOs.
    ceo_strength: f64,
    /// What managers add to the AI companies' competence: mean over all active AI
    /// companies and the largest.
    staff_mean: f64,
    staff_max: f64,
    /// Managers who changed their employer, and jobs that ended, in the year.
    moved: usize,
    left: usize,
    /// Employed managers, their mean satisfaction and those below the resignation
    /// threshold.
    employed: usize,
    satisfaction: f64,
    unhappy: usize,
    /// Yearly salaries of the AI companies' managers at the year end.
    ai_salaries_usd: f64,
}

#[derive(Default)]
pub struct Protocol {
    month: Option<(i32, u32)>,
    year: BTreeMap<ProductId, Month>,
    companies_before: usize,
    /// AI companies at the start and the player's equity at the start.
    ai_at_start: usize,
    player_start_equity_usd: f64,
    products: Vec<ProductYear>,
    companies: Vec<CompanyYear>,
    /// Owner of each site and the companies in auction at the last month end (M38).
    owners: Vec<CompanyId>,
    in_auction: Vec<CompanyId>,
    auction_sold: usize,
    auction_given_up: usize,
    /// Sites each company bought in auctions, and the bankrupt ones seen so far.
    auction_buys: BTreeMap<CompanyId, usize>,
    bankrupt_seen: Vec<CompanyId>,
    /// Bankruptcies of companies that had bought sites in auctions: (year, company, sites).
    bankrupt_buyers: Vec<(i32, String, usize)>,
    /// The employer of every employed manager at the last month end, and the changes of
    /// the year so far (MA6).
    jobs: BTreeMap<ManagerId, CompanyId>,
    moved: usize,
    left: usize,
    managers: Vec<ManagerYear>,
}

fn usd(m: wsim_core::money::Money) -> f64 {
    m.to_usd()
}

impl Protocol {
    pub fn new(game: &Game) -> Self {
        let state = game.state();
        let player = &state.companies[game.player().index()];
        Self {
            month: Some((game.date().year(), game.date().month())),
            companies_before: state.companies.len(),
            ai_at_start: state.companies.iter().filter(|c| c.ai.is_some()).count(),
            player_start_equity_usd: usd(
                player.ledger.total_assets() - player.ledger.balance(Account::Loans)
            ),
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

    /// Sites of insolvent companies that changed hands in an auction, and those left
    /// when an auction ended (M38).
    fn count_auctions(&mut self, game: &Game) {
        let state = game.state();
        for (site, &before) in self.owners.iter().enumerate() {
            let now = state.sites[site].owner;
            if now != before && state.companies[before.index()].bankrupt {
                self.auction_sold += 1;
                *self.auction_buys.entry(now).or_default() += 1;
            }
        }
        for (i, c) in state.companies.iter().enumerate() {
            let id = CompanyId(u32::try_from(i).expect("company count fits u32"));
            if c.bankrupt && !self.bankrupt_seen.contains(&id) {
                self.bankrupt_seen.push(id);
                if let Some(&n) = self.auction_buys.get(&id) {
                    self.bankrupt_buyers
                        .push((state.date.year(), c.name.clone(), n));
                }
            }
        }
        for &c in &self.in_auction {
            if state.companies[c.index()].auction_until.is_none() {
                self.auction_given_up += state.sites.iter().filter(|s| s.owner == c).count();
            }
        }
        self.owners = state.sites.iter().map(|s| s.owner).collect();
        self.in_auction = (0..state.companies.len())
            .filter(|&i| state.companies[i].auction_until.is_some())
            .map(|i| CompanyId(u32::try_from(i).expect("company count fits u32")))
            .collect();
    }

    /// Managers who changed their employer or lost their job since the last month (MA6).
    fn count_jobs(&mut self, game: &Game) {
        let state = game.state();
        let now: BTreeMap<ManagerId, CompanyId> = state
            .managers
            .iter()
            .filter_map(|(&id, m)| m.job.as_ref().map(|j| (id, j.company)))
            .collect();
        for (id, before) in &self.jobs {
            match now.get(id) {
                Some(c) if c != before => self.moved += 1,
                Some(_) => {}
                None => self.left += 1,
            }
        }
        self.jobs = now;
    }

    fn add_month(&mut self, game: &Game) {
        self.count_auctions(game);
        self.count_jobs(game);
        let state = game.state();
        let catalog = game.catalog();
        let closed = state.date.add_days(-1);
        let days = f64::from(wsim_core::calendar::days_in_month(
            closed.year(),
            closed.month(),
        ));
        for h in wsim_core::health::last_month(state, catalog) {
            let acc = self.year.entry(h.product).or_default();
            acc.outside_demand += h.outside_demand;
            acc.outside_sold += h.outside_sold;
            for &(country, demand, sold) in &h.countries {
                let e = acc.countries.entry(country).or_default();
                e.0 += demand;
                e.1 += sold;
            }
            acc.planned += h.planned * days;
            acc.input_limited += h.input_limited * days;
            acc.labor_limited += h.labor_limited * days;
            acc.cost_usd += h.cost_usd * days;
        }
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
                let planned = sl.full_runs(catalog) * sl.utilization;
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
                    // Units standing still do not count (C4): they hid the real load.
                    if r.product == product && sl.ready <= state.date && !sl.mothballed() {
                        capacity += sl.full_runs(catalog) * r.output * 365.0;
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
            let margin = match price_usd {
                Some(price) if m.planned > 1e-9 && price > 0.0 => {
                    Some(1.0 - m.cost_usd / m.planned / price)
                }
                _ => None,
            };
            let mut flags = Vec::new();
            if let (Some(p), Some(r)) = (price_usd, reference_usd) {
                if p > EXPENSIVE * r {
                    flags.push("teuer");
                } else if p < CHEAP * r {
                    flags.push("billig");
                }
            }
            // Consumer goods: demand of consumers and governments that stayed open;
            // inputs: need the facilities could not cover from the world's production.
            let coverage = (m.outside_demand > 1e-9).then(|| m.outside_sold / m.outside_demand);
            let short = if let Some(c) = coverage {
                c < SHORTAGE
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
            if let Some(margin) = margin {
                if margin > RICH_MARGIN {
                    flags.push("sehr profitabel");
                } else if margin < LOSS_MARGIN {
                    flags.push("Verlust");
                }
            }
            let undersupplied = m
                .countries
                .iter()
                .filter(|(_, (d, _))| *d >= COUNTRY_SHARE_MIN * m.outside_demand)
                .map(|(&c, &(d, s))| (c, s / d.max(1e-9)))
                .filter(|&(_, c)| c < COUNTRY_COVERAGE_MIN)
                .collect();
            let (input_shortage, labor_shortage) = if m.planned > 1e-9 {
                (m.input_limited / m.planned, m.labor_limited / m.planned)
            } else {
                (0.0, 0.0)
            };
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
                coverage,
                undersupplied,
                input_shortage,
                labor_shortage,
                margin,
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
            auction_sold: std::mem::take(&mut self.auction_sold),
            auction_given_up: std::mem::take(&mut self.auction_given_up),
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
        self.close_manager_year(game, year);
    }

    fn close_manager_year(&mut self, game: &Game, year: i32) {
        let state = game.state();
        let catalog = game.catalog();
        let ai = |c: CompanyId| state.companies[c.index()].ai.is_some();
        let mut ceos = Vec::new();
        let (mut heads, mut employed, mut unhappy) = (0, 0, 0);
        let (mut satisfaction, mut salaries) = (0.0, 0.0);
        let threshold = catalog.management.market.resignation_threshold;
        for m in state.managers.values() {
            let Some(job) = &m.job else {
                continue;
            };
            employed += 1;
            let value = wsim_core::staffing::satisfaction(catalog, job);
            satisfaction += f64::from(value);
            if value < threshold {
                unhappy += 1;
            }
            if !ai(job.company) {
                continue;
            }
            salaries += usd(job.salary);
            match (job.position.unit, &job.position.role) {
                (Unit::Board, Role::Head) => ceos.push(wsim_core::management::strength(m)),
                (Unit::Site(_), Role::Head) => heads += 1,
                _ => {}
            }
        }
        let staff: Vec<f64> = state
            .companies
            .iter()
            .filter(|c| !c.bankrupt)
            .filter_map(|c| c.ai.as_ref().map(|a| a.staff))
            .collect();
        self.managers.push(ManagerYear {
            year,
            ai_ceos: ceos.len(),
            ai_heads: heads,
            ceo_strength: if ceos.is_empty() {
                0.0
            } else {
                ceos.iter().sum::<f64>() / ceos.len() as f64
            },
            staff_mean: if staff.is_empty() {
                0.0
            } else {
                staff.iter().sum::<f64>() / staff.len() as f64
            },
            staff_max: staff.iter().copied().fold(0.0, f64::max),
            moved: std::mem::take(&mut self.moved),
            left: std::mem::take(&mut self.left),
            employed,
            satisfaction: if employed == 0 {
                0.0
            } else {
                satisfaction / employed as f64
            },
            unhappy,
            ai_salaries_usd: salaries,
        });
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
            "jahr;produkt;bedarf;konsumnachfrage;produktion;absatz_firmen;preis_usd;richtpreis_usd;kapazitaet;hersteller;lager;ergebnis_usd;einfuhr;versorgung;engpass_vorprodukte;marge_vollkosten;auffaellig\n",
        );
        for p in &self.products {
            let _ = writeln!(
                csv,
                "{};{};{:.0};{:.0};{:.0};{:.0};{};{};{:.0};{};{:.0};{:.0};{:.0};{};{:.3};{};{}",
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
                opt(p.coverage),
                p.input_shortage,
                opt(p.margin),
                p.flags.join(",")
            );
        }
        write(dir, "produkte.csv", &csv)?;

        let mut csv = String::from(
            "jahr;ki_aktiv;ki_pleite;gegruendet;versteigert;aufgegeben;mit_gewinn;median_eigenkapital_usd;spieler_eigenkapital_usd;spieler_kasse_usd;spieler_ergebnis_usd\n",
        );
        for c in &self.companies {
            let _ = writeln!(
                csv,
                "{};{};{};{};{};{};{};{:.0};{:.0};{:.0};{:.0}",
                c.year,
                c.active,
                c.bankrupt,
                c.founded,
                c.auction_sold,
                c.auction_given_up,
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
        let (section, passed, total) = self.plausibility(game, &name);
        md.push_str(&section);
        md.push_str("## Firmen je Jahr\n\nStandorte insolventer Firmen: versteigert bzw. nach der Versteigerung aufgegeben (M38).\n\n| Jahr | KI aktiv | pleite (gesamt) | gegründet | versteigert | aufgegeben | mit Gewinn | Median Eigenkapital | Spieler Eigenkapital | Spieler Ergebnis |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");
        for c in &self.companies {
            let _ = writeln!(
                md,
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                c.year,
                c.active,
                c.bankrupt,
                c.founded,
                c.auction_sold,
                c.auction_given_up,
                c.profitable,
                money(c.median_equity_usd),
                money(c.player_equity_usd),
                money(c.player_result_usd)
            );
        }
        if !self.bankrupt_buyers.is_empty() {
            let _ = writeln!(
                md,
                "\nPleiten von Firmen, die vorher Standorte ersteigert hatten (M38): {} von {}.\n",
                self.bankrupt_buyers.len(),
                self.bankrupt_seen.len()
            );
            for (year, name, n) in &self.bankrupt_buyers {
                let _ = writeln!(md, "- {year}: {name} ({n} ersteigerte Standorte)");
            }
        }
        if self
            .managers
            .iter()
            .any(|y| y.ai_ceos + y.ai_heads + y.moved + y.left > 0)
        {
            md.push_str("\n## Manager (MA6)\n\nKI-Firmen stellen CEO und Standortleitungen ein; Manager heben ihre Kompetenz (Mittel und Höchstwert über die aktiven KI-Firmen). Wechsel: Manager mit neuem Arbeitgeber (Abwerbung); beendet: Stellen, die endeten (Kündigung, Entlassung, Pleite, Standortverkauf).\n\n| Jahr | KI mit CEO | Stärke CEO | KI-Leitungen | Kompetenz + (Mittel/max) | Gehälter KI | angestellt | Zufriedenheit | unter Schwelle | Wechsel | beendet |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");
            for y in &self.managers {
                let _ = writeln!(
                    md,
                    "| {} | {} | {:.0} | {} | {:.3} / {:.3} | {} | {} | {:.0} | {} | {} | {} |",
                    y.year,
                    y.ai_ceos,
                    y.ceo_strength,
                    y.ai_heads,
                    y.staff_mean,
                    y.staff_max,
                    money(y.ai_salaries_usd),
                    y.employed,
                    y.satisfaction,
                    y.unhappy,
                    y.moved,
                    y.left
                );
            }
        }
        md.push_str("\n## Auffälligkeiten je Produkt\n\nJahre mit Auffälligkeit (teuer > 1,5 × Richtpreis, billig < 0,6 ×, Mangel < 85 % der Nachfrage von Verbrauchern und Staaten bzw. des Vorproduktbedarfs gedeckt, Überkapazität > 2 × Bedarf, sehr profitabel > 40 % Marge über Vollkosten, Verlust < −10 %).\n\n| Produkt | Jahre | teuer | billig | Mangel | Überkapazität | sehr profitabel | Verlust |\n| --- | --- | --- | --- | --- | --- | --- | --- |\n");
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
                let margin = p
                    .margin
                    .map_or("–".into(), |m| format!("{:.0} %", 100.0 * m));
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
        println!("Plausibilität: {passed} von {total} Prüfungen ohne Verstoß");
        Ok(())
    }

    /// The plausibility checks over all products, countries and years (Markdown), and
    /// how many of them passed.
    fn plausibility(
        &self,
        game: &Game,
        name: &dyn Fn(ProductId) -> String,
    ) -> (String, usize, usize) {
        let catalog = game.catalog();
        let pct = |v: f64| format!("{:.0} %", v * 100.0);
        // Per check: (title, limit, findings).
        let mut checks: Vec<(&str, String, Vec<Finding>)> = Vec::new();

        let mut found = Vec::new();
        for p in &self.products {
            if let Some(c) = p.coverage.filter(|&c| c < COVERAGE_MIN) {
                found.push((name(p.product), p.year, pct(c)));
            }
        }
        checks.push((
            "Versorgung von Verbrauchern und Staaten weltweit",
            format!("≥ {}", pct(COVERAGE_MIN)),
            found,
        ));

        let mut found = Vec::new();
        for p in &self.products {
            for &(country, c) in &p.undersupplied {
                found.push((
                    format!("{} in {}", name(p.product), catalog.countries.key(country)),
                    p.year,
                    pct(c),
                ));
            }
        }
        checks.push((
            "Versorgung je Land (ab 0,5 % der Weltnachfrage)",
            format!("≥ {}", pct(COUNTRY_COVERAGE_MIN)),
            found,
        ));

        let mut found = Vec::new();
        for p in &self.products {
            if p.input_shortage > INPUT_SHORTAGE_MAX {
                found.push((name(p.product), p.year, pct(p.input_shortage)));
            }
        }
        checks.push((
            "Erzeugung, die auf fehlende Vorprodukte wartet",
            format!("≤ {}", pct(INPUT_SHORTAGE_MAX)),
            found,
        ));

        let mut found = Vec::new();
        for p in &self.products {
            if p.labor_shortage > INPUT_SHORTAGE_MAX {
                found.push((name(p.product), p.year, pct(p.labor_shortage)));
            }
        }
        checks.push((
            "Erzeugung, die auf fehlende Arbeitskräfte wartet",
            format!("≤ {}", pct(INPUT_SHORTAGE_MAX)),
            found,
        ));

        let mut found = Vec::new();
        for p in &self.products {
            // Only made as a by-product (petrol before cracking): its price follows the
            // main product's output, a glut of it is cheap.
            if !wsim_core::health::made_as_main(catalog, p.product, p.year) {
                continue;
            }
            if let (Some(price), Some(reference)) = (p.price_usd, p.reference_usd) {
                let ratio = price / reference.max(1e-9);
                if !(PRICE_BAND.0..=PRICE_BAND.1).contains(&ratio) {
                    found.push((name(p.product), p.year, format!("{ratio:.2} ×")));
                }
            }
        }
        checks.push((
            "Preis gegen Richtpreis",
            format!("{} bis {} ×", PRICE_BAND.0, PRICE_BAND.1),
            found,
        ));

        let mut found = Vec::new();
        for p in &self.products {
            // A by-product given away has no margin of its own.
            let main = catalog.recipes.values().any(|r| r.product == p.product);
            if let Some(margin) = p.margin.filter(|_| main)
                && !(MARGIN_BAND.0..=MARGIN_BAND.1).contains(&margin)
            {
                found.push((name(p.product), p.year, pct(margin)));
            }
        }
        checks.push((
            "Marge über Vollkosten (Vorprodukte zu Marktpreisen, Löhne, Strom, Anlagen)",
            format!("{} bis {}", pct(MARGIN_BAND.0), pct(MARGIN_BAND.1)),
            found,
        ));

        let mut found = Vec::new();
        for c in &self.companies {
            let ratio = c.player_result_usd / self.player_start_equity_usd.max(1.0);
            if ratio > PLAYER_RETURN_MAX {
                found.push((
                    "Werkstatt ohne Entscheidungen".to_owned(),
                    c.year,
                    pct(ratio),
                ));
            }
        }
        checks.push((
            "Jahresergebnis des passiven Spielers gegen sein Startkapital",
            format!("≤ {}", pct(PLAYER_RETURN_MAX)),
            found,
        ));

        let mut found = Vec::new();
        let mut before = 0;
        let mut active_before = self.ai_at_start;
        for c in &self.companies {
            let failed = c.bankrupt.saturating_sub(before);
            let rate = failed as f64 / active_before.max(1) as f64;
            if rate > BANKRUPT_PER_YEAR_MAX {
                found.push(("KI-Firmen pleite".to_owned(), c.year, pct(rate)));
            }
            if (c.active as f64) < AI_ACTIVE_MIN * self.ai_at_start as f64 {
                found.push(("KI-Firmen aktiv".to_owned(), c.year, c.active.to_string()));
            }
            before = c.bankrupt;
            active_before = c.active;
        }
        checks.push((
            "Pleiten je Jahr und aktive KI-Firmen",
            format!(
                "≤ {} je Jahr, ≥ {} der Startzahl",
                pct(BANKRUPT_PER_YEAR_MAX),
                pct(AI_ACTIVE_MIN)
            ),
            found,
        ));

        let mut found = Vec::new();
        for p in &self.products {
            let extracted = catalog
                .recipes
                .values()
                .any(|r| r.product == p.product && r.extraction);
            let need = p.demand - p.consumer_demand;
            if extracted && need > 0.0 && p.produced < COVERAGE_MIN * need {
                found.push((name(p.product), p.year, pct(p.produced / need)));
            }
        }
        checks.push((
            "Förderung von Rohstoffen gegen den Bedarf der Anlagen",
            format!("≥ {}", pct(COVERAGE_MIN)),
            found,
        ));

        let total = checks.len();
        let passed = checks.iter().filter(|c| c.2.is_empty()).count();
        let mut md = format!(
            "## Plausibilität\n\n{passed} von {total} Prüfungen ohne Verstoß. Die Grenzen gelten für alle Produkte und Länder gleich.\n\n| Prüfung | Grenze | Verstöße | Betroffen (Jahre; schlechtester Wert) |\n| --- | --- | --- | --- |\n"
        );
        for (title, limit, found) in &checks {
            let mut by_subject: BTreeMap<&str, (Vec<i32>, &str)> = BTreeMap::new();
            for (subject, year, value) in found {
                let e = by_subject
                    .entry(subject.as_str())
                    .or_insert((Vec::new(), value.as_str()));
                e.0.push(*year);
                e.1 = value.as_str();
            }
            let shown: Vec<String> = by_subject
                .iter()
                .take(12)
                .map(|(s, (years, value))| format!("{s} ({}; {value})", year_ranges(years)))
                .collect();
            let more = by_subject.len().saturating_sub(12);
            let _ = writeln!(
                md,
                "| {title} | {limit} | {} | {}{} |",
                found.len(),
                if shown.is_empty() {
                    "–".to_owned()
                } else {
                    shown.join(", ")
                },
                if more > 0 {
                    format!(" und {more} weitere")
                } else {
                    String::new()
                }
            );
        }
        md.push('\n');
        (md, passed, total)
    }
}

/// Years as ranges: 1901–1903, 1905.
fn year_ranges(years: &[i32]) -> String {
    let mut sorted = years.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    let mut parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < sorted.len() {
        let mut j = i;
        while j + 1 < sorted.len() && sorted[j + 1] == sorted[j] + 1 {
            j += 1;
        }
        parts.push(if j > i {
            format!("{}–{}", sorted[i], sorted[j])
        } else {
            sorted[i].to_string()
        });
        i = j + 1;
    }
    parts.join(", ")
}

fn money(v: f64) -> String {
    wsim_data::format_number(v, 0) + " USD"
}

fn write(dir: &Path, name: &str, text: &str) -> Result<(), String> {
    let path = dir.join(name);
    fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Unit cost and margin of every recipe at reference prices in some countries (the
/// same calculation as the plausibility check of the data, docs/FORMELN.md).
pub fn recipe_margins(
    catalog: &wsim_core::catalog::Catalog,
    texts: &Texts,
    year: i32,
    countries: &[wsim_core::ids::CountryId],
) -> String {
    use wsim_core::calendar::Date;
    use wsim_core::health;
    let values: Vec<_> = countries
        .iter()
        .map(|&c| wsim_core::country_model::compute(catalog, c, Date::first_of_year(year)))
        .collect();
    let (low, high) = catalog.production_model.reference_margin;
    let text = |key: String| texts.get(&key).map_or(key.clone(), str::to_owned);
    let mut md = format!(
        "# Rezeptmargen {year}\n\nStückkosten: Vorprodukte zu Richtpreisen im Land, Arbeitsstunden \
         zu den Löhnen und der Produktivität des Landes, Strom zum Landespreis, Anlage bei \
         Normalauslastung mit Abschreibung und Instandhaltung, Gemeinkosten; Nebenprodukte zu \
         Richtpreisen abgezogen. Marge = 1 − Kosten/Richtpreis im Land; ⚠ außerhalb von {:.0} \
         bis {:.0} %.\n\n| Rezept | Produkt | erfunden | Richtpreis |",
        low * 100.0,
        high * 100.0
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
        let year = health::first_year(catalog, id);
        let _ = write!(
            md,
            "| {} | {} | {} | {} |",
            text(format!("rezept.{}", catalog.recipes.key(id))),
            text(format!("produkt.{}", catalog.products.key(r.product))),
            if year > wsim_core::EARLIEST_START_YEAR {
                year.to_string()
            } else {
                "–".into()
            },
            money(catalog.products.get(r.product).reference_price.to_usd())
        );
        let mut labor_share = 0.0;
        for v in &values {
            let price = |p| {
                catalog.products.get(p).reference_price.to_usd()
                    * wsim_core::market::level_factor(catalog, v.price_level, p)
            };
            let utilization = catalog.market_model.normal_utilization;
            let cost = health::unit_cost(catalog, v, id, utilization, price);
            labor_share = cost.labor / cost.total().max(1e-9);
            let margin = health::margin_in(catalog, v, id).unwrap_or(0.0);
            let mark = if (low..=high).contains(&margin) {
                ""
            } else {
                " ⚠"
            };
            let _ = write!(
                md,
                " {} | {:.0} %{} |",
                money(cost.total()),
                margin * 100.0,
                mark
            );
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
            let planned = sl.full_runs(catalog) * sl.utilization;
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
