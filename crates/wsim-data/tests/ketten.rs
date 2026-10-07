//! Headless scenario with the shipped data: one company runs all 12 chains of stage 1
//! (Lastenheft §17.1) in the United States, from the raw materials to the end products.
//! Rubber comes from Brazil by ship; goods without a chain from the state market.

use std::path::Path;
use std::sync::Arc;

use wsim_core::calendar::RoundLength;
use wsim_core::catalog::{Catalog, SiteType};
use wsim_core::command::Command;
use wsim_core::game::Game;
use wsim_core::ids::{CountryId, ProductId};
use wsim_core::money::Money;
use wsim_core::state::{GameSettings, SiteId, StartForm};
use wsim_data::load_dir;

fn catalog() -> Arc<Catalog> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    Arc::new(load_dir(&dir).data.expect("data loads").catalog)
}

struct Scenario {
    game: Game,
    c: Arc<Catalog>,
}

impl Scenario {
    fn apply(&mut self, command: Command) {
        if let Err(e) = self.game.apply(command.clone()) {
            panic!("{command:?}: {e:?}");
        }
    }

    fn site(&mut self, country: CountryId, kind: SiteType) -> SiteId {
        self.apply(Command::FoundSite { country, kind });
        SiteId(u32::try_from(self.game.state().sites.len() - 1).unwrap())
    }

    fn facility(&mut self, site: SiteId, facility: &str, recipe: &str, utilization: f64) {
        let c = self.c.clone();
        self.apply(Command::BuildFacility {
            site,
            facility: c.facilities.id(facility).expect(facility),
            count: 1,
            size: wsim_core::catalog::FacilitySize::Medium,
        });
        let slot = self.game.state().sites[site.index()].slots.len() - 1;
        self.apply(Command::SetProduction {
            site,
            slot,
            recipe: Some(c.recipes.id(recipe).expect(recipe)),
            utilization,
        });
    }

    fn extraction(
        &mut self,
        country: CountryId,
        deposit: &str,
        facility: &str,
        recipe: &str,
    ) -> SiteId {
        let site = self.site(country, SiteType::Extraction);
        let deposit = self.c.deposits.id(deposit).expect(deposit);
        self.apply(Command::DevelopDeposit { site, deposit });
        self.facility(site, facility, recipe, 1.0);
        site
    }

    fn product(&self, key: &str) -> ProductId {
        self.c.products.id(key).expect(key)
    }

    fn stock(&self, site: SiteId, key: &str) -> f64 {
        self.game.state().sites[site.index()]
            .inventory
            .get(&self.product(key))
            .map_or(0.0, |s| s.quantity)
    }

    fn transfer(&mut self, from: SiteId, to: SiteId, key: &str, max: f64) {
        let quantity = self.stock(from, key).min(max);
        if quantity > 1e-6 {
            let product = self.product(key);
            self.apply(Command::TransferGoods {
                from,
                to,
                product,
                quantity,
            });
        }
    }
}

#[test]
fn all_chains_run_from_raw_material_to_end_product() {
    let c = catalog();
    let usa = c.countries.id("USA").unwrap();
    let bra = c.countries.id("BRA").unwrap();
    let game = Game::new(
        c.clone(),
        GameSettings {
            seed: 12,
            start_year: 1902,
            start_country: usa,
            start_capital: Money::from_usd(1.0e10).unwrap(),
            start_form: StartForm::Workshop,
            company_name: "Alle Ketten".into(),
            research_ahead_factor: 1.0,
            market_scale: 1.0,
            ai: Default::default(),
            ventures: 1.0,
        },
    )
    .unwrap();
    let mut s = Scenario { game, c };

    // Raw materials (key, site).
    let raw = [
        (
            "eisenerz",
            s.extraction(usa, "mesabi_range", "erzbergwerk", "eisenerz_abbau"),
        ),
        (
            "kohle",
            s.extraction(usa, "appalachen", "kohlenzeche", "kohle_abbau"),
        ),
        (
            "kupfererz",
            s.extraction(usa, "butte", "kupferbergwerk", "kupfererz_abbau"),
        ),
        (
            "holz",
            s.extraction(
                usa,
                "pazifischer_nordwesten",
                "forstbetrieb",
                "holz_einschlag",
            ),
        ),
        (
            "baumwolle",
            s.extraction(
                usa,
                "baumwollguertel",
                "baumwollplantage",
                "baumwolle_ernte",
            ),
        ),
        (
            "getreide",
            s.extraction(usa, "praerie_usa", "getreidefarm", "getreide_ernte"),
        ),
        (
            "rohoel",
            s.extraction(usa, "pennsylvania", "oelfeld", "rohoel_foerdern"),
        ),
        (
            "kautschuk",
            s.extraction(
                bra,
                "amazonas_kautschuk",
                "kautschukplantage",
                "kautschuk_zapfen",
            ),
        ),
    ];

    // Own power plant for electrolysis and light bulbs (the grid covers about 10 %).
    let power = s.site(usa, SiteType::PowerPlant);
    s.facility(power, "dampfkraftwerk", "strom_dampfmaschine", 1.0);

    // One works with all factories; they share its warehouse.
    let works = s.site(usa, SiteType::Factory);
    for (facility, recipe, utilization) in [
        ("stahlwerk_herdofen", "stahl_siemens_martin", 1.0),
        ("walzwerk", "blech_walzen", 0.1),
        ("walzwerk", "stabstahl_walzen", 0.15),
        ("drahtzieherei", "draht_ziehen", 0.3),
        ("nagelmaschine", "naegel_maschine", 1.0),
        ("schraubenfabrik", "schrauben_drehen", 0.4),
        ("schraubenfabrik", "muttern_pressen", 0.2),
        ("werkzeugschmiede", "werkzeug_schmieden", 0.5),
        ("saegewerk", "schnittholz_saegen", 0.5),
        ("moebelfabrik", "moebel_tischlern", 0.5),
        ("spinnerei", "garn_spinnen", 0.5),
        ("weberei", "stoff_weben", 0.5),
        ("konfektion", "kleidung_naehen", 0.5),
        ("muehle", "mehl_mahlen", 0.2),
        ("verzinnerei", "weissblech_verzinnen", 0.2),
        ("konservenfabrik", "konserve_abfuellen", 0.25),
        ("kupferhuette", "kupfer_verhuetten", 0.4),
        ("kupferdrahtwerk", "kupferdraht_ziehen", 0.25),
        ("elektromotorenwerk", "elektromotor_bauen", 0.2),
        ("raffinerie", "rohoel_destillieren", 0.3),
        ("lampenfabrik", "petroleumlampe_bauen", 0.3),
        ("gluehlampenfabrik", "gluehlampe_kohlefaden", 0.3),
        ("gummiwerk", "gummi_vulkanisieren", 0.4),
        ("reifenwerk", "fahrradreifen_bauen", 0.4),
        ("reifenwerk", "autoreifen_bauen", 0.1),
        ("naehmaschinenfabrik", "naehmaschine_bauen", 0.2),
        ("fahrradfabrik", "fahrrad_bauen", 0.2),
        ("motorenwerk", "motor_bauen", 0.25),
        ("fahrgestellwerk", "fahrgestell_bauen", 0.25),
        ("karosseriewerk", "karosserie_holz", 0.25),
        ("automobilwerk", "automobil_montage", 0.25),
    ] {
        s.facility(works, facility, recipe, utilization);
    }
    // Goods without a chain from the state market.
    for (key, target) in [
        ("leim", 10.0),
        ("zinn", 50.0),
        ("konserveninhalt", 200.0),
        ("glas", 50.0),
        ("schwefel", 20.0),
    ] {
        let product = s.product(key);
        s.apply(Command::SetPurchase {
            site: works,
            product,
            target,
            max_price: Money::from_usd(100_000.0).unwrap(),
            min_quality: 0.0,
        });
    }

    for _ in 0..1300 {
        s.game.advance(RoundLength::Day, |_| {});
        for (key, site) in raw {
            if key == "kohle" {
                let missing = 400.0 - s.stock(power, "kohle");
                s.transfer(site, power, key, missing.max(0.0));
            }
            s.transfer(site, works, key, f64::INFINITY);
        }
    }

    let state = s.game.state();
    assert!(!state.game_over);
    let company = &state.companies[s.game.player().index()];
    assert!(company.ledger.is_balanced());
    assert!(
        state.sites[power.index()].slots[0].last_runs > 0.0,
        "Kraftwerk läuft"
    );
    let mut missing = Vec::new();
    for key in [
        "stahl",
        "blech",
        "draht",
        "stabstahl",
        "naegel",
        "schrauben",
        "muttern",
        "handwerkzeug",
        "schnittholz",
        "moebel",
        "garn",
        "stoff",
        "kleidung",
        "mehl",
        "weissblech",
        "konserve",
        "kupfer",
        "kupferdraht",
        "elektromotor",
        "petroleum",
        "benzin",
        "petroleumlampe",
        "gluehlampe",
        "gummi",
        "fahrradreifen",
        "autoreifen",
        "naehmaschine",
        "fahrrad",
        "motor",
        "fahrgestell",
        "karosserie",
        "automobil",
    ] {
        let stock = state.sites[works.index()].inventory.get(&s.product(key));
        match stock {
            Some(st) if st.quantity > 1e-6 => {
                let unit = st.value.to_usd() / st.quantity;
                let reference = s.c.products.get(s.product(key)).reference_price.to_usd();
                eprintln!(
                    "{key:<16} {:>14.2} auf Lager, Herstellkosten {unit:>10.2} USD (Richtpreis {reference:>9.0})",
                    st.quantity
                );
            }
            _ => missing.push(key),
        }
    }
    assert!(missing.is_empty(), "nicht hergestellt: {missing:?}");
}
