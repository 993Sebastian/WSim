//! The production model.

use std::collections::BTreeMap;

use wsim_core::catalog::{
    Catalog, DevelopmentModel, FacilitySize, FleetVehicle, ImprovementEffect, ImprovementModel,
    ImprovementTarget, LabRole, LabStaffModel, LeadTrait, LearningModel, Light, MaintenanceModel,
    OperationsModel, PatentModel, PhaseRole, ProductionModel, ProjectChoice, ProjectEventKind,
    ProjectEventModel, ProjectEventOption, ProjectLeadModel, ProjectModel, ProjectPhase,
    ProjectReportModel, ProjectRiskModel, ProjectTempo, ProjectTrim, ResearchModel, SiteType,
    SizeModel, StartSetup, StorageClass, StorageModel, SuggestionModel, TrainingModel,
    TransportModel, Vehicle, Way,
};
use wsim_core::catalog::{
    CouncilModel, DevPartnershipModel, FastTrackModel, FirstMoverModel, KnowledgeModel,
    LeadershipModel, PilotModel, PlatformModel, ProjectLicenseModel, SubsidyProgram,
};
use wsim_core::ids::{BranchId, Id, QualificationId, SpecializationId};
use wsim_core::state::StartForm;

use super::{
    HISTORY_YEARS, Keys, in_range, money, non_negative, per_kind, positive, provenance, resolve,
    time_series, year,
};
use crate::messages;
use crate::raw::{
    RawFacilitySizes, RawLabStaff, RawLight, RawLimits, RawPatents, RawProductDevelopment,
    RawProductionModel, RawProjectChoice, RawProjectModel, RawProjectPhase, RawSeries, RawVehicle,
    RawWay,
};
use crate::read::{Ctx, Entry, Loc, RawData};
use crate::suggest;
use crate::texts::TextIndex;

/// Text prefix of the facility sizes.
const SIZE_TEXT: &str = "anlagengroesse";

/// The one entry of a parameter section; reports a missing or repeated section.
pub(super) fn single<'r, T>(
    ctx: &mut Ctx,
    entries: &'r [Entry<T>],
    section: &str,
    file: &str,
) -> Option<&'r Entry<T>> {
    match entries {
        [] => {
            ctx.general_error(messages::section_missing(section, file));
            None
        }
        [first, rest @ ..] => {
            let first_loc = ctx.describe(&first.loc);
            for other in rest {
                ctx.error(&other.loc, messages::section_duplicate(section, &first_loc));
            }
            Some(first)
        }
    }
}

pub(super) const SITE_TYPES: &[(&str, SiteType)] = &[
    ("foerderstaette", SiteType::Extraction),
    ("werk", SiteType::Factory),
    ("kraftwerk", SiteType::PowerPlant),
    ("lager", SiteType::Warehouse),
    ("niederlassung", SiteType::SalesOffice),
    ("forschungszentrum", SiteType::ResearchCenter),
];

pub(super) fn production_model(
    ctx: &mut Ctx,
    raw: &RawData,
    (products, facilities, recipes, groups): (&Keys, &Keys, &Keys, &Keys),
) -> ProductionModel {
    let Some(entry) = single(
        ctx,
        &raw.production_model,
        "produktionsmodell",
        "parameter/produktionsmodell.yaml",
    ) else {
        return ProductionModel::default();
    };
    let m = &entry.value;
    let l = &entry.loc;
    let costs = l.field("standortkosten_usd");
    let names: Vec<&str> = SITE_TYPES.iter().map(|(n, _)| *n).collect();
    let mut site_cost = Vec::new();
    for (key, &usd) in &m.site_cost {
        match SITE_TYPES.iter().find(|(n, _)| n == key) {
            Some(&(_, kind)) => site_cost.push((kind, money(ctx, usd, &costs.field(key)))),
            None => ctx.error(
                &costs.key(key),
                messages::unknown_value(key, &names, suggest::closest(key, names.iter().copied())),
            ),
        }
    }
    for (name, _) in SITE_TYPES {
        if !m.site_cost.contains_key(*name) {
            ctx.error(&costs, messages::entry_missing("Standorttyp", name));
        }
    }
    let automation = l.field("automatisierung");
    let quality = l.field("qualitaet");
    ProductionModel {
        site_cost,
        building_lifetime_years: positive(
            ctx,
            m.building_lifetime_years,
            &l.field("gebaeude_lebensdauer_jahre"),
        ),
        development_lifetime_years: positive(
            ctx,
            m.development_lifetime_years,
            &l.field("erschliessung_lebensdauer_jahre"),
        ),
        decline_from: in_range(ctx, m.decline_from, 0.0, 1.0, &l.field("foerderkurve_ab")),
        commodity_cycle: m.commodity_cycle.as_ref().map_or_else(
            wsim_core::catalog::CommodityCycle::default,
            |c| {
                let cl = l.field("rohstoffzyklus");
                wsim_core::catalog::CommodityCycle {
                    shock: in_range(ctx, c.schwankung, 0.0, 0.5, &cl.field("schwankung")),
                    persistence: in_range(ctx, c.beharrung, 0.0, 0.99, &cl.field("beharrung")),
                    limit: in_range(ctx, c.grenze, 0.0, 1.5, &cl.field("grenze")),
                }
            },
        ),
        training: {
            let t = &m.training;
            let tl = l.field("schulung");
            TrainingModel {
                cost_share: in_range(ctx, t.cost_share, 0.0, 1.0, &tl.field("kosten_anteil_lohn")),
                gain_per_month: in_range(
                    ctx,
                    t.gain_per_month,
                    0.0,
                    1.0,
                    &tl.field("aufbau_je_monat"),
                ),
                loss_per_month: in_range(
                    ctx,
                    t.loss_per_month,
                    0.0,
                    1.0,
                    &tl.field("verlust_je_monat"),
                ),
                labor_saving: in_range(
                    ctx,
                    t.labor_saving,
                    0.0,
                    0.9,
                    &tl.field("arbeitsersparnis"),
                ),
                quality_points: in_range(
                    ctx,
                    t.quality_points,
                    0.0,
                    50.0,
                    &tl.field("qualitaet_punkte"),
                ),
            }
        },
        automation_labor_saving: in_range(
            ctx,
            m.automation.labor_saving,
            0.0,
            1.0,
            &automation.field("arbeitsersparnis"),
        ),
        automation_cost_share: in_range(
            ctx,
            m.automation.cost_share,
            0.0,
            10.0,
            &automation.field("kostenanteil"),
        ),
        quality_inputs: in_range(
            ctx,
            m.quality.inputs,
            0.0,
            1.0,
            &quality.field("vorprodukte"),
        ),
        quality_automation: in_range(
            ctx,
            m.quality.automation,
            0.0,
            100.0,
            &quality.field("automatisierung"),
        ),
        quality_condition: in_range(
            ctx,
            m.quality.condition,
            0.0,
            100.0,
            &quality.field("zustand"),
        ),
        condition_min: in_range(ctx, m.condition_min, 0.0, 1.0, &l.field("zustand_minimum")),
        electricity: m
            .electricity
            .as_ref()
            .map(|key| resolve(ctx, products, key, &l.field("strom"))),
        feed_in_share: in_range(
            ctx,
            m.feed_in_share,
            0.0,
            1.0,
            &l.field("einspeiseverguetung"),
        ),
        overhead_share: per_kind(
            ctx,
            &m.overhead_share,
            &l.field("gemeinkosten_anteil"),
            |ctx, v, loc| in_range(ctx, v, 0.0, 5.0, loc),
        ),
        reference_margin: reference_margin(ctx, &m.reference_margin, &l.field("richtpreis_marge")),
        by_product_stock_days: positive(
            ctx,
            m.by_product_stock_days,
            &l.field("nebenprodukte_lager_tage"),
        ),
        wage_premium_max: in_range(
            ctx,
            m.wage_premium_max,
            0.0,
            5.0,
            &l.field("lohnaufschlag_max"),
        ),
        mothball_maintenance_share: in_range(
            ctx,
            m.mothballing.maintenance_share,
            0.0,
            1.0,
            &l.field("stilllegung").field("instandhaltung_anteil"),
        ),
        restart_days: m.mothballing.restart_days,
        restart_cost_share: in_range(
            ctx,
            m.mothballing.restart_cost_share,
            0.0,
            1.0,
            &l.field("stilllegung").field("wiederanlauf_kosten"),
        ),
        sale_proceeds_share: in_range(
            ctx,
            m.facility_sale.proceeds_share,
            0.0,
            1.0,
            &l.field("verkauf").field("erloes_anteil"),
        ),
        scrap_share: in_range(
            ctx,
            m.facility_sale.scrap_share,
            0.0,
            1.0,
            &l.field("verkauf").field("schrottwert"),
        ),
        close_stock_share: in_range(
            ctx,
            m.closing.stock_proceeds_share,
            0.0,
            1.0,
            &l.field("schliessung").field("lager_erloes_anteil"),
        ),
        severance_months: in_range(
            ctx,
            m.closing.severance_months,
            0.0,
            36.0,
            &l.field("schliessung").field("abfindung_monate"),
        ),
        storage: m
            .storage
            .as_ref()
            .map(|v| storage(ctx, v, &l.field("lager"), groups))
            .unwrap_or_default(),
        learning: LearningModel::default(),
        trims: wsim_core::catalog::TrimModel::default(),
        maintenance: m
            .maintenance
            .as_ref()
            .map(|v| maintenance(ctx, v, &l.field("instandhaltung")))
            .unwrap_or_default(),
        operations: m
            .operations
            .as_ref()
            .map(|v| operations(ctx, v, &l.field("betrieb")))
            .unwrap_or_default(),
        start_setups: start_setups(ctx, m, l, (products, facilities, recipes)),
        sizes: sizes(ctx, &m.sizes, &l.field("anlagengroessen")),
    }
}

/// Scrap, accidents, corruption and rule of law (BP4): shares in range.
fn operations(ctx: &mut Ctx, v: &crate::raw::RawOperations, l: &Loc) -> OperationsModel {
    let sl = l.field("ausschuss");
    let cl = l.field("korruption");
    let rl = l.field("rechtssicherheit");
    let al = l.field("unfaelle_je_100000_stunden");
    for (&y, &n) in &v.accidents {
        in_range(ctx, n, 0.0, 1000.0, &al.field(&y.to_string()));
    }
    let share = |ctx: &mut Ctx, x: f64, at: &Loc| in_range(ctx, x, 0.0, 1.0, at);
    let scrap_min = share(ctx, v.scrap.min, &sl.field("min"));
    let scrap_max = share(ctx, v.scrap.max, &sl.field("max"));
    if scrap_min > scrap_max {
        ctx.error(&sl.field("min"), messages::range_order("min", "max"));
    }
    OperationsModel {
        enabled: true,
        scrap_base: per_kind(ctx, &v.scrap.base, &sl.field("basis"), |ctx, x, loc| {
            in_range(ctx, x, 0.0, 1.0, loc)
        }),
        scrap_min,
        scrap_max,
        scrap_training: share(ctx, v.scrap.training, &sl.field("schulung")),
        scrap_development: share(ctx, v.scrap.development, &sl.field("weiterentwicklung")),
        accidents: Some(time_series(ctx, &v.accidents, &al)),
        accident_hours: in_range(
            ctx,
            v.accident_hours,
            0.0,
            10_000.0,
            &l.field("unfall_kosten_stunden"),
        ),
        accident_stop_share: share(
            ctx,
            v.accident_stop_share,
            &l.field("unfall_stillstand_anteil"),
        ),
        corruption: (
            share(ctx, v.corruption.base, &cl.field("grund")),
            share(ctx, v.corruption.development, &cl.field("entwicklung")),
            share(ctx, v.corruption.instability, &cl.field("instabilitaet")),
        ),
        corruption_build_time: in_range(
            ctx,
            v.corruption.build_time,
            0.0,
            5.0,
            &cl.field("bauzeit"),
        ),
        corruption_fee: share(ctx, v.corruption.fee, &cl.field("gebuehr")),
        demand_chance: share(
            ctx,
            v.corruption.demand_chance,
            &cl.field("forderung_chance"),
        ),
        demand_share: share(
            ctx,
            v.corruption.demand_share,
            &cl.field("forderung_anteil"),
        ),
        rule_of_law: (
            share(ctx, v.rule_of_law.base, &rl.field("grund")),
            share(ctx, v.rule_of_law.development, &rl.field("entwicklung")),
            share(ctx, v.rule_of_law.stability, &rl.field("stabilitaet")),
        ),
        expropriation: share(ctx, v.expropriation, &l.field("enteignung_basis")),
        make_or_buy_tolerance: share(
            ctx,
            v.make_or_buy_tolerance,
            &l.field("make_or_buy_toleranz"),
        ),
    }
}

/// Maintenance (BP2): levels around 1, shares in range, days in order.
fn maintenance(ctx: &mut Ctx, v: &crate::raw::RawMaintenance, l: &Loc) -> MaintenanceModel {
    let fl = l.field("ausfall");
    let ol = l.field("ueberholung");
    let level_min = in_range(ctx, v.level_min, 0.1, 1.0, &l.field("stufe_min"));
    let level_max = in_range(ctx, v.level_max, 1.0, 5.0, &l.field("stufe_max"));
    if v.failure.days_min == 0 || v.failure.days_min > v.failure.days_max {
        ctx.error(
            &fl.field("tage_min"),
            messages::range_order("tage_min", "tage_max"),
        );
    }
    MaintenanceModel {
        enabled: true,
        level_min,
        level_max,
        exponent: in_range(ctx, v.exponent, 0.0, 5.0, &l.field("exponent")),
        capacity_weight: in_range(
            ctx,
            v.capacity_weight,
            0.0,
            1.0,
            &l.field("kapazitaet_zustand"),
        ),
        failure_chance: in_range(ctx, v.failure.chance, 0.0, 1.0, &fl.field("basis")),
        failure_days: (v.failure.days_min, v.failure.days_max),
        failure_cost_share: in_range(
            ctx,
            v.failure.cost_share,
            0.0,
            1.0,
            &fl.field("kosten_anteil"),
        ),
        failures_warning: v.failure.warning,
        overhaul_cost_share: in_range(
            ctx,
            v.overhaul.cost_share,
            0.0,
            1.0,
            &ol.field("kosten_anteil"),
        ),
        overhaul_days: v.overhaul.days,
        ai_overhaul_below: in_range(ctx, v.overhaul.ai_below, 0.0, 1.0, &ol.field("ki_ab")),
    }
}

/// Learning curve (BP1): rates per branch below 1, shares in range.
/// Demand trends of the market model (MN1).
pub(super) fn trends(
    ctx: &mut Ctx,
    raw: &RawData,
    (products, groups, countries): (&Keys, &Keys, &Keys),
) -> Vec<wsim_core::catalog::Trend> {
    let Some(entry) = raw.market_model.first() else {
        return Vec::new();
    };
    let l = entry.loc.field("trends");
    let mut seen = std::collections::BTreeSet::new();
    entry
        .value
        .trends
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let tl = l.index(i);
            if !seen.insert(t.id.as_str()) {
                ctx.error(&tl.field("id"), messages::trend_duplicate(&t.id));
            }
            if t.groups.is_empty() && t.products.is_empty() {
                ctx.error(&tl, messages::trend_without_goods(&t.id));
            }
            fn list<I: Id>(ctx: &mut Ctx, keys: &[String], refs: &Keys, loc: &Loc) -> Vec<I> {
                keys.iter()
                    .enumerate()
                    .map(|(j, k)| resolve(ctx, refs, k, &loc.index(j)))
                    .collect()
            }
            let curve_loc = tl.field("verlauf");
            for (&year, &f) in &t.curve {
                in_range(ctx, f, 0.1, 10.0, &curve_loc.key(&year.to_string()));
            }
            wsim_core::catalog::Trend {
                key: t.id.clone(),
                groups: list(ctx, &t.groups, groups, &tl.field("warengruppen")),
                products: list(ctx, &t.products, products, &tl.field("produkte")),
                countries: list(ctx, &t.countries, countries, &tl.field("laender")),
                curve: time_series(ctx, &t.curve, &curve_loc),
            }
        })
        .collect()
}

/// Tenders of governments (MN2).
pub(super) fn tenders(
    ctx: &mut Ctx,
    raw: &RawData,
    groups: &Keys,
) -> wsim_core::catalog::TenderModel {
    let Some(entry) = raw.market_model.first() else {
        return Default::default();
    };
    let Some(t) = entry.value.tenders.as_ref() else {
        return Default::default();
    };
    let l = entry.loc.field("ausschreibung");
    let gl = l.field("warengruppen");
    if t.groups.is_empty() {
        ctx.error(&gl, messages::list_empty());
    }
    let deadline_days = if (1..=90).contains(&t.deadline_days) {
        t.deadline_days
    } else {
        ctx.error(
            &l.field("frist_tage"),
            messages::out_of_range(f64::from(t.deadline_days), 1.0, 90.0),
        );
        30
    };
    let months = if (1..=120).contains(&t.months) {
        t.months
    } else {
        ctx.error(
            &l.field("laufzeit_monate"),
            messages::out_of_range(f64::from(t.months), 1.0, 120.0),
        );
        12
    };
    wsim_core::catalog::TenderModel {
        enabled: true,
        groups: t
            .groups
            .iter()
            .enumerate()
            .map(|(i, k)| resolve(ctx, groups, k, &gl.index(i)))
            .collect(),
        lot_share: in_range(ctx, t.lot_share, 0.0, 1.0, &l.field("los_anteil")),
        deadline_days,
        months,
        price_weight: in_range(ctx, t.price_weight, 0.0, 10.0, &l.field("preisgewicht")),
        quality_weight: in_range(
            ctx,
            t.quality_weight,
            0.0,
            10.0,
            &l.field("qualitaetsgewicht"),
        ),
        home_bonus: in_range(ctx, t.home_bonus, 0.0, 10.0, &l.field("heimatbonus")),
        war_factor: in_range(ctx, t.war_factor, 1.0, 10.0, &l.field("kriegsfaktor")),
        reputation_weight: in_range(ctx, t.reputation_weight, 0.0, 10.0, &l.field("rufgewicht")),
        ai_discount: in_range(ctx, t.ai_discount, 0.0, 0.9, &l.field("ki_nachlass")),
        penalty: in_range(ctx, t.penalty, 0.0, 2.0, &l.field("strafe")),
    }
}

/// A sales channel (MN3).
pub(super) fn channel(
    ctx: &mut Ctx,
    e: &Entry<crate::raw::RawChannel>,
    groups: &Keys,
) -> wsim_core::catalog::Channel {
    let v = &e.value;
    let l = &e.loc;
    let available_from = year(
        ctx,
        v.available_from,
        HISTORY_YEARS,
        &l.field("verfuegbar_ab"),
    );
    let gl = l.field("warengruppen");
    let groups = v
        .groups
        .iter()
        .enumerate()
        .map(|(i, k)| resolve(ctx, groups, k, &gl.index(i)))
        .collect();
    let reach_loc = l.field("reichweite");
    for (&y, &r) in &v.reach {
        in_range(ctx, r, 0.0, 1.0, &reach_loc.field(&y.to_string()));
    }
    let fixed_loc = l.field("fixkosten_je_land_usd");
    for (&y, &c) in &v.fixed_cost_usd {
        non_negative(ctx, c, &fixed_loc.field(&y.to_string()));
    }
    let fl = l.field("schichtfaktor");
    for (i, &f) in v.layer_factor.iter().enumerate() {
        in_range(ctx, f, 0.0, 5.0, &fl.index(i));
    }
    let layer_factor = <[f64; 5]>::try_from(v.layer_factor.as_slice()).unwrap_or_else(|_| {
        ctx.error(
            &fl,
            messages::wrong_length(v.layer_factor.len(), "5 Werte (ärmstes Fünftel zuerst)"),
        );
        [1.0; 5]
    });
    wsim_core::catalog::Channel {
        available_from,
        groups,
        reach: time_series(ctx, &v.reach, &reach_loc),
        layer_factor,
        fixed_cost_usd: time_series(ctx, &v.fixed_cost_usd, &fixed_loc),
        cost_per_sale: in_range(
            ctx,
            v.cost_per_sale,
            0.0,
            0.9,
            &l.field("kosten_je_verkauf"),
        ),
        reputation_factor: in_range(ctx, v.reputation_factor, 0.0, 5.0, &l.field("ruf_faktor")),
        provenance: provenance(v.approximation, v.source.as_ref()),
    }
}

/// Trim levels of end products (MN2).
pub(super) fn trims(ctx: &mut Ctx, raw: &RawData) -> wsim_core::catalog::TrimModel {
    use wsim_core::catalog::{TrimEffect, TrimModel};
    let Some(entry) = raw.production_model.first() else {
        return TrimModel::default();
    };
    let Some(v) = entry.value.trims.as_ref() else {
        return TrimModel::default();
    };
    let l = entry.loc.field("ausstattung");
    let mut level = |t: &crate::raw::RawTrim, key: &str| {
        let tl = l.field(key);
        TrimEffect {
            material: in_range(ctx, t.material, 0.3, 3.0, &tl.field("material")),
            labor: in_range(ctx, t.labor, 0.3, 3.0, &tl.field("arbeit")),
            quality: in_range(ctx, t.quality, -50.0, 50.0, &tl.field("qualitaet")),
        }
    };
    let simple = level(&v.einfach, "einfach");
    let premium = level(&v.premium, "premium");
    let ai_premium_from = in_range(ctx, v.ai_premium_from, 0.0, 1.0, &l.field("ki_premium_ab"));
    let ai_simple_up_to = in_range(ctx, v.ai_simple_up_to, 0.0, 1.0, &l.field("ki_einfach_bis"));
    if ai_simple_up_to > ai_premium_from {
        ctx.error(
            &l.field("ki_einfach_bis"),
            messages::trim_ai_bounds(ai_simple_up_to, ai_premium_from),
        );
    }
    TrimModel {
        enabled: true,
        simple,
        premium,
        ai_premium_from,
        ai_simple_up_to,
    }
}

pub(super) fn learning(ctx: &mut Ctx, raw: &RawData, branches: &Keys) -> LearningModel {
    let Some(entry) = raw.production_model.first() else {
        return LearningModel::default();
    };
    let Some(v) = entry.value.learning.as_ref() else {
        return LearningModel::default();
    };
    let l = entry.loc.field("lernkurve");
    let rl = l.field("rate");
    let rates = v
        .rates
        .iter()
        .map(|(key, &rate)| {
            (
                super::resolve(ctx, branches, key, &rl.key(key)),
                in_range(ctx, rate, 0.5, 1.0, &rl.field(key)),
            )
        })
        .collect();
    LearningModel {
        enabled: true,
        rates,
        default_rate: in_range(ctx, v.default_rate, 0.5, 1.0, &l.field("rate_sonst")),
        minimum: in_range(ctx, v.minimum, 0.05, 1.0, &l.field("untergrenze")),
        forget_per_month: in_range(
            ctx,
            v.forget_per_month,
            0.0,
            1.0,
            &l.field("vergessen_je_monat"),
        ),
        layoff_loss: in_range(ctx, v.layoff_loss, 0.0, 1.0, &l.field("entlassung")),
        transfer: in_range(ctx, v.transfer, 0.0, 1.0, &l.field("uebertrag")),
        established_years: in_range(
            ctx,
            v.established_years,
            0.0,
            1000.0,
            &l.field("start_etabliert_jahre"),
        ),
    }
}

/// Storage of the sites: a minimum for every site type, positive steps, shares in range.
fn storage(ctx: &mut Ctx, v: &crate::raw::RawStorage, l: &Loc, groups: &Keys) -> StorageModel {
    let ml = l.field("mindestens_m3");
    let names: Vec<&str> = SITE_TYPES.iter().map(|(n, _)| *n).collect();
    let mut minimum = Vec::new();
    for (key, &m3) in &v.minimum_m3 {
        match SITE_TYPES.iter().find(|(n, _)| n == key) {
            Some(&(_, kind)) => minimum.push((kind, in_range(ctx, m3, 0.0, 1e9, &ml.field(key)))),
            None => ctx.error(
                &ml.key(key),
                messages::unknown_value(key, &names, suggest::closest(key, names.iter().copied())),
            ),
        }
    }
    for (name, _) in SITE_TYPES {
        if !v.minimum_m3.contains_key(*name) {
            ctx.error(&ml, messages::entry_missing("Standorttyp", name));
        }
    }
    let sl = l.field("verderb");
    let spoilage = v
        .spoilage
        .iter()
        .map(|(key, &share)| {
            (
                super::resolve(ctx, groups, key, &sl.key(key)),
                in_range(ctx, share, 0.0, 1.0, &sl.field(key)),
            )
        })
        .collect();
    let step = l.field("ausbau");
    StorageModel {
        enabled: true,
        minimum,
        days: positive(ctx, v.days, &l.field("tage")),
        step_capacity: positive(ctx, v.step.capacity_m3, &step.field("kapazitaet_m3")),
        step_cost: money(ctx, v.step.cost_usd, &step.field("kosten_usd")),
        cost_share_year: in_range(
            ctx,
            v.cost_share_year,
            0.0,
            1.0,
            &l.field("kosten_jahr_anteil"),
        ),
        spoilage,
        ai_expand_at: in_range(ctx, v.ai_expand_at, 0.0, 1.0, &l.field("ki_ausbau_ab")),
        classes: v
            .classes
            .iter()
            .filter_map(|(key, &share)| {
                let at = l.field("klassen");
                let class = storage_class(ctx, key, &at.key(key))?;
                Some((class, in_range(ctx, share, 0.0, 1.0, &at.field(key))))
            })
            .collect(),
        product_class: Vec::new(),
        shelf_life: Vec::new(),
        volume: Vec::new(),
        external_capacity: v.external.as_ref().map_or(1.0, |e| {
            in_range(
                ctx,
                e.capacity,
                1.0,
                10.0,
                &l.field("fremdlager").field("kapazitaet_faktor"),
            )
        }),
        external_rate: v.external.as_ref().map_or(1.0, |e| {
            in_range(
                ctx,
                e.rate,
                1.0,
                10.0,
                &l.field("fremdlager").field("kosten_faktor"),
            )
        }),
        spoil_message_share: v.spoil_message_share.map_or(0.05, |s| {
            in_range(ctx, s, 0.0, 1.0, &l.field("verderb_meldung_ab"))
        }),
    }
}

fn storage_class(ctx: &mut Ctx, key: &str, at: &Loc) -> Option<StorageClass> {
    let names: Vec<&str> = StorageClass::ALL.iter().map(|c| c.key()).collect();
    let found = StorageClass::ALL.into_iter().find(|c| c.key() == key);
    if found.is_none() {
        ctx.error(
            at,
            messages::unknown_value(key, &names, suggest::closest(key, names.iter().copied())),
        );
    }
    found
}

/// Storage class and shelf life of every product (BP3): the product's own class, else by
/// goods group, else by transport class (each needs one); short-lived goods are chilled.
pub(super) fn storage_classes(
    ctx: &mut Ctx,
    raw: &RawData,
    products: &[&Entry<crate::raw::RawProduct>],
    catalog: &mut Catalog,
) {
    let Some(entry) = raw.production_model.first() else {
        return;
    };
    let Some(v) = entry.value.storage.as_ref() else {
        return;
    };
    let l = entry.loc.field("lager");
    let mut by_transport = BTreeMap::new();
    for (key, class) in &v.by_transport {
        let at = l.field("nach_transportklasse");
        if catalog.transport_classes.id(key).is_none() {
            ctx.error(
                &at.key(key),
                messages::unknown_reference("Transportklasse", key, None),
            );
        }
        if let Some(c) = storage_class(ctx, class, &at.field(key)) {
            by_transport.insert(key.clone(), c);
        }
    }
    let mut by_group = BTreeMap::new();
    for (key, class) in &v.by_group {
        let at = l.field("nach_warengruppe");
        if catalog.goods_groups.id(key).is_none() {
            ctx.error(
                &at.key(key),
                messages::unknown_reference("Warengruppe", key, None),
            );
        }
        if let Some(c) = storage_class(ctx, class, &at.field(key)) {
            by_group.insert(key.clone(), c);
        }
    }
    // The weight per unit as the catalog has it (with the defaults of the units).
    let weights: Vec<f64> = products
        .iter()
        .map(|e| {
            catalog
                .products
                .id(&e.value.id)
                .map_or(0.0, |id| catalog.products.get(id).weight_kg)
        })
        .collect();
    let m = &mut catalog.production_model.storage;
    if !v.classes.is_empty() {
        for class in StorageClass::ALL {
            if !v.classes.contains_key(class.key()) {
                ctx.error(
                    &l.field("klassen"),
                    messages::entry_missing("Lagerklasse", class.key()),
                );
            }
        }
    }
    let dl = l.field("dichte_t_je_m3");
    let mut density = BTreeMap::new();
    for (key, &d) in &v.density {
        if let Some(c) = storage_class(ctx, key, &dl.key(key)) {
            density.insert(c.key(), positive(ctx, d, &dl.field(key)));
        }
    }
    if !v.density.is_empty() {
        for class in StorageClass::ALL {
            if !v.density.contains_key(class.key()) {
                ctx.error(&dl, messages::entry_missing("Lagerklasse", class.key()));
            }
        }
    }
    let mut classes = Vec::with_capacity(products.len());
    let mut shelf = Vec::with_capacity(products.len());
    let mut volume = Vec::with_capacity(products.len());
    for (e, &weight_kg) in products.iter().zip(&weights) {
        let p = &e.value;
        let life = p.shelf_life_days.filter(|&d| {
            if d == 0 {
                ctx.error(
                    &e.loc.field("haltbarkeit_tage"),
                    messages::not_positive(0.0),
                );
            }
            d > 0
        });
        let own = p
            .storage_class
            .as_ref()
            .and_then(|k| storage_class(ctx, k, &e.loc.field("lagerklasse")));
        let class = own
            .or_else(|| {
                life.filter(|&d| d <= v.chilled_up_to_days)
                    .map(|_| StorageClass::Chilled)
            })
            .or_else(|| by_group.get(&p.goods_group).copied())
            .or_else(|| by_transport.get(&p.transport_class).copied());
        if class.is_none() && !v.classes.is_empty() {
            ctx.error(
                &e.loc,
                messages::entry_missing("Lagerklasse (nach_transportklasse)", &p.transport_class),
            );
        }
        let class = class.unwrap_or_default();
        let own_volume = p.storage_volume_m3.filter(|&x| {
            if x <= 0.0 {
                ctx.error(&e.loc.field("lagervolumen_m3"), messages::not_positive(x));
            }
            x > 0.0
        });
        volume.push(own_volume.unwrap_or_else(|| {
            density
                .get(class.key())
                .map_or(0.0, |&d| weight_kg / 1000.0 / d)
        }));
        classes.push(class);
        shelf.push(life);
    }
    m.product_class = classes;
    m.shelf_life = shelf;
    m.volume = volume;
}

/// Facility sizes (M36): every size with a capacity, growing with the size, 1 for
/// `mittel` (the size of the data values).
fn sizes(ctx: &mut Ctx, m: &RawFacilitySizes, loc: &Loc) -> SizeModel {
    let caps = loc.field("kapazitaet");
    let names: Vec<&str> = FacilitySize::ALL.iter().map(|s| s.key()).collect();
    for key in m.capacity.keys() {
        if !names.contains(&key.as_str()) {
            ctx.error(
                &caps.key(key),
                messages::unknown_value(key, &names, suggest::closest(key, names.iter().copied())),
            );
        }
    }
    let mut capacity = [1.0; 5];
    let mut complete = true;
    for size in FacilitySize::ALL {
        match m.capacity.get(size.key()) {
            Some(&k) => capacity[size.index()] = positive(ctx, k, &caps.field(size.key())),
            None => {
                complete = false;
                ctx.error(&caps, messages::entry_missing("Anlagengröße", size.key()));
            }
        }
    }
    let medium = capacity[FacilitySize::Medium.index()];
    if (medium - 1.0).abs() > 1e-9 {
        ctx.error(&caps.field("mittel"), messages::size_medium_not_one(medium));
    }
    if complete {
        for pair in FacilitySize::ALL.windows(2) {
            if capacity[pair[1].index()] <= capacity[pair[0].index()] {
                ctx.error(
                    &caps.field(pair[1].key()),
                    messages::sizes_not_increasing(pair[0].key(), pair[1].key()),
                );
            }
        }
    }
    SizeModel {
        capacity,
        investment_exponent: in_range(
            ctx,
            m.investment_exponent,
            0.0,
            1.5,
            &loc.field("investition_exponent"),
        ),
        labor_exponent: in_range(
            ctx,
            m.labor_exponent,
            -1.0,
            1.0,
            &loc.field("arbeit_exponent"),
        ),
        area_exponent: in_range(
            ctx,
            m.area_exponent,
            0.0,
            1.5,
            &loc.field("flaeche_exponent"),
        ),
        changeover_cost_share: in_range(
            ctx,
            m.changeover.kosten_anteil,
            0.0,
            1.0,
            &loc.field("umstellung").field("kosten_anteil"),
        ),
        changeover_days_share: in_range(
            ctx,
            m.changeover.dauer_anteil,
            0.0,
            1.0,
            &loc.field("umstellung").field("dauer_anteil"),
        ),
        build_exponent: in_range(
            ctx,
            m.build_exponent,
            0.0,
            1.0,
            &loc.field("bauzeit_exponent"),
        ),
    }
}

/// Every facility size has a name (`anlagengroesse.<key>`), and no name lacks its size.
pub(super) fn check_size_texts(
    ctx: &mut Ctx,
    raw: &RawData,
    texts: &TextIndex,
    report_unused: bool,
) {
    let Some(entry) = raw.production_model.first() else {
        return;
    };
    let caps = entry.loc.field("anlagengroessen").field("kapazitaet");
    for size in FacilitySize::ALL {
        let key = format!("{SIZE_TEXT}.{}", size.key());
        if texts.texts.get(&key).is_none() {
            ctx.error(
                &caps.field(size.key()),
                messages::text_missing(&key, crate::LANGUAGE),
            );
        }
    }
    if !report_unused {
        return;
    }
    for (text_key, loc) in &texts.locations {
        if let Some((SIZE_TEXT, rest)) = text_key.split_once('.')
            && !FacilitySize::ALL.iter().any(|s| s.key() == rest)
        {
            ctx.warning(loc, messages::text_unused(text_key));
        }
    }
}

fn reference_margin(ctx: &mut Ctx, m: &RawLimits, loc: &Loc) -> (f64, f64) {
    let min = in_range(ctx, m.min, -1.0, 1.0, &loc.field("minimum"));
    let max = in_range(ctx, m.max, -1.0, 1.0, &loc.field("maximum"));
    if min > max {
        ctx.error(loc, messages::range_inverted("minimum", "maximum"));
    }
    (min, max)
}

pub(super) fn finance_model(ctx: &mut Ctx, raw: &RawData) -> wsim_core::catalog::FinanceModel {
    use wsim_core::time_series::TimeSeries;
    let Some(entry) = single(
        ctx,
        &raw.finance_model,
        "finanzmodell",
        "parameter/finanzmodell.yaml",
    ) else {
        return wsim_core::catalog::FinanceModel::default();
    };
    let m = &entry.value;
    let l = &entry.loc;
    let rates = l.field("realzins");
    if m.real_rate.is_empty() {
        ctx.error(&rates, messages::time_series_empty());
    }
    for (&year, &rate) in &m.real_rate {
        let point = rates.field(&year.to_string());
        super::year(ctx, year, super::GAME_YEARS, &point);
        in_range(ctx, rate, -0.2, 0.5, &point);
    }
    let real_rate = TimeSeries::new(m.real_rate.iter().map(|(&y, &r)| (y, r)).collect())
        .unwrap_or_else(|_| TimeSeries::new(vec![(1900, 0.0)]).expect("valid"));
    let premium = l.field("risikoaufschlag");
    let overdraft = l.field("dispo");
    if m.max_term_years == 0 {
        ctx.error(&l.field("laufzeit_max_jahre"), messages::not_positive(0.0));
    }
    wsim_core::catalog::FinanceModel {
        real_rate,
        premium_min: in_range(ctx, m.premium.min, 0.0, 1.0, &premium.field("minimum")),
        premium_per_debt_ratio: in_range(
            ctx,
            m.premium.per_debt_ratio,
            0.0,
            1.0,
            &premium.field("je_verschuldung"),
        ),
        loan_to_value: in_range(ctx, m.loan_to_value, 0.0, 1.0, &l.field("beleihung")),
        overdraft_share: in_range(ctx, m.overdraft.share, 0.0, 1.0, &overdraft.field("anteil")),
        overdraft_premium: in_range(
            ctx,
            m.overdraft.premium,
            0.0,
            1.0,
            &overdraft.field("aufschlag"),
        ),
        max_term_years: m.max_term_years,
        dividends: dividend_model(ctx, &m.dividends, &l.field("dividende")),
    }
}

/// Dividends (PE4): reserve, group privilege, payout range of AI and CEO.
fn dividend_model(
    ctx: &mut Ctx,
    d: &crate::raw::RawDividends,
    l: &crate::read::Loc,
) -> wsim_core::catalog::DividendModel {
    let payout_min = in_range(ctx, d.payout_min, 0.0, 1.0, &l.field("quote_min"));
    let payout_max = in_range(ctx, d.payout_max, 0.0, 1.0, &l.field("quote_max"));
    if payout_min > payout_max {
        ctx.error(l, messages::range_inverted("quote_min", "quote_max"));
    }
    wsim_core::catalog::DividendModel {
        reserve_months_min: in_range(
            ctx,
            d.reserve_months_min,
            0.0,
            60.0,
            &l.field("reserve_monate_min"),
        ),
        group_share: in_range(ctx, d.group_share, 0.0, 1.0, &l.field("konzern_anteil")),
        payout_min,
        payout_max,
        cash_months: in_range(ctx, d.cash_months, 0.0, 60.0, &l.field("kasse_monate")),
    }
}

/// The market model and the keys of the advertising media (for the text check).
pub(super) fn market_model(
    ctx: &mut Ctx,
    raw: &RawData,
) -> (wsim_core::catalog::MarketModel, Keys) {
    let mut media_keys = Keys {
        kind: "Werbemittel",
        text_prefix: Some("werbemittel"),
        index: BTreeMap::new(),
        locations: Vec::new(),
        broken: Default::default(),
    };
    let Some(entry) = single(
        ctx,
        &raw.market_model,
        "marktmodell",
        "parameter/marktmodell.yaml",
    ) else {
        return (wsim_core::catalog::MarketModel::default(), media_keys);
    };
    let m = &entry.value;
    let l = &entry.loc;
    let mut five = |values: &[f64], field: &str| -> [f64; 5] {
        let loc = l.field(field);
        for (i, &v) in values.iter().enumerate() {
            positive(ctx, v, &loc.index(i));
        }
        <[f64; 5]>::try_from(values).unwrap_or_else(|_| {
            ctx.error(
                &loc,
                messages::wrong_length(values.len(), "5 Werte (ärmstes Fünftel zuerst)"),
            );
            [1.0; 5]
        })
    };
    let price_weight = five(&m.price_weight, "preisgewicht");
    let quality_weight = five(&m.quality_weight, "qualitaetsgewicht");
    let bl = l.field("marke");
    let brand_weight = {
        let loc = bl.field("markengewicht");
        for (i, &v) in m.brand.weight.iter().enumerate() {
            non_negative(ctx, v, &loc.index(i));
        }
        <[f64; 5]>::try_from(m.brand.weight.as_slice()).unwrap_or_else(|_| {
            ctx.error(
                &loc,
                messages::wrong_length(m.brand.weight.len(), "5 Werte (ärmstes Fünftel zuerst)"),
            );
            [0.0; 5]
        })
    };
    let b = &m.brand;
    let mut media = Vec::new();
    for (i, medium) in b.media.iter().enumerate() {
        let loc = bl.field("werbemittel").index(i);
        if let Some(&first) = media_keys.index.get(&medium.id) {
            let first = ctx.describe(&media_keys.locations[first]);
            ctx.error(
                &loc.field("id"),
                messages::duplicate_key("Werbemittel", &medium.id, &first),
            );
            continue;
        }
        media_keys.index.insert(medium.id.clone(), media.len());
        media_keys.locations.push(loc.clone());
        media.push(wsim_core::catalog::AdvertisingMedium {
            key: medium.id.clone(),
            from_year: medium.from_year,
            effect: positive(ctx, medium.effect, &loc.field("wirkung")),
        });
    }
    if media.is_empty() {
        ctx.error(&bl.field("werbemittel"), messages::list_empty());
    }
    let brand = wsim_core::catalog::BrandModel {
        weight: brand_weight,
        forgetting_per_month: in_range(
            ctx,
            b.forgetting_per_month,
            0.0,
            1.0,
            &bl.field("vergessen_je_monat"),
        ),
        word_of_mouth: in_range(ctx, b.word_of_mouth, 0.0, 1.0, &bl.field("mundpropaganda")),
        cost_per_inhabitant_usd: positive(
            ctx,
            b.cost_per_inhabitant_usd,
            &bl.field("kosten_je_einwohner_usd"),
        ),
        start_awareness: in_range(
            ctx,
            b.start_awareness,
            0.0,
            1.0,
            &bl.field("bekanntheit_start"),
        ),
        start_awareness_real: in_range(
            ctx,
            b.start_awareness_real,
            0.0,
            1.0,
            &bl.field("bekanntheit_start_real"),
        ),
        trade_awareness: in_range(
            ctx,
            b.trade_awareness,
            0.0,
            1.0,
            &bl.field("bekanntheit_handel"),
        ),
        state_market_awareness: in_range(
            ctx,
            b.state_market_awareness,
            0.0,
            1.0,
            &bl.field("bekanntheit_staatsmarkt"),
        ),
        media,
    };
    let adjust = l.field("preisanpassung");
    let traders = l.field("haendler");
    let substitution = l.field("substitution");
    let model = wsim_core::catalog::MarketModel {
        substitution_steepness: m.substitution.as_ref().map_or(0.0, |s| {
            in_range(
                ctx,
                s.steepness,
                0.0,
                20.0,
                &substitution.field("steilheit"),
            )
        }),
        substitution_per_year: m.substitution.as_ref().map_or(0.0, |s| {
            in_range(ctx, s.per_year, 0.0, 12.0, &substitution.field("je_jahr"))
        }),
        trends: Vec::new(),
        tenders: Default::default(),
        obsolescence_points: m.obsolescence.as_ref().map_or(0.0, |o| {
            in_range(
                ctx,
                o.points_per_level,
                0.0,
                50.0,
                &l.field("veralterung").field("punkte_je_stufe"),
            )
        }),
        used: m.used.as_ref().map_or_else(Default::default, |u| {
            let ul = l.field("gebraucht");
            wsim_core::catalog::UsedModel {
                enabled: true,
                pass_on: in_range(ctx, u.pass_on, 0.0, 1.0, &ul.field("weitergabe")),
                life_share: in_range(
                    ctx,
                    u.life_share,
                    0.05,
                    1.0,
                    &ul.field("lebensdauer_anteil"),
                ),
                first_share: in_range(ctx, u.first_share, 0.0, 1.0, &ul.field("anteil_erstkauf")),
            }
        }),
        trend_generator: m
            .trend_generator
            .as_ref()
            .map_or_else(Default::default, |t| {
                let tl = l.field("trendgenerator");
                let factor_min = in_range(ctx, t.faktor_min, 1.0, 5.0, &tl.field("faktor_min"));
                let factor_max = in_range(ctx, t.faktor_max, 1.0, 5.0, &tl.field("faktor_max"));
                if factor_min > factor_max {
                    ctx.error(
                        &tl.field("faktor_max"),
                        messages::range_inverted("faktor_min", "faktor_max"),
                    );
                }
                if t.jahre_min == 0 || t.jahre_min > t.jahre_max {
                    ctx.error(
                        &tl.field("jahre_max"),
                        messages::range_inverted("jahre_min", "jahre_max"),
                    );
                }
                wsim_core::catalog::TrendGenerator {
                    enabled: true,
                    from_year: year(ctx, t.from_year, HISTORY_YEARS, &tl.field("ab")),
                    chance: in_range(ctx, t.chance, 0.0, 1.0, &tl.field("chance")),
                    factor_min,
                    factor_max,
                    years_min: t.jahre_min.max(1),
                    years_max: t.jahre_max.max(t.jahre_min.max(1)),
                }
            }),
        channels: m.channels.as_ref().map_or_else(Default::default, |c| {
            let cl = l.field("vertrieb");
            wsim_core::catalog::ChannelModel {
                base_reach: in_range(ctx, c.base_reach, 0.05, 1.0, &cl.field("grundreichweite")),
                ai_delay_years: in_range(
                    ctx,
                    c.ai_delay_years,
                    0.0,
                    50.0,
                    &cl.field("ki_verzoegerung_jahre"),
                ),
                ai_margin: in_range(ctx, c.ai_margin, 0.0, 1.0, &cl.field("ki_marge")),
            }
        }),
        reputation: m.reputation.as_ref().map_or_else(Default::default, |r| {
            let rl = l.field("ruf");
            let wl = rl.field("rufgewicht");
            for (i, &v) in r.weight.iter().enumerate() {
                non_negative(ctx, v, &wl.index(i));
            }
            let weight = <[f64; 5]>::try_from(r.weight.as_slice()).unwrap_or_else(|_| {
                ctx.error(
                    &wl,
                    messages::wrong_length(r.weight.len(), "5 Werte (ärmstes Fünftel zuerst)"),
                );
                [0.0; 5]
            });
            wsim_core::catalog::ReputationModel {
                enabled: true,
                gain_per_month: in_range(
                    ctx,
                    r.gain_per_month,
                    0.0,
                    1.0,
                    &rl.field("gewinn_je_monat"),
                ),
                return_per_month: in_range(
                    ctx,
                    r.return_per_month,
                    0.0,
                    1.0,
                    &rl.field("rueckkehr_je_monat"),
                ),
                accident_loss: in_range(ctx, r.accident_loss, 0.0, 1.0, &rl.field("unfall_abzug")),
                recall_below: in_range(
                    ctx,
                    r.recall_below,
                    0.0,
                    100.0,
                    &rl.field("rueckruf_schwelle"),
                ),
                recall_chance: in_range(
                    ctx,
                    r.recall_chance,
                    0.0,
                    1.0,
                    &rl.field("rueckruf_chance"),
                ),
                recall_loss: in_range(ctx, r.recall_loss, 0.0, 1.0, &rl.field("rueckruf_abzug")),
                weight,
            }
        }),
        price_weight,
        quality_weight,
        adoption_per_year: in_range(
            ctx,
            m.adoption_per_year,
            0.0,
            1.0,
            &l.field("aneignung_je_jahr"),
        ),
        price_step_up: in_range(ctx, m.price_adjustment.up, 0.0, 0.5, &adjust.field("hoch")),
        price_step_down: in_range(
            ctx,
            m.price_adjustment.down,
            0.0,
            0.5,
            &adjust.field("runter"),
        ),
        stock_days: positive(
            ctx,
            m.price_adjustment.stock_days,
            &adjust.field("lagertage"),
        ),
        normal_utilization: in_range(
            ctx,
            m.price_adjustment.normal_utilization,
            0.0,
            1.0,
            &adjust.field("auslastung_normal"),
        ),
        price_max_factor: in_range(
            ctx,
            m.price_adjustment.max_factor,
            1.0,
            1000.0,
            &adjust.field("hoechstfaktor"),
        ),
        catch_up_max: in_range(
            ctx,
            m.price_adjustment.catch_up_max,
            1.0,
            1000.0,
            &adjust.field("aufholen_max"),
        ),
        state_price_cap: positive(ctx, m.state_price_cap, &l.field("staat_hoechstpreis")),
        state_displacement_years: positive(
            ctx,
            m.state_displacement_years,
            &l.field("verdraengung_staat_jahre"),
        ),
        history_months: {
            let loc = l.field("verlauf_monate");
            in_range(ctx, f64::from(m.history_months), 1.0, 120.0, &loc);
            m.history_months
        },
        price_cut_report: in_range(
            ctx,
            m.price_cut_report,
            0.01,
            0.9,
            &l.field("meldung_preissenkung"),
        ),
        price_level_share: per_kind(
            ctx,
            &m.price_level_share,
            &l.field("preisniveau_anteil"),
            |ctx, v, loc| in_range(ctx, v, 0.0, 1.0, loc),
        ),
        index_smoothing: in_range(
            ctx,
            m.index_smoothing,
            0.0,
            1.0,
            &l.field("index_glaettung"),
        ),
        trader_margin: in_range(ctx, m.traders.margin, 0.0, 1.0, &traders.field("marge")),
        trader_cover_days: positive(ctx, m.traders.cover_days, &traders.field("vorrat_tage")),
        demand_smoothing_days: in_range(
            ctx,
            m.traders.smoothing_days,
            1.0,
            365.0,
            &traders.field("glaettung_tage"),
        ),
        arbitrage_gap: in_range(
            ctx,
            m.traders.arbitrage.gap,
            0.0,
            10.0,
            &traders.field("arbitrage").field("abstand"),
        ),
        arbitrage_share: in_range(
            ctx,
            m.traders.arbitrage.share,
            0.0,
            1.0,
            &traders.field("arbitrage").field("anteil"),
        ),
        brand,
    };
    (model, media_keys)
}

pub(super) fn transport_model(ctx: &mut Ctx, raw: &RawData) -> TransportModel {
    let Some(entry) = single(
        ctx,
        &raw.transport_model,
        "transportmodell",
        "parameter/transportmodell.yaml",
    ) else {
        return TransportModel::default();
    };
    let m = &entry.value;
    let l = &entry.loc;
    let detour = l.field("umweg");
    let handling = l.field("umschlag");
    TransportModel {
        detour_land: in_range(ctx, m.detour.land, 1.0, 5.0, &detour.field("land")),
        detour_sea: in_range(ctx, m.detour.see, 1.0, 5.0, &detour.field("see")),
        detour_air: in_range(ctx, m.detour.luft, 1.0, 5.0, &detour.field("luft")),
        handling_cost_usd: non_negative(
            ctx,
            m.handling.cost_usd,
            &handling.field("kosten_usd_je_t"),
        ),
        handling_days: non_negative(ctx, m.handling.days, &handling.field("tage")),
        min_infrastructure: in_range(
            ctx,
            m.min_infrastructure,
            0.001,
            1.0,
            &l.field("mindestinfrastruktur"),
        ),
    }
}

/// Payload and price of a vehicle for fleets (W5): both or none.
fn fleet_vehicle(ctx: &mut Ctx, v: &RawVehicle, l: &Loc) -> Option<FleetVehicle> {
    let series = |ctx: &mut Ctx, values: &RawSeries, key: &str| {
        let at = l.field(key);
        for (&y, &x) in values {
            positive(ctx, x, &at.field(&y.to_string()));
        }
        time_series(ctx, values, &at)
    };
    match (&v.payload_t, &v.price_usd) {
        (Some(payload), Some(price)) => Some(FleetVehicle {
            payload_t: series(ctx, payload, "nutzlast_t"),
            price_usd: series(ctx, price, "kaufpreis_usd"),
        }),
        (Some(_), None) => {
            ctx.error(
                &l.field("nutzlast_t"),
                messages::fleet_fields_together("nutzlast_t", "kaufpreis_usd"),
            );
            None
        }
        (None, Some(_)) => {
            ctx.error(
                &l.field("kaufpreis_usd"),
                messages::fleet_fields_together("kaufpreis_usd", "nutzlast_t"),
            );
            None
        }
        (None, None) => None,
    }
}

pub(super) fn vehicle(ctx: &mut Ctx, e: &Entry<RawVehicle>, classes: &Keys) -> Vehicle {
    let v = &e.value;
    let l = &e.loc;
    let available_from = year(
        ctx,
        v.available_from,
        HISTORY_YEARS,
        &l.field("verfuegbar_ab"),
    );
    let available_until = v
        .available_until
        .map(|y| year(ctx, y, HISTORY_YEARS, &l.field("verfuegbar_bis")));
    if let Some(until) = available_until
        && available_from > until
    {
        ctx.error(
            &l.field("verfuegbar_bis"),
            messages::year_range_inverted(available_from, until),
        );
    }
    let classes_loc = l.field("transportklassen");
    if v.classes.is_empty() {
        ctx.error(&classes_loc, messages::vehicle_without_classes());
    }
    let classes = v
        .classes
        .iter()
        .enumerate()
        .map(|(i, c)| resolve(ctx, classes, c, &classes_loc.index(i)))
        .collect();
    let speed_loc = l.field("km_je_tag");
    for (&y, &speed) in &v.km_per_day {
        positive(ctx, speed, &speed_loc.field(&y.to_string()));
    }
    Vehicle {
        way: match v.way {
            RawWay::Terrain => Way::Terrain,
            RawWay::Road => Way::Road,
            RawWay::Rail => Way::Rail,
            RawWay::Sea => Way::Sea,
            RawWay::Air => Way::Air,
        },
        available_from,
        available_until,
        classes,
        cost_per_tkm: time_series(ctx, &v.cost_per_tkm, &l.field("kosten_usd_je_tkm")),
        km_per_day: time_series(ctx, &v.km_per_day, &speed_loc),
        fleet: fleet_vehicle(ctx, v, l),
        provenance: provenance(v.approximation, v.source.as_ref()),
    }
}

pub(super) fn research_model(
    ctx: &mut Ctx,
    catalog: &Catalog,
    raw: &RawData,
    (qualifications, branches, specializations): (&Keys, &Keys, &Keys),
) -> ResearchModel {
    let Some(entry) = single(
        ctx,
        &raw.research_model,
        "forschungsmodell",
        "parameter/forschungsmodell.yaml",
    ) else {
        return ResearchModel::default();
    };
    let m = &entry.value;
    let l = &entry.loc;
    let latecomer = l.field("nachzuegler");
    let researchers_loc = l.field("forscher");
    let qualification: QualificationId =
        resolve(ctx, qualifications, &m.researchers, &researchers_loc);
    let researchers = if catalog.qualifications.get(qualification).has_specialization {
        catalog
            .specializations
            .ids()
            .map(|s| {
                let key = format!("{}.{}", m.researchers, catalog.specializations.key(s));
                catalog.labor_groups.id(&key)
            })
            .collect()
    } else {
        if qualifications.index.contains_key(&m.researchers) {
            ctx.error(
                &researchers_loc,
                messages::researchers_need_fields(&m.researchers),
            );
        }
        Vec::new()
    };
    ResearchModel {
        ahead_base: in_range(ctx, m.ahead_base, 1.0, 10.0, &l.field("vorgriff_faktor")),
        latecomer_discount: in_range(
            ctx,
            m.latecomer.discount,
            0.0,
            1.0,
            &latecomer.field("rabatt_je_jahr"),
        ),
        latecomer_min: in_range(ctx, m.latecomer.min, 0.0, 1.0, &latecomer.field("minimum")),
        public_domain_years: i32::try_from(m.public_domain_years).unwrap_or(i32::MAX),
        material_usd_per_day: non_negative(
            ctx,
            m.material_usd_per_day,
            &l.field("sachkosten_usd_je_forschertag"),
        ),
        development: development_model(
            ctx,
            &m.development,
            &l.field("weiterentwicklung"),
            (&researchers, &m.researchers),
            (branches, specializations),
        ),
        patents: m
            .patents
            .as_ref()
            .map(|p| patent_model(ctx, p, &l.field("patente"))),
        lab_staff: m
            .lab_staff
            .as_ref()
            .map_or_else(LabStaffModel::default, |s| {
                lab_staff_model(ctx, s, &l.field("labor"))
            }),
        projects: m
            .projects
            .as_ref()
            .map(|p| project_model(ctx, p, &l.field("projekte"))),
        researchers,
    }
}

/// The roles of laboratory staff (PD1): each known role once, with its weight in research.
fn lab_staff_model(ctx: &mut Ctx, s: &RawLabStaff, l: &Loc) -> LabStaffModel {
    let mut roles: Vec<(LabRole, f64)> = Vec::new();
    for (i, r) in s.roles.iter().enumerate() {
        let rl = l.field("rollen").index(i);
        let Some(role) = LabRole::from_key(&r.id) else {
            ctx.error(&rl.field("id"), messages::unknown_lab_role(&r.id));
            continue;
        };
        if roles.iter().any(|(x, _)| *x == role) {
            ctx.error(&rl.field("id"), messages::duplicate_lab_role(&r.id));
            continue;
        }
        let weight = in_range(ctx, r.research, 0.0, 1.0, &rl.field("forschung"));
        roles.push((role, weight));
    }
    LabStaffModel { roles }
}

const PROJECT_PHASES: [&str; 6] = [
    "pm0_pm1", "pm1_pm2", "pm2_pm3", "pm4_pm5", "pm5_pm6", "pm6_pm7",
];
const PROJECT_TEMPOS: [&str; 3] = ["gruendlich", "normal", "beschleunigt"];
const PROJECT_TRIMS: [&str; 3] = ["basis", "standard", "premium"];

/// The phases of a project kind in the order of `keys`, each with roles whose effects add up
/// to one (shared by the development projects and the improvement projects).
fn project_phases(
    ctx: &mut Ctx,
    raw_phases: &[RawProjectPhase],
    keys: &[&str],
    l: &Loc,
) -> Vec<ProjectPhase> {
    let mut phases = Vec::new();
    for (i, key) in keys.iter().enumerate() {
        let pl = l.field("phasen").index(i);
        let Some(raw) = raw_phases.get(i) else {
            ctx.error(&l.field("phasen"), messages::project_phase_missing(key));
            continue;
        };
        if raw.id != *key {
            ctx.error(&pl.field("id"), messages::project_phase_order(&raw.id, key));
        }
        let mut roles: Vec<PhaseRole> = Vec::new();
        for (j, r) in raw.roles.iter().enumerate() {
            let rl = pl.field("rollen").index(j);
            let Some(role) = LabRole::from_key(&r.id) else {
                ctx.error(&rl.field("id"), messages::unknown_lab_role(&r.id));
                continue;
            };
            if roles.iter().any(|x| x.role == role) {
                ctx.error(&rl.field("id"), messages::duplicate_lab_role(&r.id));
                continue;
            }
            roles.push(PhaseRole {
                role,
                need: non_negative(ctx, r.need, &rl.field("bedarf")),
                effect: in_range(ctx, r.effect, 0.0, 1.0, &rl.field("wirkung")),
            });
        }
        let effects: f64 = roles.iter().map(|r| r.effect).sum();
        if (effects - 1.0).abs() > 1e-6 {
            ctx.error(&pl.field("rollen"), messages::project_effects_sum(effects));
        }
        phases.push(ProjectPhase {
            key: raw.id.clone(),
            cost: in_range(ctx, raw.cost, 0.0, 1.0, &pl.field("kosten")),
            time: in_range(ctx, raw.time, 0.0, 1.0, &pl.field("dauer")),
            roles,
        });
    }
    if raw_phases.len() > keys.len() {
        ctx.error(&l.field("phasen"), messages::project_phase_extra());
    }
    phases
}

/// Development projects (PD2): six phases in order, each with roles whose effects add up to
/// one; three tempos and three trims; shares of cost and time add up to one.
fn project_model(ctx: &mut Ctx, p: &RawProjectModel, l: &Loc) -> ProjectModel {
    let phases = project_phases(ctx, &p.phases, &PROJECT_PHASES, l);
    for (name, total) in [
        (
            "kosten",
            phases.iter().map(|x| x.cost).sum::<f64>() + p.gate_cost,
        ),
        ("dauer", phases.iter().map(|x| x.time).sum::<f64>()),
    ] {
        if (total - 1.0).abs() > 1e-6 {
            ctx.error(
                &l.field("phasen"),
                messages::project_shares_sum(name, total),
            );
        }
    }
    let mut tempos = Vec::new();
    for (i, key) in PROJECT_TEMPOS.iter().enumerate() {
        let tl = l.field("tempo").index(i);
        let Some(raw) = p.tempos.get(i) else {
            ctx.error(&l.field("tempo"), messages::project_choice_missing(key));
            continue;
        };
        if raw.id != *key {
            ctx.error(&tl.field("id"), messages::project_phase_order(&raw.id, key));
        }
        tempos.push(ProjectTempo {
            key: raw.id.clone(),
            time: positive(ctx, raw.time, &tl.field("dauer")),
            cost: positive(ctx, raw.cost, &tl.field("kosten")),
            risk: positive(ctx, raw.risk, &tl.field("risiko")),
        });
    }
    let mut trims = Vec::new();
    for (i, key) in PROJECT_TRIMS.iter().enumerate() {
        let tl = l.field("ausstattung").index(i);
        let Some(raw) = p.trims.get(i) else {
            ctx.error(
                &l.field("ausstattung"),
                messages::project_choice_missing(key),
            );
            continue;
        };
        if raw.id != *key {
            ctx.error(&tl.field("id"), messages::project_phase_order(&raw.id, key));
        }
        trims.push(ProjectTrim {
            key: raw.id.clone(),
            cost: positive(ctx, raw.cost, &tl.field("kosten")),
            quality: in_range(ctx, raw.quality, -0.5, 0.5, &tl.field("qualitaet")),
        });
    }
    let ll = l.field("projektleiter");
    if p.lead.age_min >= p.lead.age_max {
        ctx.error(
            &ll.field("alter_min"),
            messages::range_inverted("alter_min", "alter_max"),
        );
    }
    if p.lead.characters.is_empty() {
        ctx.error(&ll.field("charaktere"), messages::project_no_characters());
    }
    ProjectModel {
        cost_per_point_usd: positive(ctx, p.cost_per_point_usd, &l.field("punktekosten_usd")),
        reference_rate: positive(ctx, p.reference_rate, &l.field("referenzrate")),
        reference_effort: positive(ctx, p.reference_effort, &l.field("referenzaufwand")),
        need_exponent: in_range(ctx, p.need_exponent, 0.0, 2.0, &l.field("bedarf_exponent")),
        over_max: in_range(
            ctx,
            p.over.max,
            0.0,
            1.0,
            &l.field("ueberbesetzung").field("maximum"),
        ),
        over_tau: positive(ctx, p.over.tau, &l.field("ueberbesetzung").field("tau")),
        rest_share: in_range(ctx, p.rest_share, 0.0, 0.5, &l.field("restanteil")),
        shortfall_penalty: in_range(
            ctx,
            p.shortfall.penalty,
            0.0,
            1.0,
            &l.field("unterbesetzung").field("abschlag"),
        ),
        shortfall_cap: in_range(
            ctx,
            p.shortfall.cap,
            0.0,
            1.0,
            &l.field("unterbesetzung").field("hoechstens"),
        ),
        spread: in_range(ctx, p.spread, 0.0, 0.5, &l.field("streuung")),
        gate_cost: in_range(ctx, p.gate_cost, 0.0, 1.0, &l.field("gate_kosten")),
        phases,
        tempos,
        trims,
        lead: ProjectLeadModel {
            salary_usd_year: positive(ctx, p.lead.salary_usd_year, &ll.field("gehalt_usd_jahr")),
            capacity_new: p.lead.capacity_new.max(1),
            capacity_further: p.lead.capacity_further.max(1),
            candidates: p.lead.candidates,
            experience_gain: in_range(
                ctx,
                p.lead.experience_gain,
                0.0,
                1.0,
                &ll.field("erfahrung_je_projekt"),
            ),
            age_min: p.lead.age_min,
            age_max: p.lead.age_max,
            characters: p.lead.characters.clone(),
        },
        risk: ProjectRiskModel {
            base: in_range(
                ctx,
                p.risk.base,
                0.0,
                1.0,
                &l.field("risiko").field("basis"),
            ),
            shortfall: non_negative(
                ctx,
                p.risk.shortfall,
                &l.field("risiko").field("unterbesetzung"),
            ),
            experience: in_range(
                ctx,
                p.risk.experience,
                0.0,
                1.0,
                &l.field("risiko").field("erfahrung"),
            ),
            branch: in_range(
                ctx,
                p.risk.branch,
                0.0,
                1.0,
                &l.field("risiko").field("branche"),
            ),
            buffer: in_range(
                ctx,
                p.risk.buffer,
                0.0,
                1.0,
                &l.field("risiko").field("puffer"),
            ),
            fast_track: non_negative(
                ctx,
                p.risk.fast_track,
                &l.field("risiko").field("fast_track"),
            ),
            branch_ref: non_negative(
                ctx,
                p.risk.branch_ref,
                &l.field("risiko").field("branche_anlagen_ref"),
            ),
            buffer_ref: non_negative(
                ctx,
                p.risk.buffer_ref,
                &l.field("risiko").field("puffer_ref"),
            ),
        },
        reports: project_reports(ctx, p, l),
        events: project_events(ctx, p, l),
        improvement: project_improvement(ctx, p, l),
        suggestions: project_suggestions(ctx, p, l),
        council: project_council(ctx, p, l),
        fast_track: project_fast_track(ctx, p, l),
        pilot: project_pilot(ctx, p, l),
        leadership: project_leadership(ctx, p, l),
        knowledge: project_knowledge(ctx, p, l),
        platform: project_platform(ctx, p, l),
        license: project_license(ctx, p, l),
        partnership: project_partnership(ctx, p, l),
        first_mover: project_first_mover(ctx, p, l),
        subsidies: project_subsidies(ctx, p, l),
    }
}

const PROJECT_PROFILES: [&str; 3] = ["kosten", "ausgewogen", "qualitaet"];
const PROJECT_PATHS: [&str; 3] = ["bewaehrt", "standard", "innovativ"];

/// Milestone reports and decisions (PD3): the traffic lights, the leader's traits, the
/// delegation and the choices at PM1 and PM2. All four blocks belong together.
fn project_reports(ctx: &mut Ctx, p: &RawProjectModel, l: &Loc) -> Option<ProjectReportModel> {
    let (Some(r), Some(d)) = (p.reports.as_ref(), p.decision.as_ref()) else {
        if p.reports.is_some() || p.decision.is_some() {
            ctx.error(&l.field("berichte"), messages::project_reports_incomplete());
        }
        return None;
    };
    let rl = l.field("berichte");
    let choices = |ctx: &mut Ctx, raw: &[RawProjectChoice], keys: &[&str], at: &Loc| {
        let mut out = Vec::new();
        for (i, key) in keys.iter().enumerate() {
            let cl = at.index(i);
            let Some(c) = raw.get(i) else {
                ctx.error(at, messages::project_choice_missing(key));
                continue;
            };
            if c.id != *key {
                ctx.error(&cl.field("id"), messages::project_phase_order(&c.id, key));
            }
            out.push(ProjectChoice {
                key: c.id.clone(),
                cost: positive(ctx, c.cost, &cl.field("kosten")),
                time: positive(ctx, c.time, &cl.field("dauer")),
                risk: positive(ctx, c.risk, &cl.field("risiko")),
                quality: in_range(ctx, c.quality, -0.5, 0.5, &cl.field("qualitaet")),
            });
        }
        out
    };
    let profiles = choices(ctx, &p.profiles, &PROJECT_PROFILES, &l.field("profile"));
    let paths = choices(ctx, &p.paths, &PROJECT_PATHS, &l.field("loesungswege"));
    let mut traits = Vec::new();
    for key in &p.lead.characters {
        let tl = rl.field("charaktere").field(key);
        let Some(t) = r.characters.get(key) else {
            ctx.error(
                &rl.field("charaktere"),
                messages::project_character_missing(key),
            );
            continue;
        };
        let path = PROJECT_PATHS.iter().position(|x| *x == t.path);
        if path.is_none() {
            ctx.error(&tl.field("weg"), messages::project_path_unknown(&t.path));
        }
        traits.push(LeadTrait {
            optimism: in_range(ctx, t.optimism, -0.9, 0.9, &tl.field("optimismus")),
            path: path.unwrap_or(1),
            dissent: in_range(ctx, t.dissent, 0.0, 1.0, &tl.field("gegenmeinung")),
            fast_track: in_range(ctx, t.fast_track, 0.0, 1.0, &tl.field("fast_track")),
            pilot: in_range(ctx, t.pilot, 0.0, 1.0, &tl.field("pilot")),
        });
    }
    for key in r.characters.keys() {
        if !p.lead.characters.contains(key) {
            ctx.error(
                &rl.field("charaktere").field(key),
                messages::project_character_unused(key),
            );
        }
    }
    let light = |ctx: &mut Ctx, v: &RawLight, at: &Loc| {
        let yellow = non_negative(ctx, v.yellow, &at.field("gelb"));
        let red = non_negative(ctx, v.red, &at.field("rot"));
        if red <= yellow {
            ctx.error(at, messages::range_inverted("gelb", "rot"));
        }
        Light { yellow, red }
    };
    let ll = rl.field("ampel");
    let lights = [
        light(ctx, &r.lights.schedule, &ll.field("termin")),
        light(ctx, &r.lights.budget, &ll.field("budget")),
        light(ctx, &r.lights.quality, &ll.field("qualitaet")),
        light(ctx, &r.lights.risk, &ll.field("risiko")),
    ];
    let dl = l.field("entscheidung");
    for &m in &d.player_at {
        if !(1..=7).contains(&m) {
            ctx.error(
                &dl.field("spieler_bei"),
                messages::out_of_range(f64::from(m), 1.0, 7.0),
            );
        }
    }
    Some(ProjectReportModel {
        lifecycle_years: positive(ctx, r.lifecycle_years, &rl.field("lebenszyklus_jahre")),
        capture_share: in_range(ctx, r.capture_share, 0.0, 1.0, &rl.field("absatzanteil")),
        plants_without_demand: non_negative(
            ctx,
            r.plants_without_demand,
            &rl.field("anlagen_ohne_nachfrage"),
        ),
        utilization: in_range(ctx, r.utilization, 0.01, 1.0, &rl.field("auslastung")),
        uncertainty: in_range(ctx, r.uncertainty, 0.0, 0.9, &rl.field("unschaerfe")),
        accuracy_per_pm: in_range(
            ctx,
            r.accuracy_per_pm,
            0.0,
            1.0,
            &rl.field("genauigkeit_je_pm"),
        ),
        traits,
        lights,
        payback_max_years: positive(
            ctx,
            r.payback_max_years,
            &rl.field("amortisation_max_jahre"),
        ),
        conditions_risk: non_negative(ctx, r.conditions_risk, &rl.field("auflagen_risiko")),
        player_at: d.player_at.clone(),
        deadline_days: d.deadline_days,
        profiles,
        paths,
    })
}

/// First-mover bonus (PD9).
fn project_first_mover(ctx: &mut Ctx, p: &RawProjectModel, l: &Loc) -> Option<FirstMoverModel> {
    let x = p.first_mover.as_ref()?;
    let fl = l.field("erstankunft");
    Some(FirstMoverModel {
        awareness: in_range(ctx, x.awareness, 0.0, 1.0, &fl.field("bekanntheit")),
        reputation: in_range(ctx, x.reputation, 0.0, 0.5, &fl.field("ruf")),
    })
}

/// State programs for development projects (PD9).
fn project_subsidies(ctx: &mut Ctx, p: &RawProjectModel, l: &Loc) -> Vec<SubsidyProgram> {
    let mut out: Vec<SubsidyProgram> = Vec::new();
    for (i, x) in p.subsidies.iter().enumerate() {
        let sl = l.field("foerderung").index(i);
        if x.from_year > x.to_year {
            ctx.error(
                &sl.field("von"),
                messages::project_subsidy_years(x.from_year, x.to_year),
            );
        }
        if out.iter().any(|o| o.key == x.id) {
            ctx.error(&sl.field("id"), messages::project_subsidy_duplicate(&x.id));
        }
        if !x.approximation {
            ctx.error(&sl, messages::project_subsidy_estimate(&x.id));
        }
        out.push(SubsidyProgram {
            key: x.id.clone(),
            from_year: x.from_year,
            to_year: x.to_year,
            share: in_range(ctx, x.share, 0.0, 0.9, &sl.field("anteil")),
            cap_usd: positive(ctx, x.cap_usd, &sl.field("deckel_usd")),
        });
    }
    out
}

/// Partial knowledge of aborted projects (PD8).
fn project_knowledge(ctx: &mut Ctx, p: &RawProjectModel, l: &Loc) -> Option<KnowledgeModel> {
    let k = p.knowledge.as_ref()?;
    let kl = l.field("teilwissen");
    Some(KnowledgeModel {
        share: in_range(ctx, k.share, 0.0, 1.0, &kl.field("anteil")),
        effect: in_range(ctx, k.effect, 0.0, 0.9, &kl.field("wirkung")),
    })
}

/// Platform strategy (PD8).
fn project_platform(ctx: &mut Ctx, p: &RawProjectModel, l: &Loc) -> Option<PlatformModel> {
    let x = p.platform.as_ref()?;
    Some(PlatformModel {
        effect: in_range(
            ctx,
            x.effect,
            0.0,
            0.9,
            &l.field("plattform").field("wirkung"),
        ),
    })
}

/// Project licence (PD8).
fn project_license(ctx: &mut Ctx, p: &RawProjectModel, l: &Loc) -> Option<ProjectLicenseModel> {
    let x = p.license.as_ref()?;
    let ll = l.field("lizenz");
    if !(1..=5).contains(&x.from_phase) {
        ctx.error(
            &ll.field("ab_phase"),
            messages::project_license_phase(x.from_phase),
        );
    }
    Some(ProjectLicenseModel {
        price_share: positive(ctx, x.price_share, &ll.field("preis_anteil")),
        from_phase: x.from_phase.clamp(1, 5),
        fee_share: non_negative(ctx, x.fee_share, &ll.field("gebuehr_anteil")),
        fee_years: x.fee_years.max(1),
    })
}

/// Development partnership (PD8).
fn project_partnership(ctx: &mut Ctx, p: &RawProjectModel, l: &Loc) -> Option<DevPartnershipModel> {
    let x = p.partnership.as_ref()?;
    let pl = l.field("partnerschaft");
    Some(DevPartnershipModel {
        cost_share: in_range(ctx, x.cost_share, 0.0, 0.9, &pl.field("kosten_anteil")),
        ai_accept: in_range(ctx, x.ai_accept, 0.0, 1.0, &pl.field("ki_annahme")),
    })
}

/// The gate council (PD7).
fn project_council(ctx: &mut Ctx, p: &RawProjectModel, l: &Loc) -> Option<CouncilModel> {
    let c = p.council.as_ref()?;
    let cl = l.field("gremium");
    Some(CouncilModel {
        uncertainty: in_range(ctx, c.uncertainty, 0.0, 1.5, &cl.field("unschaerfe")),
        accuracy: in_range(ctx, c.accuracy, 0.0, 1.0, &cl.field("genauigkeit")),
        threshold: in_range(ctx, c.threshold, 0.0, 1.0, &cl.field("schwelle")),
        pause_majority: c.pause_majority.clamp(1, 3),
        max_loops: c.max_loops,
    })
}

/// Fast-track on the leader's proposal (PD7).
fn project_fast_track(ctx: &mut Ctx, p: &RawProjectModel, l: &Loc) -> Option<FastTrackModel> {
    let f = p.fast_track.as_ref()?;
    let fl = l.field("fast_track");
    Some(FastTrackModel {
        experience_min: in_range(ctx, f.experience_min, 0.0, 1.0, &fl.field("erfahrung_min")),
        overlap: in_range(ctx, f.overlap, 0.0, 0.9, &fl.field("ueberlappung")),
        cost_factor: in_range(ctx, f.cost_factor, 1.0, 3.0, &fl.field("kosten_faktor")),
        defect: in_range(ctx, f.defect, 0.0, 1.0, &fl.field("mangel")),
    })
}

/// Development leadership of the laboratories (PD11).
fn project_leadership(ctx: &mut Ctx, p: &RawProjectModel, l: &Loc) -> Option<LeadershipModel> {
    let x = p.leadership.as_ref()?;
    let ll = l.field("leitung");
    let months = |ctx: &mut Ctx, v: u32, max: u32, name: &str| {
        in_range(ctx, f64::from(v), 0.0, f64::from(max), &ll.field(name)) as u32
    };
    Some(LeadershipModel {
        lead_months: months(ctx, x.lead_months, 12, "vorlauf_monate"),
        induction_months: months(ctx, x.induction_months, 6, "einarbeitung_monate"),
        severance_months: months(ctx, x.severance_months, 12, "abfindung_monate"),
        horizon_months: months(ctx, x.horizon_months, 12, "vorausschau_monate").max(1),
        patience_months: months(ctx, x.patience_months, 12, "geduld_monate"),
        reserve: in_range(ctx, x.reserve, 0.0, 1.0, &ll.field("reserve")),
        reserve_thrifty: in_range(
            ctx,
            x.reserve_thrifty,
            0.0,
            5.0,
            &ll.field("reserve_sparsam"),
        ),
        reserve_generous: in_range(
            ctx,
            x.reserve_generous,
            0.0,
            5.0,
            &ll.field("reserve_grosszuegig"),
        ),
        budget_share: in_range(ctx, x.budget_share, 0.0, 1.0, &ll.field("budget_anteil")),
        base_share: in_range(ctx, x.base_share, 0.0, 1.0, &ll.field("grundausstattung")),
    })
}

/// Pilot series on the leader's proposal (PD7).
fn project_pilot(ctx: &mut Ctx, p: &RawProjectModel, l: &Loc) -> Option<PilotModel> {
    let x = p.pilot.as_ref()?;
    let pl = l.field("pilot");
    Some(PilotModel {
        risk_from: in_range(ctx, x.risk_from, 0.0, 1.0, &pl.field("risiko_ab")),
        duration_share: positive(ctx, x.duration_share, &pl.field("dauer_anteil")),
        cost_share: non_negative(ctx, x.cost_share, &pl.field("kosten_anteil")),
        detection: non_negative(ctx, x.detection, &pl.field("aufdeckung_faktor")),
        repair: in_range(ctx, x.repair, 0.0, 1.0, &pl.field("behebung")),
        rework: non_negative(ctx, x.rework, &pl.field("nacharbeit")),
        quality_bonus: in_range(ctx, x.quality_bonus, 0.0, 0.3, &pl.field("qualitaet_bonus")),
        calm: in_range(ctx, x.calm, 0.0, 1.0, &pl.field("beruhigung")),
    })
}

/// Product suggestions and after-calculation of the projects (PD6).
fn project_suggestions(ctx: &mut Ctx, p: &RawProjectModel, l: &Loc) -> Option<SuggestionModel> {
    let s = p.suggestions.as_ref()?;
    let sl = l.field("vorschlaege");
    let wl = sl.field("gewichte");
    let weights = [s.weights.schedule, s.weights.budget, s.weights.figures];
    let sum: f64 = weights.iter().sum();
    if (sum - 1.0).abs() > 1e-6 {
        ctx.error(&wl, messages::project_shares_sum("gewichte", sum));
    }
    for (w, name) in weights.iter().zip(["termin", "budget", "kennzahlen"]) {
        in_range(ctx, *w, 0.0, 1.0, &wl.field(name));
    }
    Some(SuggestionModel {
        open_max: s.open_max.max(1),
        valid_days: s.valid_days.max(1),
        branch_bonus: non_negative(ctx, s.branch_bonus, &sl.field("branchenbonus")),
        uncertainty: in_range(ctx, s.uncertainty, 0.0, 0.9, &sl.field("unschaerfe")),
        accuracy: in_range(ctx, s.accuracy, 0.0, 1.0, &sl.field("genauigkeit")),
        review_years: s.review_years.max(1),
        tolerance: positive(ctx, s.tolerance, &sl.field("toleranz")),
        hit_from: in_range(ctx, s.hit_from, 0.0, 1.0, &sl.field("treffer_ab")),
        weights,
    })
}

const IMPROVEMENT_PHASES: [&str; 3] = ["wm0_wm1", "wm1_wm2", "wm2_wm3"];
const IMPROVEMENT_TARGETS: [&str; 6] = [
    "kosten",
    "qualitaet",
    "ausschuss",
    "rate",
    "vorprodukte",
    "ausstattung",
];

/// Improvement projects WM0–WM3 (PD5): three phases, six targets in a fixed order.
fn project_improvement(ctx: &mut Ctx, p: &RawProjectModel, l: &Loc) -> Option<ImprovementModel> {
    let w = p.improvement.as_ref()?;
    let wl = l.field("weiterentwicklung");
    let phases = project_phases(ctx, &w.phases, &IMPROVEMENT_PHASES, &wl);
    for (name, total) in [
        ("kosten", phases.iter().map(|x| x.cost).sum::<f64>()),
        ("dauer", phases.iter().map(|x| x.time).sum::<f64>()),
    ] {
        if (total - 1.0).abs() > 1e-6 {
            ctx.error(
                &wl.field("phasen"),
                messages::project_shares_sum(name, total),
            );
        }
    }
    let al = wl.field("ansaetze");
    if w.approaches.min == 0 || w.approaches.min > w.approaches.max || w.approaches.max > 6 {
        ctx.error(&al, messages::improvement_approaches());
    }
    let mut targets = Vec::new();
    for (i, key) in IMPROVEMENT_TARGETS.iter().enumerate() {
        let tl = wl.field("ziele").index(i);
        let Some(raw) = w.targets.get(i) else {
            ctx.error(&wl.field("ziele"), messages::project_choice_missing(key));
            continue;
        };
        if raw.id != *key {
            ctx.error(&tl.field("id"), messages::project_phase_order(&raw.id, key));
        }
        let el = tl.field("wirkung");
        targets.push(ImprovementTarget {
            key: raw.id.clone(),
            per_stage: in_range(ctx, raw.per_stage, 0.0, 1.0, &tl.field("je_stufe")),
            cap: in_range(ctx, raw.cap, 0.0, 1.0, &tl.field("obergrenze")),
            effect: ImprovementEffect {
                labor: in_range(ctx, raw.effect.labor, 0.0, 1.0, &el.field("arbeit")),
                inputs: in_range(ctx, raw.effect.inputs, 0.0, 1.0, &el.field("vorprodukte")),
                quality: in_range(ctx, raw.effect.quality, 0.0, 50.0, &el.field("qualitaet")),
                scrap: in_range(ctx, raw.effect.scrap, 0.0, 1.0, &el.field("ausschuss")),
                output: in_range(ctx, raw.effect.output, 0.0, 1.0, &el.field("menge")),
            },
        });
    }
    if w.targets.len() > IMPROVEMENT_TARGETS.len() {
        ctx.error(&wl.field("ziele"), messages::project_phase_extra());
    }
    Some(ImprovementModel {
        stages: w.stages.max(1),
        rest_share: in_range(ctx, w.rest_share, 0.0, 1.0, &wl.field("restanteil")),
        effort_share: positive(ctx, w.effort.share, &wl.field("aufwand").field("anteil")),
        effort_growth: in_range(
            ctx,
            w.effort.growth,
            1.0,
            5.0,
            &wl.field("aufwand").field("wachstum"),
        ),
        retrofit_days: w.retrofit_days,
        immediate_cost_share: in_range(
            ctx,
            w.immediate_cost_share,
            0.0,
            2.0,
            &wl.field("sofort_kosten_anteil"),
        ),
        deadline_days: w.deadline_days.max(1),
        approaches: (w.approaches.min, w.approaches.max),
        approach_spread: in_range(ctx, w.approaches.spread, 0.0, 0.9, &al.field("streuung")),
        approach_cost_slope: non_negative(
            ctx,
            w.approaches.cost_slope,
            &al.field("kosten_steigung"),
        ),
        outcome_spread: in_range(
            ctx,
            w.outcome.spread,
            0.0,
            0.9,
            &wl.field("ergebnis").field("streuung"),
        ),
        outcome_experience: in_range(
            ctx,
            w.outcome.experience,
            0.0,
            1.0,
            &wl.field("ergebnis").field("erfahrung"),
        ),
        outcome_shortfall: non_negative(
            ctx,
            w.outcome.shortfall,
            &wl.field("ergebnis").field("unterbesetzung"),
        ),
        phases,
        targets,
    })
}

/// Events, hidden defects and recalls of the projects (PD4).
fn project_events(ctx: &mut Ctx, p: &RawProjectModel, l: &Loc) -> Option<ProjectEventModel> {
    let e = p.events.as_ref()?;
    let el = l.field("ereignisse");
    let dl = el.field("mangel");
    if e.defect.per_tempo.len() != p.tempos.len() {
        ctx.error(
            &dl.field("je_tempo"),
            messages::project_event_tempo_count(p.tempos.len()),
        );
    }
    let per_tempo: Vec<f64> = e
        .defect
        .per_tempo
        .iter()
        .enumerate()
        .map(|(i, v)| in_range(ctx, *v, 0.0, 1.0, &dl.field("je_tempo").index(i)))
        .collect();
    let rl = el.field("rueckruf");
    if e.recall.days[0] > e.recall.days[1] {
        ctx.error(
            &rl.field("verzug_tage"),
            messages::range_inverted("von", "bis"),
        );
    }
    if e.kinds.is_empty() {
        ctx.error(&el.field("katalog"), messages::project_event_empty());
    }
    let mut kinds = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    for (i, k) in e.kinds.iter().enumerate() {
        let kl = el.field("katalog").index(i);
        if seen.contains(&k.id.as_str()) {
            ctx.error(&kl.field("id"), messages::project_event_duplicate(&k.id));
        }
        seen.push(&k.id);
        for &ph in &k.phases {
            if ph > 5 {
                ctx.error(
                    &kl.field("phasen"),
                    messages::out_of_range(f64::from(ph), 0.0, 5.0),
                );
            }
        }
        if k.phases.is_empty() {
            ctx.error(&kl.field("phasen"), messages::project_event_no_phase(&k.id));
        }
        if !(2..=3).contains(&k.options.len()) {
            ctx.error(
                &kl.field("optionen"),
                messages::project_event_options(&k.id),
            );
        }
        if k.recommended >= k.options.len() {
            ctx.error(
                &kl.field("empfehlung"),
                messages::out_of_range(k.recommended as f64, 0.0, k.options.len() as f64 - 1.0),
            );
        }
        let options = k
            .options
            .iter()
            .enumerate()
            .map(|(j, o)| {
                let ol = kl.field("optionen").index(j);
                ProjectEventOption {
                    key: o.id.clone(),
                    cost: in_range(ctx, o.cost, 0.0, 2.0, &ol.field("kosten")),
                    delay: in_range(ctx, o.delay, -1.0, 1.0, &ol.field("verzug")),
                    quality: in_range(ctx, o.quality, -0.3, 0.3, &ol.field("qualitaet")),
                    defect: in_range(ctx, o.defect, 0.0, 1.0, &ol.field("mangel")),
                }
            })
            .collect();
        kinds.push(ProjectEventKind {
            key: k.id.clone(),
            phases: k.phases.clone(),
            weight: positive(ctx, k.weight, &kl.field("gewicht")),
            recommended: k.recommended,
            options,
        });
    }
    Some(ProjectEventModel {
        deadline_days: e.deadline_days.max(1),
        defect_cap: in_range(ctx, e.defect.cap, 0.0, 1.0, &dl.field("obergrenze")),
        defect_per_condition: in_range(
            ctx,
            e.defect.per_condition,
            0.0,
            1.0,
            &dl.field("je_auflage"),
        ),
        defect_per_shortfall: in_range(
            ctx,
            e.defect.per_shortfall,
            0.0,
            1.0,
            &dl.field("je_unterbesetzung"),
        ),
        defect_per_tempo: per_tempo,
        recall_days: (e.recall.days[0], e.recall.days[1]),
        recall_cost_share: non_negative(ctx, e.recall.cost_share, &rl.field("kosten_anteil")),
        recall_awareness_loss: in_range(
            ctx,
            e.recall.awareness_loss,
            0.0,
            1.0,
            &rl.field("bekanntheit_verlust"),
        ),
        kinds,
    })
}

/// Patents (P7): term, filing period, cost per country, AI filing.
fn patent_model(ctx: &mut Ctx, p: &RawPatents, l: &Loc) -> PatentModel {
    let term = in_range(
        ctx,
        f64::from(p.term_years),
        1.0,
        50.0,
        &l.field("laufzeit_jahre"),
    );
    let days = in_range(
        ctx,
        f64::from(p.filing_days),
        1.0,
        3650.0,
        &l.field("anmeldefrist_tage"),
    );
    let markets = in_range(
        ctx,
        f64::from(p.ai_largest_markets),
        0.0,
        200.0,
        &l.field("ki_groesste_maerkte"),
    );
    PatentModel {
        // Checked ranges: the casts cannot truncate.
        term_years: term as i32,
        filing_days: days as i32,
        cost_per_country_usd: non_negative(
            ctx,
            p.cost_per_country_usd,
            &l.field("kosten_je_land_usd"),
        ),
        ai_largest_markets: markets as usize,
        provenance: super::provenance(p.approximation, p.source.as_ref()),
    }
}

/// Development of researched products (M37): levels, effects, effort and the research
/// field by branch for products without a technology.
fn development_model(
    ctx: &mut Ctx,
    d: &RawProductDevelopment,
    l: &Loc,
    (researchers, qualification): (&[Option<wsim_core::ids::LaborGroupId>], &str),
    (branches, specializations): (&Keys, &Keys),
) -> DevelopmentModel {
    let per_level = l.field("wirkung_max");
    let effort = l.field("aufwand");
    let levels = in_range(ctx, f64::from(d.levels), 1.0, 10.0, &l.field("stufen"));
    let mut fields: Vec<Option<SpecializationId>> = vec![None; branches.index.len()];
    for (branch, field) in &d.fields {
        let loc = l.field("fachgebiete").field(branch);
        let b: BranchId = resolve(ctx, branches, branch, &loc);
        let f: SpecializationId = resolve(ctx, specializations, field, &loc);
        if specializations.index.contains_key(field)
            && researchers.get(f.index()).copied().flatten().is_none()
        {
            ctx.error(
                &loc,
                messages::development_field_without_researchers(field, qualification),
            );
        }
        if let Some(slot) = fields.get_mut(b.index()) {
            *slot = Some(f);
        }
    }
    DevelopmentModel {
        // At most 10 levels: the cast cannot truncate.
        levels: levels as u8,
        quality_max: in_range(
            ctx,
            d.per_level.quality,
            0.0,
            60.0,
            &per_level.field("qualitaet"),
        ),
        labor_max: in_range(ctx, d.per_level.labor, 0.0, 0.9, &per_level.field("arbeit")),
        inputs_max: in_range(
            ctx,
            d.per_level.inputs,
            0.0,
            0.9,
            &per_level.field("vorprodukte"),
        ),
        process_output_max: d.process.as_ref().map_or(0.0, |x| {
            in_range(
                ctx,
                x.output,
                0.0,
                2.0,
                &l.field("fertigung").field("ausstoss"),
            )
        }),
        process_inputs_max: d.process.as_ref().map_or(0.0, |x| {
            in_range(
                ctx,
                x.inputs,
                0.0,
                0.9,
                &l.field("fertigung").field("vorprodukte"),
            )
        }),
        effort_share: in_range(ctx, d.effort.share, 0.01, 2.0, &effort.field("anteil")),
        effort_growth: in_range(ctx, d.effort.growth, 1.0, 3.0, &effort.field("wachstum")),
        base_effort: positive(ctx, d.effort.base, &effort.field("grundaufwand")),
        complexity: d.effort.complexity.as_ref().map(|c| {
            let at = effort.field("komplexitaet");
            wsim_core::catalog::Complexity {
                per_level: in_range(ctx, c.per_level, 0.0, 2.0, &at.field("zuschlag_je_stufe")),
                per_part: in_range(ctx, c.per_part, 0.0, 2.0, &at.field("zuschlag_je_bauteil")),
                min: positive(ctx, c.min, &at.field("faktor_min")),
                max: in_range(ctx, c.max, c.min, 1000.0, &at.field("faktor_max")),
            }
        }),
        fields,
    }
}

/// Every product made without any technology needs a research field for its branch
/// (`forschungsmodell.weiterentwicklung.fachgebiete`, M37).
pub(super) fn check_development_fields(ctx: &mut Ctx, catalog: &Catalog, raw: &RawData) {
    let Some(entry) = raw.research_model.first() else {
        return;
    };
    let loc = entry.loc.field("weiterentwicklung").field("fachgebiete");
    for (product, p) in catalog.products.iter() {
        let mut recipes = catalog.recipes.values().filter(|r| r.product == product);
        let Some(first) = recipes.next() else {
            continue;
        };
        let with_technology = first.technology.is_some() || recipes.any(|r| r.technology.is_some());
        let field = catalog
            .research_model
            .development
            .fields
            .get(p.branch.index())
            .copied()
            .flatten();
        if !with_technology && field.is_none() {
            ctx.error(
                &loc,
                messages::development_field_missing(
                    catalog.products.key(product),
                    catalog.branches.key(p.branch),
                ),
            );
        }
    }
}

const START_FORMS: &[(&str, StartForm)] = &[
    ("werkstatt", StartForm::Workshop),
    ("handel", StartForm::Trading),
];

fn start_setups(
    ctx: &mut Ctx,
    m: &RawProductionModel,
    l: &Loc,
    (products, facilities, recipes): (&Keys, &Keys, &Keys),
) -> Vec<(StartForm, StartSetup)> {
    let forms_loc = l.field("startformen");
    let names: Vec<&str> = START_FORMS.iter().map(|(n, _)| *n).collect();
    let mut setups = Vec::new();
    for (key, raw) in &m.start_setups {
        let Some(&(_, form)) = START_FORMS.iter().find(|(n, _)| n == key) else {
            ctx.error(
                &forms_loc.key(key),
                messages::unknown_value(key, &names, suggest::closest(key, names.iter().copied())),
            );
            continue;
        };
        let loc = forms_loc.field(key);
        let facilities_loc = loc.field("anlagen");
        let purchases_loc = loc.field("einkauf");
        let sales_loc = loc.field("verkauf");
        let setup = StartSetup {
            site_type: super::site_type(raw.site_type),
            building: money(ctx, raw.building_usd, &loc.field("gebaeude_usd")),
            facilities: raw
                .facilities
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let f_loc = facilities_loc.index(i);
                    (
                        resolve(ctx, facilities, &f.facility, &f_loc.field("anlage")),
                        f.recipe
                            .as_ref()
                            .map(|r| resolve(ctx, recipes, r, &f_loc.field("rezept"))),
                        in_range(ctx, f.utilization, 0.0, 1.0, &f_loc.field("auslastung")),
                    )
                })
                .collect(),
            purchases: raw
                .purchases
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let p_loc = purchases_loc.index(i);
                    (
                        resolve(ctx, products, &p.product, &p_loc.field("produkt")),
                        positive(ctx, p.target, &p_loc.field("ziel")),
                        money(ctx, p.max_price_usd, &p_loc.field("hoechstpreis_usd")),
                    )
                })
                .collect(),
            sales: raw
                .sales
                .iter()
                .enumerate()
                .map(|(i, p)| resolve(ctx, products, p, &sales_loc.index(i)))
                .collect(),
        };
        setups.push((form, setup));
    }
    for (name, _) in START_FORMS {
        if !m.start_setups.contains_key(*name) {
            ctx.error(&forms_loc, messages::entry_missing("Startform", name));
        }
    }
    setups
}

/// Facilities of a start form fit its site type, recipes fit their facility, and both
/// are known in the earliest start year.
pub(super) fn check_start_setups(ctx: &mut Ctx, catalog: &Catalog, raw: &RawData) {
    let Some(entry) = raw.production_model.first() else {
        return;
    };
    for (key, setup) in &entry.value.start_setups {
        let Some(&(_, form)) = START_FORMS.iter().find(|(n, _)| n == key) else {
            continue;
        };
        let Some(built) = catalog.production_model.start_setup(form) else {
            continue;
        };
        let loc = entry.loc.field("startformen").field(key).field("anlagen");
        for (i, (&(facility, recipe, _), raw_f)) in
            built.facilities.iter().zip(&setup.facilities).enumerate()
        {
            if catalog.facilities.id(&raw_f.facility) != Some(facility) {
                continue;
            }
            let f = catalog.facilities.get(facility);
            if f.site_type != built.site_type {
                ctx.error(
                    &loc.index(i).field("anlage"),
                    messages::start_facility_wrong_site(&raw_f.facility),
                );
            }
            let too_new = |t: Option<wsim_core::ids::TechnologyId>| {
                t.is_some_and(|t| {
                    catalog.technologies.get(t).invention_year > wsim_core::EARLIEST_START_YEAR
                })
            };
            if too_new(f.technology) {
                ctx.error(
                    &loc.index(i).field("anlage"),
                    messages::start_needs_new_technology(&raw_f.facility),
                );
            }
            if let (Some(recipe), Some(raw_r)) = (recipe, raw_f.recipe.as_deref())
                && catalog.recipes.id(raw_r) == Some(recipe)
            {
                let r = catalog.recipes.get(recipe);
                if r.facility != facility {
                    ctx.error(
                        &loc.index(i).field("rezept"),
                        messages::start_recipe_wrong_facility(raw_r, &raw_f.facility),
                    );
                }
                if too_new(r.technology) {
                    ctx.error(
                        &loc.index(i).field("rezept"),
                        messages::start_needs_new_technology(raw_r),
                    );
                }
            }
        }
    }
}
