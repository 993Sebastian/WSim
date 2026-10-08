//! Logistics (W5, Lastenheft §8.1; formulas in docs/FORMELN.md, W5).
//!
//! A company sends its own loads (transfers between its sites, deliveries of its
//! contracts) by the freight market, by state transport or with its own fleet. A fleet
//! is bought by vehicle; its capacity per month carries loads on land or at sea, and
//! free capacity may carry the freight of others. Every load may be lost on the way.

use serde::{Deserialize, Serialize};

use crate::calendar::{self, Date};
use crate::catalog::{Catalog, Vehicle, Way};
use crate::ids::{CountryId, ProductId, TransportClassId, VehicleId};
use crate::ledger::{Account, CostCenter, CostType};
use crate::message::{Message, MessageKind, Param, keys};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{CompanyId, GameState};

/// How a company sends its own loads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FreightMode {
    /// Logistics companies at the cost of the route (stage 1).
    #[default]
    Market,
    /// State transport: dearer, safer.
    State,
    /// The own fleet while it has room, then the market.
    Fleet,
}

/// Vehicles of one kind a company holds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FleetHolding {
    pub vehicle: VehicleId,
    pub count: u32,
    /// Purchase value of the vehicles held.
    pub cost: Money,
    /// Book value.
    pub value: Money,
    /// Tonne-kilometres carried this month and in the month before.
    #[serde(default)]
    pub used: f64,
    #[serde(default)]
    pub used_last: f64,
}

/// Tonne-kilometres of the loads of one month on one way for one transport class.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Load {
    pub sea: bool,
    pub class: TransportClassId,
    /// All loads.
    pub tkm: f64,
    /// Of those, the loads the fleet did not carry.
    pub market: f64,
}

/// Figures of one month.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LogisticsMonth {
    pub fleet_tkm: f64,
    pub market_tkm: f64,
    pub state_tkm: f64,
    /// Loads and their stock value lost on the way.
    pub losses: u32,
    pub lost_value: Money,
    pub rental: Money,
    pub upkeep: Money,
    pub depreciation: Money,
}

impl LogisticsMonth {
    fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// Logistics of a company.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Logistics {
    #[serde(default)]
    pub mode: FreightMode,
    /// Free capacity of the fleet carries the freight of others.
    #[serde(default)]
    pub carry_for_others: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fleet: Vec<FleetHolding>,
    /// Loads of this month and the month before.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub loads: Vec<Load>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub loads_last: Vec<Load>,
    #[serde(default, skip_serializing_if = "LogisticsMonth::is_empty")]
    pub month: LogisticsMonth,
    #[serde(default, skip_serializing_if = "LogisticsMonth::is_empty")]
    pub last_month: LogisticsMonth,
    /// Loads sent so far; numbers the draws of the freight risk.
    #[serde(default)]
    pub sent: u32,
}

impl Logistics {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

/// Whether a vehicle runs at sea (`Some(true)`) or on land; `None` for air.
fn at_sea(way: Way) -> Option<bool> {
    match way {
        Way::Sea => Some(true),
        Way::Air => None,
        Way::Terrain | Way::Road | Way::Rail => Some(false),
    }
}

/// Payload, kilometres per day and price in USD of a vehicle a fleet may hold, in a year.
fn performance(v: &Vehicle, year: i32) -> Option<(f64, f64, f64)> {
    at_sea(v.way)?;
    let y = f64::from(year);
    let fleet = v.fleet.as_ref()?;
    let payload = fleet.payload_t.value_at(y);
    let km = v.km_per_day.value_at(y);
    let price = fleet.price_usd.value_at(y);
    (payload > 0.0 && km > 0.0 && price > 0.0).then_some((payload, km, price))
}

/// Whether a company may buy the vehicle in a year.
pub fn for_sale(catalog: &Catalog, vehicle: VehicleId, year: i32) -> bool {
    let v = catalog.vehicles.get(vehicle);
    catalog.logistics.enabled && v.available(year) && performance(v, year).is_some()
}

/// Price of one vehicle.
pub fn price(catalog: &Catalog, vehicle: VehicleId, year: i32) -> Option<Money> {
    let (_, _, usd) = performance(catalog.vehicles.get(vehicle), year)?;
    Money::from_usd(usd)
}

/// Tonne-kilometres one vehicle carries in the month of `date`.
pub fn capacity_per_vehicle(catalog: &Catalog, vehicle: VehicleId, date: Date) -> f64 {
    performance(catalog.vehicles.get(vehicle), date.year()).map_or(0.0, |(payload, km, _)| {
        payload
            * km
            * f64::from(calendar::days_in_month(date.year(), date.month()))
            * catalog.logistics.load
    })
}

/// What the freight market charges per tkm on a way for a transport class: the cost of
/// the cheapest vehicle available, times the factor of the class.
pub fn market_rate(
    catalog: &Catalog,
    sea: bool,
    class: TransportClassId,
    year: i32,
) -> Option<f64> {
    let y = f64::from(year);
    catalog
        .vehicles
        .values()
        .filter(|v| v.available(year) && at_sea(v.way) == Some(sea) && v.classes.contains(&class))
        .map(|v| v.cost_per_tkm.value_at(y))
        .filter(|&r| r > 0.0)
        .min_by(f64::total_cmp)
        .map(|r| r * catalog.transport_classes.get(class).cost_factor)
}

/// Share of the market freight a run of an own vehicle costs for a transport class: the
/// vehicle's own cost per tkm without the margin of the logistics companies and without
/// its capital costs (paid monthly), against the market rate of its way. Above 1 the
/// vehicle runs dearer than the market; infinite where it cannot carry the class.
pub fn running_share(
    catalog: &Catalog,
    vehicle: VehicleId,
    class: TransportClassId,
    year: i32,
) -> f64 {
    let m = &catalog.logistics;
    let v = catalog.vehicles.get(vehicle);
    let (Some((payload, km, price)), Some(sea)) = (performance(v, year), at_sea(v.way)) else {
        return f64::INFINITY;
    };
    let Some(market) = market_rate(catalog, sea, class, year) else {
        return f64::INFINITY;
    };
    if !v.classes.contains(&class) {
        return f64::INFINITY;
    }
    let per_year = payload * km * f64::from(calendar::days_in_year(year)) * m.load;
    let capital = price * (m.upkeep_share + 1.0 / m.life_years) / per_year;
    let rate =
        v.cost_per_tkm.value_at(f64::from(year)) * catalog.transport_classes.get(class).cost_factor;
    (rate * (1.0 - m.market_margin) - capital).max(0.0) / market
}

/// `running_share` for the first class the vehicle carries (for the view and for the
/// freight of others).
pub fn running_share_first(catalog: &Catalog, vehicle: VehicleId, year: i32) -> f64 {
    catalog
        .vehicles
        .get(vehicle)
        .classes
        .first()
        .map_or(f64::INFINITY, |&c| running_share(catalog, vehicle, c, year))
}

fn capacity(catalog: &Catalog, h: &FleetHolding, date: Date) -> f64 {
    f64::from(h.count) * capacity_per_vehicle(catalog, h.vehicle, date)
}

/// Whether a holding carries loads of a class on a way.
fn carries(catalog: &Catalog, h: &FleetHolding, sea: bool, class: TransportClassId) -> bool {
    let v = catalog.vehicles.get(h.vehicle);
    at_sea(v.way) == Some(sea) && v.classes.contains(&class)
}

/// Chance of losing a load on a way in a year, before the factor of the mode.
pub fn risk(catalog: &Catalog, sea: bool, year: i32) -> f64 {
    let m = &catalog.logistics;
    let series = if sea { &m.risk_sea } else { &m.risk_land };
    series
        .as_ref()
        .map_or(0.0, |s| s.value_at(f64::from(year)).clamp(0.0, 1.0))
}

/// How a load would travel and what it would cost the company.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Plan {
    /// Freight the company pays.
    pub cost: Money,
    sea: bool,
    class: TransportClassId,
    tkm: f64,
    /// Tonne-kilometres per holding of the fleet.
    fleet: Vec<(usize, f64)>,
    market: f64,
    state: f64,
    risk: f64,
}

/// The way of a load of `quantity` from one country to another whose freight on the
/// market is `market`.
pub(crate) fn plan(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
    (product, quantity): (ProductId, f64),
    (from, to): (CountryId, CountryId),
    market: Money,
) -> Plan {
    let p = catalog.products.get(product);
    let class = p.transport_class;
    let route = state.routes.get(class, from, to);
    let sea = route.is_some_and(|r| r.by_sea);
    let tm = &catalog.transport_model;
    let detour = if sea { tm.detour_sea } else { tm.detour_land };
    let km = state.routes.distance_km(from, to).unwrap_or(0.0) * detour;
    let tkm = (quantity * p.weight_kg / 1000.0 * km).max(0.0);
    let mut plan = Plan {
        cost: market,
        sea,
        class,
        tkm,
        fleet: Vec::new(),
        market: tkm,
        state: 0.0,
        risk: 0.0,
    };
    let m = &catalog.logistics;
    if !m.enabled || from == to {
        return plan;
    }
    let base = risk(catalog, sea, state.date.year());
    plan.risk = base;
    let l = &state.companies[company.index()].logistics;
    match l.mode {
        FreightMode::Market => {}
        FreightMode::State => {
            plan.cost = market.scale(1.0 + m.state_surcharge);
            plan.state = tkm;
            plan.market = 0.0;
            plan.risk = (base * m.state_risk_factor).min(1.0);
        }
        FreightMode::Fleet => {
            let year = state.date.year();
            let mut rest = tkm;
            let mut share = 0.0;
            for (i, h) in l.fleet.iter().enumerate() {
                if rest <= 0.0 {
                    break;
                }
                if !carries(catalog, h, sea, class) {
                    continue;
                }
                // Vehicles dearer than the market stay at home.
                let own = running_share(catalog, h.vehicle, class, year);
                if own >= 1.0 {
                    continue;
                }
                let free = (capacity(catalog, h, state.date) - h.used).max(0.0);
                let take = free.min(rest);
                if take > 0.0 {
                    plan.fleet.push((i, take));
                    rest -= take;
                    share += take / tkm * own;
                }
            }
            if tkm > 0.0 {
                share += rest / tkm;
            } else {
                share = 1.0;
            }
            plan.cost = market.scale(share);
            plan.market = rest;
        }
    }
    plan
}

/// Records a planned load; `true` when it is lost on the way.
pub(crate) fn book(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    plan: &Plan,
) -> bool {
    if !catalog.logistics.enabled {
        return false;
    }
    let seed = state.settings.seed;
    let l = &mut state.companies[company.index()].logistics;
    let mut own = 0.0;
    for &(i, tkm) in &plan.fleet {
        l.fleet[i].used += tkm;
        own += tkm;
    }
    l.month.fleet_tkm += own;
    l.month.market_tkm += plan.market;
    l.month.state_tkm += plan.state;
    match l
        .loads
        .iter_mut()
        .find(|x| x.sea == plan.sea && x.class == plan.class)
    {
        Some(x) => {
            x.tkm += plan.tkm;
            x.market += plan.market;
        }
        None => l.loads.push(Load {
            sea: plan.sea,
            class: plan.class,
            tkm: plan.tkm,
            market: plan.market,
        }),
    }
    l.sent = l.sent.wrapping_add(1);
    if plan.risk <= 0.0 {
        return false;
    }
    let mut rng = SimRng::for_stream(
        seed,
        Stream::Freight {
            company: company.0,
            load: l.sent,
        },
    );
    rng.next_f64() < plan.risk
}

/// Notes a lost load in the month's figures.
pub(crate) fn note_loss(state: &mut GameState, company: CompanyId, value: Money) {
    let m = &mut state.companies[company.index()].logistics.month;
    m.losses += 1;
    m.lost_value += value;
}

/// The message to the player about a lost load.
pub(crate) fn loss_message(
    catalog: &Catalog,
    (product, quantity): (ProductId, f64),
    (from, to): (CountryId, CountryId),
) -> Message {
    Message::new(MessageKind::Warning, keys::FREIGHT_LOST)
        .with(
            "produkt",
            Param::TextKey(format!("produkt.{}", catalog.products.key(product))),
        )
        .with("menge", Param::Number(quantity))
        .with(
            "von",
            Param::TextKey(format!("land.{}", catalog.countries.key(from))),
        )
        .with(
            "nach",
            Param::TextKey(format!("land.{}", catalog.countries.key(to))),
        )
}

/// Buys vehicles for the fleet.
pub(crate) fn buy(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    vehicle: VehicleId,
    count: u32,
) -> Result<(), crate::command::CommandError> {
    use crate::command::CommandError;
    if !catalog.logistics.enabled {
        return Err(CommandError::NoLogistics);
    }
    if count == 0 {
        return Err(CommandError::InvalidQuantity);
    }
    let year = state.date.year();
    if !for_sale(catalog, vehicle, year) {
        return Err(CommandError::VehicleNotForFleet);
    }
    let cost = Money::times(
        price(catalog, vehicle, year).ok_or(CommandError::VehicleNotForFleet)?,
        f64::from(count),
    );
    let c = &mut state.companies[company.index()];
    if c.ledger.cash() < cost {
        return Err(CommandError::NotEnoughCash { needed: cost });
    }
    c.ledger.transfer(Account::FixedAssets, Account::Cash, cost);
    let fleet = &mut c.logistics.fleet;
    match fleet.iter_mut().find(|h| h.vehicle == vehicle) {
        Some(h) => {
            h.count += count;
            h.cost += cost;
            h.value += cost;
        }
        None => fleet.push(FleetHolding {
            vehicle,
            count,
            cost,
            value: cost,
            used: 0.0,
            used_last: 0.0,
        }),
    }
    Ok(())
}

/// Sells vehicles of the fleet at a share of their book value.
pub(crate) fn sell(
    state: &mut GameState,
    catalog: &Catalog,
    company: CompanyId,
    vehicle: VehicleId,
    count: u32,
) -> Result<(), crate::command::CommandError> {
    use crate::command::CommandError;
    if count == 0 {
        return Err(CommandError::InvalidQuantity);
    }
    let c = &mut state.companies[company.index()];
    let fleet = &mut c.logistics.fleet;
    let i = fleet
        .iter()
        .position(|h| h.vehicle == vehicle)
        .ok_or(CommandError::TooManyVehicles { count: 0 })?;
    let h = &mut fleet[i];
    if count > h.count {
        return Err(CommandError::TooManyVehicles { count: h.count });
    }
    let share = f64::from(count) / f64::from(h.count);
    let (cost, book) = if count == h.count {
        (h.cost, h.value)
    } else {
        (h.cost.scale(share), h.value.scale(share))
    };
    h.count -= count;
    h.cost -= cost;
    h.value -= book;
    h.used *= 1.0 - share;
    if h.count == 0 {
        fleet.remove(i);
    }
    let proceeds = book.scale(catalog.logistics.sale_share);
    let ledger = &mut c.ledger;
    let center = CostCenter::default();
    if book > proceeds {
        ledger.expense(
            CostType::Other,
            center,
            Account::FixedAssets,
            book - proceeds,
        );
    } else if proceeds > book {
        ledger.income(
            CostType::Other,
            center,
            Account::FixedAssets,
            proceeds - book,
        );
    }
    ledger.transfer(Account::Cash, Account::FixedAssets, proceeds);
    Ok(())
}

/// At the end of a month: upkeep and depreciation of the fleets, freight for others,
/// and the month's figures kept for the month before.
pub(crate) fn month_end(state: &mut GameState, catalog: &Catalog, today: Date) {
    let m = &catalog.logistics;
    if !m.enabled {
        return;
    }
    for c in &mut state.companies {
        let l = &mut c.logistics;
        if l.is_default() {
            continue;
        }
        let (mut upkeep, mut depreciation) = (Money::ZERO, Money::ZERO);
        let (mut rental, mut running) = (Money::ZERO, Money::ZERO);
        if !c.bankrupt {
            for h in &mut l.fleet {
                upkeep += h.cost.scale(m.upkeep_share / 12.0);
                let d = h.cost.scale(1.0 / (m.life_years * 12.0)).min(h.value);
                h.value -= d;
                depreciation += d;
                // Others pay the market rate; nobody hires a vehicle dearer than that.
                let v = catalog.vehicles.get(h.vehicle);
                let own = running_share_first(catalog, h.vehicle, today.year());
                let rate = match (v.classes.first(), at_sea(v.way)) {
                    (Some(&class), Some(sea)) => market_rate(catalog, sea, class, today.year()),
                    _ => None,
                };
                if let (true, Some(rate), true) = (l.carry_for_others, rate, own < 1.0) {
                    let free = (capacity(catalog, h, today) - h.used).max(0.0) * m.rental_share;
                    let revenue = Money::from_usd(free * rate).unwrap_or(Money::ZERO);
                    rental += revenue;
                    running += revenue.scale(own);
                }
            }
        }
        let center = CostCenter::default();
        let ledger = &mut c.ledger;
        if upkeep > Money::ZERO {
            ledger.expense(CostType::Maintenance, center, Account::Cash, upkeep);
        }
        if depreciation > Money::ZERO {
            ledger.expense(
                CostType::Depreciation,
                center,
                Account::FixedAssets,
                depreciation,
            );
        }
        if rental > Money::ZERO {
            ledger.income(CostType::Revenue, center, Account::Cash, rental);
        }
        if running > Money::ZERO {
            ledger.expense(CostType::Transport, center, Account::Cash, running);
        }
        for h in &mut l.fleet {
            h.used_last = h.used;
            h.used = 0.0;
        }
        l.month.upkeep = upkeep;
        l.month.depreciation = depreciation;
        l.month.rental = rental;
        l.last_month = std::mem::take(&mut l.month);
        l.loads_last = std::mem::take(&mut l.loads);
    }
}

/// Vehicles an AI company buys at the start of a month: where its fleet could carry less
/// than `ki.anteil` of its loads of the month before, the cheapest vehicle per
/// tonne-kilometre that it would fill and that saves more than it costs, within
/// `ki.kasse_anteil` of its cash.
pub(crate) fn ai_purchases(
    state: &GameState,
    catalog: &Catalog,
    company: CompanyId,
) -> Vec<(VehicleId, u32)> {
    let m = &catalog.logistics;
    let mut out = Vec::new();
    if !m.enabled || m.ai_share <= 0.0 {
        return out;
    }
    let c = &state.companies[company.index()];
    let mut budget = c.ledger.cash().scale(m.ai_cash_share);
    if budget <= Money::ZERO {
        return out;
    }
    let date = state.date;
    let year = date.year();
    let l = &c.logistics;
    for load in &l.loads_last {
        let have: f64 = l
            .fleet
            .iter()
            .filter(|h| carries(catalog, h, load.sea, load.class))
            .map(|h| capacity(catalog, h, date))
            .sum();
        let need = m.ai_share * load.tkm - have;
        if need <= 0.0 {
            continue;
        }
        let Some(rate) = market_rate(catalog, load.sea, load.class, year) else {
            continue;
        };
        let best = catalog
            .vehicles
            .ids()
            .filter(|&v| for_sale(catalog, v, year))
            .filter(|&v| {
                let x = catalog.vehicles.get(v);
                at_sea(x.way) == Some(load.sea) && x.classes.contains(&load.class)
            })
            .filter_map(|v| {
                let per = capacity_per_vehicle(catalog, v, date);
                let cost = price(catalog, v, year)?;
                let own = running_share(catalog, v, load.class, year);
                // Saving at full use against the upkeep and depreciation of a month.
                let saving = per * rate * (1.0 - own);
                let fixed = cost.to_usd() * (m.upkeep_share / 12.0 + 1.0 / (m.life_years * 12.0));
                (per > 0.0 && per <= need && saving > fixed).then_some((v, per, cost, own))
            })
            .min_by(|a, b| a.3.total_cmp(&b.3).then(a.0.cmp(&b.0)));
        let Some((vehicle, per, cost, _)) = best else {
            continue;
        };
        // Counts of vehicles are small; the casts cannot overflow.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let wanted = (need / per).floor() as u32;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let affordable = if cost > Money::ZERO {
            (budget.to_usd() / cost.to_usd()).floor() as u32
        } else {
            0
        };
        let n = wanted.min(affordable);
        if n == 0 {
            continue;
        }
        budget -= Money::times(cost, f64::from(n));
        out.push((vehicle, n));
    }
    out
}
