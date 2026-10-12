//! Machine tenders (MAA, docs/MASCHINENANGEBOTE.md, docs/FORMELN.md): a facility is not
//! bought at the catalog price but asked for. For 28 days machine builders send offers
//! whose hidden performance L, price position P and build time Z scatter around the
//! catalog; the buyer takes one or lets the tender lapse.

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::{Catalog, FacilitySize, MachineOfferModel};
use crate::command::CommandError;
use crate::ids::{MachineBuilderId, ProductId};
use crate::message::{Message, MessageKind, Param};
use crate::money::Money;
use crate::rng::{SimRng, Stream};
use crate::state::{CompanyId, GameState, SiteId};

/// What an accepted offer made of a facility: factors on the catalog values.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MachineFactors {
    pub output: f64,
    pub labor: f64,
    pub inputs: f64,
    pub scrap: f64,
    pub area: f64,
    /// Price paid against the catalog investment (volume discount included); the base of
    /// overhaul and resale values.
    pub price: f64,
    pub builder: Option<MachineBuilderId>,
}

/// What a tender asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenderKind {
    /// New units of the main facility of a product.
    Build {
        product: ProductId,
        size: FacilitySize,
        count: u32,
    },
    /// Changing an existing facility over to another product of its branch.
    Switch { slot: usize, product: ProductId },
}

/// An offer of a machine builder.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MachineOffer {
    pub id: u32,
    pub builder: MachineBuilderId,
    pub received: Date,
    /// Last day the offer can be taken.
    pub valid_until: Date,
    pub factors: MachineFactors,
    /// Price of all units (volume discount included) and the build time in days.
    pub price: Money,
    pub build_days: u32,
    /// Market prices (USD per unit) of the recipe's inputs on the day the offer came in;
    /// fixed then, so the material cost shown does not move with the market.
    #[serde(default)]
    pub input_prices: Vec<(ProductId, f64)>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MachineTender {
    pub id: u32,
    pub company: CompanyId,
    pub site: SiteId,
    pub kind: TenderKind,
    pub start: Date,
    /// Day offsets (from the start) on which an offer arrives; drawn on the first day.
    #[serde(default)]
    pub schedule: Vec<u32>,
    #[serde(default)]
    pub offers: Vec<MachineOffer>,
    #[serde(default)]
    pub next_offer: u32,
}

impl MachineTender {
    /// Day of the tender today (1 on the first day).
    pub fn day(&self, date: Date) -> i32 {
        self.start.days_until(date) + 1
    }

    /// Whether the tender still waits for offers or has some to decide on.
    fn over(&self, model: &MachineOfferModel, date: Date) -> bool {
        let running = self.start.days_until(date) < i32::try_from(model.tender_days).unwrap_or(0);
        !running && self.offers.is_empty()
    }
}

/// The price of one offer of a tender against the catalog (before the volume discount):
/// what a tender would cost and take at the catalog.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Base {
    pub facility: crate::ids::FacilityId,
    pub recipe: crate::ids::RecipeId,
    pub size: FacilitySize,
    pub count: u32,
    /// The catalog investment of one unit and what it is scaled by (size, count and, for
    /// a changeover, its share): `unit * scale` is the price at the catalog.
    pub unit: Money,
    pub scale: f64,
    pub days: u32,
}

impl Base {
    pub fn price(&self) -> Money {
        self.unit.scale(self.scale)
    }
}

/// Price and time at the catalog for a kind of tender.
pub fn base(
    state: &GameState,
    catalog: &Catalog,
    site: SiteId,
    kind: TenderKind,
) -> Result<Base, CommandError> {
    let sizes = &catalog.production_model.sizes;
    match kind {
        TenderKind::Build {
            product,
            size,
            count,
        } => {
            let recipe = crate::product_research::main_recipe(catalog, product)
                .ok_or_else(|| CommandError::NoRecipe(catalog.products.key(product).to_owned()))?;
            let facility = catalog.recipes.get(recipe).facility;
            let f = catalog.facilities.get(facility);
            Ok(Base {
                facility,
                recipe,
                size,
                count,
                unit: f.investment,
                scale: sizes.investment(size) * f64::from(count),
                days: crate::operations::build_days(
                    catalog,
                    state,
                    site,
                    sizes.build_days(size, f.build_days),
                ),
            })
        }
        TenderKind::Switch { slot, product } => {
            let sl = state
                .site(site)
                .and_then(|s| s.slots.get(slot))
                .ok_or(CommandError::UnknownSlot)?;
            let recipe = crate::product_research::main_recipe(catalog, product)
                .ok_or_else(|| CommandError::NoRecipe(catalog.products.key(product).to_owned()))?;
            let facility = catalog.recipes.get(recipe).facility;
            let f = catalog.facilities.get(facility);
            Ok(Base {
                facility,
                recipe,
                size: sl.size,
                count: sl.count,
                unit: f.investment,
                scale: sizes.investment(sl.size)
                    * f64::from(sl.count)
                    * sizes.changeover_cost_share,
                days: (f64::from(sizes.build_days(sl.size, f.build_days))
                    * sizes.changeover_days_share)
                    .ceil() as u32,
            })
        }
    }
}

/// Starts a tender (`RequestMachineOffers`, `RequestSwitchOffers`).
pub fn request(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    site: SiteId,
    kind: TenderKind,
) -> Result<(), CommandError> {
    if !catalog.machine_offers.enabled {
        return Err(CommandError::NoTenders);
    }
    let s = state.site(site).ok_or(CommandError::UnknownSite)?;
    if !s.held_by(actor) {
        return Err(CommandError::NotOwner);
    }
    match kind {
        TenderKind::Build {
            product,
            size: _,
            count,
        } => {
            if count == 0 {
                return Err(CommandError::InvalidQuantity);
            }
            crate::purchase::check_product(state, catalog, actor, s, product)?;
            let b = base(state, catalog, site, kind)?;
            let f = catalog.facilities.get(b.facility);
            if s.kind != f.site_type {
                return Err(CommandError::WrongSiteType {
                    required: f.site_type,
                });
            }
        }
        TenderKind::Switch { slot, product } => {
            let sl = s.slots.get(slot).ok_or(CommandError::UnknownSlot)?;
            crate::purchase::check_product(state, catalog, actor, s, product)?;
            if sl.ready > state.date {
                return Err(CommandError::UnderConstruction);
            }
        }
    }
    let same = |k: &TenderKind| match (k, &kind) {
        (
            TenderKind::Build {
                product: a,
                size: x,
                ..
            },
            TenderKind::Build {
                product: b,
                size: y,
                ..
            },
        ) => a == b && x == y,
        (TenderKind::Switch { slot: a, .. }, TenderKind::Switch { slot: b, .. }) => a == b,
        _ => false,
    };
    if state
        .machine_tenders
        .iter()
        .any(|t| t.site == site && same(&t.kind))
    {
        return Err(CommandError::TenderRunning);
    }
    let id = state.next_machine_tender;
    state.next_machine_tender += 1;
    state.machine_tenders.push(MachineTender {
        id,
        company: actor,
        site,
        kind,
        start: state.date,
        schedule: Vec::new(),
        offers: Vec::new(),
        next_offer: 0,
    });
    Ok(())
}

fn tender_index(state: &GameState, actor: CompanyId, tender: u32) -> Result<usize, CommandError> {
    let i = state
        .machine_tenders
        .iter()
        .position(|t| t.id == tender)
        .ok_or(CommandError::UnknownTender)?;
    if state.machine_tenders[i].company != actor {
        return Err(CommandError::NotOwner);
    }
    Ok(i)
}

/// Takes an offer (`AcceptMachineOffer`): the build or changeover starts with its values
/// and the tender ends.
pub fn accept(
    state: &mut GameState,
    catalog: &Catalog,
    actor: CompanyId,
    tender: u32,
    offer: u32,
) -> Result<(), CommandError> {
    let i = tender_index(state, actor, tender)?;
    let t = state.machine_tenders[i].clone();
    let o = t
        .offers
        .iter()
        .find(|o| o.id == offer)
        .cloned()
        .ok_or(CommandError::UnknownMachineOffer)?;
    if o.valid_until < state.date {
        return Err(CommandError::OfferExpired);
    }
    match t.kind {
        TenderKind::Build {
            product,
            size,
            count,
        } => crate::command::build_for_product(
            state,
            catalog,
            actor,
            (t.site, product, count, size),
            Some(&o),
        )?,
        TenderKind::Switch { slot, product } => {
            crate::command::switch_product(
                state,
                catalog,
                actor,
                (t.site, slot, product),
                Some(&o),
            )?;
        }
    }
    state.machine_tenders.remove(i);
    Ok(())
}

/// Turns an offer down (`DeclineMachineOffer`); the tender goes on.
pub fn decline(
    state: &mut GameState,
    actor: CompanyId,
    tender: u32,
    offer: u32,
) -> Result<(), CommandError> {
    let i = tender_index(state, actor, tender)?;
    let t = &mut state.machine_tenders[i];
    let before = t.offers.len();
    t.offers.retain(|o| o.id != offer);
    if t.offers.len() == before {
        return Err(CommandError::UnknownMachineOffer);
    }
    Ok(())
}

/// Ends a tender without taking an offer (`CancelMachineTender`).
pub fn cancel(state: &mut GameState, actor: CompanyId, tender: u32) -> Result<(), CommandError> {
    let i = tender_index(state, actor, tender)?;
    state.machine_tenders.remove(i);
    Ok(())
}

/// A hidden value from -1 to 1: mostly normal around 0, now and then an outlier.
fn hidden(rng: &mut SimRng, m: &MachineOfferModel) -> f64 {
    let v = if rng.chance(1.0 - m.normal_share) {
        let magnitude = m.outlier.0 + rng.next_f64() * (m.outlier.1 - m.outlier.0);
        if rng.chance(0.5) {
            -magnitude
        } else {
            magnitude
        }
    } else {
        // Box-Muller; `1 - u` keeps the logarithm finite.
        let u1 = 1.0 - rng.next_f64();
        let u2 = rng.next_f64();
        m.deviation * libm::sqrt(-2.0 * libm::log(u1)) * libm::cos(std::f64::consts::TAU * u2)
    };
    v.clamp(-1.0, 1.0)
}

/// A factor around 1 with the noise of a figure, within the limit.
fn figure(rng: &mut SimRng, m: &MachineOfferModel, factor: f64) -> f64 {
    let noisy = factor + m.noise * (2.0 * rng.next_f64() - 1.0);
    noisy.clamp(1.0 - m.limit, 1.0 + m.limit)
}

/// The factors, price and build time of one offer from the hidden values.
pub fn draw_offer(
    rng: &mut SimRng,
    m: &MachineOfferModel,
    tendency: f64,
    discount_units: u32,
    base: &Base,
) -> (MachineFactors, Money, u32) {
    let l = (hidden(rng, m) + tendency).clamp(-1.0, 1.0);
    let p = hidden(rng, m);
    let z = hidden(rng, m);
    let a = hidden(rng, m);
    let output = figure(rng, m, 1.0 + m.output * l);
    let labor = figure(rng, m, 1.0 - m.labor * l);
    let inputs = figure(rng, m, 1.0 - m.inputs * l);
    let scrap = figure(rng, m, 1.0 - m.scrap * l);
    let area = figure(rng, m, 1.0 + m.area_spread * a);
    let price = figure(rng, m, 1.0 + m.price_performance * l + m.price_position * p);
    let build = (1.0 + m.build_time * z).clamp(1.0 - m.limit, 1.0 + m.limit);
    let n = f64::from(discount_units.max(1));
    let price_factor = price * (1.0 - m.volume_discount * (1.0 - 1.0 / n));
    let days = (f64::from(base.days) * build).round().max(1.0) as u32;
    (
        MachineFactors {
            output,
            labor,
            inputs,
            scrap,
            area,
            price: price_factor,
            builder: None,
        },
        base.unit.scale(base.scale * price_factor),
        days,
    )
}

/// Offers arrive, old ones lapse and tenders end (daily).
pub fn simulate_day(state: &mut GameState, catalog: &Catalog, date: Date) -> Vec<Message> {
    let m = &catalog.machine_offers;
    let mut messages = Vec::new();
    if !m.enabled || state.machine_tenders.is_empty() {
        return messages;
    }
    let seed = state.settings.seed;
    for i in 0..state.machine_tenders.len() {
        let (id, company, site, kind, start) = {
            let t = &state.machine_tenders[i];
            (t.id, t.company, t.site, t.kind, t.start)
        };
        if state.machine_tenders[i].schedule.is_empty() {
            // The weeks of the tender: some offers on random days of each.
            let mut rng = SimRng::for_stream(
                seed,
                Stream::Procurement {
                    tender: id,
                    index: u32::MAX,
                },
            );
            let weeks = m.tender_days.div_ceil(7);
            let mut days = Vec::new();
            for week in 0..weeks {
                let span = u64::from(m.offers_per_week.1 - m.offers_per_week.0) + 1;
                let n = m.offers_per_week.0 + rng.below(span) as u32;
                for _ in 0..n {
                    let day = week * 7 + rng.below(7) as u32;
                    if day < m.tender_days {
                        days.push(day);
                    }
                }
            }
            days.sort_unstable();
            if days.is_empty() {
                days.push(0);
            }
            state.machine_tenders[i].schedule = days;
        }
        let input_prices: Vec<(ProductId, f64)> = base(state, catalog, site, kind)
            .ok()
            .zip(state.site(site).map(|s| s.country))
            .map(|(b, country)| {
                catalog
                    .recipes
                    .get(b.recipe)
                    .inputs
                    .iter()
                    .map(|&(p, _)| {
                        (
                            p,
                            crate::market::market_price(catalog, state, country, p).to_usd(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        let offset = start.days_until(date);
        let due: Vec<u32> = state.machine_tenders[i]
            .schedule
            .iter()
            .copied()
            .filter(|&d| i32::try_from(d).is_ok_and(|d| d == offset))
            .collect();
        let mut received = 0;
        if let Ok(base) = base(state, catalog, site, kind) {
            let country = state.site(site).map(|s| s.country);
            for _ in due {
                let t = &mut state.machine_tenders[i];
                let index = t.next_offer;
                t.next_offer += 1;
                let mut rng = SimRng::for_stream(seed, Stream::Procurement { tender: id, index });
                let eligible: Vec<(MachineBuilderId, f64)> = {
                    let active = |b: &crate::catalog::MachineBuilder| b.active(date.year());
                    let own: Vec<_> = catalog
                        .machine_builders
                        .iter()
                        .filter(|(_, b)| active(b) && Some(b.country) == country)
                        .map(|(id, b)| (id, b.tendency))
                        .collect();
                    if own.is_empty() {
                        catalog
                            .machine_builders
                            .iter()
                            .filter(|(_, b)| active(b))
                            .map(|(id, b)| (id, b.tendency))
                            .collect()
                    } else {
                        own
                    }
                };
                if eligible.is_empty() {
                    continue;
                }
                let (builder, tendency) = eligible[rng.below(eligible.len() as u64) as usize];
                let (mut factors, price, build_days) =
                    draw_offer(&mut rng, m, tendency, base.count, &base);
                factors.builder = Some(builder);
                t.offers.push(MachineOffer {
                    id: index,
                    builder,
                    received: date,
                    valid_until: date.add_days(i32::try_from(m.valid_days).unwrap_or(1) - 1),
                    factors,
                    price,
                    build_days,
                    input_prices: input_prices.clone(),
                });
                received += 1;
            }
        }
        if received > 0
            && let Some(label) = state.news_of(company)
        {
            let country = state.site(site).map(|s| s.country);
            let mut message =
                Message::new(MessageKind::Info, crate::message::keys::MACHINE_OFFERS_NEW)
                    .with("anzahl", Param::Integer(received));
            if let Some(c) = country {
                message = message.with("land", Param::Country(catalog.countries.key(c).to_owned()));
            }
            messages.push(message.of(label));
        }
    }
    for t in &mut state.machine_tenders {
        t.offers.retain(|o| o.valid_until >= date);
    }
    state.machine_tenders.retain(|t| !t.over(m, date));
    messages
}

/// One input of an offer's material cost.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialLine {
    pub product: ProductId,
    /// Quantity per unit made and its price (fixed on the day the offer came in).
    pub quantity: f64,
    pub price_usd: f64,
}

/// What making one unit would cost with the machine of an offer, and with the machine
/// as the catalog has it (for the comparison).
#[derive(Clone, Debug, PartialEq)]
pub struct OfferCosts {
    pub materials: Vec<MaterialLine>,
    pub material_usd: f64,
    /// All of it: material, labor, energy, overhead, rent, depreciation and maintenance.
    pub unit_cost_usd: f64,
    pub catalog_unit_cost_usd: f64,
}

/// The facility an offer would make of the site: for a build the new units, for a
/// changeover the existing unit with the offer's values.
fn probe_slot(
    state: &GameState,
    catalog: &Catalog,
    tender: &MachineTender,
    machine: Option<MachineFactors>,
    cost: Money,
) -> Option<(crate::state::Slot, crate::ids::RecipeId)> {
    let b = base(state, catalog, tender.site, tender.kind).ok()?;
    let slot = match tender.kind {
        TenderKind::Build { .. } => crate::state::Slot {
            facility: b.facility,
            ready: state.date,
            count: b.count,
            cost,
            recipe: Some(b.recipe),
            utilization: 1.0,
            automation: 0.0,
            condition: 1.0,
            down: None,
            batches: Vec::new(),
            last_runs: 0.0,
            limit: None,
            operation: crate::state::Operation::Running,
            size: b.size,
            retrofit: 0,
            machine,
        },
        TenderKind::Switch { slot, .. } => {
            let mut sl = state.site(tender.site)?.slots.get(slot)?.clone();
            sl.facility = b.facility;
            sl.recipe = Some(b.recipe);
            sl.utilization = 1.0;
            sl.machine = machine.map(|m| MachineFactors {
                price: sl.machine.map_or(1.0, |old| old.price),
                ..m
            });
            sl
        }
    };
    Some((slot, b.recipe))
}

/// Material and total cost per unit of an offer (MAA §5): the inputs at the prices of the
/// day the offer came in, labor, energy and the rest as in `production::unit_costs`.
pub fn offer_costs(
    state: &GameState,
    catalog: &Catalog,
    tender: &MachineTender,
    offer: &MachineOffer,
) -> Option<OfferCosts> {
    let site = state.site(tender.site)?;
    let price = |p: ProductId| {
        offer
            .input_prices
            .iter()
            .find(|&&(q, _)| q == p)
            .map_or_else(
                || crate::market::market_price(catalog, state, site.country, p).to_usd(),
                |&(_, v)| v,
            )
    };
    let (slot, recipe_id) = probe_slot(state, catalog, tender, Some(offer.factors), offer.price)?;
    let recipe = catalog.recipes.get(recipe_id);
    let line =
        crate::production::slot_cost_line(catalog, state, tender.site, &slot, recipe, &price);
    let per_day = line.output_per_day.max(1e-12);
    let developed = crate::production::slot_effect(catalog, state, site, &slot, recipe.product);
    let runs = slot.full_runs(catalog) * slot.utilization;
    let materials: Vec<MaterialLine> = recipe
        .inputs
        .iter()
        .map(|&(p, q)| MaterialLine {
            product: p,
            quantity: q * developed.inputs * runs / per_day,
            price_usd: price(p),
        })
        .collect();
    let unit = line.per_unit();
    // The same facility as the catalog has it, at the catalog price.
    let at_catalog = probe_slot(
        state,
        catalog,
        tender,
        None,
        base(state, catalog, tender.site, tender.kind).ok()?.price(),
    )
    .map(|(sl, r)| {
        crate::production::slot_cost_line(
            catalog,
            state,
            tender.site,
            &sl,
            catalog.recipes.get(r),
            &price,
        )
        .per_unit()
        .total()
    })?;
    Some(OfferCosts {
        material_usd: unit.material,
        unit_cost_usd: unit.total(),
        materials,
        catalog_unit_cost_usd: at_catalog,
    })
}
