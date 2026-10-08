//! Import tariffs, trade zones and embargoes (W3; formulas in docs/FORMELN.md).

use crate::calendar::Date;
use crate::catalog::{Catalog, EffectKind};
use crate::ids::{CountryId, GoodsGroupId, Id, ProductId};
use crate::rng::{SimRng, Stream};
use crate::state::{GameState, PerId};

/// Zone factor of a pair under an embargo.
const EMBARGO: f64 = -1.0;

/// Tariffs between all countries in one year; derived, not saved.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TariffTable {
    year: i32,
    countries: usize,
    /// Average tariff of each importing country; empty without tariffs.
    base: Vec<f64>,
    /// Zone factor per pair at `from * countries + to`; `EMBARGO` blocks the trade.
    pair: Vec<f64>,
    /// Extra tariff per pair from the events (H1); empty without.
    extra: Vec<f64>,
    /// The month the events were applied for.
    month: Option<Date>,
}

impl TariffTable {
    pub fn new(catalog: &Catalog, year: i32, offsets: &PerId<CountryId, f64>) -> Self {
        let model = &catalog.tariffs;
        let n = catalog.countries.len();
        if !model.enabled() {
            return Self {
                year,
                countries: n,
                base: Vec::new(),
                pair: Vec::new(),
                extra: Vec::new(),
                month: None,
            };
        };
        let dynamic = year > model.last_year() && offsets.len() == n;
        let base = catalog
            .countries
            .ids()
            .map(|c| {
                let z = data_rate(catalog, c, year).unwrap_or(0.0);
                if dynamic {
                    let d = &model.dynamics;
                    (z + offsets.get(c)).clamp(d.min, d.max)
                } else {
                    z
                }
            })
            .collect();
        let mut pair = vec![1.0_f64; n * n];
        for zone in &model.zones {
            let members: Vec<usize> = zone
                .members
                .iter()
                .filter(|&&(_, from, until)| active(year, from, until))
                .map(|(c, _, _)| c.index())
                .collect();
            for &a in &members {
                for &b in &members {
                    let p = &mut pair[a * n + b];
                    *p = (*p).min(zone.factor);
                }
            }
        }
        for e in &model.embargoes {
            if active(year, e.from, e.until) {
                let (a, b) = (e.countries.0.index(), e.countries.1.index());
                pair[a * n + b] = EMBARGO;
                pair[b * n + a] = EMBARGO;
            }
        }
        Self {
            year,
            countries: n,
            base,
            pair,
            extra: Vec::new(),
            month: None,
        }
    }

    /// The embargoes and extra tariffs of the events acting in the month starting on
    /// `month` (H1), unless the game has the effects off.
    pub fn with_events(mut self, catalog: &Catalog, month: Date, enabled: bool) -> Self {
        self.month = Some(month);
        if !enabled {
            return self;
        }
        let n = self.countries;
        for effect in crate::events::active(catalog, month) {
            match &effect.kind {
                EffectKind::Embargo { against } => {
                    if self.pair.is_empty() {
                        self.pair = vec![1.0; n * n];
                    }
                    for a in &effect.countries {
                        for b in against {
                            self.pair[a.index() * n + b.index()] = EMBARGO;
                            self.pair[b.index() * n + a.index()] = EMBARGO;
                        }
                    }
                }
                EffectKind::Tariff { against, surcharge } => {
                    if self.extra.is_empty() {
                        self.extra = vec![0.0; n * n];
                    }
                    for to in &effect.countries {
                        for from in 0..n {
                            let listed =
                                against.is_empty() || against.iter().any(|c| c.index() == from);
                            if listed && from != to.index() {
                                self.extra[from * n + to.index()] += surcharge;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        self
    }

    pub fn year(&self) -> i32 {
        self.year
    }

    pub fn month(&self) -> Option<Date> {
        self.month
    }

    /// Whether an embargo blocks the trade between two countries.
    pub fn blocked(&self, a: CountryId, b: CountryId) -> bool {
        self.pair
            .get(a.index() * self.countries + b.index())
            .is_some_and(|&p| p < 0.0)
    }

    /// Tariff on goods of a group from one country into another, as a share of their
    /// value; `None` under an embargo.
    pub fn rate(
        &self,
        catalog: &Catalog,
        from: CountryId,
        to: CountryId,
        group: GoodsGroupId,
    ) -> Option<f64> {
        let factor = catalog
            .tariffs
            .groups
            .get(group.index())
            .copied()
            .unwrap_or(1.0);
        self.rate_with(from, to, factor)
    }

    /// Tariff on a product: its own factor where the data give one, else its group's;
    /// `None` under an embargo.
    pub fn for_product(
        &self,
        catalog: &Catalog,
        from: CountryId,
        to: CountryId,
        product: ProductId,
    ) -> Option<f64> {
        let own = catalog
            .tariffs
            .products
            .get(product.index())
            .copied()
            .flatten();
        match own {
            Some(factor) => self.rate_with(from, to, factor),
            None => self.rate(catalog, from, to, catalog.products.get(product).goods_group),
        }
    }

    /// The tariff with the factor of the goods: 0 within a country.
    fn rate_with(&self, from: CountryId, to: CountryId, factor: f64) -> Option<f64> {
        if from == to {
            return Some(0.0);
        }
        let at = from.index() * self.countries + to.index();
        let pair = self.pair.get(at).copied().unwrap_or(1.0);
        let base = self.base.get(to.index()).copied().unwrap_or(0.0);
        let extra = self.extra.get(at).copied().unwrap_or(0.0);
        (pair >= 0.0).then_some(base * factor * pair + extra)
    }

    /// Average tariff of a country before the goods group and the zones.
    pub fn average(&self, country: CountryId) -> f64 {
        self.base.get(country.index()).copied().unwrap_or(0.0)
    }
}

/// Whether a membership or an embargo holds in a year: from its first year until before
/// its end.
fn active(year: i32, from: i32, until: Option<i32>) -> bool {
    from <= year && until.is_none_or(|u| year < u)
}

/// The tariff of a country by the data, without the dynamics.
fn data_rate(catalog: &Catalog, country: CountryId, year: i32) -> Option<f64> {
    let model = &catalog.tariffs;
    let series = model
        .countries
        .get(country.index())
        .and_then(Option::as_ref)
        .or(model.default.as_ref())?;
    Some(series.value_at(f64::from(year)))
}

/// Zones a country belongs to in a year.
pub fn zones_of(catalog: &Catalog, country: CountryId, year: i32) -> Vec<&str> {
    catalog
        .tariffs
        .zones
        .iter()
        .filter(|z| {
            z.members
                .iter()
                .any(|&(c, from, until)| c == country && active(year, from, until))
        })
        .map(|z| z.key.as_str())
        .collect()
}

/// Countries a country has an embargo with in a year.
pub fn embargoes_of(catalog: &Catalog, country: CountryId, year: i32) -> Vec<CountryId> {
    catalog
        .tariffs
        .embargoes
        .iter()
        .filter(|e| active(year, e.from, e.until))
        .filter_map(|e| match e.countries {
            (a, b) if a == country => Some(b),
            (a, b) if b == country => Some(a),
            _ => None,
        })
        .collect()
}

/// At the start of each year after the data, the tariff of every country changes by a
/// uniform random step with the standard deviation of the chosen dynamics.
pub(crate) fn new_year(state: &mut GameState, catalog: &Catalog, year: i32) {
    let model = &catalog.tariffs;
    let d = &model.dynamics;
    let deviation = d.deviation * state.settings.tariff_dynamics;
    if !model.enabled() || year <= model.last_year() || deviation <= 0.0 {
        return;
    }
    let n = catalog.countries.len();
    if state.tariff_offsets.len() != n {
        state.tariff_offsets.resize_with(n, || 0.0);
    }
    let stream = Stream::Tariffs {
        year: u16::try_from(year).unwrap_or(u16::MAX),
    };
    let mut rng = SimRng::for_stream(state.settings.seed, stream);
    // A uniform distribution on [-h, h] has the standard deviation h / sqrt(3).
    let half = libm::sqrt(3.0) * deviation;
    for (country, offset) in state.tariff_offsets.iter_mut() {
        let base = data_rate(catalog, country, year).unwrap_or(0.0);
        let step = (2.0 * rng.next_f64() - 1.0) * half;
        // The offset keeps the tariff within its bounds, so a run of steps cannot push
        // it far beyond them.
        *offset = (base + *offset + step).clamp(d.min, d.max) - base;
    }
}
