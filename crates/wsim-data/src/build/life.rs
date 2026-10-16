//! Age, retirement and death of the managers of `lebenslauf` (docs/FORMELN.md, PE1).

use wsim_core::catalog::{
    AgeSpan, AssetModel, BackgroundEffect, Catalog, CountrySeries, FoundingModel, FundTerms,
    LegacyModel, LifeModel, LifestyleLevel, PersonModel, PrivateLifeModel, PropertyModel,
    PropertyTerms, ServiceModel, StakeRules, WellbeingModel,
};
use wsim_core::ids::Id;
use wsim_core::state::{Background, Lifestyle};

use super::{in_range, non_negative, positive, provenance, time_series};
use crate::messages;
use crate::raw::{RawAgeSpan, RawCountrySeries, RawLifestyle, RawPerson};
use crate::read::{Ctx, Loc, RawData};

/// Ages a manager may have, in years.
const AGES: (f64, f64) = (16.0, 100.0);

fn span(ctx: &mut Ctx, s: &RawAgeSpan, l: &Loc) -> AgeSpan {
    let min = in_range(ctx, s.min, AGES.0, AGES.1, &l.field("von"));
    let max = in_range(ctx, s.max, AGES.0, AGES.1, &l.field("bis"));
    if min > max {
        ctx.error(l, messages::range_inverted("von", "bis"));
    }
    if s.mean < min || s.mean > max {
        ctx.error(
            &l.field("mittel"),
            messages::age_mean_outside(s.mean, min, max),
        );
    }
    AgeSpan {
        min,
        max,
        mean: s.mean,
        spread: positive(ctx, s.spread, &l.field("streuung")),
    }
}

/// A value per country and year: values within `range`, countries known.
pub(super) fn country_series(
    ctx: &mut Ctx,
    catalog: &Catalog,
    raw: &RawCountrySeries,
    l: &Loc,
    range: (f64, f64),
) -> CountrySeries {
    let check = |ctx: &mut Ctx, values: &crate::raw::RawSeries, at: &Loc| {
        for (&y, &v) in values {
            in_range(ctx, v, range.0, range.1, &at.field(&y.to_string()));
        }
        time_series(ctx, values, at)
    };
    let default = Some(check(ctx, &raw.default, &l.field("standard")));
    let mut countries = vec![None; catalog.countries.len()];
    let cl = l.field("laender");
    for (key, values) in &raw.countries {
        match catalog.countries.id(key) {
            Some(id) => countries[id.index()] = Some(check(ctx, values, &cl.field(key))),
            None => {
                let suggested = crate::suggest::closest(
                    key,
                    catalog.countries.keys().iter().map(String::as_str),
                );
                ctx.error(
                    &cl.key(key),
                    messages::unknown_reference("Land", key, suggested),
                );
            }
        }
    }
    CountrySeries { default, countries }
}

pub(super) fn life_model(ctx: &mut Ctx, catalog: &Catalog, raw: &RawData) -> LifeModel {
    // Optional: without the section managers neither age nor retire.
    let Some((entry, rest)) = raw.life.split_first() else {
        return LifeModel::default();
    };
    let first = ctx.describe(&entry.loc);
    for other in rest {
        ctx.error(
            &other.loc,
            messages::section_duplicate("lebenslauf", &first),
        );
    }
    let v = &entry.value;
    let l = &entry.loc;
    let el = l.field("eintrittsalter");
    let entry_age = [
        span(ctx, &v.entry_age.site, &el.field("standort")),
        span(ctx, &v.entry_age.country, &el.field("land")),
        span(ctx, &v.entry_age.continent, &el.field("kontinent")),
        span(ctx, &v.entry_age.board, &el.field("vorstand")),
    ];
    let sl = l.field("ebene_nach_staerke");
    let level_strength = [
        in_range(ctx, v.level_strength.country, 0.0, 100.0, &sl.field("land")),
        in_range(
            ctx,
            v.level_strength.continent,
            0.0,
            100.0,
            &sl.field("kontinent"),
        ),
        in_range(
            ctx,
            v.level_strength.board,
            0.0,
            100.0,
            &sl.field("vorstand"),
        ),
    ];
    if level_strength[0] > level_strength[1] || level_strength[1] > level_strength[2] {
        ctx.error(&sl, messages::levels_not_ascending());
    }
    let (xl, rl, dl, tl, ml) = (
        l.field("erfahrung"),
        l.field("risiko"),
        l.field("abbau"),
        l.field("ruhestand"),
        l.field("sterbetafel"),
    );
    let age = |ctx: &mut Ctx, x: f64, at: &Loc| in_range(ctx, x, AGES.0, AGES.1, at);
    let factor = |ctx: &mut Ctx, x: f64, at: &Loc| in_range(ctx, x, 0.0, 10.0, at);
    let r = &v.retirement;
    // Whole numbers of months and years; the casts are exact after the range checks.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let warning_months = in_range(
        ctx,
        r.warning_months,
        1.0,
        60.0,
        &tl.field("vorwarnung_monate"),
    )
    .round() as u32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let extension_years_max = in_range(
        ctx,
        r.extension_years_max,
        0.0,
        10.0,
        &tl.field("verlaengerung_jahre_max"),
    )
    .round() as u32;
    let acceptance_ages = [
        age(ctx, r.acceptance_ages[0], &tl.field("zusage_alter")),
        age(ctx, r.acceptance_ages[1], &tl.field("zusage_alter")),
    ];
    if acceptance_ages[0] >= acceptance_ages[1] {
        ctx.error(
            &tl.field("zusage_alter"),
            messages::range_inverted("zusage_alter[0]", "zusage_alter[1]"),
        );
    }
    LifeModel {
        enabled: true,
        entry_age,
        level_strength,
        young_until: age(ctx, v.experience.jung_bis, &xl.field("jung_bis")),
        young_factor: factor(ctx, v.experience.jung, &xl.field("jung")),
        old_from: age(ctx, v.experience.alt_ab, &xl.field("alt_ab")),
        old_factor: factor(ctx, v.experience.alt, &xl.field("alt")),
        risk_from: age(ctx, v.risk.from, &rl.field("ab")),
        risk_per_year: in_range(ctx, v.risk.per_year, 0.0, 10.0, &rl.field("je_jahr")),
        risk_max: in_range(ctx, v.risk.max, 0.0, 100.0, &rl.field("hoechstens")),
        decline_from: age(ctx, v.decline.from, &dl.field("ab")),
        decline_chance: in_range(ctx, v.decline.chance, 0.0, 1.0, &dl.field("chance")),
        retirement_spread: in_range(ctx, r.spread, 0.0, 20.0, &tl.field("abweichung")),
        warning_months,
        extension_years_max,
        extension_raise: in_range(
            ctx,
            r.extension_raise,
            0.0,
            5.0,
            &tl.field("verlaengerung_aufschlag"),
        ),
        reference_age: age(ctx, r.reference_age, &tl.field("bezugsalter")),
        acceptance_ages,
        mortality_from: age(ctx, v.mortality.from, &ml.field("ab")),
        mortality_chance: in_range(ctx, v.mortality.chance, 0.0, 1.0, &ml.field("chance")),
        doubling_years: positive(
            ctx,
            v.mortality.doubling_years,
            &ml.field("verdopplung_jahre"),
        ),
        retirement_age: country_series(
            ctx,
            catalog,
            &v.retirement_age,
            &l.field("ruhestandsalter"),
            (40.0, 90.0),
        ),
        life_expectancy: country_series(
            ctx,
            catalog,
            &v.life_expectancy,
            &l.field("lebenserwartung"),
            (30.0, 110.0),
        ),
        provenance: provenance(v.approximation, v.source.as_ref()),
    }
}

/// The player as a person (`person`, PE2); optional.
pub(super) fn person_model(ctx: &mut Ctx, catalog: &Catalog, raw: &RawData) -> PersonModel {
    let Some((entry, rest)) = raw.person.split_first() else {
        return PersonModel::default();
    };
    let first = ctx.describe(&entry.loc);
    for other in rest {
        ctx.error(&other.loc, messages::section_duplicate("person", &first));
    }
    let v = &entry.value;
    let l = &entry.loc;
    let (al, fl) = (l.field("alter_start"), l.field("familie"));
    // Whole years and counts within the checked ranges; the casts are exact.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let years =
        |ctx: &mut Ctx, x: f64, at: &Loc| in_range(ctx, x, AGES.0, AGES.1, at).round() as u32;
    let start_age = [
        years(ctx, v.start_age.standard, &al.field("standard")),
        years(ctx, v.start_age.min, &al.field("von")),
        years(ctx, v.start_age.max, &al.field("bis")),
    ];
    if start_age[1] > start_age[0] || start_age[0] > start_age[2] {
        ctx.error(&al, messages::start_age_outside());
    }
    let f = &v.family;
    let children_ages = [
        years(ctx, f.children_from, &fl.field("kinder_ab")),
        years(ctx, f.children_until, &fl.field("kinder_bis")),
    ];
    if children_ages[0] >= children_ages[1] {
        ctx.error(&fl, messages::range_inverted("kinder_ab", "kinder_bis"));
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let children_max = in_range(
        ctx,
        f.children_max,
        0.0,
        20.0,
        &fl.field("kinder_hoechstens"),
    )
    .round() as u32;
    PersonModel {
        enabled: true,
        start_age,
        children_ages,
        children_max,
        child_chance: in_range(
            ctx,
            f.child_chance,
            0.0,
            1.0,
            &fl.field("kinder_chance_jahr"),
        ),
        card_age: years(ctx, f.card_age, &fl.field("managerkarte_ab")),
        ..money_model(ctx, catalog, v, l)
    }
}

/// The private account, founding and lifestyle of `person` (docs/FORMELN.md, PE3).
fn money_model(ctx: &mut Ctx, catalog: &Catalog, v: &RawPerson, l: &Loc) -> PersonModel {
    let gl = l.field("gruendung");
    let founding = FoundingModel {
        cost_share: in_range(
            ctx,
            v.founding.cost_share,
            0.0,
            1.0,
            &gl.field("kosten_anteil"),
        ),
        cost_min_months: in_range(
            ctx,
            v.founding.cost_min_months,
            0.0,
            120.0,
            &gl.field("kosten_mindestens_monatsloehne"),
        ),
        capital_suggestion: in_range(
            ctx,
            v.founding.capital_suggestion,
            0.0,
            1.0,
            &gl.field("einlage_vorschlag"),
        ),
    };
    let dl = l.field("darlehen");
    // Whole years within the checked range; the cast is exact.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let loan_max_years = in_range(
        ctx,
        v.loans.max_years,
        1.0,
        100.0,
        &dl.field("jahre_hoechstens"),
    )
    .round() as u32;
    let ll = l.field("lebensstil");
    let sl = ll.field("stufen");
    let s = &v.lifestyle.levels;
    let lifestyles = [
        lifestyle(ctx, &s.modest, &sl.field(Lifestyle::Modest.key())),
        lifestyle(ctx, &s.middle, &sl.field(Lifestyle::Middle.key())),
        lifestyle(ctx, &s.upscale, &sl.field(Lifestyle::Upscale.key())),
        lifestyle(ctx, &s.luxury, &sl.field(Lifestyle::Luxury.key())),
    ];
    if lifestyles.windows(2).any(|w| w[0].cost >= w[1].cost) {
        ctx.error(&sl, messages::lifestyle_costs_not_ascending());
    }
    let default_lifestyle = Lifestyle::from_key(&v.lifestyle.standard).unwrap_or_else(|| {
        let suggested = crate::suggest::closest(
            &v.lifestyle.standard,
            Lifestyle::ALL.iter().map(|l| l.key()),
        );
        ctx.error(
            &ll.field("standard"),
            messages::unknown_reference("Lebensstil", &v.lifestyle.standard, suggested),
        );
        Lifestyle::Middle
    });
    // Whole months within the checked range; the cast is exact.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let lifestyle_months = in_range(
        ctx,
        v.lifestyle.change_months,
        0.0,
        120.0,
        &ll.field("wechsel_monate"),
    )
    .round() as u32;
    PersonModel {
        founding,
        salary_co_owner_max: in_range(
            ctx,
            v.salary.co_owner_max,
            1.0,
            100.0,
            &l.field("gehalt").field("hoechstens_mitgesellschafter"),
        ),
        loan_max_rate: in_range(
            ctx,
            v.loans.max_rate,
            0.0,
            1.0,
            &dl.field("zins_hoechstens"),
        ),
        loan_max_years,
        lifestyles,
        default_lifestyle,
        lifestyle_months,
        income_tax: country_series(
            ctx,
            catalog,
            &v.income_tax,
            &l.field("einkommensteuer"),
            (0.0, 1.0),
        ),
        income_brackets: tax_brackets(ctx, &v.income_brackets, &l.field("einkommensteuer_stufen")),
        capital_tax: v
            .capital_tax
            .as_ref()
            .map(|s| country_series(ctx, catalog, s, &l.field("kapitalertragsteuer"), (0.0, 1.0))),
        savings_rate: country_series(
            ctx,
            catalog,
            &v.savings_rate,
            &l.field("sparzins"),
            (-0.5, 0.5),
        ),
        nephew_age: heirs_age(
            ctx,
            &v.heirs.nephew_age,
            &l.field("erbe").field("neffe_alter"),
        ),
        // Whole years within the checked range; the cast is exact.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        succession_hint_from: in_range(
            ctx,
            v.heirs.hint_from,
            18.0,
            120.0,
            &l.field("erbe").field("hinweis_ab"),
        )
        .round() as u32,
        inheritance_tax: country_series(
            ctx,
            catalog,
            &v.inheritance_tax,
            &l.field("erbschaftsteuer"),
            (0.0, 1.0),
        ),
        private_life: v
            .private_life
            .as_ref()
            .map_or_else(PrivateLifeModel::default, |p| {
                private_life(ctx, p, &l.field("privatleben"))
            }),
        legacy: v
            .legacy
            .as_ref()
            .map(|f| legacy(ctx, catalog, f, &l.field("nachwuchs"))),
        assets: v
            .assets
            .as_ref()
            .map(|a| assets(ctx, a, &l.field("anlagen"))),
        wellbeing: v
            .wellbeing
            .as_ref()
            .map(|w| wellbeing(ctx, w, &l.field("wohlbefinden"))),
        standing: v
            .standing
            .as_ref()
            .map(|s| standing(ctx, s, &l.field("ansehen"))),
        personal: v
            .personal
            .as_ref()
            .map(|p| personal(ctx, p, &l.field("persoenliches"))),
        ai_persons: v.ai_persons.as_ref().map(|a| {
            let al = l.field("ki_unternehmer");
            let span =
                |ctx: &mut Ctx, s: &crate::raw::RawFloatSpan, hi: f64, at: Loc| -> [f64; 2] {
                    let lo = non_negative(ctx, s.min, &at.field("von"));
                    let up = non_negative(ctx, s.max, &at.field("bis"));
                    if lo > up {
                        ctx.error(&at, messages::range_inverted("von", "bis"));
                    }
                    [lo.min(hi), up.min(hi)]
                };
            wsim_core::catalog::AiPersonModel {
                start_wealth: span(
                    ctx,
                    &a.start_wealth,
                    10_000.0,
                    al.field("startvermoegen_jahreslohn"),
                ),
                age: span(ctx, &a.age, 100.0, al.field("alter")),
                profile_weights: [
                    non_negative(
                        ctx,
                        a.profiles.cautious,
                        &al.field("profile").field("vorsichtig"),
                    ),
                    non_negative(
                        ctx,
                        a.profiles.balanced,
                        &al.field("profile").field("ausgewogen"),
                    ),
                    non_negative(ctx, a.profiles.risky, &al.field("profile").field("riskant")),
                ],
                budget: in_range(ctx, a.budget, 0.0, 1.0, &al.field("budget")),
                successor_age: span(ctx, &a.successor_age, 100.0, al.field("nachfolger_alter")),
                successor_share: in_range(
                    ctx,
                    a.successor_share,
                    0.0,
                    1.0,
                    &al.field("nachfolger_anteil"),
                ),
            }
        }),
        speculation: v.speculation.as_ref().map(|s| {
            let sl = l.field("spekulation");
            let ll = sl.field("wertpapierkredit");
            let cl = sl.field("sammler");
            let bl = ll.field("beleihung");
            wsim_core::catalog::SpeculationModel {
                lombard: wsim_core::catalog::LombardTerms {
                    lending: [
                        in_range(ctx, s.lombard.lending.funds, 0.0, 0.95, &bl.field("fonds")),
                        in_range(ctx, s.lombard.lending.gold, 0.0, 0.95, &bl.field("gold")),
                        in_range(
                            ctx,
                            s.lombard.lending.collectibles,
                            0.0,
                            0.95,
                            &bl.field("sammler"),
                        ),
                    ],
                    rate_premium: in_range(
                        ctx,
                        s.lombard.rate_premium,
                        0.0,
                        1.0,
                        &ll.field("zins_aufschlag"),
                    ),
                    call_months: s.lombard.call_months,
                },
                collectors: wsim_core::catalog::CollectorTerms {
                    volatility: in_range(
                        ctx,
                        s.collectors.volatility,
                        0.0,
                        1.0,
                        &cl.field("schwankung"),
                    ),
                    reversion: in_range(
                        ctx,
                        s.collectors.reversion,
                        0.0,
                        1.0,
                        &cl.field("rueckkehr"),
                    ),
                    spread: in_range(ctx, s.collectors.spread, 0.0, 0.5, &cl.field("spanne")),
                },
            }
        }),
        trust: v.trust.as_ref().map(|t| {
            let bl = l.field("bindung");
            let fl = bl.field("stiftung");
            wsim_core::catalog::TrustModel {
                holding_cost: non_negative(ctx, t.holding_cost, &bl.field("holding_kosten")),
                foundation_min: non_negative(ctx, t.foundation.min, &fl.field("mindest")),
                gift_tax: in_range(
                    ctx,
                    t.foundation.gift_tax,
                    0.0,
                    1.0,
                    &fl.field("einlage_steuer"),
                ),
                yield_year: in_range(
                    ctx,
                    t.foundation.yield_year,
                    -0.5,
                    1.0,
                    &fl.field("rendite_jahr"),
                ),
                payout_year: in_range(
                    ctx,
                    t.foundation.payout_year,
                    0.0,
                    1.0,
                    &fl.field("ausschuettung_jahr"),
                ),
                standing_per_wage: non_negative(
                    ctx,
                    t.foundation.standing_per_wage,
                    &fl.field("ansehen_je_jahreslohn"),
                ),
                standing_cap: in_range(
                    ctx,
                    t.foundation.standing_cap,
                    0.0,
                    100.0,
                    &fl.field("ansehen_deckel"),
                ),
            }
        }),
        provenance: provenance(v.approximation, v.source.as_ref()),
        ..PersonModel::default()
    }
}

/// Partner search, private schools and health care: costs 0–50 monthly wages, the chance
/// 0–1, education 0–50 points, the factor on the death chance 0.1–1.
fn private_life(ctx: &mut Ctx, p: &crate::raw::RawPrivateLife, l: &Loc) -> PrivateLifeModel {
    let (pl, sl, cl) = (
        l.field("partnersuche"),
        l.field("privatschule"),
        l.field("vorsorge"),
    );
    PrivateLifeModel {
        enabled: true,
        partner_cost: in_range(ctx, p.partner.cost, 0.0, 50.0, &pl.field("kosten_monat")),
        partner_chance: in_range(ctx, p.partner.chance, 0.0, 1.0, &pl.field("chance_monat")),
        school_cost: in_range(ctx, p.school.cost, 0.0, 50.0, &sl.field("kosten_monat")),
        school_education: in_range(ctx, p.school.education, 0.0, 50.0, &sl.field("ausbildung")),
        care_cost: in_range(ctx, p.care.cost, 0.0, 50.0, &cl.field("kosten_monat")),
        care_mortality: in_range(ctx, p.care.mortality, 0.1, 1.0, &cl.field("sterblichkeit")),
    }
}

/// The age of a nephew or niece who inherits (PE6): from below to, both 18–80.
fn heirs_age(ctx: &mut Ctx, r: &crate::raw::RawAgeRange, l: &Loc) -> [u32; 2] {
    let from = in_range(ctx, r.from, 18.0, 80.0, &l.field("von"));
    let to = in_range(ctx, r.to, 18.0, 80.0, &l.field("bis"));
    if from > to {
        ctx.error(l, messages::range_inverted("von", "bis"));
    }
    // Whole years within the checked range; the casts are exact.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    [from.round() as u32, to.round() as u32]
}

fn lifestyle(ctx: &mut Ctx, r: &RawLifestyle, l: &Loc) -> LifestyleLevel {
    LifestyleLevel {
        cost: in_range(ctx, r.cost, 0.0, 1000.0, &l.field("kosten")),
        interest: in_range(ctx, r.interest, -0.1, 0.1, &l.field("zins")),
        salary_demand: in_range(
            ctx,
            r.salary_demand,
            -0.9,
            1.0,
            &l.field("gehaltsforderung"),
        ),
        education: in_range(ctx, r.education, 0.0, 50.0, &l.field("ausbildung")),
        mortality: in_range(ctx, r.mortality, 0.0, 10.0, &l.field("sterblichkeit")),
    }
}

/// The brackets of the income tax: rising starts from 0, factors 0–5 (PV1).
fn tax_brackets(ctx: &mut Ctx, raw: &[crate::raw::RawTaxBracket], l: &Loc) -> Vec<(f64, f64)> {
    let mut out: Vec<(f64, f64)> = Vec::new();
    for (i, x) in raw.iter().enumerate() {
        let at = l.index(i);
        let from = non_negative(ctx, x.from, &at.field("ab"));
        let factor = in_range(ctx, x.factor, 0.0, 5.0, &at.field("faktor"));
        if out.last().is_some_and(|&(prev, _)| from <= prev) {
            ctx.error(&at.field("ab"), messages::brackets_not_ascending());
        }
        out.push((from, factor));
    }
    out
}

/// Background, time budget and health (PV1b): every background present, shares and
/// factors within sensible bounds, the default time adding up to 100.
fn personal(
    ctx: &mut Ctx,
    p: &crate::raw::RawPersonal,
    l: &Loc,
) -> wsim_core::catalog::PersonalModel {
    use wsim_core::catalog::{EventCondition, EventOption, EventTerms};
    let el = l.field("liste");
    if p.events.is_empty() {
        ctx.error(&el, messages::standing_list_empty("liste"));
    }
    let mut events: Vec<EventTerms> = Vec::new();
    for (i, e) in p.events.iter().enumerate() {
        let at = el.index(i);
        if events.iter().any(|x| x.key == e.id) {
            ctx.error(&at.field("id"), messages::standing_id_duplicate(&e.id));
        }
        let condition = match e.condition.as_str() {
            "immer" => EventCondition::Always,
            "verheiratet" => EventCondition::Married,
            "kinder" => EventCondition::Children,
            "gesundheit_niedrig" => EventCondition::LowHealth,
            "firma" => EventCondition::Leader,
            "mieter" => EventCondition::Tenants,
            other => {
                ctx.error(
                    &at.field("bedingung"),
                    messages::event_condition_unknown(other),
                );
                EventCondition::Always
            }
        };
        if e.options.len() < 2 {
            ctx.error(
                &at.field("optionen"),
                messages::standing_list_empty("optionen (mindestens zwei)"),
            );
        }
        let options = e
            .options
            .iter()
            .enumerate()
            .map(|(j, o)| {
                let ol = at.field("optionen").index(j);
                EventOption {
                    key: o.id.clone(),
                    cost: non_negative(ctx, o.cost, &ol.field("kosten")),
                    bond: in_range(ctx, o.bond, -100.0, 100.0, &ol.field("beziehung")),
                    standing: in_range(ctx, o.standing, -100.0, 100.0, &ol.field("ansehen")),
                    health: in_range(ctx, o.health, -100.0, 100.0, &ol.field("gesundheit")),
                    interest: in_range(ctx, o.interest, -100.0, 100.0, &ol.field("interesse")),
                    loyalty: in_range(ctx, o.loyalty, -100.0, 100.0, &ol.field("loyalitaet")),
                    gain: non_negative(ctx, o.gain, &ol.field("gewinn")),
                    detect: in_range(ctx, o.detect, 0.0, 1.0, &ol.field("entdeckung")),
                    fine: non_negative(ctx, o.fine, &ol.field("strafe")),
                    shame: in_range(ctx, o.shame, 0.0, 100.0, &ol.field("schande")),
                }
            })
            .collect();
        events.push(EventTerms {
            key: e.id.clone(),
            condition,
            chance_year: in_range(ctx, e.chance_year, 0.0, 1.0, &at.field("chance_jahr")),
            options,
        });
    }
    let bl = l.field("beziehung");
    let sl = l.field("trennung");
    wsim_core::catalog::PersonalModel {
        chance_factor: non_negative(ctx, p.chance_factor, &l.field("chance_faktor")),
        wait_days: p.wait_days.clamp(1, 365),
        bond_start: in_range(ctx, p.bond.start, 0.0, 100.0, &bl.field("start")),
        bond_holds_at: in_range(ctx, p.bond.holds_at, 0.0, 100.0, &bl.field("haelt_bei")),
        bond_rate: non_negative(ctx, p.bond.rate, &bl.field("aenderung")),
        bond_warn: in_range(ctx, p.bond.warn, 0.0, 100.0, &bl.field("warnung_unter")),
        bond_separate: in_range(ctx, p.separation.below, 0.0, 100.0, &sl.field("unter")),
        share_without: in_range(
            ctx,
            p.separation.without,
            0.0,
            1.0,
            &sl.field("anteil_ohne_vertrag"),
        ),
        share_with: in_range(
            ctx,
            p.separation.with,
            0.0,
            1.0,
            &sl.field("anteil_mit_vertrag"),
        ),
        contract_cost: non_negative(ctx, p.contract_cost, &l.field("ehevertrag_kosten")),
        nepotism_malus: p.nepotism_malus.min(100),
        low_health: in_range(ctx, p.low_health, 0.0, 100.0, &l.field("gesundheit_unter")),
        pending_max: p.pending_max.clamp(1, 10),
        events,
    }
}

fn standing(
    ctx: &mut Ctx,
    s: &crate::raw::RawStanding,
    l: &Loc,
) -> wsim_core::catalog::StandingModel {
    use wsim_core::catalog::{ClubTerms, ContactRole, GoodTerms};
    let target_lifestyle = std::array::from_fn(|i| {
        in_range(
            ctx,
            s.target_lifestyle[i],
            0.0,
            100.0,
            &l.field("ziel_lebensstil").index(i),
        )
    });
    let years = {
        let at = l.field("kontakt_jahre");
        let a = in_range(ctx, s.contact_years.min, 1.0, 80.0, &at.field("von"));
        let b = in_range(ctx, s.contact_years.max, 1.0, 80.0, &at.field("bis"));
        if a > b {
            ctx.error(&at, messages::range_inverted("von", "bis"));
        }
        [a, b]
    };
    let cl = l.field("clubs");
    if s.clubs.is_empty() {
        ctx.error(&cl, messages::standing_list_empty("clubs"));
    }
    let mut clubs: Vec<ClubTerms> = Vec::new();
    for (i, c) in s.clubs.iter().enumerate() {
        let at = cl.index(i);
        if clubs.iter().any(|x| x.key == c.id) {
            ctx.error(&at.field("id"), messages::standing_id_duplicate(&c.id));
        }
        let mut roles = Vec::new();
        for (j, r) in c.roles.iter().enumerate() {
            match ContactRole::from_key(r) {
                Some(role) => roles.push(role),
                None => ctx.error(
                    &at.field("rollen").index(j),
                    messages::contact_role_unknown(r),
                ),
            }
        }
        if c.roles.is_empty() {
            ctx.error(&at.field("rollen"), messages::standing_list_empty("rollen"));
        }
        clubs.push(ClubTerms {
            key: c.id.clone(),
            standing: in_range(ctx, c.standing, 0.0, 100.0, &at.field("ansehen")),
            lifestyle: u8::try_from(c.lifestyle.min(3)).unwrap_or(3),
            fee: non_negative(ctx, c.fee, &at.field("beitrag")),
            contacts_year: non_negative(ctx, c.contacts_year, &at.field("kontakte_jahr")),
            roles,
        });
    }
    let gl = l.field("besitz");
    let mut goods: Vec<GoodTerms> = Vec::new();
    for (i, g) in s.goods.iter().enumerate() {
        let at = gl.index(i);
        if goods.iter().any(|x| x.key == g.id) {
            ctx.error(&at.field("id"), messages::standing_id_duplicate(&g.id));
        }
        goods.push(GoodTerms {
            key: g.id.clone(),
            from: g.from,
            price: positive(ctx, g.price, &at.field("preis")),
            upkeep: in_range(ctx, g.upkeep, 0.0, 1.0, &at.field("unterhalt")),
            representation: in_range(
                ctx,
                g.representation,
                0.0,
                100.0,
                &at.field("repraesentation"),
            ),
            collectible: g.collectible,
        });
    }
    wsim_core::catalog::StandingModel {
        target_base: in_range(ctx, s.target_base, 0.0, 100.0, &l.field("ziel_basis")),
        target_lifestyle,
        target_per_company: in_range(
            ctx,
            s.target_per_company,
            0.0,
            100.0,
            &l.field("ziel_je_firma"),
        ),
        rate: in_range(ctx, s.rate, 0.0, 1.0, &l.field("annaeherung_monat")),
        donation_points: non_negative(
            ctx,
            s.donation_points,
            &l.field("spende_punkte_je_jahreslohn"),
        ),
        donation_cap: non_negative(ctx, s.donation_cap, &l.field("spende_deckel")),
        interest_per_point: in_range(
            ctx,
            s.interest_per_point,
            -0.001,
            0.001,
            &l.field("kreditzins_je_punkt"),
        ),
        contact_years: years,
        care_gain: non_negative(ctx, s.care_gain, &l.field("pflege_gewinn")),
        care_decay: non_negative(ctx, s.care_decay, &l.field("pflege_verfall")),
        banker_interest: in_range(
            ctx,
            s.banker_interest,
            -0.05,
            0.05,
            &l.field("bankier_zins"),
        ),
        broker_discount: in_range(ctx, s.broker_discount, 0.0, 0.5, &l.field("makler_rabatt")),
        tip_chance: in_range(ctx, s.tip_chance, 0.0, 1.0, &l.field("hinweis_chance")),
        clubs,
        goods,
    }
}

fn wellbeing(ctx: &mut Ctx, w: &crate::raw::RawWellbeing, l: &Loc) -> WellbeingModel {
    let bl = l.field("hintergruende");
    let mut backgrounds = [BackgroundEffect::default(); 4];
    for key in w.backgrounds.keys() {
        if Background::from_key(key).is_none() {
            ctx.error(&bl.field(key), messages::background_unknown(key));
        }
    }
    for b in Background::ALL {
        let at = bl.field(b.key());
        let Some(r) = w.backgrounds.get(b.key()) else {
            ctx.error(&bl, messages::background_missing(b.key()));
            continue;
        };
        backgrounds[b as usize] = BackgroundEffect {
            research: in_range(ctx, r.research, 0.0, 1.0, &at.field("forschung")),
            loan_rate: in_range(ctx, r.loan_rate, -0.05, 0.05, &at.field("kreditzins")),
            order_fee: in_range(ctx, r.order_fee, -1.0, 1.0, &at.field("ordergebuehr")),
            loyalty: in_range(ctx, r.loyalty, 0.0, 1.0, &at.field("loyalitaet")),
            standing: in_range(ctx, r.standing, 0.0, 100.0, &at.field("ansehen")),
            network: in_range(ctx, r.network, 0.0, 100.0, &at.field("netzwerk")),
        };
    }
    let tl = l.field("zeit");
    let d = &w.time.default;
    if d.firm + d.family + d.network + d.rest != 100 {
        ctx.error(&tl.field("standard"), messages::time_not_hundred());
    }
    let mf = tl.field("mindest_firma");
    let min_firm = [
        in_range(ctx, w.time.min_firm.base, 0.0, 100.0, &mf.field("basis")),
        in_range(
            ctx,
            w.time.min_firm.per_company,
            0.0,
            100.0,
            &mf.field("je_firma"),
        ),
        in_range(
            ctx,
            w.time.min_firm.most,
            0.0,
            100.0,
            &mf.field("hoechstens"),
        ),
    ];
    let hl = l.field("gesundheit");
    let h = &w.health;
    let lifestyle_gain = std::array::from_fn(|i| {
        in_range(
            ctx,
            h.lifestyle_gain[i],
            -10.0,
            10.0,
            &hl.field("lebensstil_gewinn").index(i),
        )
    });
    WellbeingModel {
        backgrounds,
        cap: in_range(ctx, w.cap, 0.0, 1.0, &l.field("deckel")),
        experience_per_year: in_range(
            ctx,
            w.experience_per_year,
            0.0,
            1.0,
            &l.field("erfahrung_je_jahr"),
        ),
        time_default: [d.firm, d.family, d.network, d.rest],
        min_firm,
        delay_days: in_range(
            ctx,
            w.time.delay_days,
            0.0,
            365.0,
            &tl.field("verzoegerung_tage"),
        ),
        health_start: in_range(ctx, h.start, 1.0, 100.0, &hl.field("start")),
        aging_from: h.aging_from,
        aging_per_year: in_range(
            ctx,
            h.aging_per_year,
            0.0,
            10.0,
            &hl.field("altern_je_jahr"),
        ),
        overload_from: in_range(
            ctx,
            h.overload_from,
            0.0,
            99.0,
            &hl.field("ueberlastung_ab"),
        ),
        overload_loss: in_range(
            ctx,
            h.overload_loss,
            0.0,
            50.0,
            &hl.field("ueberlastung_verlust"),
        ),
        rest_gain: in_range(ctx, h.rest_gain, 0.0, 50.0, &hl.field("erholung_gewinn")),
        lifestyle_gain,
        care_gain: in_range(ctx, h.care_gain, 0.0, 10.0, &hl.field("vorsorge_gewinn")),
        healthy_from: in_range(ctx, h.healthy_from, 1.0, 100.0, &hl.field("gesund_ab")),
        mortality_at_zero: in_range(
            ctx,
            h.mortality_at_zero,
            1.0,
            20.0,
            &hl.field("sterblichkeit_null"),
        ),
        ill_below: in_range(ctx, h.ill_below, 0.0, 100.0, &hl.field("krank_unter")),
        ill_chance: in_range(ctx, h.ill_chance, 0.0, 1.0, &hl.field("krank_chance")),
        ill_months: h.ill_months,
    }
}

/// Safe investments, gold and funds (PV2): spans ascending, rates and fees within bounds.
fn assets(ctx: &mut Ctx, a: &crate::raw::RawAssets, l: &Loc) -> AssetModel {
    let fl = l.field("festgeld");
    let bl = l.field("staatsanleihen");
    let gl = l.field("gold");
    let span = |ctx: &mut Ctx, s: &crate::raw::RawMonthsSpan, at: &Loc| -> [u32; 2] {
        let lm = at.field("laufzeit_monate");
        if s.min == 0 || s.min > s.max || s.max > 600 {
            ctx.error(&lm, messages::months_span_invalid());
        }
        [s.min, s.max]
    };
    let fund = |ctx: &mut Ctx, f: &crate::raw::RawFundTerms, at: Loc| FundTerms {
        from: i32::try_from(
            in_range(ctx, f64::from(f.from), 1800.0, 2200.0, &at.field("ab")) as i64,
        )
        .unwrap_or(1900),
        cost: in_range(ctx, f.cost, 0.0, 0.1, &at.field("kosten")),
        load: in_range(ctx, f.load, 0.0, 0.2, &at.field("aufschlag")),
        fee: in_range(ctx, f.fee, 0.0, 0.2, &at.field("gebuehr")),
    };
    let fnl = l.field("fonds");
    AssetModel {
        deposit_premium: in_range(ctx, a.deposit.premium, -0.05, 0.05, &fl.field("aufschlag")),
        deposit_months: span(ctx, &a.deposit.months, &fl),
        deposit_penalty: in_range(
            ctx,
            a.deposit.penalty,
            0.0,
            1.0,
            &fl.field("kuendigung_abschlag"),
        ),
        bond_premium: in_range(ctx, a.bonds.premium, -0.05, 0.05, &bl.field("aufschlag")),
        bond_months: span(ctx, &a.bonds.months, &bl),
        bond_duration_factor: in_range(
            ctx,
            a.bonds.duration_factor,
            0.0,
            5.0,
            &bl.field("dauer_faktor"),
        ),
        bond_spread: in_range(ctx, a.bonds.spread, 0.0, 0.2, &bl.field("spanne")),
        gold_price: {
            let points: Vec<(i32, f64)> = a
                .gold
                .price
                .iter()
                .map(|(&y, &v)| {
                    (
                        y,
                        positive(ctx, v, &gl.field("preis_usd").field(&y.to_string())),
                    )
                })
                .collect();
            if points.is_empty() {
                ctx.error(&gl.field("preis_usd"), messages::time_series_empty());
            }
            wsim_core::time_series::TimeSeries::new(points).unwrap_or_else(|_| {
                wsim_core::time_series::TimeSeries::new(vec![(1900, 1.0)]).expect("valid")
            })
        },
        gold_crisis_beta: in_range(ctx, a.gold.crisis_beta, 0.0, 5.0, &gl.field("krisen_beta")),
        gold_noise: in_range(ctx, a.gold.noise, 0.0, 0.5, &gl.field("rauschen")),
        gold_reversion: in_range(ctx, a.gold.reversion, 0.0, 1.0, &gl.field("rueckkehr")),
        gold_spread: in_range(ctx, a.gold.spread, 0.0, 0.2, &gl.field("spanne")),
        gold_fee: in_range(ctx, a.gold.fee, 0.0, 0.2, &gl.field("gebuehr")),
        funds: [
            fund(ctx, &a.funds.managed, fnl.field("gemanagt")),
            fund(ctx, &a.funds.index, fnl.field("index")),
        ],
        property: a
            .property
            .as_ref()
            .map(|p| property(ctx, p, &l.field("immobilien"))),
        services: a
            .services
            .as_ref()
            .map(|s| services(ctx, s, &l.field("dienste"))),
        stakes: a
            .stakes
            .as_ref()
            .map(|s| stake_rules(ctx, s, &l.field("beteiligung"))),
    }
}

/// Private real estate (PV3): shares and spans within bounds, spans ascending.
fn property(ctx: &mut Ctx, p: &crate::raw::RawProperty, l: &Loc) -> PropertyModel {
    let kl = l.field("arten");
    let kind = |ctx: &mut Ctx, k: &crate::raw::RawPropertyKind, key: &str| {
        let at = kl.field(key);
        PropertyTerms {
            price_years: in_range(
                ctx,
                k.price_years,
                0.1,
                500.0,
                &at.field("preis_jahreslohn"),
            ),
            rent_yield: in_range(ctx, k.yield_, 0.0, 0.3, &at.field("rendite")),
            vacancy: in_range(ctx, k.vacancy, 0.0, 1.0, &at.field("leerstand")),
            upkeep: in_range(ctx, k.upkeep, 0.0, 0.2, &at.field("instandhaltung")),
            decay: in_range(ctx, k.decay, 0.0, 50.0, &at.field("verfall")),
        }
    };
    let span = |ctx: &mut Ctx, s: &crate::raw::RawFloatSpan, lo: f64, hi: f64, at: Loc| {
        let a = in_range(ctx, s.min, lo, hi, &at.field("von"));
        let b = in_range(ctx, s.max, lo, hi, &at.field("bis"));
        if a > b {
            ctx.error(&at, messages::range_inverted("von", "bis"));
        }
        [a, b]
    };
    let ol = l.field("angebot");
    let dl = l.field("schaden");
    let rl = l.field("sanierung");
    let wl = l.field("werk_effekt");
    PropertyModel {
        kinds: [
            kind(ctx, &p.kinds.apartment, "wohnung"),
            kind(ctx, &p.kinds.multi, "mehrfamilienhaus"),
            kind(ctx, &p.kinds.office, "buero"),
            kind(ctx, &p.kinds.hall, "halle"),
            kind(ctx, &p.kinds.land, "grundstueck"),
        ],
        costs: in_range(ctx, p.costs, 0.0, 0.3, &l.field("nebenkosten")),
        equity_min: in_range(ctx, p.equity_min, 0.0, 1.0, &l.field("eigenkapital_min")),
        mortgage_premium: in_range(
            ctx,
            p.mortgage_premium,
            0.0,
            0.2,
            &l.field("hypothek_aufschlag"),
        ),
        repay_year: in_range(ctx, p.repay_year, 0.0, 1.0, &l.field("tilgung_jahr")),
        rent_band: span(ctx, &p.rent_band, 0.3, 3.0, l.field("mietband")),
        vacancy_slope: in_range(
            ctx,
            p.vacancy_slope,
            0.0,
            10.0,
            &l.field("leerstand_steigung"),
        ),
        damage_chance: in_range(ctx, p.damage.chance, 0.0, 1.0, &dl.field("chance")),
        damage_cost: in_range(ctx, p.damage.cost, 0.0, 1.0, &dl.field("kosten")),
        damage_points: in_range(ctx, p.damage.points, 0.0, 100.0, &dl.field("punkte")),
        reno_cost: in_range(ctx, p.renovation.cost, 0.0, 1.0, &rl.field("kosten")),
        reno_points: in_range(ctx, p.renovation.points, 0.0, 100.0, &rl.field("punkte")),
        reno_months: p.renovation.months,
        site_effect: in_range(
            ctx,
            p.site_effect.per_site,
            0.0,
            1.0,
            &wl.field("je_standort"),
        ),
        site_effect_max: in_range(ctx, p.site_effect.most, 0.0, 2.0, &wl.field("hoechstens")),
        sale_discount: in_range(ctx, p.sale_discount, 0.0, 0.5, &l.field("verkauf_abschlag")),
        distress_discount: in_range(
            ctx,
            p.distress_discount,
            0.0,
            0.9,
            &l.field("notverkauf_abschlag"),
        ),
        condition_weight: in_range(
            ctx,
            p.condition_weight,
            0.0,
            1.0,
            &l.field("zustandsgewicht"),
        ),
        offer_months: p.offers.months.max(1),
        offer_spread: span(ctx, &p.offers.spread, -0.9, 3.0, ol.field("spanne")),
        offer_quality: span(ctx, &p.offers.quality, 0.1, 5.0, ol.field("guete")),
        offer_condition: span(ctx, &p.offers.condition, 0.0, 100.0, ol.field("zustand")),
        offer_countries: p.offers.countries_max.max(1),
        company_relief: p.company.as_ref().map_or(0.0, |c| {
            in_range(
                ctx,
                c.relief,
                0.0,
                1.0,
                &l.field("firma").field("entlastung"),
            )
        }),
        develop: p.develop.as_ref().map(|d| {
            let dl = l.field("entwicklung");
            wsim_core::catalog::DevelopTerms {
                cost: positive(ctx, d.cost, &dl.field("kosten")),
                cost_spread: in_range(ctx, d.cost_spread, 0.0, 0.9, &dl.field("kosten_streuung")),
                months: d.months.max(1),
                months_spread: in_range(
                    ctx,
                    d.months_spread,
                    0.0,
                    0.9,
                    &dl.field("monate_streuung"),
                ),
            }
        }),
        company_relief_max: p.company.as_ref().map_or(0.0, |c| {
            in_range(
                ctx,
                c.relief_max,
                0.0,
                1.0,
                &l.field("firma").field("entlastung_max"),
            )
        }),
    }
}

/// Children, education and succession (PV4): shares within 0–1, traits ascending within
/// 0–100, the functions of an education known to the management.
fn legacy(ctx: &mut Ctx, catalog: &Catalog, f: &crate::raw::RawLegacy, l: &Loc) -> LegacyModel {
    let el = l.field("ausbildungen");
    let list = |ctx: &mut Ctx, keys: &[String], name: &str| -> Vec<usize> {
        let at = el.field(name);
        keys.iter()
            .enumerate()
            .filter_map(|(i, key)| {
                let found = catalog.management.function(key);
                if found.is_none() {
                    ctx.error(&at.index(i), messages::function_unknown(key));
                }
                found
            })
            .collect()
    };
    let educations = [
        list(ctx, &f.educations.engineer, "ingenieur"),
        list(ctx, &f.educations.merchant, "kaufmann"),
        list(ctx, &f.educations.lawyer, "jurist"),
        list(ctx, &f.educations.scientist, "naturwissenschaftler"),
    ];
    let tl = l.field("eigenschaften");
    let a = in_range(ctx, f.traits.min, 0.0, 100.0, &tl.field("von"));
    let b = in_range(ctx, f.traits.max, 0.0, 100.0, &tl.field("bis"));
    if a > b {
        ctx.error(&tl, messages::range_inverted("von", "bis"));
    }
    LegacyModel {
        education_from: f.education_from.clamp(10, 25),
        education_cost: in_range(
            ctx,
            f.education_cost,
            0.0,
            50.0,
            &l.field("ausbildung_kosten"),
        ),
        education_points: in_range(
            ctx,
            f.education_points,
            0.0,
            100.0,
            &l.field("ausbildung_punkte"),
        ),
        ambition_points: in_range(
            ctx,
            f.ambition_points,
            0.0,
            100.0,
            &l.field("ehrgeiz_punkte"),
        ),
        talent_points: in_range(ctx, f.talent_points, 0.0, 100.0, &l.field("talent_punkte")),
        traits: [a, b],
        interest_per_month: in_range(
            ctx,
            f.interest_per_month,
            0.0,
            10.0,
            &l.field("interesse_je_monat"),
        ),
        interest_base: in_range(
            ctx,
            f.interest_base,
            0.0,
            100.0,
            &l.field("interesse_basis"),
        ),
        educations,
        partner_share: in_range(ctx, f.partner_share, 0.0, 1.0, &l.field("partner_anteil")),
        firm_relief: in_range(
            ctx,
            f.firm_relief,
            0.0,
            1.0,
            &l.field("firmenbeguenstigung"),
        ),
        dispute_chance: in_range(
            ctx,
            f.dispute_chance,
            0.0,
            1.0,
            &l.field("streit_chance_jahr"),
        ),
        dispute_sale: in_range(ctx, f.dispute_sale, 0.0, 1.0, &l.field("streit_verkauf")),
    }
}

/// Thresholds ascending within 0–1, at most as many price premiums as thresholds (PV5a).
fn stake_rules(ctx: &mut Ctx, s: &crate::raw::RawStakeRules, l: &Loc) -> StakeRules {
    let tl = l.field("schwellen");
    let mut last = 0.0;
    for (i, &t) in s.thresholds.iter().enumerate() {
        in_range(ctx, t, 0.0, 1.0, &tl.index(i));
        if t <= last {
            ctx.error(&tl.index(i), messages::brackets_not_ascending());
        }
        last = t;
    }
    if s.announce_premium.len() > s.thresholds.len() {
        ctx.error(&l.field("meldung_aufschlag"), messages::too_many_premiums());
    }
    let ml = l.field("meldung_aufschlag");
    let announce_premium = s
        .announce_premium
        .iter()
        .enumerate()
        .map(|(i, &p)| in_range(ctx, p, 0.0, 1.0, &ml.index(i)))
        .collect();
    let ol = l.field("pflichtangebot");
    StakeRules {
        thresholds: s.thresholds.clone(),
        announce_premium,
        blocking: in_range(ctx, s.blocking, 0.0, 1.0, &l.field("sperrminoritaet")),
        offer_from: in_range(ctx, s.offer.from, 0.0, 1.0, &ol.field("ab")),
        offer_premium: in_range(ctx, s.offer.premium, 0.0, 1.0, &ol.field("aufschlag")),
        offer_acceptance: in_range(ctx, s.offer.acceptance, 0.0, 1.0, &ol.field("annahme")),
    }
}

/// Property management, tax advisor and asset manager (PV5b): shares within 0–1, the
/// weights of a strategy adding up to 1.
fn services(ctx: &mut Ctx, s: &crate::raw::RawServices, l: &Loc) -> ServiceModel {
    let hl = l.field("hausverwaltung").field("stufen");
    let housing = s
        .housing
        .levels
        .iter()
        .enumerate()
        .map(|(i, x)| {
            let at = hl.index(i);
            [
                in_range(ctx, x.fee, 0.0, 1.0, &at.field("gebuehr")),
                in_range(ctx, x.vacancy, 0.0, 1.0, &at.field("leerstand")),
                in_range(ctx, x.damage, 0.0, 1.0, &at.field("schaden")),
            ]
        })
        .collect();
    let tl = l.field("steuerberater").field("stufen");
    let tax = s
        .tax
        .levels
        .iter()
        .enumerate()
        .map(|(i, x)| {
            let at = tl.index(i);
            [
                in_range(ctx, x.cost, 0.0, 50.0, &at.field("kosten")),
                in_range(ctx, x.relief, 0.0, 1.0, &at.field("entlastung")),
            ]
        })
        .collect();
    let ml = l.field("verwalter");
    let sl = ml.field("strategien");
    let mix = |ctx: &mut Ctx, m: &crate::raw::RawMix, name: &str| -> [f64; 4] {
        let at = sl.field(name);
        let w = [
            in_range(ctx, m.deposit, 0.0, 1.0, &at.field("festgeld")),
            in_range(ctx, m.bonds, 0.0, 1.0, &at.field("anleihen")),
            in_range(ctx, m.funds, 0.0, 1.0, &at.field("fonds")),
            in_range(ctx, m.gold, 0.0, 1.0, &at.field("gold")),
        ];
        if (w.iter().sum::<f64>() - 1.0).abs() > 1e-6 {
            ctx.error(&at, messages::mix_not_one());
        }
        w
    };
    let m = &s.manager;
    ServiceModel {
        housing,
        tax,
        manager_fee: in_range(ctx, m.fee, 0.0, 0.2, &ml.field("gebuehr")),
        dishonest_chance: in_range(
            ctx,
            m.dishonest_chance,
            0.0,
            1.0,
            &ml.field("unehrlich_chance"),
        ),
        embezzlement: in_range(ctx, m.embezzlement, 0.0, 1.0, &ml.field("veruntreuung")),
        reserve_months: m.reserve_months,
        strategies: [
            mix(ctx, &m.strategies.cautious, "vorsichtig"),
            mix(ctx, &m.strategies.balanced, "ausgewogen"),
            mix(ctx, &m.strategies.risky, "riskant"),
        ],
    }
}
