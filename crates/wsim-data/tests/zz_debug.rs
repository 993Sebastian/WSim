use std::sync::Arc;
use wsim_core::calendar::RoundLength;
use wsim_core::game::Game;
use wsim_core::money::Money;
use wsim_core::state::{AiSettings, GameSettings, StartForm};

#[test]
#[ignore]
fn debug_coverage() {
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
    let months: Vec<u32> = std::env::var("MONATE")
        .map(|m| m.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or(vec![1, 3, 6, 12, 24]);
    for month in 1..=*months.iter().max().unwrap() {
        game.advance(RoundLength::Month, |_| {});
        if !months.contains(&month) {
            continue;
        }
        let st = game.state();
        let mut line = format!("{} |", st.date);
        let mut worst = Vec::new();
        for (p, prod) in c.products.iter() {
            if prod.consumer_demand.is_none() {
                continue;
            }
            let (mut d, mut s) = (0.0, 0.0);
            for k in c.countries.ids() {
                let m = st.markets.get(p).get(k);
                d += m.last_month.demand;
                s += m.last_month.sold - m.last_month.exported;
            }
            if d <= 0.0 {
                continue;
            }
            let cov = s / d;
            worst.push((cov, c.products.key(p).to_string()));
        }
        worst.sort_by(|a, b| a.0.total_cmp(&b.0));
        let avg: f64 = worst.iter().map(|x| x.0).sum::<f64>() / worst.len() as f64;
        line += &format!(" mittel {:.0}% |", avg * 100.0);
        for (cv, k) in &worst {
            line += &format!(" {k} {:.0}", cv * 100.0);
        }
        println!("{line}");
    }
}

#[test]
#[ignore]
fn debug_countries() {
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
    let key = std::env::var("PRODUKT").unwrap_or("naegel".into());
    let p = c.products.id(&key).unwrap();
    let months: Vec<u32> = std::env::var("MONATE")
        .map(|m| m.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or(vec![3, 6]);
    for month in 1..=*months.iter().max().unwrap() {
        game.advance(RoundLength::Month, |_| {});
        if !months.contains(&month) {
            continue;
        }
        let st = game.state();
        println!("== {}", st.date);
        let mut rows = Vec::new();
        for k in c.countries.ids() {
            let m = st.markets.get(p).get(k);
            let demand = m.consumer_rate.iter().sum::<f64>() + m.state_rate;
            let (mut full, mut plan, mut stock, mut sold, mut price, mut lim) =
                (0.0, 0.0, 0.0, 0.0, 0.0, String::new());
            let mut sellers = 0;
            for site in st.sites.iter().filter(|s| s.country == k) {
                if st.companies[site.owner.index()].bankrupt {
                    continue;
                }
                for sl in &site.slots {
                    if let Some(r) = sl.recipe {
                        let r = c.recipes.get(r);
                        if r.product == p && sl.ready <= st.date {
                            let f = c.facilities.get(sl.facility).runs_per_day
                                * f64::from(sl.count)
                                * r.output;
                            full += f;
                            plan += f * sl.utilization;
                            if let Some(l) = sl.limit {
                                lim = format!("{l:?}");
                            }
                        }
                    }
                }
                stock += site.inventory.get(&p).map_or(0.0, |x| x.quantity);
                if let Some(o) = site.offers.get(&p) {
                    sold += o.sold_last_month / 30.0;
                    price = o.price.to_usd();
                    sellers += 1;
                }
            }
            let t = &m.last_month;
            if full + demand * 10.0 < 1.0 {
                continue;
            }
            rows.push((demand.max(full), format!("  {:3} bedarf/t {:8.1} bedient/t {:8.1} einf {:7.1} ausf {:7.1} | voll {:8.1} plan {:8.1} lager {:8.0} absatz/t {:7.1} preis {:7.1} markt {:7.1} imp {:7.0} offen {:6.1} anb {} {}",
                c.countries.key(k), t.demand/30.0, (t.sold - t.exported)/30.0, t.imported/30.0, t.exported/30.0, full, plan, stock, sold, price, m.price.to_usd(), m.imports.quantity, m.open_demand, sellers, lim)));
        }
        rows.sort_by(|a, b| b.0.total_cmp(&a.0));
        for (_, r) in rows.iter().take(25) {
            println!("{r}");
        }
    }
}

#[test]
#[ignore]
fn debug_leim() {
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
    let leim = c.products.id("leim").unwrap();
    let land = c
        .countries
        .id(&std::env::var("LAND").unwrap_or("CHN".into()))
        .unwrap();
    for day in 0..60 {
        game.advance(RoundLength::Day, |_| {});
        if day % 10 != 0 {
            continue;
        }
        let st = game.state();
        let m = st.markets.get(leim).get(land);
        let sp = wsim_core::market::state_price(&c, st, land, leim).map(|x| x.to_usd());
        println!(
            "{} markt {:.0} staatspreis {:?} heute {:?} idle {:?}",
            st.date,
            m.price.to_usd(),
            sp,
            m.today,
            m.idle_since
        );
        for s in st.sites.iter().filter(|s| s.country == land) {
            if let Some(o) = s.orders.get(&leim) {
                let stock = s.inventory.get(&leim).map_or(0.0, |x| x.quantity);
                println!(
                    "   {:30} ziel {:.2} lager {:.2} hoechst {:.0} gekauft_monat {:.2}",
                    st.companies[s.owner.index()].name,
                    o.target,
                    stock,
                    o.max_price.to_usd(),
                    o.bought_month
                );
            }
        }
    }
}
