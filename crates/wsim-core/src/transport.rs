//! Transport between countries (Lastenheft §3.5, §8.1; formulas in docs/FORMELN.md, M8).
//!
//! Stage 1 has an abstract freight service instead of fleets: every country is a node
//! at its capital. Land legs join neighbors, sea legs join coastal countries, air legs
//! join countries with airports. Costs and speeds follow the vehicles available in the
//! year and the infrastructure of the countries (values of 1 January). The cheapest
//! route per transport class is found with Dijkstra, one origin country at a time when
//! it is first needed; within a country goods move freely.

use std::fmt;
use std::sync::{Arc, OnceLock};

use crate::calendar::Date;
use crate::catalog::{Catalog, Way};
use crate::country_model::{self, Infrastructure};
use crate::ids::{CountryId, Id, ProductId, TransportClassId};
use crate::math;
use crate::money::Money;

/// Cheapest way from one country to another for one transport class.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Route {
    /// USD per tonne.
    pub cost_per_t: f64,
    pub days: f64,
    /// Whether the route uses a sea leg.
    pub by_sea: bool,
}

impl Route {
    const LOCAL: Route = Route {
        cost_per_t: 0.0,
        days: 0.0,
        by_sea: false,
    };

    /// Whole days on the way, at least one.
    pub fn whole_days(&self) -> u32 {
        // Routes take far fewer days than u32::MAX.
        (self.days.ceil().max(1.0)) as u32
    }
}

/// Cheapest routes between all countries in one year. Derived from catalog and year
/// only, so it does not matter when a row is computed.
#[derive(Clone, Default)]
pub struct Routes {
    year: i32,
    network: Option<Arc<Network>>,
    /// Per transport class and origin country, filled on first use.
    rows: Vec<Vec<OnceLock<Vec<Option<Route>>>>>,
}

impl PartialEq for Routes {
    fn eq(&self, other: &Self) -> bool {
        self.year == other.year
    }
}

impl fmt::Debug for Routes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Routes({})", self.year)
    }
}

/// Distances and the legs of every transport class in one year.
struct Network {
    countries: usize,
    /// Great-circle distance in km, `from * countries + to`.
    distance: Arc<Vec<f64>>,
    classes: Vec<Option<ClassLegs>>,
}

/// Legs of one transport class: land legs per country, port and airport handling, and
/// cost and time per great-circle km at sea and in the air.
struct ClassLegs {
    land: Vec<Vec<(usize, f64, f64)>>,
    sea: Option<Open>,
    air: Option<Open>,
}

/// A way that joins every pair of countries with a terminal (port or airport).
struct Open {
    cost_per_km: f64,
    days_per_km: f64,
    /// Handling per country (cost, days); infinite where there is no terminal.
    terminal_cost: Vec<f64>,
    terminal_days: Vec<f64>,
}

/// Cost and speed of the best vehicle of one way.
#[derive(Clone, Copy)]
struct Mode {
    /// USD per tonne-kilometre, already times the class factor.
    cost: f64,
    km_per_day: f64,
}

impl Routes {
    /// Routes of `year`. `previous` lends its distance table if the countries are the
    /// same.
    pub fn new(catalog: &Catalog, year: i32, previous: Option<&Routes>) -> Self {
        let n = catalog.countries.len();
        let distance = previous
            .and_then(|p| p.network.as_ref())
            .filter(|net| net.countries == n)
            .map(|net| Arc::clone(&net.distance))
            .unwrap_or_else(|| Arc::new(distances(catalog)));
        let infrastructure: Vec<Infrastructure> =
            country_model::compute_all(catalog, Date::first_of_year(year))
                .into_iter()
                .map(|s| s.infrastructure)
                .collect();
        let classes: Vec<Option<ClassLegs>> = catalog
            .transport_classes
            .ids()
            .map(|class| ClassLegs::new(catalog, &infrastructure, &distance, year, class))
            .collect();
        let rows = classes
            .iter()
            .map(|_| (0..n).map(|_| OnceLock::new()).collect())
            .collect();
        Self {
            year,
            network: Some(Arc::new(Network {
                countries: n,
                distance,
                classes,
            })),
            rows,
        }
    }

    pub fn year(&self) -> i32 {
        self.year
    }

    /// Great-circle distance between the capitals of two countries in km.
    pub fn distance_km(&self, from: CountryId, to: CountryId) -> Option<f64> {
        let network = self.network.as_ref()?;
        network
            .distance
            .get(from.index() * network.countries + to.index())
            .copied()
    }

    pub fn get(&self, class: TransportClassId, from: CountryId, to: CountryId) -> Option<Route> {
        if from == to {
            return Some(Route::LOCAL);
        }
        let network = self.network.as_ref()?;
        let legs = network.classes.get(class.index())?.as_ref()?;
        let row = self
            .rows
            .get(class.index())?
            .get(from.index())?
            .get_or_init(|| legs.cheapest_from(network, from.index()));
        *row.get(to.index())?
    }

    /// `for_product` for goods sent from or to a site at a port (M35): freight by sea
    /// costs `sea_freight` times as much.
    pub fn for_product_via(
        &self,
        catalog: &Catalog,
        product: ProductId,
        (from, to): (CountryId, CountryId),
        sea_freight: f64,
    ) -> Option<(Money, u32)> {
        let p = catalog.products.get(product);
        let route = self.get(p.transport_class, from, to)?;
        let factor = if route.by_sea { sea_freight } else { 1.0 };
        let cost = Money::from_usd(route.cost_per_t * p.weight_kg / 1000.0 * factor)?;
        Some((cost, route.whole_days()))
    }

    /// Transport cost per unit of a product and the whole days on the way.
    pub fn for_product(
        &self,
        catalog: &Catalog,
        product: ProductId,
        from: CountryId,
        to: CountryId,
    ) -> Option<(Money, u32)> {
        let p = catalog.products.get(product);
        let route = self.get(p.transport_class, from, to)?;
        let cost = Money::from_usd(route.cost_per_t * p.weight_kg / 1000.0)?;
        Some((cost, route.whole_days()))
    }
}

fn distances(catalog: &Catalog) -> Vec<f64> {
    let capitals: Vec<_> = catalog.countries.values().map(|c| c.capital).collect();
    let n = capitals.len();
    let mut distance = vec![0.0; n * n];
    for a in 0..n {
        for b in (a + 1)..n {
            let (p, q) = (capitals[a], capitals[b]);
            let d = math::great_circle_km(p.lat, p.lon, q.lat, q.lon);
            distance[a * n + b] = d;
            distance[b * n + a] = d;
        }
    }
    distance
}

impl ClassLegs {
    fn new(
        catalog: &Catalog,
        infrastructure: &[Infrastructure],
        distance: &[f64],
        year: i32,
        class: TransportClassId,
    ) -> Option<Self> {
        let model = &catalog.transport_model;
        let factor = catalog.transport_classes.get(class).cost_factor;
        let best = |way: Way| {
            catalog
                .vehicles
                .values()
                .filter(|v| v.way == way && v.available(year) && v.classes.contains(&class))
                .map(|v| Mode {
                    cost: v.cost_per_tkm.value_at(f64::from(year)) * factor,
                    km_per_day: v.km_per_day.value_at(f64::from(year)),
                })
                .filter(|m| m.km_per_day > 0.0)
                .min_by(|a, b| a.cost.total_cmp(&b.cost))
        };
        let (terrain, road, rail) = (best(Way::Terrain), best(Way::Road), best(Way::Rail));
        let n = infrastructure.len();
        let usable = |level: f64| level >= model.min_infrastructure;

        // Land legs between neighbors: the cheapest of cart, road and rail; road and
        // rail get dearer and slower with poor infrastructure of the two countries.
        let land: Vec<Vec<(usize, f64, f64)>> = (0..n)
            .map(|a| {
                catalog
                    .countries
                    .get(CountryId::from_index(a))
                    .neighbors
                    .iter()
                    .filter_map(|b| {
                        let b = b.index();
                        let km = distance[a * n + b] * model.detour_land;
                        let (ia, ib) = (&infrastructure[a], &infrastructure[b]);
                        [
                            (terrain, 1.0),
                            (road, (ia.road + ib.road) / 2.0),
                            (rail, (ia.rail + ib.rail) / 2.0),
                        ]
                        .into_iter()
                        .filter_map(|(mode, level)| {
                            let mode = mode?;
                            usable(level).then(|| {
                                (b, km * mode.cost / level, km / (mode.km_per_day * level))
                            })
                        })
                        .min_by(|x, y| x.1.total_cmp(&y.1))
                    })
                    .collect()
            })
            .collect();

        // Sea and air: every pair of countries with a usable port or airport.
        let open = |way: Way, detour: f64, level: fn(&Infrastructure) -> f64| {
            let mode = best(way)?;
            let (terminal_cost, terminal_days) = infrastructure
                .iter()
                .map(|i| {
                    let l = level(i);
                    if usable(l) {
                        (
                            model.handling_cost_usd * factor / l,
                            model.handling_days / l,
                        )
                    } else {
                        (f64::INFINITY, f64::INFINITY)
                    }
                })
                .unzip();
            Some(Open {
                cost_per_km: mode.cost * detour,
                days_per_km: detour / mode.km_per_day,
                terminal_cost,
                terminal_days,
            })
        };
        let sea = open(Way::Sea, model.detour_sea, |i| i.port);
        let air = open(Way::Air, model.detour_air, |i| i.air);

        let any_land = land.iter().any(|legs| !legs.is_empty());
        (any_land || sea.is_some() || air.is_some()).then_some(Self { land, sea, air })
    }

    /// Dijkstra from one country over all legs; one entry per destination.
    fn cheapest_from(&self, network: &Network, from: usize) -> Vec<Option<Route>> {
        let n = network.countries;
        let mut cost = vec![f64::INFINITY; n];
        let mut days = vec![0.0; n];
        let mut sea = vec![false; n];
        let mut done = vec![false; n];
        cost[from] = 0.0;
        loop {
            let mut u = usize::MAX;
            let mut lowest = f64::INFINITY;
            for c in 0..n {
                if !done[c] && cost[c] < lowest {
                    lowest = cost[c];
                    u = c;
                }
            }
            if u == usize::MAX {
                break;
            }
            done[u] = true;
            for &(v, c, d) in &self.land[u] {
                if !done[v] && cost[u] + c < cost[v] {
                    cost[v] = cost[u] + c;
                    days[v] = days[u] + d;
                    sea[v] = sea[u];
                }
            }
            for (open, by_sea) in [(&self.sea, true), (&self.air, false)] {
                let Some(o) = open else { continue };
                if !o.terminal_cost[u].is_finite() {
                    continue;
                }
                let row = &network.distance[u * n..(u + 1) * n];
                let base = cost[u] + o.terminal_cost[u];
                let base_days = days[u] + o.terminal_days[u];
                for v in 0..n {
                    let candidate = base + row[v] * o.cost_per_km + o.terminal_cost[v];
                    if !done[v] && candidate < cost[v] {
                        cost[v] = candidate;
                        days[v] = base_days + row[v] * o.days_per_km + o.terminal_days[v];
                        sea[v] = sea[u] || by_sea;
                    }
                }
            }
        }
        (0..n)
            .map(|v| {
                cost[v].is_finite().then_some(Route {
                    cost_per_t: cost[v],
                    days: days[v],
                    by_sea: sea[v],
                })
            })
            .collect()
    }
}
