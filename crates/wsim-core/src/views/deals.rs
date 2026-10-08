//! Offers between companies and the companies to buy from (M30; docs/BEDIENUNG.md,
//! "Wettbewerb").

use serde::{Deserialize, Serialize};

use super::{iso, usd};
use crate::command::site_type_key;
use crate::deals::{self, DealObject, Offer, OfferStatus, SiteValue};
use crate::game::Game;
use crate::message::{Message, MessageKind, Param, keys};
use crate::ranking::{equity, revenue_of_year};
use crate::state::{CompanyId, GameState, SiteId};

/// What a site is worth (docs/FORMELN.md, M30).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SiteValueView {
    /// B: facilities, building, development and goodwill on the books.
    pub book_usd: f64,
    /// L: stocks at the site and on the way there.
    pub inventory_usd: f64,
    /// R: result of the last twelve months; `None` with too few months.
    pub result_year_usd: Option<f64>,
    /// E = max(0, R) × earnings years.
    pub earnings_value_usd: f64,
    /// Q: proceeds of selling the finished facilities.
    pub liquidation_usd: f64,
    /// U: facilities and development under construction.
    pub under_construction_usd: f64,
    /// A bought plot at today's value (M35); 0 for a leased one.
    #[serde(default)]
    pub land_usd: f64,
    /// G = max(E, Q) + U + L + land; for an area the sum over its sites and the brand.
    pub base_usd: f64,
    pub earnings_years: f64,
    /// W: the brand of an area (M31), 0 for a site.
    pub brand_usd: f64,
}

fn value_view(game: &Game, v: &SiteValue) -> SiteValueView {
    SiteValueView {
        book_usd: usd(v.book()),
        inventory_usd: usd(v.inventory),
        result_year_usd: v.result_year.map(usd),
        earnings_value_usd: usd(v.earnings_value),
        liquidation_usd: usd(v.liquidation),
        under_construction_usd: usd(v.under_construction),
        land_usd: usd(v.land),
        base_usd: usd(v.base),
        earnings_years: game.catalog().deal_model.earnings_years,
        brand_usd: 0.0,
    }
}

fn area_value_view(game: &Game, v: &deals::AreaValue) -> SiteValueView {
    SiteValueView {
        brand_usd: usd(v.brand),
        ..value_view(game, &v.sum())
    }
}

/// The object of an offer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DealObjectView {
    /// `standort`, `lizenz` or `bereich`.
    pub kind: String,
    /// Goods group of an area.
    pub group: Option<String>,
    /// Number of sites of an area.
    pub site_count: u32,
    pub site: Option<u32>,
    /// Text key of the site type, e.g. `standorttyp.werk`.
    pub site_type: Option<String>,
    pub country: Option<String>,
    /// Products the site makes or offers.
    pub products: Vec<String>,
    pub technology: Option<String>,
}

fn site_index(site: SiteId) -> u32 {
    site.0
}

fn products_of(game: &Game, site: SiteId) -> Vec<String> {
    let catalog = game.catalog();
    let s = &game.state().sites[site.index()];
    let mut products: Vec<String> = Vec::new();
    let made = s
        .slots
        .iter()
        .filter_map(|sl| sl.recipe.map(|r| catalog.recipes.get(r).product));
    for p in made.chain(s.offers.keys().copied()) {
        let key = catalog.products.key(p).to_owned();
        if !products.contains(&key) {
            products.push(key);
        }
    }
    products
}

fn object_view(game: &Game, seller: CompanyId, object: DealObject) -> DealObjectView {
    let catalog = game.catalog();
    match object {
        DealObject::Site(site) => {
            let s = &game.state().sites[site.index()];
            DealObjectView {
                kind: "standort".to_owned(),
                group: None,
                site_count: 1,
                site: Some(site_index(site)),
                site_type: Some(site_type_key(s.kind)),
                country: Some(catalog.countries.key(s.country).to_owned()),
                products: products_of(game, site),
                technology: None,
            }
        }
        DealObject::License(t) => DealObjectView {
            kind: "lizenz".to_owned(),
            group: None,
            site_count: 0,
            site: None,
            site_type: None,
            country: None,
            products: Vec::new(),
            technology: Some(catalog.technologies.key(t).to_owned()),
        },
        DealObject::Area(group) => {
            let sites = deals::area_sites(game.state(), catalog, seller, group);
            let mut products: Vec<String> = Vec::new();
            for &site in &sites {
                for p in products_of(game, site) {
                    if !products.contains(&p) {
                        products.push(p);
                    }
                }
            }
            DealObjectView {
                kind: "bereich".to_owned(),
                group: Some(catalog.goods_groups.key(group).to_owned()),
                site_count: u32::try_from(sites.len()).unwrap_or(u32::MAX),
                site: None,
                site_type: None,
                country: None,
                products,
                technology: None,
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfferView {
    pub id: u32,
    /// `kaeufer`: the player buys; `verkaeufer`: the player sells.
    pub role: String,
    /// The other company.
    pub company: String,
    pub company_index: u32,
    pub object: DealObjectView,
    pub price_usd: f64,
    /// The seller named the price (counter-offer).
    pub counter: bool,
    /// Day of the current price and last day for the answer.
    pub date: String,
    pub deadline: String,
    /// `offen`, `angenommen`, `abgelehnt`, `abgelaufen`, `zurueckgezogen`.
    pub status: String,
    pub closed: Option<String>,
    /// The player answers now.
    pub answer: bool,
    /// The player may name a higher price (seller on an offer).
    pub can_counter: bool,
    /// The player named the price and may withdraw it.
    pub can_withdraw: bool,
    /// What the site is worth.
    pub value: Option<SiteValueView>,
    /// What the licence saves the buyer.
    pub license_value_usd: Option<f64>,
}

fn status_key(status: OfferStatus) -> &'static str {
    match status {
        OfferStatus::Open => "offen",
        OfferStatus::Accepted => "angenommen",
        OfferStatus::Declined => "abgelehnt",
        OfferStatus::Expired => "abgelaufen",
        OfferStatus::Withdrawn => "zurueckgezogen",
    }
}

fn offer_view(game: &Game, offer: &Offer) -> OfferView {
    let state = game.state();
    let catalog = game.catalog();
    let player = game.player();
    let other = if offer.buyer == player {
        offer.seller
    } else {
        offer.buyer
    };
    let open = offer.status == OfferStatus::Open;
    let (value, license_value_usd) = match offer.object {
        // A sold site's value is the new owner's business now.
        DealObject::Site(site) if open => (
            Some(value_view(game, &deals::site_value(state, catalog, site))),
            None,
        ),
        DealObject::License(t) if open => (
            None,
            deals::license_value(state, catalog, offer.buyer, t).map(usd),
        ),
        DealObject::Area(group) if open => (
            Some(area_value_view(
                game,
                &deals::area_value(state, catalog, offer.seller, group),
            )),
            None,
        ),
        _ => (None, None),
    };
    OfferView {
        id: offer.id,
        role: if offer.buyer == player {
            "kaeufer"
        } else {
            "verkaeufer"
        }
        .to_owned(),
        company: state.companies[other.index()].name.clone(),
        company_index: other.0,
        object: object_view(game, offer.seller, offer.object),
        price_usd: usd(offer.price),
        counter: offer.counter,
        date: iso(offer.date),
        deadline: iso(offer.deadline(catalog)),
        status: status_key(offer.status).to_owned(),
        closed: offer.closed.map(iso),
        answer: open && offer.answering() == player,
        can_counter: open && offer.seller == player && !offer.counter,
        can_withdraw: open && offer.bidding() == player,
        value,
        license_value_usd,
    }
}

/// The player's offers: open ones first (those to answer on top), then the closed ones
/// of the blocking period, newest first.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OffersView {
    pub offers: Vec<OfferView>,
}

pub fn offers(game: &Game) -> OffersView {
    let state = game.state();
    let player = game.player();
    let mut list: Vec<OfferView> = state
        .offers
        .iter()
        .filter(|o| o.buyer == player || o.seller == player)
        .map(|o| offer_view(game, o))
        .collect();
    let rank = |o: &OfferView| match (o.status.as_str(), o.answer) {
        ("offen", true) => 0,
        ("offen", false) => 1,
        _ => 2,
    };
    list.sort_by(|a, b| {
        rank(a)
            .cmp(&rank(b))
            .then(b.closed.cmp(&a.closed))
            .then(b.id.cmp(&a.id))
    });
    OffersView { offers: list }
}

/// A company in the list of the competition tab.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompanyRowView {
    pub index: u32,
    pub name: String,
    pub headquarters: String,
    pub equity_usd: f64,
    /// Revenue of the last twelve closed months.
    pub revenue_year_usd: f64,
    pub sites: u32,
    pub real: bool,
    pub player: bool,
    /// Last day of the auction of an insolvent company's sites (M38).
    #[serde(default)]
    pub auction_until: Option<String>,
    /// The central departments it runs (`abteilung.<key>`) and their employees in all
    /// (ZA2, ZA4).
    #[serde(default)]
    pub departments: Vec<String>,
    #[serde(default)]
    pub central_staff: u32,
    /// A move of its headquarters under way: the new country and the first day there
    /// (ZA1, ZA4).
    #[serde(default)]
    pub moving_to: Option<String>,
    #[serde(default)]
    pub moving_until: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompaniesView {
    /// Active companies and insolvent ones whose sites are auctioned (M38), largest
    /// equity first.
    pub companies: Vec<CompanyRowView>,
}

fn row(state: &GameState, catalog: &crate::catalog::Catalog, id: CompanyId) -> CompanyRowView {
    let c = &state.companies[id.index()];
    CompanyRowView {
        index: id.0,
        name: c.name.clone(),
        headquarters: catalog.countries.key(c.headquarters).to_owned(),
        equity_usd: usd(equity(c)),
        revenue_year_usd: usd(revenue_of_year(c)),
        sites: u32::try_from(state.sites.iter().filter(|s| s.owner == id).count())
            .unwrap_or(u32::MAX),
        real: c.ai.as_ref().is_some_and(|a| a.real.is_some()),
        player: state.is_main(id),
        auction_until: deals::in_auction(state, id)
            .then_some(c.auction_until)
            .flatten()
            .map(iso),
        departments: catalog
            .central
            .departments
            .iter()
            .filter(|d| c.departments.contains_key(&d.kind))
            .map(|d| d.kind.key().to_owned())
            .collect(),
        central_staff: c.departments.values().sum(),
        moving_to: c
            .relocation
            .as_ref()
            .map(|r| catalog.countries.key(r.country).to_owned()),
        moving_until: c.relocation.as_ref().map(|r| iso(r.until)),
    }
}

pub fn companies(game: &Game) -> CompaniesView {
    let state = game.state();
    let mut list: Vec<CompanyRowView> = state
        .companies
        .iter()
        .enumerate()
        .filter(|(i, c)| {
            let id = CompanyId(u32::try_from(*i).unwrap_or(0));
            // The player's subsidiaries show under the organisation (W6).
            (state.is_main(id)
                || !state
                    .main_company
                    .is_some_and(|p| crate::group::same_group(state, id, p)))
                && (!c.bankrupt || deals::in_auction(state, id))
        })
        .map(|(i, _)| {
            row(
                state,
                game.catalog(),
                CompanyId(u32::try_from(i).unwrap_or(u32::MAX)),
            )
        })
        .collect();
    list.sort_by(|a, b| {
        b.equity_usd
            .total_cmp(&a.equity_usd)
            .then(a.name.cmp(&b.name))
    });
    CompaniesView { companies: list }
}

/// A site of another company, with what it is worth and whether the player can bid.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ForeignSiteView {
    pub site: u32,
    /// Text key of the site type.
    pub site_type: String,
    pub country: String,
    /// Facility keys with their number of units and their size (M36).
    pub facilities: Vec<(String, u32, String)>,
    pub products: Vec<String>,
    pub workers: f64,
    pub value: SiteValueView,
    /// What the same site would cost to build today.
    pub new_build_usd: f64,
    /// The owner needs the site itself (its power, its only laboratory) and sells it only
    /// for what a new one costs.
    pub needed: bool,
    /// Why the player cannot bid now: `zu_jung` (until `blocked_until`), `angebot_offen`
    /// (its number in `open_offer`), `gesperrt` (until `blocked_until`).
    pub blocked: Option<String>,
    pub blocked_until: Option<String>,
    pub open_offer: Option<u32>,
    /// Lowest bid while the site is auctioned (M38).
    #[serde(default)]
    pub min_bid_usd: Option<f64>,
}

/// A technology of another company the player could license.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LicenseView {
    pub technology: String,
    /// Research cost it saves the player.
    pub value_usd: f64,
    pub open_offer: Option<u32>,
    pub blocked_until: Option<String>,
}

/// An area of a company: all its sites of a goods group with the brand (M31).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AreaView {
    pub group: String,
    /// Its sites, as listed in `sites` of the company.
    pub sites: Vec<u32>,
    /// Awareness of the brand per country, highest first.
    pub brand: Vec<(String, f64)>,
    pub value: SiteValueView,
    /// What the sites and the advertising for the brand would cost today.
    pub new_build_usd: f64,
    /// Why the player cannot bid now, as for sites.
    pub blocked: Option<String>,
    pub blocked_until: Option<String>,
    pub open_offer: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompanyDetailView {
    pub company: CompanyRowView,
    /// The names the company gave its products (M42), in product order.
    #[serde(default)]
    pub products: Vec<NamedProductView>,
    pub sites: Vec<ForeignSiteView>,
    pub areas: Vec<AreaView>,
    pub licenses: Vec<LicenseView>,
    /// The player's cash (for the offer form).
    pub cash_usd: f64,
    pub min_age_months: u32,
    /// The company's managers with the player's offers to poach them (N46).
    #[serde(default)]
    pub managers: Vec<super::organisation::RivalManagerView>,
    /// The player's free positions the offers name.
    #[serde(default)]
    pub free_positions: Vec<super::organisation::FreePositionView>,
}

/// A product with the name a company gave it (M42).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NamedProductView {
    pub product: String,
    pub name: String,
}

/// Until when the player is blocked for an object of `seller`, and its open offer.
fn player_offer_state(
    game: &Game,
    seller: CompanyId,
    object: DealObject,
) -> (Option<u32>, Option<crate::calendar::Date>) {
    let state = game.state();
    let catalog = game.catalog();
    let player = game.player();
    let open = state
        .offers
        .iter()
        .find(|o| {
            o.status == OfferStatus::Open
                && o.buyer == player
                && o.seller == seller
                && o.object == object
        })
        .map(|o| o.id);
    let blocked = state
        .offers
        .iter()
        .filter(|o| {
            o.buyer == player
                && o.seller == seller
                && o.object == object
                && matches!(o.status, OfferStatus::Declined | OfferStatus::Expired)
        })
        .filter_map(|o| o.closed)
        .map(|c| c.add_months(catalog.deal_model.block_months))
        .filter(|&until| until > state.date)
        .max();
    (open, blocked)
}

pub fn company_detail(game: &Game, index: u32) -> Option<CompanyDetailView> {
    let state = game.state();
    let catalog = game.catalog();
    let id = CompanyId(index);
    let company = state.companies.get(id.index())?;
    // Insolvent companies show while their sites are auctioned (M38).
    let auction = deals::in_auction(state, id);
    if company.bankrupt && !auction {
        return None;
    }
    let player = game.player();
    let min_age = catalog.deal_model.min_age_months;
    let sites = state
        .sites
        .iter()
        .enumerate()
        .filter(|(_, s)| s.owner == id)
        .map(|(i, s)| {
            let site = SiteId(u32::try_from(i).unwrap_or(u32::MAX));
            let (open, blocked) = player_offer_state(game, id, DealObject::Site(site));
            let ready = s.founded.add_months(min_age);
            let (reason, until) = if id == player {
                (None, None)
            } else if let Some(_offer) = open {
                (Some("angebot_offen"), None)
            } else if auction {
                (None, None)
            } else if ready > state.date {
                (Some("zu_jung"), Some(ready))
            } else if let Some(until) = blocked {
                (Some("gesperrt"), Some(until))
            } else {
                (None, None)
            };
            ForeignSiteView {
                site: site.0,
                site_type: site_type_key(s.kind),
                country: catalog.countries.key(s.country).to_owned(),
                facilities: s
                    .slots
                    .iter()
                    .map(|sl| {
                        (
                            catalog.facilities.key(sl.facility).to_owned(),
                            sl.count,
                            sl.size.key().to_owned(),
                        )
                    })
                    .collect(),
                products: products_of(game, site),
                workers: s.workforce.values().sum(),
                value: value_view(game, &deals::site_value(state, catalog, site)),
                new_build_usd: usd(deals::new_site_cost(state, catalog, site)),
                needed: id != player && deals::needed_by_owner(state, catalog, site),
                blocked: reason.map(str::to_owned),
                blocked_until: until.map(iso),
                open_offer: open,
                min_bid_usd: auction.then(|| usd(deals::auction_minimum(state, catalog, site))),
            }
        })
        .collect();
    let groups: std::collections::BTreeSet<crate::ids::GoodsGroupId> = state
        .sites
        .iter()
        .enumerate()
        .filter(|(_, s)| s.owner == id && !company.bankrupt && deals::area_kind(s.kind))
        .flat_map(|(i, _)| {
            deals::site_groups(state, catalog, SiteId(u32::try_from(i).unwrap_or(u32::MAX)))
        })
        .collect();
    let areas = groups
        .into_iter()
        .map(|group| {
            let sites = deals::area_sites(state, catalog, id, group);
            let value = deals::area_value_of(state, catalog, id, group, &sites);
            let (open, blocked) = player_offer_state(game, id, DealObject::Area(group));
            let ready = sites
                .iter()
                .map(|s| state.sites[s.index()].founded.add_months(min_age))
                .min();
            let (reason, until) = if id == player {
                (None, None)
            } else if let Some(_offer) = open {
                (Some("angebot_offen"), None)
            } else if let Some(ready) = ready.filter(|&r| r > state.date) {
                (Some("zu_jung"), Some(ready))
            } else if let Some(until) = blocked {
                (Some("gesperrt"), Some(until))
            } else {
                (None, None)
            };
            let mut brand: Vec<(String, f64)> = company
                .brands
                .iter()
                .filter(|b| b.group == group)
                .map(|b| (catalog.countries.key(b.country).to_owned(), b.awareness))
                .collect();
            brand.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            AreaView {
                group: catalog.goods_groups.key(group).to_owned(),
                sites: sites.iter().map(|s| s.0).collect(),
                brand,
                value: area_value_view(game, &value),
                new_build_usd: usd(value.new_build),
                blocked: reason.map(str::to_owned),
                blocked_until: until.map(iso),
                open_offer: open,
            }
        })
        .collect();
    let licenses = if id == player || company.bankrupt {
        Vec::new()
    } else {
        catalog
            .technologies
            .ids()
            .filter(|&t| state.knows(catalog, id, t) && !state.knows(catalog, player, t))
            .filter_map(|t| {
                let value = deals::license_value(state, catalog, player, t)?;
                let (open, blocked) = player_offer_state(game, id, DealObject::License(t));
                Some(LicenseView {
                    technology: catalog.technologies.key(t).to_owned(),
                    value_usd: usd(value),
                    open_offer: open,
                    blocked_until: blocked.map(iso),
                })
            })
            .collect()
    };
    let products = state.companies[id.index()]
        .product_names
        .iter()
        .map(|(p, name)| NamedProductView {
            product: catalog.products.key(*p).to_owned(),
            name: name.clone(),
        })
        .collect();
    let (managers, free_positions) = if id == player {
        (Vec::new(), Vec::new())
    } else {
        let free = super::organisation::free_positions(game);
        let managers = super::organisation::rival_managers(game, id, &free);
        let free = free
            .iter()
            .map(|p| super::organisation::FreePositionView {
                place: super::organisation::place_view(catalog, state, p.unit),
                role: super::organisation::role_key(&p.role),
            })
            .collect();
        (managers, free)
    };
    Some(CompanyDetailView {
        company: row(state, catalog, id),
        products,
        sites,
        areas,
        licenses,
        cash_usd: usd(state.companies[player.index()].ledger.cash()),
        min_age_months: min_age,
        managers,
        free_positions,
    })
}

/// Hints for offers the player has to answer (overview "Zu erledigen").
pub(super) fn offer_hints(game: &Game) -> Vec<Message> {
    let state = game.state();
    let catalog = game.catalog();
    let player = game.player();
    state
        .offers
        .iter()
        .filter(|o| o.status == OfferStatus::Open && o.answering() == player)
        .map(|o| {
            let other = if o.buyer == player { o.seller } else { o.buyer };
            let key = if o.counter {
                deals::object_key(
                    o.object,
                    [
                        keys::HINT_COUNTER_SITE,
                        keys::HINT_COUNTER_LICENSE,
                        keys::HINT_COUNTER_AREA,
                    ],
                )
            } else {
                deals::object_key(
                    o.object,
                    [
                        keys::HINT_OFFER_SITE,
                        keys::HINT_OFFER_LICENSE,
                        keys::HINT_OFFER_AREA,
                    ],
                )
            };
            let m = Message::new(MessageKind::Info, key)
                .with(
                    "firma",
                    Param::Text(state.companies[other.index()].name.clone()),
                )
                .with("preis", Param::Money(o.price))
                .with("frist", Param::Date(o.deadline(catalog)));
            deals::describe(m, state, catalog, o.seller, o.object)
        })
        .collect()
}
