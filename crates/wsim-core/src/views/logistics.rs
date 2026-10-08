//! Logistics (W5; docs/BEDIENUNG.md, "Markt → Logistik").

use serde::{Deserialize, Serialize};

use super::usd;
use crate::catalog::Way;
use crate::game::Game;
use crate::logistics::{self, FreightMode, LogisticsMonth};

/// Vehicles of one kind the player holds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FleetHoldingView {
    pub vehicle: String,
    /// `gelaende`, `strasse`, `schiene`, `see`.
    pub way: String,
    pub count: u32,
    /// Tonne-kilometres the vehicles carry in the current month.
    pub capacity_tkm: f64,
    /// Tonne-kilometres carried this month and in the month before.
    pub used_tkm: f64,
    pub used_last_tkm: f64,
    pub purchase_usd: f64,
    pub book_value_usd: f64,
    /// What selling one vehicle brings.
    pub sale_usd: f64,
    /// Share of the market freight a run costs (first class it carries); `None` where
    /// there is no market to compare with.
    pub running_share: Option<f64>,
}

/// A vehicle the player can buy now.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VehicleOfferView {
    pub vehicle: String,
    pub way: String,
    /// Keys of the transport classes it carries.
    pub classes: Vec<String>,
    pub payload_t: f64,
    pub km_per_day: f64,
    pub cost_per_tkm_usd: f64,
    /// Tonne-kilometres one vehicle carries in the current month.
    pub capacity_tkm: f64,
    pub price_usd: f64,
    /// Upkeep and depreciation of one vehicle per month.
    pub monthly_cost_usd: f64,
    /// Share of the market freight a run costs (first class it carries); `None` where
    /// there is no market to compare with.
    pub running_share: Option<f64>,
}

/// Figures of one month.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LogisticsMonthView {
    pub fleet_tkm: f64,
    pub market_tkm: f64,
    pub state_tkm: f64,
    pub losses: u32,
    pub lost_value_usd: f64,
    pub rental_usd: f64,
    pub upkeep_usd: f64,
    pub depreciation_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogisticsView {
    /// Without logistics in the data only the freight market exists.
    pub enabled: bool,
    /// `markt`, `staat` or `flotte`.
    pub mode: String,
    pub carry_for_others: bool,
    /// State transport: surcharge on the freight and factor on the risk.
    pub state_surcharge: f64,
    pub state_risk_factor: f64,
    /// Margin of the logistics companies: what a fully used fleet saves.
    pub market_margin: f64,
    /// Share of free capacity that finds freight of others.
    pub rental_share: f64,
    /// Sale at this share of the book value.
    pub sale_share: f64,
    /// Chance of losing a load this year by land and by sea.
    pub risk_land: f64,
    pub risk_sea: f64,
    pub fleet: Vec<FleetHoldingView>,
    pub vehicles: Vec<VehicleOfferView>,
    pub month: LogisticsMonthView,
    pub last_month: LogisticsMonthView,
}

fn way_key(way: Way) -> &'static str {
    match way {
        Way::Terrain => "gelaende",
        Way::Road => "strasse",
        Way::Rail => "schiene",
        Way::Sea => "see",
        Way::Air => "luft",
    }
}

fn mode_key(mode: FreightMode) -> &'static str {
    match mode {
        FreightMode::Market => "markt",
        FreightMode::State => "staat",
        FreightMode::Fleet => "flotte",
    }
}

fn finite(x: f64) -> Option<f64> {
    x.is_finite().then_some(x)
}

fn month_view(m: &LogisticsMonth) -> LogisticsMonthView {
    LogisticsMonthView {
        fleet_tkm: m.fleet_tkm,
        market_tkm: m.market_tkm,
        state_tkm: m.state_tkm,
        losses: m.losses,
        lost_value_usd: usd(m.lost_value),
        rental_usd: usd(m.rental),
        upkeep_usd: usd(m.upkeep),
        depreciation_usd: usd(m.depreciation),
    }
}

pub fn logistics(game: &Game) -> LogisticsView {
    let (state, catalog) = (game.state(), game.catalog());
    let m = &catalog.logistics;
    let l = &state.companies[state.player.index()].logistics;
    let date = state.date;
    let year = date.year();
    let fleet = l
        .fleet
        .iter()
        .map(|h| {
            let v = catalog.vehicles.get(h.vehicle);
            let one = h.value.scale(1.0 / f64::from(h.count.max(1)));
            FleetHoldingView {
                vehicle: catalog.vehicles.key(h.vehicle).to_owned(),
                way: way_key(v.way).to_owned(),
                count: h.count,
                capacity_tkm: f64::from(h.count)
                    * logistics::capacity_per_vehicle(catalog, h.vehicle, date),
                used_tkm: h.used,
                used_last_tkm: h.used_last,
                purchase_usd: usd(h.cost),
                book_value_usd: usd(h.value),
                sale_usd: usd(one.scale(m.sale_share)),
                running_share: finite(logistics::running_share_first(catalog, h.vehicle, year)),
            }
        })
        .collect();
    let vehicles = catalog
        .vehicles
        .ids()
        .filter(|&v| logistics::for_sale(catalog, v, year))
        .filter_map(|id| {
            let v = catalog.vehicles.get(id);
            let price = logistics::price(catalog, id, year)?;
            let y = f64::from(year);
            Some(VehicleOfferView {
                vehicle: catalog.vehicles.key(id).to_owned(),
                way: way_key(v.way).to_owned(),
                classes: v
                    .classes
                    .iter()
                    .map(|&c| catalog.transport_classes.key(c).to_owned())
                    .collect(),
                payload_t: v.fleet.as_ref().map_or(0.0, |f| f.payload_t.value_at(y)),
                km_per_day: v.km_per_day.value_at(y),
                cost_per_tkm_usd: v.cost_per_tkm.value_at(y),
                capacity_tkm: logistics::capacity_per_vehicle(catalog, id, date),
                price_usd: usd(price),
                monthly_cost_usd: usd(price.scale(m.upkeep_share / 12.0))
                    + usd(price.scale(1.0 / (m.life_years * 12.0))),
                running_share: finite(logistics::running_share_first(catalog, id, year)),
            })
        })
        .collect();
    LogisticsView {
        enabled: m.enabled,
        mode: mode_key(l.mode).to_owned(),
        carry_for_others: l.carry_for_others,
        state_surcharge: m.state_surcharge,
        state_risk_factor: m.state_risk_factor,
        market_margin: m.market_margin,
        rental_share: m.rental_share,
        sale_share: m.sale_share,
        risk_land: logistics::risk(catalog, false, year),
        risk_sea: logistics::risk(catalog, true, year),
        fleet,
        vehicles,
        month: month_view(&l.month),
        last_month: month_view(&l.last_month),
    }
}
