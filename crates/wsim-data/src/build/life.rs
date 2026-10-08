//! Age, retirement and death of the managers of `lebenslauf` (docs/FORMELN.md, PE1).

use wsim_core::catalog::{
    AgeSpan, Catalog, CountrySeries, FoundingModel, LifeModel, LifestyleLevel, PersonModel,
};
use wsim_core::ids::Id;
use wsim_core::state::Lifestyle;

use super::{in_range, positive, provenance, time_series};
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
        savings_rate: country_series(
            ctx,
            catalog,
            &v.savings_rate,
            &l.field("sparzins"),
            (-0.5, 0.5),
        ),
        provenance: provenance(v.approximation, v.source.as_ref()),
        ..PersonModel::default()
    }
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
