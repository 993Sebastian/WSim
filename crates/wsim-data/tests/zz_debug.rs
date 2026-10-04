use std::sync::Arc;
use wsim_core::calendar::RoundLength;
use wsim_core::game::Game;
use wsim_core::money::Money;
use wsim_core::state::{AiSettings, Consignee, GameSettings, StartForm};

#[test]
#[ignore]
fn debug_trade() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let data = wsim_data::load_dir(&dir).data.unwrap();
    let c = Arc::new(data.catalog);
    let settings = GameSettings {
        seed: 1,
        start_year: 1900,
        start_country: c.countries.id("DEU").unwrap(),
        start_capital: Money::from_usd(100_000.0).unwrap(),
        start_form: StartForm::Workshop,
        company_name: "X".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: AiSettings {
            companies: 100,
            competence: 0.5,
            aggressiveness: 0.5,
        },
    };
    let mut game = Game::new(c.clone(), settings).unwrap();
    let key = std::env::var("PRODUKT").unwrap_or("blech".into());
    let p = c.products.id(&key).unwrap();
    let lands: Vec<String> = std::env::var("LAENDER")
        .unwrap_or("JPN,IND,RUS,ITA".into())
        .split(',')
        .map(String::from)
        .collect();
    for month in 1..=14 {
        game.advance(RoundLength::Month, |_| {});
        if month % 3 != 2 {
            continue;
        }
        let st = game.state();
        println!("== {}", st.date);
        for k in &lands {
            let k = c.countries.id(k).unwrap();
            let m = st.markets.get(p).get(k);
            let transit: f64 = st
                .shipments
                .iter()
                .filter(|s| s.product == p && s.to == Consignee::Importer(k))
                .map(|s| s.quantity)
                .sum();
            let ceiling = wsim_core::market::local_reference(&c, st, k, p).to_usd()
                * c.market_model.price_max_factor;
            println!(
                "  {} konsum/t {:7.1} bedient/t {:7.1} offen {:7.1} importlager {:8.0} unterwegs {:8.0} importpreis {:6.0} index {:6.0} decke {:6.0}",
                c.countries.key(k),
                m.consumer_rate.iter().sum::<f64>(),
                m.last_month.outside_sold / 30.0,
                m.open_demand,
                m.imports.quantity,
                transit,
                m.import_price.to_usd(),
                m.price.to_usd(),
                ceiling
            );
        }
        for k in ["USA", "DEU", "GBR", "FRA"] {
            let k = c.countries.id(k).unwrap();
            for s in st.sites.iter().filter(|s| s.country == k) {
                if let Some(o) = s.offers.get(&p) {
                    let stock = s.inventory.get(&p).map_or(0.0, |x| x.quantity);
                    println!(
                        "  quelle {} preis {:6.0} lager {:8.0} behalten {:6.0} verkauft {:8.0} an_haendler {:8.0} regel {:?}",
                        c.countries.key(k),
                        o.price.to_usd(),
                        stock,
                        o.keep,
                        o.sold_last_month,
                        o.to_traders_month,
                        st.companies[s.owner.index()].name
                    );
                }
            }
        }
    }
}

#[test]
#[ignore]
fn debug_margins() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let data = wsim_data::load_dir(&dir).data.unwrap();
    let c = data.catalog;
    for (pid, p) in c.products.iter() {
        let mut rows = Vec::new();
        for (rid, r) in c.recipes.iter().filter(|(_, r)| r.product == pid) {
            let year = wsim_core::health::first_year(&c, rid);
            let m = wsim_core::health::reference_margin(&c, rid).unwrap();
            rows.push((
                year,
                format!(
                    "{:14} {:10} ref {:8.0} marge {:5.0}% ab {} {:22} abbau {}",
                    c.products.key(pid),
                    format!("{:?}", p.kind),
                    p.reference_price.to_usd(),
                    m * 100.0,
                    year,
                    c.recipes.key(rid),
                    r.extraction
                ),
            ));
        }
        rows.sort_by_key(|a| a.0);
        for (_, r) in rows {
            println!("{r}");
        }
    }
}

#[test]
#[ignore]
fn debug_breakdown() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let data = wsim_data::load_dir(&dir).data.unwrap();
    let c = data.catalog;
    let keys: Vec<String> = std::env::var("REZEPTE")
        .unwrap_or_default()
        .split(',')
        .map(String::from)
        .collect();
    for (rid, r) in c.recipes.iter() {
        if !keys.iter().any(|k| k == c.recipes.key(rid)) {
            continue;
        }
        let year = wsim_core::health::first_year(&c, rid);
        let cs = wsim_core::country_model::compute(
            &c,
            c.country_model.price_reference.unwrap(),
            wsim_core::calendar::Date::new(year, 7, 1).unwrap(),
        );
        let f = c.facilities.get(r.facility);
        let price = |p: wsim_core::ids::ProductId| c.products.get(p).reference_price.to_usd();
        let out = r.output;
        println!(
            "== {} ({}), ausbringung {} je lauf, {} laeufe/tag, produktivitaet {:.2}, strom {:.0}",
            c.recipes.key(rid),
            year,
            out,
            f.runs_per_day,
            cs.labor_productivity,
            cs.electricity_price_usd_mwh
        );
        for &(p, q) in &r.inputs {
            println!(
                "   eingang {:14} {:8.3} x {:8.0} = {:9.1} je einheit",
                c.products.key(p),
                q,
                price(p),
                q * price(p) / out
            );
        }
        for &(p, q) in &r.by_products {
            println!(
                "   neben   {:14} {:8.3} x {:8.0} = {:9.1} je einheit",
                c.products.key(p),
                q,
                price(p),
                q * price(p) / out
            );
        }
        for &(g, h) in &r.labor_hours {
            let w = cs.hourly_wage_usd[wsim_core::ids::Id::index(g)];
            println!(
                "   arbeit  {:14} {:8.2} h x {:6.2} = {:9.1} je einheit",
                c.labor_groups.key(g),
                h,
                w,
                h * w / cs.labor_productivity / out
            );
        }
        println!(
            "   strom {:.3} MWh = {:.1}",
            r.energy_mwh,
            r.energy_mwh * cs.electricity_price_usd_mwh / out
        );
        let cap = f.investment.to_usd()
            * (1.0 / f64::from(f.lifetime_years.max(1)) + f.maintenance_share)
            / (365.0 * f.runs_per_day * c.market_model.normal_utilization);
        println!(
            "   kapital {:.1} (invest {:.0}, {} j, wartung {})",
            cap / out,
            f.investment.to_usd(),
            f.lifetime_years,
            f.maintenance_share
        );
        let p = c.products.get(r.product);
        println!(
            "   gemein {:.1}; richtpreis {:.0}; marge {:.0}%",
            c.production_model.overhead_share(p.kind) * price(r.product),
            price(r.product),
            wsim_core::health::reference_margin(&c, rid).unwrap() * 100.0
        );
    }
}

#[test]
#[ignore]
fn debug_parts() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let data = wsim_data::load_dir(&dir).data.unwrap();
    let c = data.catalog;
    println!("rezept;produkt;art;jahr;abbau;preis;eingang;umwandlung");
    for (rid, r) in c.recipes.iter() {
        let year = wsim_core::health::first_year(&c, rid);
        let cs = wsim_core::country_model::compute(
            &c,
            c.country_model.price_reference.unwrap(),
            wsim_core::calendar::Date::new(year, 7, 1).unwrap(),
        );
        let f = c.facilities.get(r.facility);
        let price = |p: wsim_core::ids::ProductId| c.products.get(p).reference_price.to_usd();
        let out = r.output;
        let inputs: f64 = r.inputs.iter().map(|&(p, q)| q * price(p)).sum::<f64>()
            - r.by_products
                .iter()
                .map(|&(p, q)| q * price(p))
                .sum::<f64>();
        let labor: f64 = r
            .labor_hours
            .iter()
            .map(|&(g, h)| h * cs.hourly_wage_usd[wsim_core::ids::Id::index(g)])
            .sum::<f64>()
            / cs.labor_productivity;
        let cap = f.investment.to_usd()
            * (1.0 / f64::from(f.lifetime_years.max(1)) + f.maintenance_share)
            / (365.0 * f.runs_per_day * c.market_model.normal_utilization);
        let energy = r.energy_mwh * cs.electricity_price_usd_mwh;
        let p = c.products.get(r.product);
        println!(
            "{};{};{:?};{};{};{:.2};{:.3};{:.3}",
            c.recipes.key(rid),
            c.products.key(r.product),
            p.kind,
            year,
            r.extraction,
            price(r.product),
            inputs / out,
            (labor + cap + energy) / out
        );
    }
}

#[test]
#[ignore]
fn debug_start() {
    use wsim_core::state::PriceMode;
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let data = wsim_data::load_dir(&dir).data.unwrap();
    let c = Arc::new(data.catalog);
    let settings = GameSettings {
        seed: 1,
        start_year: 1900,
        start_country: c.countries.id("DEU").unwrap(),
        start_capital: Money::from_usd(100_000.0).unwrap(),
        start_form: StartForm::Workshop,
        company_name: "X".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: AiSettings {
            companies: 100,
            competence: 0.5,
            aggressiveness: 0.5,
        },
    };
    let mut game = Game::new(c.clone(), settings).unwrap();
    let products: Vec<String> = std::env::var("PRODUKTE")
        .unwrap_or("stahl,draht,naegel".into())
        .split(',')
        .map(String::from)
        .collect();
    let days: i32 = std::env::var("TAGE")
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(90);
    for day in 0..days {
        if day > 0 {
            game.advance(RoundLength::Day, |_| {});
        }
        if day % 7 != 0 {
            continue;
        }
        let st = game.state();
        println!("== {}", st.date);
        for key in &products {
            let p = c.products.id(key).unwrap();
            let (mut stock, mut sold, mut n, mut price, mut floor, mut cap, mut made) =
                (0.0, 0.0, 0.0f64, 0.0, 0.0, 0.0, 0.0);
            for s in &st.sites {
                if let Some(o) = s.offers.get(&p) {
                    stock += s.inventory.get(&p).map_or(0.0, |x| x.quantity);
                    sold += o.sold_month;
                    n += 1.0;
                    price += o.price.to_usd()
                        / wsim_core::market::local_reference(&c, st, s.country, p).to_usd();
                    if let PriceMode::Market { floor: f, .. } = o.mode {
                        floor += f.to_usd()
                            / wsim_core::market::local_reference(&c, st, s.country, p).to_usd();
                    }
                }
                for sl in &s.slots {
                    if let Some(r) = sl.recipe {
                        let r2 = c.recipes.get(r);
                        if r2.product == p && sl.ready <= st.date {
                            let runs =
                                c.facilities.get(sl.facility).runs_per_day * f64::from(sl.count);
                            cap += runs * r2.output;
                            made += sl.last_runs * r2.output;
                        }
                    }
                }
            }
            let mut demand = 0.0;
            let mut open = 0.0;
            for k in c.countries.ids() {
                let m = st.markets.get(p).get(k);
                demand += m.consumer_rate.iter().sum::<f64>();
                open += m.open_demand;
            }
            println!(
                "  {:8} anbieter {:4} preis/ref {:5.2} boden/ref {:5.2} lager {:9.0} kapaz/t {:8.0} gemacht/t {:8.0} konsum/t {:7.0} offen {:8.0} verkauft_monat {:9.0}",
                key,
                n,
                price / n.max(1.0),
                floor / n.max(1.0),
                stock,
                cap,
                made,
                demand,
                open,
                sold
            );
        }
    }
}

#[test]
#[ignore]
fn debug_buyers() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let data = wsim_data::load_dir(&dir).data.unwrap();
    let c = Arc::new(data.catalog);
    let settings = GameSettings {
        seed: 1,
        start_year: 1900,
        start_country: c.countries.id("DEU").unwrap(),
        start_capital: Money::from_usd(100_000.0).unwrap(),
        start_form: StartForm::Workshop,
        company_name: "X".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: AiSettings {
            companies: 100,
            competence: 0.5,
            aggressiveness: 0.5,
        },
    };
    let mut game = Game::new(c.clone(), settings).unwrap();
    let product = c
        .products
        .id(&std::env::var("PRODUKT").unwrap_or("draht".into()))
        .unwrap();
    let days: i32 = std::env::var("TAGE")
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(30);
    let skip: i32 = std::env::var("AB")
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(0);
    let every: i32 = std::env::var("ALLE")
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(3);
    for day in 0..days {
        if day > 0 {
            game.advance(RoundLength::Day, |_| {});
        }
        if day < skip || day % every != 0 {
            continue;
        }
        let st = game.state();
        println!("== {}", st.date);
        for s in &st.sites {
            for sl in &s.slots {
                let Some(r) = sl.recipe else { continue };
                let r = c.recipes.get(r);
                if r.product != product || sl.ready > st.date {
                    continue;
                }
                let runs = c.facilities.get(sl.facility).runs_per_day * f64::from(sl.count);
                print!(
                    "  {} {:28} ausl {:4.2} laeufe {:6.1}/{:6.1} grenze {:?}",
                    c.countries.key(s.country),
                    st.companies[s.owner.index()]
                        .name
                        .chars()
                        .take(28)
                        .collect::<String>(),
                    sl.utilization,
                    sl.last_runs,
                    runs * sl.utilization,
                    sl.limit
                );
                for &(p, q) in &r.inputs {
                    let stock = s.inventory.get(&p).map_or(0.0, |x| x.quantity);
                    let po = s.orders.get(&p);
                    let m = st.markets.get(p).get(s.country);
                    print!(
                        " | {} lager {:7.0} bedarf/t {:6.1} ziel {:7.0} max {:6.0} gekauft {:7.0} markt {:6.0} import {:6.0} importlager {:7.0}",
                        c.products.key(p),
                        stock,
                        q * runs * sl.utilization,
                        po.map_or(0.0, |o| o.target),
                        po.map_or(0.0, |o| o.max_price.to_usd()),
                        po.map_or(0.0, |o| o.bought_month),
                        wsim_core::market::market_price(&c, st, s.country, p).to_usd(),
                        m.import_price.to_usd(),
                        m.imports.quantity
                    );
                }
                println!();
            }
        }
    }
}

#[test]
#[ignore]
fn debug_floors() {
    use wsim_core::state::PriceMode;
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let data = wsim_data::load_dir(&dir).data.unwrap();
    let c = Arc::new(data.catalog);
    let settings = GameSettings {
        seed: 1,
        start_year: 1900,
        start_country: c.countries.id("DEU").unwrap(),
        start_capital: Money::from_usd(100_000.0).unwrap(),
        start_form: StartForm::Workshop,
        company_name: "X".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: AiSettings {
            companies: 100,
            competence: 0.5,
            aggressiveness: 0.5,
        },
    };
    let mut game = Game::new(c.clone(), settings).unwrap();
    let p = c
        .products
        .id(&std::env::var("PRODUKT").unwrap_or("naegel".into()))
        .unwrap();
    let days: i32 = std::env::var("TAGE")
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(30);
    for _ in 0..days {
        game.advance(RoundLength::Day, |_| {});
    }
    let st = game.state();
    println!("== {}", st.date);
    let mut rows = Vec::new();
    for s in &st.sites {
        let Some(o) = s.offers.get(&p) else { continue };
        let lref = wsim_core::market::local_reference(&c, st, s.country, p).to_usd();
        let floor = if let PriceMode::Market { floor, .. } = o.mode {
            floor.to_usd()
        } else {
            0.0
        };
        let mut inputs = String::new();
        for sl in &s.slots {
            if let Some(r) = sl.recipe {
                let r = c.recipes.get(r);
                if r.product == p {
                    for &(i, _) in &r.inputs {
                        let mp = wsim_core::market::market_price(&c, st, s.country, i).to_usd();
                        let ir = wsim_core::market::local_reference(&c, st, s.country, i).to_usd();
                        inputs += &format!(
                            " {} {:.2}xLref ({:.2}xWelt)",
                            c.products.key(i),
                            mp / ir,
                            mp / c.products.get(i).reference_price.to_usd()
                        );
                    }
                }
            }
        }
        rows.push((
            st.countries.get(s.country).price_level,
            format!(
                "{} niveau {:.2} boden {:.2}xLref preis {:.2}xLref verkauft {:7.0}{}",
                c.countries.key(s.country),
                st.countries.get(s.country).price_level,
                floor / lref,
                o.price.to_usd() / lref,
                o.sold_last_month + o.sold_month,
                inputs
            ),
        ));
    }
    rows.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (_, r) in rows {
        println!("  {r}");
    }
}

#[test]
#[ignore]
fn debug_deposits() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let data = wsim_data::load_dir(&dir).data.unwrap();
    let c = Arc::new(data.catalog);
    let settings = GameSettings {
        seed: 1,
        start_year: 1900,
        start_country: c.countries.id("DEU").unwrap(),
        start_capital: Money::from_usd(100_000.0).unwrap(),
        start_form: StartForm::Workshop,
        company_name: "X".into(),
        research_ahead_factor: 1.0,
        market_scale: 1.0,
        ai: AiSettings {
            companies: 100,
            competence: 0.5,
            aggressiveness: 0.5,
        },
    };
    let mut game = Game::new(c.clone(), settings).unwrap();
    let p = c
        .products
        .id(&std::env::var("PRODUKT").unwrap_or("kautschuk".into()))
        .unwrap();
    let years: i32 = std::env::var("JAHRE")
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(10);
    for _ in 0..years {
        for _ in 0..12 {
            game.advance(RoundLength::Month, |_| {});
        }
        let st = game.state();
        // world need of processing plants and extraction
        let (mut need, mut extract_cap, mut extracted) = (0.0, 0.0, 0.0);
        for s in &st.sites {
            for sl in s.slots.iter().filter(|sl| sl.ready <= st.date) {
                let Some(r) = sl.recipe else { continue };
                let r = c.recipes.get(r);
                let runs = c.facilities.get(sl.facility).runs_per_day * f64::from(sl.count);
                for &(i, q) in &r.inputs {
                    if i == p {
                        need += runs * sl.utilization * q;
                    }
                }
                if r.product == p {
                    extract_cap += runs * r.output;
                    extracted += sl.last_runs * r.output;
                }
            }
        }
        let mut price = 0.0;
        let mut n = 0.0f64;
        for k in c.countries.ids() {
            let m = st.markets.get(p).get(k);
            if m.price > Money::ZERO {
                price += m.price.to_usd();
                n += 1.0;
            }
        }
        println!(
            "== {} bedarf/t {:8.1} kapaz/t {:8.1} gefoerdert/t {:8.1} mittl.index {:8.0}",
            st.date,
            need,
            extract_cap,
            extracted,
            price / n.max(1.0)
        );
        for (d, dep) in c.deposits.iter().filter(|(_, d)| d.resource == p) {
            let ds = st.deposits.get(d);
            let dev = ds.concessions.iter().filter(|x| x.site.is_some()).count();
            let ready = ds
                .concessions
                .iter()
                .filter(|x| x.ready.is_some_and(|r| r <= st.date))
                .count();
            println!(
                "   {:24} {} entdeckt {:?} max/j {:8.0} konz {}/{} fertig {} dieses_jahr {:8.0}",
                c.deposits.key(d),
                c.countries.key(dep.country),
                dep.discovered,
                dep.max_output_per_year * 0.1,
                dev,
                ds.concessions.len(),
                ready,
                ds.concessions
                    .iter()
                    .map(|x| x.extracted_this_year)
                    .sum::<f64>()
            );
        }
    }
}
