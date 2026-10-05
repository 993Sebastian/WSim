//! Views of the play screens (Lastenheft §14.1): production with its causes, the
//! options to build, markets, research and finances. Like all views they hold keys
//! and finished numbers only.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{iso, usd};
use crate::catalog::SiteType;
use crate::command::site_type_key;
use crate::finance;
use crate::game::Game;
use crate::ids::{CountryId, GoodsGroupId, Id};
use crate::market;
use crate::money::Money;
use crate::reports;
use crate::research;
use crate::state::{CompanyId, Limit, PriceMode};

/// Why a facility made less than planned on the last day, as text key and product.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cause {
    /// `ursache.vorprodukt`, `ursache.arbeitskraefte`, `ursache.strom`,
    /// `ursache.lagerstaette`, `ursache.im_bau`, `ursache.erschliessung`,
    /// `ursache.kein_rezept`, `ursache.ruht`.
    pub key: String,
    /// Missing input or labor group.
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SlotDetail {
    pub index: usize,
    pub facility: String,
    pub count: u32,
    pub recipe: Option<String>,
    pub product: Option<String>,
    pub utilization: f64,
    pub automation: f64,
    pub automation_max: f64,
    pub ready: String,
    /// Output per day at the planned utilization.
    pub planned_per_day: f64,
    /// Output on the last day.
    pub made_per_day: f64,
    pub cause: Option<Cause>,
    /// Condition of the facilities (1 = new).
    pub condition: f64,
    /// Inputs per day at the planned utilization.
    pub inputs_per_day: Vec<(String, f64)>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfferView {
    pub product: String,
    /// `markt` or `fest`
    pub mode: String,
    pub price_usd: f64,
    pub floor_usd: f64,
    pub markup: f64,
    pub keep: f64,
    /// Sold in the running month.
    pub sold_month: f64,
    pub sold_last_month: f64,
    /// In stock at the site.
    pub stock: f64,
    /// Price index of the product in the site's country.
    pub market_price_usd: f64,
    /// Reference price in the site's country.
    pub reference_usd: f64,
    /// Expected cost per unit: of making it here, else of what is in stock.
    pub unit_cost_usd: Option<f64>,
    /// (price − unit cost) / price.
    pub margin: Option<f64>,
    /// The site uses the product itself (an input of its facilities).
    pub used_here: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrderView {
    pub product: String,
    pub target: f64,
    pub max_price_usd: f64,
    pub min_quality: f64,
    /// Bought in the running month.
    pub bought_month: f64,
    pub bought_last_month: f64,
}

/// An input of the site's production: daily need, stock and how long it lasts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputSupply {
    pub product: String,
    pub need_per_day: f64,
    pub stock: f64,
    /// Days the stock lasts at the planned production (`None` without need).
    pub days: Option<f64>,
    /// Made at the same site.
    pub own: bool,
    /// Bought by a purchase order of the site.
    pub ordered: bool,
    /// Price index in the site's country.
    pub market_price_usd: f64,
}

/// A labor group at a site (M18): what production needs, who works there, who is free
/// in the country, and the wage the site pays.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StaffLine {
    pub group: String,
    pub needed: f64,
    pub employed: f64,
    /// Workers of the group in the country that no site employs.
    pub free_in_country: f64,
    /// Wage per hour at the site, with its premium.
    pub wage_usd: f64,
    /// Wage per hour in the country.
    pub country_wage_usd: f64,
}

/// Expected cost of one unit of a product made at the site, by kind (USD per unit).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnitCostView {
    pub product: String,
    pub output_per_day: f64,
    pub material_usd: f64,
    pub labor_usd: f64,
    pub energy_usd: f64,
    pub overhead_usd: f64,
    pub rent_usd: f64,
    /// Depreciation and maintenance of the facilities.
    pub facility_usd: f64,
    pub total_usd: f64,
    /// Without the facilities: what one more unit costs.
    pub variable_usd: f64,
}

/// An amount by cost type, e.g. `kostenart.material` (income positive).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResultLine {
    pub key: String,
    pub usd: f64,
}

/// Revenue and gross margin of a product at a site (M18): revenue less the production
/// cost of what was sold.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProductResult {
    pub product: String,
    pub revenue_usd: f64,
    pub margin_usd: f64,
}

/// A site's result in the last closed month (M18).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SiteResult {
    /// First day of the month.
    pub month: String,
    pub revenue_usd: f64,
    /// Everything but revenue by cost type, in the order of the income statement.
    pub lines: Vec<ResultLine>,
    pub result_usd: f64,
    pub products: Vec<ProductResult>,
}

/// A deposit an extraction site without deposit could develop.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DepositOption {
    pub key: String,
    pub resource: String,
    pub cost_usd: f64,
    pub days: u32,
    /// Yearly output of the free field.
    pub output_per_year: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SiteDetail {
    pub index: u32,
    pub country: String,
    /// For commands, e.g. `Factory`.
    pub kind: SiteType,
    /// Text key, e.g. `standorttyp.werk`.
    pub kind_text: String,
    pub deposit: Option<String>,
    /// Day the development of the site's deposit field is finished.
    pub deposit_ready: Option<String>,
    /// Deposits to develop (extraction sites without deposit only).
    pub free_deposits: Vec<DepositOption>,
    pub workers: f64,
    pub slots: Vec<SlotDetail>,
    pub stock: Vec<super::StockView>,
    pub offers: Vec<OfferView>,
    pub orders: Vec<OrderView>,
    pub inputs: Vec<InputSupply>,
    pub research: Option<String>,
    /// Premium over the country's wages (0.1 = 10 %) and its highest allowed value.
    pub wage_premium: f64,
    pub wage_premium_max: f64,
    /// Highest premium another company pays at a site in the same country.
    pub rival_premium_max: f64,
    pub staff: Vec<StaffLine>,
    /// Wages per day of the staff employed now.
    pub wage_cost_per_day_usd: f64,
    pub unit_costs: Vec<UnitCostView>,
    /// Result of the last closed month, if the site existed then.
    pub last_month: Option<SiteResult>,
    /// Prices in the site's country of the products it makes, uses, keeps, buys or sells.
    pub prices: BTreeMap<String, PriceInfo>,
}

/// Price index and reference price of a product in a country.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PriceInfo {
    pub market_usd: f64,
    pub reference_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SiteTypeOption {
    pub kind: SiteType,
    pub kind_text: String,
    pub cost_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FacilityOption {
    pub key: String,
    pub site_type: SiteType,
    pub investment_usd: f64,
    pub build_days: u32,
    pub runs_per_day: f64,
    pub automation_max: f64,
    /// Recipes of this facility the company knows.
    pub recipes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecipeOption {
    pub key: String,
    pub facility: String,
    pub product: String,
    pub output: f64,
    /// Output of one facility per day at full utilization, and the inputs it takes.
    pub output_per_day: f64,
    pub inputs_per_day: Vec<(String, f64)>,
    pub duration_days: u32,
    pub extraction: bool,
    pub inputs: Vec<(String, f64)>,
    pub labor_hours: Vec<(String, f64)>,
    pub energy_mwh: f64,
}

/// The player's sites and what the player could build (technologies known today).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProductionView {
    pub date: String,
    pub cash_usd: f64,
    pub sites: Vec<SiteDetail>,
    pub site_types: Vec<SiteTypeOption>,
    pub facilities: Vec<FacilityOption>,
    pub recipes: Vec<RecipeOption>,
    /// All products (for new purchase orders and offers).
    pub products: Vec<String>,
    /// Unit of each product (key of `einheit.<key>`).
    pub units: BTreeMap<String, String>,
}

pub fn production(game: &Game) -> ProductionView {
    let state = game.state();
    let catalog = game.catalog();
    let player = state.player;
    let knows =
        |t: Option<crate::ids::TechnologyId>| t.is_none_or(|t| state.knows(catalog, player, t));
    let recipes: Vec<RecipeOption> = catalog
        .recipes
        .iter()
        .filter(|(_, r)| {
            knows(r.technology) && knows(catalog.facilities.get(r.facility).technology)
        })
        .map(|(id, r)| RecipeOption {
            key: catalog.recipes.key(id).to_owned(),
            facility: catalog.facilities.key(r.facility).to_owned(),
            product: catalog.products.key(r.product).to_owned(),
            output: r.output,
            output_per_day: r.output * catalog.facilities.get(r.facility).runs_per_day,
            inputs_per_day: r
                .inputs
                .iter()
                .map(|&(p, q)| {
                    (
                        catalog.products.key(p).to_owned(),
                        q * catalog.facilities.get(r.facility).runs_per_day,
                    )
                })
                .collect(),
            duration_days: r.duration_days,
            extraction: r.extraction,
            inputs: r
                .inputs
                .iter()
                .map(|&(p, q)| (catalog.products.key(p).to_owned(), q))
                .collect(),
            labor_hours: r
                .labor_hours
                .iter()
                .map(|&(g, h)| (catalog.labor_groups.key(g).to_owned(), h))
                .collect(),
            energy_mwh: r.energy_mwh,
        })
        .collect();
    let facilities = catalog
        .facilities
        .iter()
        .filter(|(_, f)| knows(f.technology))
        .map(|(id, f)| {
            let key = catalog.facilities.key(id).to_owned();
            FacilityOption {
                recipes: recipes
                    .iter()
                    .filter(|r| r.facility == key)
                    .map(|r| r.key.clone())
                    .collect(),
                key,
                site_type: f.site_type,
                investment_usd: usd(f.investment),
                build_days: f.build_days,
                runs_per_day: f.runs_per_day,
                automation_max: f.automation_max,
            }
        })
        .collect();
    let site_types = [
        SiteType::Extraction,
        SiteType::Factory,
        SiteType::PowerPlant,
        SiteType::Warehouse,
        SiteType::SalesOffice,
        SiteType::ResearchCenter,
    ]
    .into_iter()
    .map(|kind| SiteTypeOption {
        kind,
        kind_text: site_type_key(kind),
        cost_usd: usd(catalog.production_model.site_cost(kind)),
    })
    .collect();

    let sites = state
        .sites
        .iter()
        .enumerate()
        .filter(|(_, s)| s.owner == player)
        .map(|(i, s)| {
            let site_id = crate::state::SiteId(u32::try_from(i).unwrap_or(u32::MAX));
            let concession = s.deposit.and_then(|d| {
                state
                    .deposits
                    .get(d)
                    .concessions
                    .iter()
                    .find(|c| c.site == Some(site_id))
            });
            let developing = concession.is_none_or(|c| c.ready.is_none_or(|r| r > state.date));
            let mut need: Vec<(crate::ids::ProductId, f64)> = Vec::new();
            let slots = s
                .slots
                .iter()
                .enumerate()
                .map(|(index, sl)| {
                    let f = catalog.facilities.get(sl.facility);
                    let recipe = sl.recipe.map(|r| catalog.recipes.get(r));
                    let runs = f.runs_per_day * f64::from(sl.count) * sl.utilization;
                    if let Some(r) = recipe {
                        for &(p, q) in &r.inputs {
                            match need.iter_mut().find(|(x, _)| *x == p) {
                                Some((_, n)) => *n += q * runs,
                                None => need.push((p, q * runs)),
                            }
                        }
                    }
                    let lab = f.site_type == SiteType::ResearchCenter;
                    let cause = if sl.ready > state.date {
                        Some(("ursache.im_bau", None))
                    } else if recipe.is_none() && !lab {
                        Some(("ursache.kein_rezept", None))
                    } else if sl.utilization <= 0.0 {
                        Some(("ursache.ruht", None))
                    } else if recipe.is_some_and(|r| r.extraction) && developing {
                        Some(("ursache.erschliessung", None))
                    } else {
                        sl.limit.map(|l| match l {
                            Limit::Input(p) => (
                                "ursache.vorprodukt",
                                Some(catalog.products.key(p).to_owned()),
                            ),
                            Limit::Labor(g) => (
                                "ursache.arbeitskraefte",
                                Some(catalog.labor_groups.key(g).to_owned()),
                            ),
                            Limit::Electricity => ("ursache.strom", None),
                            Limit::Deposit => ("ursache.lagerstaette", None),
                        })
                    };
                    SlotDetail {
                        index,
                        facility: catalog.facilities.key(sl.facility).to_owned(),
                        count: sl.count,
                        recipe: sl.recipe.map(|r| catalog.recipes.key(r).to_owned()),
                        product: recipe.map(|r| catalog.products.key(r.product).to_owned()),
                        utilization: sl.utilization,
                        automation: sl.automation,
                        automation_max: f.automation_max,
                        ready: iso(sl.ready),
                        planned_per_day: recipe.map_or(0.0, |r| runs * r.output),
                        made_per_day: recipe.map_or(0.0, |r| sl.last_runs * r.output),
                        cause: cause.map(|(key, detail)| Cause {
                            key: key.to_owned(),
                            detail,
                        }),
                        condition: sl.condition,
                        inputs_per_day: recipe.map_or_else(Vec::new, |r| {
                            r.inputs
                                .iter()
                                .map(|&(p, q)| (catalog.products.key(p).to_owned(), q * runs))
                                .collect()
                        }),
                    }
                })
                .collect();
            let made: Vec<crate::ids::ProductId> = s
                .slots
                .iter()
                .filter_map(|sl| sl.recipe.map(|r| catalog.recipes.get(r).product))
                .collect();
            let inputs = need
                .iter()
                .map(|&(p, per_day)| {
                    let stock = s.inventory.get(&p).map_or(0.0, |x| x.quantity);
                    InputSupply {
                        product: catalog.products.key(p).to_owned(),
                        need_per_day: per_day,
                        stock,
                        days: (per_day > 1e-12).then(|| stock / per_day),
                        own: made.contains(&p),
                        ordered: s.orders.contains_key(&p),
                        market_price_usd: usd(market::market_price(catalog, state, s.country, p)),
                    }
                })
                .collect();
            let unit_costs = crate::production::unit_costs(catalog, state, site_id);
            let used_here: Vec<crate::ids::ProductId> = need.iter().map(|&(p, _)| p).collect();
            let (staff, rival_premium_max, wage_cost_per_day_usd) = site_staff(game, site_id);
            SiteDetail {
                index: u32::try_from(i).unwrap_or(u32::MAX),
                country: catalog.countries.key(s.country).to_owned(),
                kind: s.kind,
                kind_text: site_type_key(s.kind),
                deposit: s.deposit.map(|d| catalog.deposits.key(d).to_owned()),
                deposit_ready: concession.and_then(|c| c.ready).map(iso),
                free_deposits: if s.kind == SiteType::Extraction && s.deposit.is_none() {
                    free_deposits(game, s.country)
                } else {
                    Vec::new()
                },
                workers: s.workforce.values().sum(),
                slots,
                stock: s
                    .inventory
                    .iter()
                    .filter(|(_, st)| st.quantity > 1e-9)
                    .map(|(p, st)| super::StockView {
                        product: catalog.products.key(*p).to_owned(),
                        quantity: st.quantity,
                        value_usd: usd(st.value),
                    })
                    .collect(),
                offers: s
                    .offers
                    .iter()
                    .map(|(p, o)| {
                        let (mode, markup, floor) = match o.mode {
                            PriceMode::Fixed(_) => ("fest", 0.0, Money::ZERO),
                            PriceMode::Market { markup, floor } => ("markt", markup, floor),
                        };
                        let stock = s.inventory.get(p);
                        let unit_cost_usd = unit_costs
                            .iter()
                            .find(|u| u.product == *p)
                            .map(crate::production::UnitCost::total)
                            .or_else(|| {
                                stock
                                    .filter(|st| st.quantity > 1e-9)
                                    .map(|st| usd(st.value) / st.quantity)
                            });
                        OfferView {
                            product: catalog.products.key(*p).to_owned(),
                            mode: mode.to_owned(),
                            price_usd: usd(o.price),
                            floor_usd: usd(floor),
                            markup,
                            keep: o.keep,
                            sold_month: o.sold_month,
                            sold_last_month: o.sold_last_month,
                            stock: stock.map_or(0.0, |st| st.quantity),
                            market_price_usd: usd(market::market_price(
                                catalog, state, s.country, *p,
                            )),
                            reference_usd: usd(market::local_reference(
                                catalog, state, s.country, *p,
                            )),
                            margin: unit_cost_usd
                                .filter(|_| o.price > Money::ZERO)
                                .map(|c| 1.0 - c / usd(o.price)),
                            unit_cost_usd,
                            used_here: used_here.contains(p),
                        }
                    })
                    .collect(),
                orders: s
                    .orders
                    .iter()
                    .map(|(p, o)| OrderView {
                        product: catalog.products.key(*p).to_owned(),
                        target: o.target,
                        max_price_usd: usd(o.max_price),
                        min_quality: o.min_quality,
                        bought_month: o.bought_month,
                        bought_last_month: o.bought_last_month,
                    })
                    .collect(),
                inputs,
                research: s.research.map(|t| catalog.technologies.key(t).to_owned()),
                wage_premium: s.wage_premium,
                wage_premium_max: catalog.production_model.wage_premium_max,
                rival_premium_max,
                staff,
                wage_cost_per_day_usd,
                unit_costs: unit_costs
                    .iter()
                    .map(|u| UnitCostView {
                        product: catalog.products.key(u.product).to_owned(),
                        output_per_day: u.output_per_day,
                        material_usd: u.material,
                        labor_usd: u.labor,
                        energy_usd: u.energy,
                        overhead_usd: u.overhead,
                        rent_usd: u.rent,
                        facility_usd: u.facility,
                        total_usd: u.total(),
                        variable_usd: u.variable(),
                    })
                    .collect(),
                last_month: site_result(game, site_id),
                prices: {
                    let mut products: BTreeSet<crate::ids::ProductId> =
                        made.iter().chain(used_here.iter()).copied().collect();
                    products.extend(s.inventory.keys());
                    products.extend(s.offers.keys());
                    products.extend(s.orders.keys());
                    products
                        .into_iter()
                        .map(|p| {
                            (
                                catalog.products.key(p).to_owned(),
                                PriceInfo {
                                    market_usd: usd(market::market_price(
                                        catalog, state, s.country, p,
                                    )),
                                    reference_usd: usd(market::local_reference(
                                        catalog, state, s.country, p,
                                    )),
                                },
                            )
                        })
                        .collect()
                },
            }
        })
        .collect();
    ProductionView {
        date: iso(state.date),
        cash_usd: usd(state.companies[player.index()].ledger.cash()),
        sites,
        site_types,
        facilities,
        recipes,
        products: catalog
            .products
            .iter()
            .map(|(p, _)| catalog.products.key(p).to_owned())
            .collect(),
        units: units(catalog),
    }
}

/// Unit of each product, by product key.
pub fn units(catalog: &crate::catalog::Catalog) -> BTreeMap<String, String> {
    catalog
        .products
        .iter()
        .map(|(p, x)| {
            (
                catalog.products.key(p).to_owned(),
                catalog.units.key(x.unit).to_owned(),
            )
        })
        .collect()
}

/// Staff of a site by labor group, the highest premium of other companies' sites in its
/// country, and the wages per day of the staff employed now.
fn site_staff(game: &Game, site: crate::state::SiteId) -> (Vec<StaffLine>, f64, f64) {
    let state = game.state();
    let catalog = game.catalog();
    let s = &state.sites[site.index()];
    let country = state.countries.get(s.country);
    let needed = crate::production::needed_workers(catalog, state, site, state.date);
    let hours = crate::production::hours_per_worker_day(catalog, state.date);
    let factor = 1.0 + s.wage_premium;
    let mut wage_cost = 0.0;
    let staff = catalog
        .labor_groups
        .ids()
        .map(|g| {
            let employed_in_country: f64 = state
                .sites
                .iter()
                .filter(|o| o.country == s.country)
                .map(|o| *o.workforce.get(g))
                .sum();
            let employed = *s.workforce.get(g);
            let country_wage = country
                .hourly_wage_usd
                .get(g.index())
                .copied()
                .unwrap_or(0.0);
            wage_cost += employed * hours * country_wage * factor;
            StaffLine {
                group: catalog.labor_groups.key(g).to_owned(),
                needed: needed.get(g.index()).copied().unwrap_or(0.0),
                employed,
                free_in_country: (country
                    .labor_available
                    .get(g.index())
                    .copied()
                    .unwrap_or(0.0)
                    - employed_in_country)
                    .max(0.0),
                wage_usd: country_wage * factor,
                country_wage_usd: country_wage,
            }
        })
        .filter(|l| l.needed > 1e-9 || l.employed > 1e-9)
        .collect();
    let rival_premium_max = state
        .sites
        .iter()
        .filter(|o| o.country == s.country && o.owner != s.owner)
        .map(|o| o.wage_premium)
        .fold(0.0, f64::max);
    (staff, rival_premium_max, wage_cost)
}

/// A site's result in the last closed month by cost type, and per product.
fn site_result(game: &Game, site: crate::state::SiteId) -> Option<SiteResult> {
    let state = game.state();
    let catalog = game.catalog();
    let owner = state.sites[site.index()].owner;
    let period = state.companies[owner.index()].ledger.months.last()?;
    if period.by_center.keys().all(|c| c.site != Some(site)) {
        return None;
    }
    let lines: Vec<ResultLine> = crate::ledger::CostType::ALL
        .iter()
        .filter(|&&t| t != crate::ledger::CostType::Revenue)
        .map(|&t| (t, period.site_type(site, t)))
        .filter(|(_, m)| *m != Money::ZERO)
        .map(|(t, m)| ResultLine {
            key: t.text_key().to_owned(),
            usd: usd(m),
        })
        .collect();
    let revenue = period.site_type(site, crate::ledger::CostType::Revenue);
    let products = period
        .by_center
        .iter()
        .filter(|(c, _)| c.site == Some(site))
        .filter_map(|(c, by_type)| {
            let product = c.product?;
            let revenue = by_type
                .get(&crate::ledger::CostType::Revenue)
                .copied()
                .unwrap_or(Money::ZERO);
            Some(ProductResult {
                product: catalog.products.key(product).to_owned(),
                revenue_usd: usd(revenue),
                margin_usd: usd(by_type.values().copied().sum()),
            })
        })
        .collect();
    Some(SiteResult {
        month: super::iso(period.start?),
        revenue_usd: usd(revenue),
        result_usd: usd(revenue) + lines.iter().map(|l| l.usd).sum::<f64>(),
        lines,
        products,
    })
}

/// Discovered deposits of a country with a free field, priced as `DevelopDeposit` does.
fn free_deposits(game: &Game, country: crate::ids::CountryId) -> Vec<DepositOption> {
    let state = game.state();
    let catalog = game.catalog();
    catalog
        .deposits
        .iter()
        .filter(|(_, d)| d.country == country)
        .filter(|(_, d)| d.discovered.is_none_or(|y| y <= state.date.year()))
        .filter_map(|(id, d)| {
            let field = state
                .deposits
                .get(id)
                .concessions
                .iter()
                .find(|c| c.site.is_none())?;
            Some(DepositOption {
                key: catalog.deposits.key(id).to_owned(),
                resource: catalog.products.key(d.resource).to_owned(),
                cost_usd: usd(d
                    .development_cost
                    .scale(state.settings.market_scale * field.share)),
                days: d.development_days,
                // What the concession allows this year, at the market scale like the
                // production itself.
                output_per_year: catalog.max_output(id, state.date.year())
                    * state.settings.market_scale
                    * field.share,
            })
        })
        .collect()
}

/// One product on the market of a country.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketLine {
    pub product: String,
    pub price_usd: f64,
    pub reference_usd: f64,
    /// Price of the state market, if it sells the product.
    pub state_price_usd: Option<f64>,
    pub demand_last_month: f64,
    pub sold_last_month: f64,
    pub imported_last_month: f64,
    pub exported_last_month: f64,
    pub sellers: u32,
    /// The player's asking price here, if the player offers the product.
    pub own_price_usd: Option<f64>,
    /// Own sales in the last closed month.
    pub own_sold_last_month: f64,
    /// Goods group; brands and advertising work per group (M16).
    pub group: String,
    /// Own share of the sales in the country in the last closed month (0–1).
    pub own_share: f64,
    /// Company that sold most here in the last closed month, and its share.
    pub leader: Option<String>,
    pub leader_share: f64,
}

/// Brands of a goods group in a country (M16).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BrandLine {
    pub group: String,
    /// Own awareness, 0–1.
    pub own_awareness: f64,
    /// Best known other company selling the group here, and its awareness.
    pub top: Option<String>,
    pub top_awareness: f64,
    /// Own advertising budget per month.
    pub budget_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketView {
    pub date: String,
    pub country: String,
    pub lines: Vec<MarketLine>,
    /// Goods groups that consumers buy here or the player advertises or is known for.
    pub brands: Vec<BrandLine>,
    /// Best advertising medium of the year (text `werbemittel.<key>`).
    pub medium: Option<String>,
    /// Budget per month that reaches the whole country once.
    pub reach_usd: f64,
}

pub fn market(game: &Game, country: &str) -> Option<MarketView> {
    let state = game.state();
    let catalog = game.catalog();
    let c = catalog.countries.id(country)?;
    let lines = catalog
        .products
        .ids()
        .filter(|&p| catalog.products.get(p).kind != crate::catalog::ProductKind::Energy)
        .map(|p| {
            let m = state.markets.get(p).get(c);
            let t = &m.last_month;
            let mut sellers = 0;
            let mut own_price = None;
            let mut own_sold = 0.0;
            let mut by_company: BTreeMap<CompanyId, f64> = BTreeMap::new();
            for s in state.sites.iter().filter(|s| s.country == c) {
                if let Some(o) = s.offers.get(&p) {
                    if state.companies[s.owner.index()].bankrupt {
                        continue;
                    }
                    sellers += 1;
                    *by_company.entry(s.owner).or_default() += o.sold_last_month;
                    if s.owner == state.player {
                        own_price = Some(usd(o.price));
                        own_sold += o.sold_last_month;
                    }
                }
            }
            let share = |sold: f64| if t.sold > 1e-9 { sold / t.sold } else { 0.0 };
            let leader = by_company
                .iter()
                .filter(|&(_, &sold)| sold > 1e-9)
                .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(a.0)));
            MarketLine {
                product: catalog.products.key(p).to_owned(),
                price_usd: usd(market::market_price(catalog, state, c, p)),
                reference_usd: usd(market::local_reference(catalog, state, c, p)),
                state_price_usd: market::state_price(catalog, state, c, p).map(usd),
                demand_last_month: t.demand,
                sold_last_month: t.sold,
                imported_last_month: t.imported,
                exported_last_month: t.exported,
                sellers,
                own_price_usd: own_price,
                own_sold_last_month: own_sold,
                group: catalog
                    .goods_groups
                    .key(catalog.products.get(p).goods_group)
                    .to_owned(),
                own_share: share(own_sold),
                leader: leader.map(|(&id, _)| state.companies[id.index()].name.clone()),
                leader_share: leader.map_or(0.0, |(_, &sold)| share(sold)),
            }
        })
        .collect();
    Some(MarketView {
        date: iso(state.date),
        country: country.to_owned(),
        lines,
        brands: brands(game, c),
        medium: catalog
            .market_model
            .brand
            .medium(state.date.year())
            .map(|m| m.key.clone()),
        reach_usd: crate::brand::reach_usd(state, catalog, c),
    })
}

/// Awareness and advertising per goods group in a country: groups with consumer demand
/// here and groups the player advertises or is known for.
fn brands(game: &Game, country: CountryId) -> Vec<BrandLine> {
    let state = game.state();
    let catalog = game.catalog();
    let player = &state.companies[state.player.index()];
    let mut groups: BTreeSet<GoodsGroupId> = catalog
        .products
        .iter()
        .filter(|&(p, _)| {
            state
                .markets
                .get(p)
                .get(country)
                .consumer_rate
                .iter()
                .sum::<f64>()
                > 0.0
        })
        .map(|(_, product)| product.goods_group)
        .collect();
    groups.extend(
        player
            .brands
            .iter()
            .filter(|b| b.country == country)
            .map(|b| b.group),
    );
    groups.extend(
        player
            .advertising
            .iter()
            .filter(|a| a.country == country)
            .map(|a| a.group),
    );
    groups
        .into_iter()
        .map(|group| {
            let top = state
                .sites
                .iter()
                .filter(|s| s.country == country && s.owner != state.player)
                .filter(|s| !state.companies[s.owner.index()].bankrupt)
                .filter(|s| {
                    s.offers
                        .keys()
                        .any(|&p| catalog.products.get(p).goods_group == group)
                })
                .map(|s| {
                    let company = &state.companies[s.owner.index()];
                    (company.awareness(country, group), s.owner)
                })
                .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)));
            BrandLine {
                group: catalog.goods_groups.key(group).to_owned(),
                own_awareness: player.awareness(country, group),
                top: top.map(|(_, id)| state.companies[id.index()].name.clone()),
                top_awareness: top.map_or(0.0, |(a, _)| a),
                budget_usd: player
                    .advertising
                    .iter()
                    .find(|a| a.country == country && a.group == group)
                    .map_or(0.0, |a| usd(a.budget)),
            }
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TechnologyView {
    pub key: String,
    pub field: String,
    pub invention_year: i32,
    pub prerequisites: Vec<String>,
    pub known: bool,
    pub researchable: bool,
    /// Points needed today (`None` for technologies known from the start).
    pub needed: Option<f64>,
    /// Factor against the historical effort (above 1 ahead of history).
    pub factor: Option<f64>,
    pub points: f64,
    /// Sites working on it.
    pub sites: Vec<u32>,
    /// Facilities and recipes it opens.
    pub opens: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResearchCenterView {
    pub site: u32,
    pub country: String,
    pub researchers: f64,
    pub project: Option<String>,
    /// A laboratory is ready.
    pub ready: bool,
    /// Day the first laboratory is ready while all are still being built.
    pub building_until: Option<String>,
    pub labs: Vec<LabView>,
}

/// A laboratory slot of a research center; its utilization sets the researcher posts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LabView {
    pub slot: usize,
    pub count: u32,
    pub utilization: f64,
    pub ready: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResearchOverview {
    pub date: String,
    pub technologies: Vec<TechnologyView>,
    pub centers: Vec<ResearchCenterView>,
    /// Facility for research centers (to build a laboratory).
    pub laboratory: Option<String>,
}

pub fn research_overview(game: &Game) -> ResearchOverview {
    let state = game.state();
    let catalog = game.catalog();
    let player = state.player;
    let company = &state.companies[player.index()];
    let technologies = catalog
        .technologies
        .iter()
        .map(|(id, t)| {
            let effort = research::effort(catalog, state, id, state.date);
            let mut opens: Vec<String> = catalog
                .facilities
                .iter()
                .filter(|(_, f)| f.technology == Some(id))
                .map(|(f, _)| format!("anlage.{}", catalog.facilities.key(f)))
                .collect();
            opens.extend(
                catalog
                    .recipes
                    .iter()
                    .filter(|(_, r)| r.technology == Some(id))
                    .map(|(r, _)| format!("rezept.{}", catalog.recipes.key(r))),
            );
            TechnologyView {
                key: catalog.technologies.key(id).to_owned(),
                field: catalog.specializations.key(t.field).to_owned(),
                invention_year: t.invention_year,
                prerequisites: t
                    .prerequisites
                    .iter()
                    .map(|&p| catalog.technologies.key(p).to_owned())
                    .collect(),
                known: state.knows(catalog, player, id),
                researchable: research::can_research(catalog, state, player, id),
                needed: effort.map(|e| e.points),
                factor: effort.map(|e| e.factor),
                points: company.research.get(&id).copied().unwrap_or(0.0),
                sites: state
                    .sites
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| s.owner == player && s.research == Some(id))
                    .map(|(i, _)| u32::try_from(i).unwrap_or(u32::MAX))
                    .collect(),
                opens,
            }
        })
        .collect();
    let centers = state
        .sites
        .iter()
        .enumerate()
        .filter(|(_, s)| s.owner == player && s.kind == SiteType::ResearchCenter)
        .map(|(i, s)| ResearchCenterView {
            site: u32::try_from(i).unwrap_or(u32::MAX),
            country: catalog.countries.key(s.country).to_owned(),
            researchers: s.workforce.values().sum(),
            project: s.research.map(|t| catalog.technologies.key(t).to_owned()),
            ready: s.slots.iter().any(|sl| sl.ready <= state.date),
            building_until: s
                .slots
                .iter()
                .map(|sl| sl.ready)
                .min()
                .filter(|&r| r > state.date)
                .map(iso),
            labs: s
                .slots
                .iter()
                .enumerate()
                .map(|(slot, sl)| LabView {
                    slot,
                    count: sl.count,
                    utilization: sl.utilization,
                    ready: iso(sl.ready),
                })
                .collect(),
        })
        .collect();
    ResearchOverview {
        date: iso(state.date),
        technologies,
        centers,
        laboratory: catalog
            .facilities
            .iter()
            .find(|(_, f)| f.site_type == SiteType::ResearchCenter)
            .map(|(f, _)| catalog.facilities.key(f).to_owned()),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoanView {
    pub index: usize,
    pub principal_usd: f64,
    pub balance_usd: f64,
    pub rate: f64,
    pub start: String,
    pub months: u32,
    pub instalment_usd: f64,
}

/// Income statement lines by cost type (text key, amount; costs negative).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Statement {
    pub lines: Vec<(String, f64)>,
    pub result_usd: f64,
    /// Cash flow from operations, investments and financing.
    pub cash_flow_usd: [f64; 3],
}

fn statement(period: &crate::ledger::PeriodResult) -> Statement {
    let s = reports::income_statement(period);
    // Every cost type, so that the periods line up in one table.
    Statement {
        lines: crate::ledger::CostType::ALL
            .iter()
            .map(|t| {
                let amount = s
                    .lines
                    .iter()
                    .find(|(x, _)| x == t)
                    .map_or(Money::ZERO, |l| l.1);
                (t.text_key().to_owned(), usd(amount))
            })
            .collect(),
        result_usd: usd(s.result),
        cash_flow_usd: [
            usd(s.cash_flow.operating),
            usd(s.cash_flow.investing),
            usd(s.cash_flow.financing),
        ],
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FinanceView {
    pub date: String,
    /// Balance sheet: (text key of the account, amount).
    pub assets: Vec<(String, f64)>,
    pub claims: Vec<(String, f64)>,
    pub total_usd: f64,
    pub year: Statement,
    pub last_year: Option<Statement>,
    pub last_month: Option<Statement>,
    pub loans: Vec<LoanView>,
    pub credit_limit_usd: f64,
    pub overdraft_limit_usd: f64,
    /// Interest rate a loan of 10 % of the credit limit would get today.
    pub loan_rate: f64,
    pub max_term_years: u32,
    pub loss_carryforward_usd: f64,
}

pub fn finance_overview(game: &Game) -> FinanceView {
    let state = game.state();
    let catalog = game.catalog();
    let company = &state.companies[state.player.index()];
    let ledger = &company.ledger;
    let sheet = reports::balance_sheet(ledger);
    let limit = finance::credit_limit(catalog, company);
    FinanceView {
        date: iso(state.date),
        assets: sheet
            .assets
            .iter()
            .map(|(a, m)| (a.text_key().to_owned(), usd(*m)))
            .collect(),
        claims: sheet
            .claims
            .iter()
            .map(|(a, m)| (a.text_key().to_owned(), usd(*m)))
            .collect(),
        total_usd: usd(sheet.total),
        year: statement(&ledger.year),
        last_year: ledger.years.last().map(statement),
        last_month: ledger.months.last().map(statement),
        loans: company
            .loans
            .iter()
            .enumerate()
            .map(|(index, l)| LoanView {
                index,
                principal_usd: usd(l.principal),
                balance_usd: usd(l.balance),
                rate: l.rate,
                start: iso(l.start),
                months: l.months,
                instalment_usd: usd(l.instalment),
            })
            .collect(),
        credit_limit_usd: usd(limit),
        overdraft_limit_usd: usd(finance::overdraft_limit(catalog, ledger)),
        loan_rate: finance::loan_rate(catalog, company, limit.scale(0.1), state.date),
        max_term_years: catalog.finance_model.max_term_years,
        loss_carryforward_usd: usd(company.loss_carryforward),
    }
}
