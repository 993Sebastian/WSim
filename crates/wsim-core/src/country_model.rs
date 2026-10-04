//! Current values of a country, derived from the yearly data and the country model
//! (Lastenheft §3.2, §9.1; formulas in docs/FORMELN.md, section M4).
//!
//! Everything here is a pure function of catalog and date, so the values never need
//! to be saved separately and are identical after loading.

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::Catalog;
use crate::ids::{CountryId, Id};
use crate::math;
use crate::time_series::TimeSeries;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Infrastructure {
    pub rail: f64,
    pub road: f64,
    pub port: f64,
    pub air: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CountryState {
    pub population: f64,
    /// GDP per capita at purchasing power parity (USD).
    pub gdp_per_capita_usd: f64,
    pub gini: f64,
    /// Market prices relative to purchasing power parity.
    pub price_level: f64,
    /// Yearly income per capita of each fifth of households at market prices (USD),
    /// poorest first.
    pub income_quintiles_usd: [f64; 5],
    pub labor_force: f64,
    /// Persons per labor group, indexed by `LaborGroupId`.
    pub labor_pool: Vec<f64>,
    /// Inhabitants whose demand the markets serve (population × market scale).
    #[serde(default)]
    pub market_population: f64,
    /// Workers the companies can hire per labor group (pool × market scale).
    #[serde(default)]
    pub labor_available: Vec<f64>,
    /// Hourly wage at market prices (USD) per labor group.
    pub hourly_wage_usd: Vec<f64>,
    pub electricity_price_usd_mwh: f64,
    /// Share of industrial electricity demand the public grid can supply.
    pub grid_share: f64,
    pub corporate_tax: f64,
    pub dividend_tax: f64,
    /// Development level 0–1.
    pub development: f64,
    pub infrastructure: Infrastructure,
    /// 0 = civil war, 1 = very stable.
    pub stability: f64,
    /// Research efficiency per field, indexed by `SpecializationId`.
    pub research_efficiency: Vec<f64>,
    /// Suitability for automated production (0 = hand assembly, 1 = highly automated).
    pub automation_affinity: f64,
}

/// Point in time for the yearly data: values apply to the middle of the year.
fn data_time(date: Date) -> f64 {
    date.year_fraction() - 0.5
}

/// Values of all countries for one day.
pub fn compute_all(catalog: &Catalog, date: Date) -> Vec<CountryState> {
    catalog
        .countries
        .ids()
        .map(|id| compute(catalog, id, date))
        .collect()
}

pub fn compute(catalog: &Catalog, id: CountryId, date: Date) -> CountryState {
    let model = &catalog.country_model;
    let country = catalog.countries.get(id);
    let t = data_time(date);
    let year = f64::from(date.year());
    let v = &country.values;

    let population = v.population.value_at(t);
    let gdp = v.gdp_per_capita_usd.value_at(t).max(1.0);
    let gini = v.gini.value_at(t).clamp(0.01, 0.95);

    let price_level = match model.price_reference {
        Some(reference) => {
            let reference_gdp = catalog
                .countries
                .get(reference)
                .values
                .gdp_per_capita_usd
                .value_at(t)
                .max(1.0);
            math::pow(gdp / reference_gdp, model.price_elasticity)
                .clamp(model.price_min, model.price_max)
        }
        None => 1.0,
    };
    let market_income = gdp * price_level;
    let shares = math::quintile_shares(gini);
    let income_quintiles_usd = shares.map(|s| market_income * s * 5.0);

    // Labor market
    let labor_force = population * model.participation_rate;
    let qualification_shares = interpolate_rows(&model.qualification_shares, gdp);
    let wage_factors = interpolate_rows(&model.wage_factors, gdp);
    let average_wage = model.labor_share * gdp / model.participation_rate.max(0.01);
    let weighted: f64 = qualification_shares
        .iter()
        .zip(&wage_factors)
        .map(|(s, f)| s * f)
        .sum();
    let hours = model.annual_hours.value_at(year).max(1.0);
    let specializations = catalog.specializations.len();
    let weights = |spec: usize| {
        country
            .profile
            .specialization_weights
            .get(spec)
            .copied()
            .unwrap_or(1.0)
    };
    let mut labor_pool = Vec::with_capacity(catalog.labor_groups.len());
    let mut hourly_wage_usd = Vec::with_capacity(catalog.labor_groups.len());
    for (_, group) in catalog.labor_groups.iter() {
        let q = group.qualification.index();
        let share = qualification_shares.get(q).copied().unwrap_or(0.0);
        let spec_share = match group.specialization {
            None => 1.0,
            Some(spec) => {
                let base = model
                    .specialization_shares
                    .get(q)
                    .filter(|s| s.len() == specializations);
                match base {
                    Some(base) => {
                        let total: f64 = (0..specializations).map(|s| base[s] * weights(s)).sum();
                        base[spec.index()] * weights(spec.index()) / total.max(f64::MIN_POSITIVE)
                    }
                    None => 1.0 / specializations.max(1) as f64,
                }
            }
        };
        labor_pool.push(labor_force * share * spec_share);
        let factor = wage_factors.get(q).copied().unwrap_or(1.0);
        let yearly = if weighted > 0.0 {
            average_wage * factor / weighted
        } else {
            average_wage
        };
        hourly_wage_usd.push(yearly / hours * price_level);
    }

    let development = math::log_position(gdp, model.development_from_usd, model.development_to_usd);
    let grid_share = (model.grid_reach.value_at(year)
        * math::pow(gdp / model.grid_reference_usd, 0.7))
    .clamp(0.0, 1.0);
    let infrastructure = Infrastructure {
        rail: math::sqrt(development) * model.rail.value_at(year),
        road: development * model.road.value_at(year),
        port: if country.landlocked {
            0.0
        } else {
            math::sqrt(development).max(0.2) * model.port.value_at(year)
        },
        air: development * model.air.value_at(year),
    };
    let own = |series: &Option<TimeSeries>, default: &TimeSeries| match series {
        Some(s) => s.value_at(t),
        None => default.value_at(year),
    };
    let base_research = math::pow(
        gdp / model.research_reference_usd,
        model.research_elasticity,
    )
    .clamp(model.research_min, model.research_max);
    let research_efficiency = (0..specializations)
        .map(|s| {
            base_research
                * country
                    .profile
                    .research_weights
                    .get(s)
                    .copied()
                    .unwrap_or(1.0)
        })
        .collect();
    let automation_affinity = (model.automation_base
        + model.automation_per_doubling * math::log2(gdp / model.automation_reference_usd)
        + country.profile.automation_bonus)
        .clamp(0.0, 1.0);

    CountryState {
        population,
        gdp_per_capita_usd: gdp,
        gini,
        price_level,
        income_quintiles_usd,
        labor_force,
        market_population: population,
        labor_available: labor_pool.clone(),
        labor_pool,
        hourly_wage_usd,
        electricity_price_usd_mwh: model.electricity_price_usd_mwh.value_at(year) * price_level,
        grid_share,
        corporate_tax: own(&v.corporate_tax, &model.corporate_tax).clamp(0.0, 1.0),
        dividend_tax: own(&v.dividend_tax, &model.dividend_tax).clamp(0.0, 1.0),
        development,
        infrastructure,
        stability: v
            .stability
            .as_ref()
            .map_or(model.stability, |s| s.value_at(t))
            .clamp(0.0, 1.0),
        research_efficiency,
        automation_affinity,
    }
}

/// Interpolates table rows logarithmically in GDP per capita.
fn interpolate_rows(rows: &[(f64, Vec<f64>)], gdp: f64) -> Vec<f64> {
    let Some(first) = rows.first() else {
        return Vec::new();
    };
    let last = &rows[rows.len() - 1];
    if gdp <= first.0 {
        return first.1.clone();
    }
    if gdp >= last.0 {
        return last.1.clone();
    }
    let next = rows.partition_point(|(g, _)| *g <= gdp);
    let (g0, a) = &rows[next - 1];
    let (g1, b) = &rows[next];
    let w = math::log_position(gdp, *g0, *g1);
    a.iter().zip(b).map(|(x, y)| x + (y - x) * w).collect()
}

/// Shrinks the quantities markets and companies work with to the market scale
/// (docs/FORMELN.md, M10); the real values stay for display. Small labor pools keep at
/// least `min_pool` persons, so that a single real facility can still be staffed.
pub fn apply_market_scale(state: &mut CountryState, scale: f64, min_pool: f64) {
    state.market_population = state.population * scale;
    state.labor_available = state
        .labor_pool
        .iter()
        .map(|&pool| (pool * scale).max(pool.min(min_pool)))
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::test_support;

    #[test]
    fn rows_interpolate_in_log_space() {
        let rows = vec![(1_000.0, vec![1.0, 0.0]), (100_000.0, vec![0.0, 1.0])];
        assert_eq!(interpolate_rows(&rows, 500.0), vec![1.0, 0.0]);
        assert_eq!(interpolate_rows(&rows, 1e6), vec![0.0, 1.0]);
        let mid = interpolate_rows(&rows, 10_000.0);
        assert!(
            (mid[0] - 0.5).abs() < 1e-12 && (mid[1] - 0.5).abs() < 1e-12,
            "{mid:?}"
        );
    }

    #[test]
    fn works_with_an_empty_model() {
        let catalog = test_support::sample();
        let state = compute(
            &catalog,
            CountryId::from_index(0),
            Date::first_of_year(1900),
        );
        assert_eq!(state.population, 1_000_000.0);
        assert_eq!(state.price_level, 1.0);
        let total: f64 = state.income_quintiles_usd.iter().sum();
        assert!((total / 5.0 - 5_000.0).abs() < 1e-6, "{total}");
    }
}
