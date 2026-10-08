// Views of the simulation core (crates/wsim-core/src/views.rs, wsim-session).
// Amounts in USD, dates as YYYY-MM-DD, names as text keys or catalog keys.

export interface Bereich<T> {
  min: T;
  max: T;
  default: T;
}

export interface Optionen {
  start_year: Bereich<number>;
  countries: string[];
  default_country: string;
  start_forms: { key: string; cost_usd: number }[];
  start_capital_usd: Bereich<number>;
  companies: Bereich<number>;
  difficulties: { key: string; competence: number; aggressiveness: number }[];
  default_difficulty: string;
  research_factor: Bereich<number>;
  /** How many start-ups there are (SU1); empty without start-ups in the data. */
  startups: { key: string; per_year: number }[];
  default_startups: string | null;
  /** How the tariffs change after the data (W3; texts `zoll.dynamik.<key>`). */
  tariffs?: string[];
  default_tariffs?: string | null;
  /** Age of the person at the start and the most children (PE2). */
  person_age?: Bereich<number>;
  children_max?: number;
  /** The least start money in the default country and year (PE3). */
  start_money_min_usd?: number;
  /** Share of the account the founding dialog suggests as capital (PE3). */
  capital_suggestion?: number;
}

export interface NeuesSpiel {
  seed: number;
  start_year: number;
  country: string;
  /** Start money of the person (PE3); with `found_at_start` the company's capital. */
  capital_usd: number;
  /** Only with `found_at_start`: the company founded at the start. */
  start_form?: string;
  company_name?: string;
  found_at_start?: boolean;
  companies: number;
  difficulty: string;
  research_factor: number;
  /** Key of the frequency of start-ups; none for the default. */
  startups?: string | null;
  /** Key of the dynamics of the tariffs; none for the default. */
  tariffs?: string | null;
  /** Whether the historical events act on markets, trade and companies (H1). */
  event_effects?: boolean;
  /** The player as a person (PE2): name (empty: drawn), year of birth, family. */
  person_name?: string;
  birth_year?: number | null;
  married?: boolean;
  children?: number;
}

export interface Anlage {
  facility: string;
  count: number;
  recipe: string | null;
  product: string | null;
  utilization: number;
  ready: string;
  output_per_day: number;
  /** Shut down (M22). */
  mothballed: boolean;
}

export interface Standort {
  index: number;
  country: string;
  /** Text key of the site type, e.g. standorttyp.werk. */
  kind: string;
  deposit: string | null;
  workers: number;
  facilities: Anlage[];
  stock: { product: string; quantity: number; value_usd: number }[];
}

export interface Firma {
  name: string;
  headquarters: string;
  cash_usd: number;
  equity_usd: number;
  loans_usd: number;
  total_assets_usd: number;
  result_year_usd: number;
  result_last_month_usd: number | null;
  revenue_last_month_usd: number | null;
  sites: Standort[];
}

export interface Wettbewerber {
  name: string;
  headquarters: string;
  equity_usd: number;
  real: boolean;
}

export interface Uebersicht {
  date: string;
  game_over: boolean;
  start_year: number;
  market_scale: number;
  /** The main company; null before the person founded one (PE3). */
  company: Firma | null;
  competitors_active: number;
  competitors_bankrupt: number;
  competitors: Wettbewerber[];
  /** What needs the player's attention now, most urgent first. */
  hints: Hinweis[];
  /** Closed months, oldest first. */
  history: Monat[];
  /** Ways to show amounts (M21); null without currency data. */
  money: Geldoptionen | null;
  /** Goals after the introduction, in data order (M23). */
  milestones?: Etappe[];
  /** The player's places among all active companies (M29); null without others. */
  rank?: Rang | null;
  /** Concerns of the player's positions waiting for an answer (MA2). */
  concerns_open?: number;
  /** The player as a person: name and age (PE2), money and place by wealth (PE3). */
  person?: PersonKurz | null;
  /** The companies the person can choose as main company (PE5), the main one marked. */
  companies?: { index: number; name: string; main: boolean }[];
}

/** The person in the header (PE2, PE3). */
export interface PersonKurz {
  name: string;
  age: number;
  cash_usd?: number;
  wealth_usd?: number;
  /** Place of the wealth among the equity of the active companies, and their number. */
  place?: number;
  companies?: number;
  /** The account fell short of the lifestyle. */
  short?: boolean;
}

/** Places by equity and by revenue of the last twelve closed months (M29). */
export interface Platzierung {
  date: string;
  equity: number;
  /** Null without revenue in the last twelve months. */
  revenue: number | null;
  /** Active companies, the player included. */
  companies: number;
}

export interface Rang {
  now: Platzierung;
  /** A year before (start of the same month), null while not recorded. */
  year_before: Platzierung | null;
}

/** A goal of the player (M23); texts `etappe.<key>` and `etappe.<key>.hinweis`. */
export interface Etappe {
  key: string;
  /** Day it was reached (ISO), null while open. */
  reached: string | null;
  /** Way there for measurable goals. */
  progress: { current: number; target: number; unit: "anzahl" | "anteil" | "geld" } | null;
}

/** How amounts are shown: a currency and the factor from game dollars to its units. */
export interface Geldanzeige {
  /** Key of the currency (text `waehrung.<key>`). */
  currency: string;
  symbol: string;
  factor: number;
}

/**
 * The headquarters' currency or the US dollar, each at the purchasing power of the
 * base year or at the prices of the game date (Lastenheft §3.6, §18.2).
 */
export interface Geldoptionen {
  home_base: Geldanzeige;
  home_then: Geldanzeige;
  lead_base: Geldanzeige;
  lead_then: Geldanzeige;
  base_year: number;
}

/** A hint with the place to act on it (site and area of the plant view). */
export interface Hinweis {
  message: Meldung;
  site: number | null;
  area: string | null;
}

export interface Monat {
  /** First day of the month. */
  month: string;
  revenue_usd: number;
  result_usd: number;
  /** Cash at the end of the month. */
  cash_usd: number;
}

/** Revenue and gross margin of a product. */
export interface ProduktErgebnis {
  product: string;
  revenue_usd: number;
  margin_usd: number;
}

export type Parameter =
  | { type: "text"; value: string }
  | { type: "integer"; value: number }
  | { type: "number"; value: number }
  | { type: "money"; value: number }
  | { type: "date"; value: string }
  | { type: "country"; value: string }
  | { type: "text_key"; value: string }
  | { type: "countries"; value: string[] }
  | { type: "text_keys"; value: string[] };

export type MeldungsArt =
  "info" | "success" | "warning" | "crisis" | "world_event" | "stock" | "error";

/** Section of the round report. */
export type MeldungsGruppe =
  "erfolg" | "welt" | "wettbewerb" | "warnung" | "forschung" | "allgemein";

export interface Meldung {
  kind: MeldungsArt;
  group: MeldungsGruppe;
  key: string;
  params: Record<string, Parameter>;
  target: string | null;
}

export interface Periode {
  revenue_usd: number;
  costs_usd: number;
  /** Goods made but not sold yet, at production cost: revenue − costs + this = result. */
  inventory_change_usd: number;
  result_usd: number;
  /** [text key of the cost type, amount; costs negative] */
  lines: [string, number][];
}

export interface Forschungsprojekt {
  technology: string;
  points: number;
  needed: number;
}

export interface Rundenbericht {
  from: string;
  to: string;
  days: number;
  cash_before_usd: number;
  cash_after_usd: number;
  equity_change_usd: number;
  period: Periode;
  previous: Periode | null;
  research: Forschungsprojekt[];
  messages: Meldung[];
  game_over: boolean;
  /** Revenue and gross margin by product in this round. */
  products: ProduktErgebnis[];
  hints: Hinweis[];
  /** Rounds the report covers (M26). */
  rounds?: number;
  /** Why several rounds stopped: `jahresende`, `warnung`, `weltereignis`, `ein_jahr`,
   * `spielende`; null for a single round. */
  stop?: string | null;
}

export interface Spielstand {
  name: string;
  date: string;
  company: string;
}

export type Rundenlaenge = "tag" | "woche" | "monat" | "quartal";

/** How far "end round" goes (M26): one round, rounds up to the year's end, or up to the
 * next warning or world event (at most a year). */
export type Weiterlaufen = "runde" | "jahresende" | "meldung";

/** Which new concerns halt a run of rounds (MA2). */
export type Anhalten = "alle" | "wichtige" | "nie";

export interface Fortschritt {
  done: number;
  total: number;
}

export interface KartenLand {
  key: string;
  lat: number;
  lon: number;
  population: number;
  gdp_per_capita_usd: number;
  wage_usd: number;
  development: number;
  grid_share: number;
  own_sites: number;
  other_sites: number;
  /** Average import tariff (share of the value, W3). */
  tariff?: number;
}

export interface Lagerstaette {
  key: string;
  country: string;
  resource: string;
  max_output_per_year: number;
  concessions: number;
  free_concessions: number;
  undiscovered: boolean;
}

export interface Weltkarte {
  date: string;
  countries: KartenLand[];
  deposits: Lagerstaette[];
  resources: string[];
  /** Traded products, for the layer of sales chances. */
  products: string[];
}

export interface Landdetail {
  key: string;
  /** Countries merged into this region (ISO codes, texts `teilland.<ISO>`), else empty. */
  members: string[];
  date: string;
  population: number;
  gdp_per_capita_usd: number;
  price_level: number;
  gini: number;
  income_quintiles_usd: number[];
  labor_force: number;
  labor: { group: string; persons: number; available: number; wage_usd: number }[];
  electricity_price_usd_mwh: number;
  grid_share: number;
  corporate_tax: number;
  dividend_tax: number;
  development: number;
  infrastructure: number[];
  stability: number;
  deposits: Lagerstaette[];
  companies: { name: string; sites: number; own: boolean }[];
  markets: {
    product: string;
    price_usd: number;
    demand_last_month: number;
    sold_last_month: number;
  }[];
  /** The country's currencies from their first month ("1923-12") on, oldest first. */
  currencies: { from: string; currency: string; symbol: string; current: boolean }[];
  /** Units of the current currency per US dollar of the game date (null: the dollar). */
  currency_per_usd: number | null;
  /** Commercial land and free plots (null without plots). */
  land: Gewerbeflaeche | null;
  /** Import tariffs, trade zones and embargoes (W3; null without tariffs). */
  tariffs?: Zoelle | null;
  /** Effects of historical events acting on the country this month (H1). */
  events?: Meldung[];
  /** Regulations in force (H2) and the CO₂ price in USD per t. */
  regulations?: Meldung[];
  co2_price_usd?: number;
}

/** Import tariffs of a country (W3). */
export interface Zoelle {
  /** Average import tariff (share of the value). */
  average: number;
  /** Tariff per goods group (key, share), highest first. */
  groups: [string, number][];
  /** Trade zones (texts `zoll.zone.<key>`). */
  zones: string[];
  /** Countries under an embargo with this one. */
  embargoes: string[];
}

export interface Ursache {
  key: string;
  detail: string | null;
}

/** A size of a facility to build (M36; text `anlagengroesse.<key>`). */
export interface Anlagengroesse {
  /** For `BuildFacility`, e.g. `Large`. */
  size: string;
  key: string;
  /** Capacity as a multiple of the data size. */
  capacity: number;
  investment_usd: number;
  build_days: number;
  area_ha: number;
  /** Labor per unit made as a multiple of the data size. */
  labor_per_unit: number;
}

export interface AnlageDetail {
  index: number;
  facility: string;
  count: number;
  /** Size of the units (text `anlagengroesse.<key>`) and their capacity factor (M36). */
  size: string;
  capacity: number;
  recipe: string | null;
  product: string | null;
  utilization: number;
  automation: number;
  automation_max: number;
  ready: string;
  planned_per_day: number;
  made_per_day: number;
  cause: Ursache | null;
  /** Condition of the facilities (1 = new). */
  condition: number;
  /** [product, quantity per day at the planned utilization] */
  inputs_per_day: [string, number][];
  /** Development level of the product made here (M37). */
  development_level: number;
  /** Running, shut down or starting up again (M22). */
  operation: "laeuft" | "stillgelegt" | "wiederanlauf";
  /** Shut down since, or producing again from. */
  operation_date: string | null;
  /** Book value of all units and what selling them brings now. */
  book_value_usd: number;
  sale_value_usd: number;
  restart_cost_usd: number;
  restart_days: number;
  /** Maintenance per month while running and while shut down. */
  maintenance_month_usd: number;
  maintenance_mothballed_month_usd: number;
  /** Emissions per day at the planned utilization (H2). */
  co2_t_per_day?: number;
  pollutant_kg_per_day?: number;
  /** Retrofit level, the level the country requires, the cost of the next level. */
  retrofit?: number;
  retrofit_required?: number;
  retrofit_cost_usd?: number | null;
}

export interface Angebot {
  product: string;
  mode: "markt" | "fest";
  price_usd: number;
  floor_usd: number;
  markup: number;
  keep: number;
  sold_month: number;
  sold_last_month: number;
  stock: number;
  market_price_usd: number;
  reference_usd: number;
  /** Expected cost per unit (made here, else the value of the stock). */
  unit_cost_usd: number | null;
  /** (price − unit cost) / price */
  margin: number | null;
  /** The site uses the product itself. */
  used_here: boolean;
  /** The company's own name for the product (M42). */
  product_name?: string | null;
}

export interface Einkauf {
  product: string;
  target: number;
  max_price_usd: number;
  min_quality: number;
  bought_month: number;
  bought_last_month: number;
}

export interface Versorgung {
  product: string;
  need_per_day: number;
  stock: number;
  days: number | null;
  own: boolean;
  ordered: boolean;
  market_price_usd: number;
}

export interface Personal {
  group: string;
  needed: number;
  employed: number;
  free_in_country: number;
  wage_usd: number;
  country_wage_usd: number;
}

export interface Stueckkosten {
  product: string;
  output_per_day: number;
  material_usd: number;
  labor_usd: number;
  energy_usd: number;
  overhead_usd: number;
  rent_usd: number;
  facility_usd: number;
  total_usd: number;
  variable_usd: number;
}

export interface StandortErgebnis {
  /** First day of the month. */
  month: string;
  revenue_usd: number;
  /** Everything but revenue by cost type (text keys), costs negative. */
  lines: { key: string; usd: number }[];
  result_usd: number;
  products: { product: string; revenue_usd: number; margin_usd: number }[];
}

export interface StandortDetail {
  index: number;
  country: string;
  kind: string;
  kind_text: string;
  deposit: string | null;
  deposit_ready: string | null;
  free_deposits: {
    key: string;
    resource: string;
    cost_usd: number;
    days: number;
    output_per_year: number;
  }[];
  workers: number;
  slots: AnlageDetail[];
  stock: { product: string; quantity: number; value_usd: number }[];
  offers: Angebot[];
  orders: Einkauf[];
  inputs: Versorgung[];
  research: string | null;
  /** Product a research center develops (M37). */
  development: string | null;
  wage_premium: number;
  wage_premium_max: number;
  /** Training (W1): level reached and target, 0–1, and where the target comes from. */
  training?: number;
  training_target?: number;
  training_source?: "standort" | "strategie" | "keine";
  training_cost_per_day_usd?: number;
  /** Effect of the level: share of labor hours saved, quality points. */
  training_labor_saving?: number;
  training_quality?: number;
  rival_premium_max: number;
  staff: Personal[];
  wage_cost_per_day_usd: number;
  unit_costs: Stueckkosten[];
  last_month: StandortErgebnis | null;
  /** Market and reference price in the site's country, by product. */
  prices: Record<string, { market_usd: number; reference_usd: number }>;
  /** The plot the site stands on (none for extraction sites). */
  plot: StandortGrundstueck | null;
}

/** The plot of a site: location (text `lage.<key>`), area and land in use (ha). */
export interface StandortGrundstueck {
  location: string;
  area_ha: number;
  used_ha: number;
  owned: boolean;
  value_usd: number;
  /** Rent per year while leased. */
  rent_usd_year: number | null;
  /** Units of each facility of the site's type that still fit on the plot, by size
   * (smallest first). */
  fits: Record<string, number[]>;
}

/** A free plot of a country (texts `lage.<location>`, `grundstuecksklasse.<class>`). */
export interface Grundstueck {
  id: number;
  location: string;
  class: string;
  area_ha: number;
  value_usd: number;
  rent_usd_year: number;
}

/** Commercial land of a country with its free plots. */
export interface Gewerbeflaeche {
  area_ha: number;
  occupied_ha: number;
  /** Land price per ha by location key. */
  price_per_ha_usd: [string, number][];
  /** Size classes, the smallest first (texts `grundstuecksklasse.<key>`). */
  classes: string[];
  free: Grundstueck[];
}

export interface Produktion {
  date: string;
  cash_usd: number;
  sites: StandortDetail[];
  site_types: { kind: string; kind_text: string; cost_usd: number }[];
  facilities: {
    key: string;
    site_type: string;
    investment_usd: number;
    build_days: number;
    runs_per_day: number;
    automation_max: number;
    recipes: string[];
    /** Land per unit with ways and offices (ha; 0 without plots). */
    area_ha: number;
    /** Sizes to build (M36), the smallest first. */
    sizes: Anlagengroesse[];
  }[];
  recipes: {
    key: string;
    facility: string;
    product: string;
    output: number;
    output_per_day: number;
    inputs_per_day: [string, number][];
    duration_days: number;
    extraction: boolean;
    inputs: [string, number][];
    labor_hours: [string, number][];
    energy_mwh: number;
  }[];
  products: string[];
  /** Unit key of each product (einheit.<key>). */
  units: Record<string, string>;
}

export interface Markt {
  date: string;
  country: string;
  lines: {
    product: string;
    price_usd: number;
    reference_usd: number;
    state_price_usd: number | null;
    demand_last_month: number;
    sold_last_month: number;
    imported_last_month: number;
    exported_last_month: number;
    sellers: number;
    own_price_usd: number | null;
    own_sold_last_month: number;
    group: string;
    own_share: number;
    leader: string | null;
    leader_share: number;
    price_last_month_usd: number | null;
    /** Share of consumers' and government demand served last month. */
    supply: number | null;
    unmet_last_month: number;
    /** mangel, teuer, wenige_anbieter */
    chances: string[];
  }[];
  brands: {
    group: string;
    own_awareness: number;
    top: string | null;
    top_awareness: number;
    budget_usd: number;
  }[];
  medium: string | null;
  reach_usd: number;
  units: Record<string, string>;
}

export interface ProduktMarkt {
  date: string;
  country: string;
  product: string;
  unit: string;
  group: string;
  price_usd: number;
  reference_usd: number;
  price_last_month_usd: number | null;
  state_price_usd: number | null;
  import_price_usd: number | null;
  /** Consumer demand per month by income fifth, poorest first. */
  consumers_per_month: number[];
  state_per_month: number;
  demand_last_month: number;
  outside_demand_last_month: number;
  sold_last_month: number;
  unmet_last_month: number;
  imported_last_month: number;
  exported_last_month: number;
  supply: number | null;
  sellers: {
    company: string;
    own: boolean;
    real: boolean;
    price_usd: number;
    sold_last_month: number;
    share: number;
    /** Development level of the seller for the product (M37). */
    level: number;
    /** The seller's own name for the product (M42). */
    product_name?: string | null;
  }[];
  own_awareness: number;
  chances: string[];
  /** The last closed months, oldest first (M24). */
  history?: Marktmonat[];
  /** How the market price comes about (M27). */
  price_parts?: {
    world_reference_usd: number;
    price_level: number;
    level_share: number;
    level_factor: number;
    reference_usd: number;
    price_usd: number;
    /** Market price / reference price in the country. */
    situation: number;
    supply: number | null;
    unmet_share: number;
    import_share: number;
    sellers: number;
    price_max_factor: number;
  } | null;
  /** How the consumer demand comes about (M27); null without consumer demand. */
  demand_parts?: {
    kind: "verbrauch" | "gebrauch" | "ergaenzung";
    complement_of: string | null;
    population: number;
    gdp_per_capita_usd: number;
    reference_usd: number;
    price_usd: number;
    grid: number | null;
    season: number;
    fifths: {
      income_usd: number;
      propensity: number;
      base: number;
      owned: number | null;
      per_head_year: number;
    }[];
  } | null;
  /** Companies give the product their own names (M42). */
  nameable?: boolean;
  /** The player's name for the product. */
  own_name?: string | null;
  /** Three free names from the name parts of the data. */
  name_suggestions?: string[];
}

/** One closed month of a market (M24). */
export interface Marktmonat {
  /** First day of the month. */
  month: string;
  /** Average price paid; null when nothing was sold. */
  price_usd: number | null;
  sold: number;
  /** The player's share; null while the player never sold here. */
  own_share: number | null;
}

/** Production chains of the end products (M25); prices and costs in the home country. */
export interface Ketten {
  country: string;
  /** Tops of the chains in data order: products nothing else is made of. */
  roots: string[];
  products: KettenProdukt[];
}

export interface KettenProdukt {
  product: string;
  kind: "rohstoff" | "halbzeug" | "komponente" | "endprodukt" | "energie";
  unit: string;
  reference_usd: number;
  price_usd: number;
  /** How it is made; null for state-market goods, electricity and goods not invented. */
  recipe: {
    key: string;
    facility: string;
    extraction: boolean;
    /** Technologies the player still needs. */
    missing: string[];
    /** Inputs per unit of output. */
    inputs: [string, number][];
    unit_cost_usd: number;
    margin: number | null;
  } | null;
  state_market: boolean;
  makes: boolean;
  buys: boolean;
  sells: boolean;
  /** Why own plants made less: `vorprodukt` (with the input), `arbeitskraefte`, … */
  stuck: [string, string | null][];
}

export interface WeltMarkt {
  date: string;
  product: string;
  unit: string;
  countries: {
    country: string;
    price_usd: number;
    reference_usd: number;
    demand_last_month: number;
    unmet_last_month: number;
    supply: number | null;
    sellers: number;
    own_sellers: number;
  }[];
}

/** What a development level changes (M37): quality points, shares saved per run. */
export interface Entwicklungswirkung {
  quality: number;
  labor_saved: number;
  inputs_saved: number;
}

/** A product the player may develop or has developed (M37). */
export interface Weiterentwicklung {
  product: string;
  /** The player makes it in a facility of its own. */
  own: boolean;
  level: number;
  own_level: number;
  public_level: number;
  best_rival: number;
  next: number | null;
  needed: number | null;
  points: number;
  points_per_day: number;
  days: number | null;
  one_lab: {
    country: string;
    researchers: number;
    points_per_day: number;
    days: number;
    cost_usd: number;
  } | null;
  field: string;
  effect: Entwicklungswirkung;
  next_effect: Entwicklungswirkung | null;
  sites: number[];
}

export interface Forschung {
  date: string;
  technologies: {
    key: string;
    field: string;
    invention_year: number;
    prerequisites: string[];
    known: boolean;
    researchable: boolean;
    needed: number | null;
    factor: number | null;
    points: number;
    sites: number[];
    opens: string[];
    /** bekannt, in_arbeit, erforschbar, gesperrt */
    status: "bekannt" | "in_arbeit" | "erforschbar" | "gesperrt";
    leads_to: string[];
    remaining: number | null;
    points_per_day: number;
    days: number | null;
    one_lab: {
      country: string;
      researchers: number;
      points_per_day: number;
      days: number;
      cost_usd: number;
    } | null;
    facilities: {
      key: string;
      site_type: string;
      kind_text: string;
      investment_usd: number;
      build_days: number;
      recipes: string[];
    }[];
    recipes: {
      key: string;
      facility: string;
      facility_missing: string | null;
      product: string;
      output_per_day: number;
      inputs_per_day: [string, number][];
    }[];
    products: string[];
  }[];
  /** Products to develop (M37) and the highest level. */
  developments: Weiterentwicklung[];
  development_levels: number;
  development_per_level: Entwicklungswirkung;
  centers: {
    site: number;
    country: string;
    researchers: number;
    project: string | null;
    /** Product developed instead of a technology (M37). */
    development: string | null;
    ready: boolean;
    building_until: string | null;
    labs: { slot: number; count: number; utilization: number; ready: string }[];
  }[];
  laboratory: string | null;
  laboratory_usd: number;
  laboratory_posts: number;
  units: Record<string, string>;
}

export type Technologie = Forschung["technologies"][number];

export interface Abrechnung {
  lines: [string, number][];
  result_usd: number;
  cash_flow_usd: [number, number, number];
}

/** A bond of the player's company (K2). */
export interface Anleihe {
  index: number;
  principal_usd: number;
  coupon: number;
  issued: string;
  maturity: string;
  grade: string;
  redeem_usd: number;
}

export interface Anleihen {
  enabled: boolean;
  grade: string | null;
  debt_ratio: number;
  coverage: number | null;
  has_figures: boolean;
  equity_usd: number;
  equity_min_usd: number;
  volume_min_usd: number;
  term_min_years: number;
  term_max_years: number;
  cost_share: number;
  redeem_premium: number;
  max_usd: number;
  quotes: { amount_usd: number; grade: string; coupon: number }[];
  bonds: Anleihe[];
}

export interface Finanzen {
  date: string;
  assets: [string, number][];
  claims: [string, number][];
  total_usd: number;
  year: Abrechnung;
  last_year: Abrechnung | null;
  last_month: Abrechnung | null;
  loans: {
    index: number;
    principal_usd: number;
    balance_usd: number;
    rate: number;
    start: string;
    months: number;
    instalment_usd: number;
  }[];
  bonds: Anleihen;
  credit_limit_usd: number;
  overdraft_limit_usd: number;
  loan_rate: number;
  max_term_years: number;
  loss_carryforward_usd: number;
  history: Monat[];
  centers_last_month: Zentren | null;
  centers_year: Zentren | null;
}

/** Results by site and product of a period, and the company's own items. */
export interface Zentren {
  sites: {
    site: number;
    kind_text: string;
    country: string;
    revenue_usd: number;
    result_usd: number;
  }[];
  products: ProduktErgebnis[];
  company_usd: number;
}

/** What a site is worth (M30, docs/FORMELN.md). */
export interface Standortwert {
  book_usd: number;
  inventory_usd: number;
  /** Result of the last twelve months; null with too few months. */
  result_year_usd: number | null;
  earnings_value_usd: number;
  liquidation_usd: number;
  under_construction_usd: number;
  /** A bought plot at today's value (M35); 0 for a leased one. */
  land_usd: number;
  /** Base value G = max(earnings value, liquidation) + under construction + stocks +
   * bought plot; for an area the sum over its sites and the brand. */
  base_usd: number;
  earnings_years: number;
  /** Brand value W of an area (advertising for the same awareness); 0 for a site. */
  brand_usd: number;
}

/** The object of an offer. */
export interface Gegenstandssicht {
  kind: "standort" | "lizenz" | "bereich";
  /** Goods group of an area. */
  group: string | null;
  /** Number of sites of an area. */
  site_count: number;
  site: number | null;
  /** Text key of the site type. */
  site_type: string | null;
  country: string | null;
  products: string[];
  technology: string | null;
}

export interface Angebot {
  id: number;
  /** `kaeufer`: the player buys; `verkaeufer`: the player sells. */
  role: "kaeufer" | "verkaeufer";
  company: string;
  company_index: number;
  object: Gegenstandssicht;
  price_usd: number;
  /** The seller named the price (counter-offer). */
  counter: boolean;
  date: string;
  deadline: string;
  status: "offen" | "angenommen" | "abgelehnt" | "abgelaufen" | "zurueckgezogen";
  closed: string | null;
  /** The player answers now. */
  answer: boolean;
  can_counter: boolean;
  can_withdraw: boolean;
  value: Standortwert | null;
  license_value_usd: number | null;
}

export interface Angebote {
  offers: Angebot[];
}

export interface Firmenzeile {
  index: number;
  name: string;
  headquarters: string;
  equity_usd: number;
  revenue_year_usd: number;
  sites: number;
  real: boolean;
  player: boolean;
  /** Last day of the auction of an insolvent company's sites (M38). */
  auction_until: string | null;
  /** Its central departments (`abteilung.<key>`) and their employees (ZA4). */
  departments: string[];
  central_staff: number;
  /** A move of its headquarters under way: the new country and the first day there. */
  moving_to: string | null;
  moving_until: string | null;
}

export interface Firmen {
  companies: Firmenzeile[];
}

export interface FremderStandort {
  site: number;
  site_type: string;
  country: string;
  /** Facility, number of units and size (M36). */
  facilities: [string, number, string][];
  products: string[];
  workers: number;
  value: Standortwert;
  /** What the same site would cost to build today. */
  new_build_usd: number;
  /** The owner needs the site itself and sells it only for what a new one costs. */
  needed: boolean;
  /** Why no offer is possible now: `zu_jung`, `angebot_offen`, `gesperrt`. */
  blocked: "zu_jung" | "angebot_offen" | "gesperrt" | null;
  blocked_until: string | null;
  open_offer: number | null;
  /** Lowest bid while the site is auctioned (M38). */
  min_bid_usd: number | null;
}

export interface Lizenzmoeglichkeit {
  technology: string;
  value_usd: number;
  open_offer: number | null;
  blocked_until: string | null;
}

/** All sites of a company in a goods group with its brand (M31). */
export interface Geschaeftsbereich {
  group: string;
  /** Its sites, as in `sites` of the company. */
  sites: number[];
  /** Awareness of the brand per country, highest first. */
  brand: [string, number][];
  value: Standortwert;
  new_build_usd: number;
  /** Why no offer is possible now, as for sites. */
  blocked: "zu_jung" | "angebot_offen" | "gesperrt" | null;
  blocked_until: string | null;
  open_offer: number | null;
}

export interface Firmendetail {
  company: Firmenzeile;
  /** The names the company gave its products (M42). */
  products?: { product: string; name: string }[];
  sites: FremderStandort[];
  areas: Geschaeftsbereich[];
  licenses: Lizenzmoeglichkeit[];
  cash_usd: number;
  min_age_months: number;
  /** The company's managers with the player's offers to poach them (N46). */
  managers?: FremderManager[];
  /** The player's free positions the offers name. */
  free_positions?: FreieStelle[];
  /** The company's value and owners with the prices they ask (PE5). */
  value_usd?: number;
  owners?: Anteilseigner[];
  /** The person's private account. */
  private_cash_usd?: number;
}

/** Who holds shares, as commands name it. */
export type Halter = "Player" | "Private" | "Investors" | { Company: number };

/** An owner of a company and what its shares cost (PE5). */
export interface Anteilseigner {
  holder: Halter;
  kind: "du" | "gruender" | "anleger" | "firma";
  name: string | null;
  share: number;
  /** It sells to the person and the person's companies. */
  sells: boolean;
  /** What it asks for all of its share. */
  ask_usd: number;
}

/** Where a unit is, for lists outside the chart (same fields as the chart's units). */
export interface Ort {
  unit: string;
  site: number | null;
  kind_text: string;
  country: string | null;
  continent: string | null;
}

/** A free position of the player that an offer can name. */
export interface FreieStelle {
  place: Ort;
  role: string;
}

/** A manager of another company as the player sees him (N46). */
export interface FremderManager {
  manager: Manager;
  place: Ort;
  role: string;
  /** Offers for the player's free positions, in the order of `free_positions`. */
  options: { salary_usd: number; over_cash: boolean }[];
  /** Not to be courted again before this day. */
  courted_until: string | null;
  /** An offer of some company is open: `own` for the player's, else the company. */
  offer: string | null;
}

/** A skill as the player sees it (MA1): a level, never the number. */
export interface Faehigkeit {
  /** `fach.<bereich>`, `erkennen`, `urteil`, `fuehrung`, `risiko`, `fragefreude`. */
  key: string;
  /** 0 (weak) … 4 (outstanding). */
  level: number;
}

export interface Manager {
  id: number;
  name: string;
  /** Home country (ISO). */
  home: string;
  continent: string;
  /** Function the manager is best at. */
  focus: string;
  skills: Faehigkeit[];
  /** Estimates of targets judged so far, and the hit rate (null before the first; ZA3). */
  judged: number;
  hit_rate: number | null;
  /** Age today, the age at which he plans to retire and the day (PE1). */
  age?: number | null;
  retires_at?: number | null;
  retirement?: string | null;
  /** A child of the person (PE2). */
  family?: boolean;
}

/** The successor waiting for a position until its holder retires (PE1). */
export interface Nachfolger {
  manager: Manager;
  salary_usd: number;
  since: string;
}

/** A manager who retired or died in the company's service (PE1). */
export interface EhemaligerManager {
  name: string;
  /** Age when he left. */
  age: number | null;
  position: AnliegenStelle;
  since: string;
  until: string;
  judged: number;
  reason: "ruhestand" | "tod";
}

/** Another company's offer to a manager of the player (MA6). */
export interface Abwerbeangebot {
  company: string;
  salary_usd: number;
  /** Last day it stands. */
  until: string;
}

export interface Stelleninhaber {
  manager: Manager;
  salary_usd: number;
  since: string;
  /** What a dismissal costs now. */
  severance_usd: number;
  /** What he would ask for the position today (MA6). */
  market_usd: number;
  /** 0 unhappy, 1 mixed, 2 happy (MA6). */
  satisfaction: number;
  offer: Abwerbeangebot | null;
  /** His successor, appointed before he retires (PE1). */
  successor?: Nachfolger | null;
}

/** What a position may spend without asking (MA2). */
export interface Budget {
  /** Shares of the reference per decision and per year. */
  shares: [number, number];
  /** The shares of its level and role. */
  defaults: [number, number];
  /** Set by the player. */
  custom: boolean;
  /** The site's revenue in the last twelve closed months, without revenue its costs. */
  base_usd: number;
  per_decision_usd: number;
  per_year_usd: number;
  spent_usd: number;
}

/** A decision a position took itself. */
export interface Stellenentscheidung {
  date: string;
  /** Topic (text `thema.<topic>`). */
  topic: string;
  /** Kind of the option (text `option.<kind>`). */
  kind: string;
  product: string | null;
  amount_usd: number;
  /** The position's estimate of the effect on a year's result. */
  effect_usd: number | null;
}

/** A topic the position does not ask about. */
export interface StillesThema {
  /** For the command `AskAgain`. */
  id: string;
  /** Text `thema.<topic>`. */
  topic: string;
  /** Declined: quiet until then; null: not to be asked again. */
  until: string | null;
}

export interface Stelle {
  /** `leitung` or the key of the function of a specialist position. */
  role: string;
  /** Topics (`thema.<key>`) the position takes care of now. */
  topics: string[];
  holder: Stelleninhaber | null;
  /** Only for a filled position (MA2). */
  budget?: Budget | null;
  /** Its latest own decisions, the newest first. */
  log?: Stellenentscheidung[];
  quiet?: StillesThema[];
  /** Its concerns waiting for an answer. */
  open_concerns?: number;
  /** Text key of what it does besides topics (MA5: the personnel member of the board). */
  effect?: string | null;
  /** For heads: whether it fills the free positions of its unit itself (MA6). */
  hires?: boolean | null;
}

/** A country the headquarters could move to (ZA1). */
export interface Sitzland {
  country: string;
  /** Profit tax (share). */
  tax: number;
  /** Yearly wage of the managers' salary group: what the board's salaries follow. */
  wage_usd: number;
  /** Keys of its cities, the most populous first (W2; texts `stadt.<ISO>.<key>`). */
  cities: string[];
  /** The city a move without a choice goes to: the capital. */
  default_city: string | null;
}

/** A city of the country of the headquarters (W2). */
export interface Stadt {
  key: string;
  capital: boolean;
  /** Inhabitants this year. */
  population: number;
  /** Academics open to the central departments of all companies there. */
  academics: number;
  /** Posts all companies with their headquarters there want. */
  wanted: number;
  /** Office costs per employee and month (USD). */
  office_per_employee_usd: number;
  here: boolean;
}

/** The headquarters and the central departments (ZA1–ZA3). */
export interface Zentrale {
  country: string;
  /** The city of the headquarters (W2); null for a country without cities. */
  city: string | null;
  tax: number;
  wage_usd: number;
  /** A move under way: the new seat from the first month start on or after `until`. */
  relocation: { country: string; city: string | null; until: string } | null;
  move_cost_usd: number;
  move_months: number;
  moving_share: number;
  /** A move to another city of the country (W2). */
  city_move_cost_usd: number;
  city_move_months: number;
  /** The cities of the country of the headquarters (W2). */
  cities: Stadt[];
  countries: Sitzland[];
  /** The departments of the data, in their order (ZA2). */
  departments: Abteilung[];
  employees: number;
  /** Salaries and offices of all departments a month. */
  monthly_cost_usd: number;
  participations: Beteiligungen;
}

/** A central department (ZA2). */
export interface Abteilung {
  /** As `StaffDepartment` names it, e.g. `Finance`. */
  kind: string;
  /** Text key `abteilung.<key>`. */
  key: string;
  /** The board's function whose member heads it (`bereich.<function>`). */
  function: string;
  /** Posts wanted. */
  staff: number;
  /** Posts filled from the academics of the city (W2). */
  staffed: number;
  /** The head's name; null while the position is vacant. */
  head: string | null;
  /** The head's expertise as the player sees it (1–5). */
  head_level: number | null;
  /** The head's hit rate and judged estimates (strategy, legal; ZA3). */
  head_hit_rate: number | null;
  head_judged: number;
  /** Cases per month: employees · cases each. */
  capacity: number;
  cases_each: number;
  workload: number;
  /** Share of the workload covered (0–1). */
  coverage: number;
  /** With head and employees. */
  working: boolean;
  /** Its effect at full quality and coverage. */
  effect: number;
  /** Countries observed (strategy), technologies checked (legal); 0 for the others. */
  reach: number;
  monthly_cost_usd: number;
  cost_per_employee_usd: number;
  release_limit_usd: number | null;
}

/** The policy „Beteiligungen“ (ZA2). */
export interface Beteiligungen {
  /** Per year; null: no limit. */
  budget_usd: number | null;
  risk: number;
  /** Bought this year: takeovers and licences. */
  spent_usd: number;
  open_bids_usd: number;
  left_usd: number | null;
  limits: { kind: string; key: string; limit_usd: number | null }[];
}

/** The positions of a unit: a site, a country, a continent (MA1, MA3) or the board (MA5). */
export interface EinheitOrganisation {
  level: "standort" | "land" | "kontinent" | "vorstand";
  /** The site's number (sites). */
  site: number | null;
  /** The country (sites and countries). */
  country: string | null;
  /** The continent (continents). */
  continent: string | null;
  /** Text key of the site type, else `ebene.land`, `ebene.kontinent` or `ebene.vorstand`. */
  kind_text: string;
  /**
   * The unit for the market of managers: `standort:3`, `land:DEU`, `kontinent:europa`,
   * `vorstand`.
   */
  key: string;
  positions: Stelle[];
  /** Next check of the positions; null without a manager there. */
  next_check: string | null;
  /** Topics nobody takes care of here: the player decides them. */
  own_topics: string[];
}

/** A type of position budget rules apply to (MA3). */
export interface Stellentyp {
  level: "standort" | "land" | "kontinent" | "vorstand";
  /** The site type as commands name it (`Factory` …), for sites. */
  site_type: string | null;
  kind_text: string;
  /** `leitung` or the function. */
  role: string;
}

/** A budget rule of the player (MA3). */
export interface Budgetvorgabe {
  kind: Stellentyp;
  scope: "firma" | "kontinent" | "land";
  scope_key: string | null;
  shares: [number, number];
}

export interface Organisation {
  /** False without manager data. */
  enabled: boolean;
  /** The board: CEO and members (MA5); null without a site. */
  board?: EinheitOrganisation | null;
  continents: {
    continent: string;
    /** The positions of the continent (MA3). */
    unit: EinheitOrganisation | null;
    countries: {
      country: string;
      /** The positions of the country (MA3). */
      unit: EinheitOrganisation | null;
      sites: EinheitOrganisation[];
    }[];
  }[];
  managers: number;
  /** Salaries of all managers per year. */
  salaries_usd: number;
  /** Days between the checks of a site's positions. */
  check_days: number;
  severance_months: number;
  /** Free candidates in all markets. */
  candidates: number;
  /** Least budget of a position in its yearly salaries: per decision and per year. */
  budget_floor?: [number, number];
  /** The player's budget rules (MA3). */
  rules?: Budgetvorgabe[];
  /** Types of positions the company has, for new rules. */
  kinds?: Stellentyp[];
  /** The headquarters and the central departments (ZA1–ZA3). */
  central?: Zentrale;
  /** Managers who retired or died in its service, the latest first (PE1). */
  former?: EhemaligerManager[];
  /** Holders who retire within two years, the earliest first (PE1). */
  retiring?: {
    name: string;
    age: number | null;
    position: AnliegenStelle;
    retirement: string;
    successor: string | null;
  }[];
}

export interface Kandidat {
  manager: Manager;
  /** Salary per year for the position. */
  demand_usd: number;
  /** The salary for a year is more than the company's cash (N37). */
  over_cash?: boolean;
  /** An own manager's position now. */
  current: { unit: string; site: number | null; role: string } | null;
}

/** An option of a concern as the position assessed it (MA2). */
export interface AnliegenOption {
  /** Kind of the option (text `option.<kind>`). */
  kind: string;
  /** What the option does, step by step. */
  steps: Meldung[];
  /** Counted against a budget. */
  amount_usd: number;
  /** Forecast of the effect on a year's result as a range; null without an estimate. */
  forecast_usd: [number, number] | null;
  /** One-off effect on the result. */
  once_usd: number;
}

export type AnliegenStatus =
  | "offen"
  | "gewaehlt"
  | "delegiert"
  | "nicht_mehr_fragen"
  | "abgelehnt"
  | "abgelaufen"
  | "erledigt";

/** A question of a position to the player (MA2). */
/** A position in concerns: its role and unit (MA3). */
export interface AnliegenStelle {
  /** `leitung` or the function of a specialist position. */
  role: string;
  level: "standort" | "land" | "kontinent";
  /** Text key of the site type, `ebene.land` or `ebene.kontinent`. */
  kind_text: string;
  country: string | null;
  continent: string | null;
  unit: string;
}

/** A position a concern passed on its way (MA3). */
export interface AnliegenWeg {
  position: AnliegenStelle;
  manager: string;
  /** Kind of the option it recommended. */
  recommended: string;
}

/** A site's part of a strategic concern (MA3). */
export interface AnliegenTeil {
  site: number | null;
  kind_text: string | null;
  country: string | null;
  product: string | null;
  option: AnliegenOption;
}

export interface Anliegen {
  id: number;
  /** The site the decision is about, if any. */
  site: number | null;
  site_kind_text: string | null;
  site_country: string | null;
  /** The position that asks the player. */
  asker: AnliegenStelle;
  /** The positions on the way, the first first; empty where it asks itself (MA3). */
  path: AnliegenWeg[];
  /** The sites' parts of a strategic concern (MA3). */
  parts: AnliegenTeil[];
  /** The manager who asks. */
  manager: string;
  /** Topic (text `thema.<topic>`). */
  topic: string;
  product: string | null;
  /** Why the position asks (text `anliegen.grund.<reason>`). */
  reason:
    | "entscheidung"
    | "jahr"
    | "immer"
    | "kredit"
    | "reserve"
    | "investition"
    | "verschuldung"
    | "antrag"
    | "abwerbung"
    | "freigabe"
    | "beteiligung"
    | "ruhestand"
    | "unbesetzt";
  options: AnliegenOption[];
  recommended: number;
  /** Why the position recommends its option. */
  because: Meldung;
  per_decision_usd: number;
  left_usd: number;
  /**
   * For the reasons `reserve` and `investition` (MA4): the liquidity reserve, or what is
   * left of the investment budget that binds first and where it is set (`firma`,
   * `land:DEU` …).
   */
  strategy_limit_usd: number | null;
  strategy_scope: string | null;
  /** Mean monthly result of the site in the last closed months; null without a site. */
  site_result_usd: number | null;
  created: string;
  deadline: string;
  status: AnliegenStatus;
  /** The option carried out. */
  carried_out: number | null;
  closed: string | null;
  /** Beyond the routine of the sites: a run of rounds halts for it. */
  important: boolean;
}

/** Open concerns of several sites alike: the same topic and recommendation. */
export interface AnliegenGruppe {
  topic: string;
  /** Kind of the recommended option. */
  kind: string;
  concerns: Anliegen[];
}

export interface AnliegenListe {
  /** Open concerns in groups, the earliest deadline first. */
  open: AnliegenGruppe[];
  /** Concerns closed in the last year, the newest first. */
  closed: Anliegen[];
  deadline_days: number;
  block_days: number;
}

export interface Managermarkt {
  /** The unit as in `EinheitOrganisation.key`. */
  unit: string;
  site: number | null;
  role: string;
  kind_text: string;
  country: string | null;
  continent: string;
  /** Free candidates, those of the site's continent first. */
  candidates: Kandidat[];
  /** The company's managers on other positions. */
  own: Kandidat[];
  /** The company's cash, for the warning on salaries above it. */
  cash_usd?: number;
}

/** How the positions price their offers (MA4). */
export type Preisstrategie = "Market" | "Premium" | "Fight" | { MinMargin: number };

/** Whether inputs come from the company's own sites or from the market (MA4). */
export type Bezugsweg = "OwnFirst" | "ByPrice" | "Buy";

/** Stock reach in days (MA4). */
export interface Lagervorgabe {
  input_min_days: number;
  input_max_days: number;
  output_days: number;
}

/** Bounds of the wage premium (shares, MA4). */
export interface Lohnvorgabe {
  min: number;
  max: number;
}

/** A strategy as `SetStrategy` takes it; the budget in Money units (MA4). */
export type Vorgabe =
  | { Price: Preisstrategie }
  | { Stock: Lagervorgabe }
  | { Wages: Lohnvorgabe }
  | { Supply: Bezugsweg }
  | { Investment: number }
  | { Reserve: number }
  | { Training: number }
  | { Environment: boolean };

export type Vorgabefeld =
  "Price" | "Stock" | "Wages" | "Supply" | "Investment" | "Reserve" | "Training" | "Environment";

/** Where a strategy holds (MA4). */
export type Geltung = "Company" | { Continent: string } | { Country: string } | { Site: number };

/** One field at one unit (MA4). */
export interface VorgabeEintrag {
  field:
    | "preis"
    | "lager"
    | "personal"
    | "eigenfertigung"
    | "investition"
    | "reserve"
    | "schulung"
    | "umwelt";
  /** What holds; null: no investment budget. */
  value: Vorgabe | null;
  /** The unit it comes from (`firma`, `kontinent:europa`, `land:DEU`, `standort:3`); null: the default. */
  origin: string | null;
  /** The unit sets it itself. */
  own: boolean;
  budget_usd: number | null;
  /** What is left this year of the budget that binds first, and where it is set. */
  left_usd: number | null;
  binding: string | null;
  /** The liquidity reserve at today's running costs. */
  reserve_usd: number | null;
  /** At sites: the position that follows it; null where the player decides. */
  carrier: { position: AnliegenStelle; manager: string } | null;
}

/** A unit of the company with its strategies (MA4). */
export interface VorgabeEinheit {
  /** `firma`, `kontinent:europa`, `land:DEU`, `standort:3`. */
  key: string;
  level: "firma" | "kontinent" | "land" | "standort";
  /** Text key: the site type, `ebene.land`, `ebene.kontinent` or `ebene.firma`. */
  kind_text: string;
  country: string | null;
  continent: string | null;
  site: number | null;
  /** The unit above; null for the company. */
  parent: string | null;
  /** The company's sites in the unit. */
  sites: number;
  entries: VorgabeEintrag[];
}

/** The strategies of the company's units (MA4). */
export interface Strategie {
  enabled: boolean;
  units: VorgabeEinheit[];
  limits: {
    min_margin_max: number;
    stock_days_max: number;
    wage_premium_max: number;
    reserve_months_max: number;
  };
  /** Price floor on the full unit cost and start markup: `marktpreis`, `premium`, `kampfpreis`. */
  prices: { kind: string; floor: number; markup: number }[];
  /** Running costs of a month of all sites. */
  monthly_cost_usd: number;
  settings: number;
  /** Who may buy the company's goods besides consumers and governments (M8). */
  sales: Verkaufsweg[];
  /** Products the company offers or has rules for. */
  sale_products: { product: string; unit: string }[];
  /** Countries of its sites and rules. */
  sale_countries: string[];
  /** The policy „Beteiligungen“ (ZA2). */
  participations: Beteiligungen;
  /** The dividend policy (PE4). */
  dividend: Dividendenpolitik;
}

/** The dividend policy of the company and what it means now (PE4). */
export interface Dividendenpolitik {
  kind: "anteil" | "betrag";
  /** Share of the net profit (0 with a fixed amount). */
  share: number;
  /** Amount per year (0 with a share). */
  amount_usd: number;
  /** Set by the player or the CEO; otherwise the default of 0 %. */
  own: boolean;
  /** The last closed year; null in the first year. */
  year: number | null;
  profit_usd: number;
  /** What the policy wants for the closed year and what could be paid today. */
  wanted_usd: number;
  payable_usd: number;
  distributable_usd: number;
  reserve_usd: number;
  /** Withholding tax of the headquarters' country. */
  tax: number;
  person_share: number;
  /** The last payout: year and amount. */
  last: [number, number] | null;
  /** A CEO proposes the dividend at the year's start. */
  ceo: boolean;
}

/** A rule for traders or other companies buying the company's goods (M8). */
export interface Verkaufsweg {
  buyer: "haendler" | "firmen";
  /** The product and country it holds for; null: all. */
  product: string | null;
  country: string | null;
  allowed: boolean;
  min_price_usd: number | null;
  max_per_month: number | null;
  /** Text key of the product's unit. */
  unit: string | null;
}

/** The guideline of the mandate as `SetMandate` takes it (MA5). */
export type Leitlinie = "Growth" | "Profit" | "Safety" | { Leadership: string };

/** How often the CEO reviews the strategy (MA5). */
export type Ruecksprachetakt = "Monthly" | "Quarterly" | "HalfYearly" | "Yearly";

/** The mandate to the board as `SetMandate` takes it (MA5). */
export interface Auftrag {
  guideline: Leitlinie;
  goals: {
    growth?: number | null;
    margin?: number | null;
    equity_ratio?: number | null;
    rank?: number | null;
  };
  max_debt?: number | null;
  blocked_countries?: string[];
  blocked_groups?: string[];
  review: Ruecksprachetakt;
}

/** Revenue and result of a part of the company in a review (MA5). */
export interface Kennzahlen {
  /** Key of the continent or goods group. */
  key: string;
  revenue_usd: number;
  /** Result; for a goods group the margin of its products. */
  result_usd: number;
}

/** A goal against what was reached (MA5). */
export interface Zielstand {
  goal: "wachstum" | "rendite" | "eigenkapitalquote" | "rang";
  /** Shares as fractions, the rank as a place. */
  target: number;
  actual: number | null;
  met: boolean | null;
}

export interface Chance {
  kind: "produkt" | "antrag";
  product: string | null;
  revenue_usd: number;
  margin_usd: number;
  concern: number | null;
  topic: string | null;
  open: boolean;
}

export interface Risiko {
  kind: "verlust" | "ziel" | "reserve" | "verschuldung";
  site: number | null;
  site_kind_text: string | null;
  country: string | null;
  /** The loss, or the cash. */
  amount_usd: number | null;
  /** The liquidity reserve. */
  limit_usd: number | null;
  goal: string | null;
  share: number | null;
  max: number | null;
}

/** One strategy review of the CEO (MA5). */
export interface Ruecksprache {
  date: string;
  from: string;
  to: string;
  interval: string;
  manager: string;
  revenue_usd: number;
  result_usd: number;
  overhead_usd: number;
  continents: Kennzahlen[];
  groups: Kennzahlen[];
  goals: Zielstand[];
  chances: Chance[];
  risks: Risiko[];
}

/** The mandate as the form shows it (MA5). */
export interface AuftragSicht {
  guideline: "wachstum" | "ertrag" | "sicherheit" | "marktfuehrung";
  leading_group: string | null;
  growth: number | null;
  margin: number | null;
  equity_ratio: number | null;
  rank: number | null;
  max_debt: number | null;
  blocked_countries: string[];
  blocked_groups: string[];
  review: "monatlich" | "quartalsweise" | "halbjaehrlich" | "jaehrlich";
}

/** The mandate to the board and the CEO's strategy reviews (MA5). */
export interface Ruecksprachen {
  enabled: boolean;
  mandate: AuftragSicht;
  guidelines: { key: string; aggressiveness: number }[];
  intervals: string[];
  groups: string[];
  countries: string[];
  /** The CEO's name; null without a CEO. */
  ceo: string | null;
  next_review: string | null;
  goals_now: Zielstand[];
  /** The reviews kept, the newest first. */
  reviews: Ruecksprache[];
}

/** An owner of a start-up (SU1). */
export interface StartUpEigner {
  holder: "gruender" | "investoren" | "spieler" | "firma";
  /** The company's name for `firma`. */
  company: string | null;
  share: number;
}

/** A start-up as the player sees it (SU1). */
export interface StartUp {
  id: number;
  /** The inventor or the founder. */
  name: string;
  /** Named after a historical inventor. */
  inventor: boolean;
  country: string;
  /** A new technology or the next level of a product. */
  kind: "technologie" | "verbesserung";
  /** Key of the technology or of the product. */
  target: string;
  level: number | null;
  /** Years ahead of history at the founding. */
  lead: number;
  founded: string;
  /** Key of the phase (`startup.phase.<key>`); none once closed. */
  phase: string | null;
  phase_number: number;
  phases: number;
  capital_usd: number;
  raised_usd: number;
  /** Last day of the open round; none once funded. */
  round_until: string | null;
  /** When the funded phase is decided. */
  phase_until: string | null;
  /** The strategy department's estimate; none without one or once closed. */
  chance: number | null;
  chance_level: "gering" | "mittel" | "hoch" | null;
  owners: StartUpEigner[];
  status: "aktiv" | "erfolg" | "gescheitert" | "ohne_geld" | "ueberholt";
  ended: string | null;
  /** Value of the whole start-up now, and of a success as it looks today (SU2). */
  value_usd: number;
  success_value_usd: number;
  /** The player's company: share, book value, pledge to the open round, grants. */
  own_share: number;
  /** The person's private share and pledge (PE5). */
  person_share?: number;
  person_pledge_usd?: number;
  private_cash_usd?: number;
  own_book_usd: number;
  own_pledge_usd: number;
  own_grants_usd: number;
  /** Pledge to the open round or shares of founders and investors, up to the maximum. */
  invest_mode: "runde" | "anteile" | null;
  invest_max_usd: number | null;
  /** What all of it fetches when selling now. */
  sale_value_usd: number;
  /** The value of the player's shares now and the minimum price it offers them to the
   * companies for until the next month start (ZA4). */
  own_value_usd: number;
  own_offer_usd: number | null;
  majority: boolean;
  /** Key of the pace (`startup.lenkung.<key>`). */
  pace: string;
  /** Cost of buying out the others; none where not possible. */
  integration_usd: number | null;
  blocked: boolean;
  subsidiary: boolean;
  /** The company it belongs to, if another one's. */
  parent: string | null;
  /** Expected return per dollar pledged (strategy department). */
  expected_return: number | null;
  exit: "tochter" | "boerse" | null;
  exit_company: string | null;
  /** The company whose research project it was (SU3). */
  origin: string | null;
}

/** A research project of the player that could become a start-up (SU3). */
export interface Ausgruendung {
  site: number;
  country: string;
  kind: "technologie" | "verbesserung";
  target: string;
  level: number | null;
  /** Share of the effort done (0–1). */
  progress: number;
  /** Years until history invents the technology (none for a level), and the months its
   * phases would still take. */
  lead: number | null;
  months: number | null;
  /** Other companies whose research centers work on the same target. */
  rivals: number;
  /** Phase it would begin in, its value then and what all of it fetches from investors. */
  phase: string | null;
  value_usd: number | null;
  sale_value_usd: number | null;
  /** Why not: too early or invented/reached. */
  reason: "fortschritt" | "nicht_moeglich" | null;
}

/** The start-ups of the world (SU1). */
export interface StartUps {
  /** What the game calls them in this year (`startup.bezeichnung.<key>`). */
  label: string | null;
  per_year: number;
  /** Whether the strategy department estimates their chances. */
  estimated: boolean;
  /** Newest first. */
  active: StartUp[];
  /** Ended most recently first. */
  closed: StartUp[];
  founded: number;
  succeeded: number;
  failed: number;
  keep_years: number;
  /** The player's stakes (SU2). */
  portfolio_book_usd: number;
  portfolio_value_usd: number;
  holdings: number;
  cash_usd: number;
  blocking: number;
  majority: number;
  buy_premium: number;
  sale_discount: number;
  grant_effect: number;
  /** Keys of the paces (`startup.lenkung.<key>`). */
  paces: string[];
  /** When stakes offered to the companies go to the best bid, and the highest premium on
   * the value a company bids (ZA4). */
  offers_settle: string;
  company_premium_max: number;
  /** Projects of the player's research centers and the progress from which they may be
   * spun off (SU3). */
  spin_offs: Ausgruendung[];
  spin_off_min: number;
}

/** An own site and the products it could contract (W4). */
export interface VertragsStandort {
  site: number;
  country: string;
  kind_text: string;
  /** Products the site makes or offers. */
  sells: string[];
  /** Products the site uses or orders. */
  buys: string[];
}

/** A supply contract of the player (W4). */
export interface Vertrag {
  id: number;
  /** The player's side. */
  role: "verkauf" | "einkauf";
  status:
    "angeboten" | "laufend" | "beendet" | "gekuendigt" | "abgelehnt" | "verfallen" | "nichtig";
  product: string;
  unit: string;
  own_site: number;
  own_country: string;
  own_kind_text: string;
  partner: string;
  partner_country: string;
  per_month: number;
  price_usd: number;
  months: number;
  min_quality: number;
  penalty: number;
  proposed: string;
  start: string | null;
  end: string | null;
  closed: string | null;
  delivered_month: number;
  delivered_total: number;
  penalties_paid_usd: number;
  penalties_received_usd: number;
  market_price_usd: number;
  /** The player must answer this proposal. */
  answer: boolean;
  can_cancel: boolean;
  cancel_fee_usd: number;
}

export interface Vertraege {
  enabled: boolean;
  months_max: number;
  months_default: number;
  penalty_max: number;
  penalty_default: number;
  cancel_months: number;
  contracts: Vertrag[];
  sites: VertragsStandort[];
}

export interface Vertragspartner {
  site: number;
  company: string;
  country: string;
  kind_text: string;
  free_per_month: number;
  suggested_price_usd: number;
  delivery_cost_usd: number;
}

export interface Vertragspartnerliste {
  site: number;
  product: string;
  unit: string;
  role: "verkauf" | "einkauf";
  own_per_month: number;
  partners: Vertragspartner[];
}

/** Vehicles of one kind the player holds (W5). */
export interface Flottenbestand {
  vehicle: string;
  way: string;
  count: number;
  capacity_tkm: number;
  used_tkm: number;
  used_last_tkm: number;
  purchase_usd: number;
  book_value_usd: number;
  sale_usd: number;
  running_share: number | null;
}

/** A vehicle the player can buy now (W5). */
export interface Fahrzeugangebot {
  vehicle: string;
  way: string;
  classes: string[];
  payload_t: number;
  km_per_day: number;
  cost_per_tkm_usd: number;
  capacity_tkm: number;
  price_usd: number;
  monthly_cost_usd: number;
  running_share: number | null;
}

export interface Logistikmonat {
  fleet_tkm: number;
  market_tkm: number;
  state_tkm: number;
  losses: number;
  lost_value_usd: number;
  rental_usd: number;
  upkeep_usd: number;
  depreciation_usd: number;
}

export interface Logistik {
  enabled: boolean;
  mode: "markt" | "staat" | "flotte";
  carry_for_others: boolean;
  state_surcharge: number;
  state_risk_factor: number;
  market_margin: number;
  rental_share: number;
  sale_share: number;
  risk_land: number;
  risk_sea: number;
  fleet: Flottenbestand[];
  vehicles: Fahrzeugangebot[];
  month: Logistikmonat;
  last_month: Logistikmonat;
}

/** A subsidiary of the player's group (W6). */
export interface Tochter {
  company: number;
  name: string;
  country: string;
  parent: string;
  direct: boolean;
  focus: "produktion" | "logistik" | "investment" | "bank";
  cash_usd: number;
  equity_usd: number;
  paid_in_usd: number;
  revenue_year_usd: number;
  result_year_usd: number;
  sites: number;
  vehicles: number;
  payout_max_usd: number;
}

export interface KonzernStandort {
  site: number;
  company: number;
  company_name: string;
  country: string;
  kind_text: string;
  book_usd: number;
}

export interface KonzernZeile {
  key: string;
  usd: number;
}

export interface Konzern {
  enabled: boolean;
  min_capital_usd: number;
  founding_cost_usd: number;
  company: number;
  company_name: string;
  own_equity_usd: number;
  subsidiaries: Tochter[];
  sites: KonzernStandort[];
  assets: KonzernZeile[];
  liabilities: KonzernZeile[];
  total_assets_usd: number;
  group_equity_usd: number;
  income: KonzernZeile[];
  result_year_usd: number;
}

/** One level of the controlling drill-down (W7). */
export interface ControllingKnoten {
  key: string;
  level:
    "konzern" | "firma" | "kontinent" | "land" | "standort" | "produkt" | "allgemein" | "zentrale";
  /** Name of a company, else a text key. */
  name: string;
  country: string | null;
  revenue_usd: number;
  variable_usd: number;
  margin1_usd: number;
  fixed_usd: number;
  margin2_usd: number;
  other_usd: number;
  result_usd: number;
  previous_result_usd: number | null;
  series_usd: number[];
  costs: { key: string; usd: number }[];
  children: ControllingKnoten[];
}

export interface Controlling {
  period: "monat" | "jahr" | "vorjahr";
  periods: ("monat" | "jahr" | "vorjahr")[];
  start: string | null;
  months: string[];
  root: ControllingKnoten | null;
}

/** A share with what it costs or brings (K1). */
export interface AktienKurs {
  share: number;
  usd: number;
}

export interface AktienAusgabe {
  share: number;
  proceeds_usd: number;
  cost_usd: number;
  stake_after: number;
  loses_majority: boolean;
}

/** An owner of the player's company (K3). */
export interface Eigner {
  kind: "du" | "anleger" | "gruender" | "firma";
  name: string | null;
  share: number;
}

export interface BoersenFirma {
  company: number;
  name: string;
  country: string;
  relation: "eigen" | "konzern" | "fremd";
  since: string;
  value_usd: number;
  price_usd: number;
  change_month: number | null;
  change_year: number | null;
  equity_usd: number;
  earnings_usd: number;
  pe: number | null;
  dividend_usd: number;
  dividend_yield: number | null;
  free_float: number;
  fair_usd: number;
  takeover_usd: number | null;
  held: number;
  held_cost_usd: number;
  held_value_usd: number;
  buy: AktienKurs[];
  sell: AktienKurs[];
  series_usd: number[];
}

export interface EigeneNotierung {
  listed: boolean;
  subsidiary: boolean;
  equity_usd: number;
  equity_min_usd: number;
  player_stake: number;
  free_float: number;
  issue_value_usd: number;
  discount: number;
  share_max: number;
  issue: AktienAusgabe[];
  buyback: AktienKurs[];
  owners: Eigner[];
  payout: number;
  profit_last_year_usd: number;
  dividend_estimate_usd: number;
  last_dividend_usd: number;
  player_dividends_usd: number;
}

/** Finances → stock market (K1). */
export interface Boerse {
  enabled: boolean;
  index: number;
  mood: number;
  months: string[];
  index_series: number[];
  own: EigeneNotierung;
  companies: BoersenFirma[];
  portfolio_cost_usd: number;
  portfolio_value_usd: number;
  trade_share_max: number;
}

/** A loan a bank of the player gave (K4). */
export interface BankKredit {
  borrower: string;
  balance_usd: number;
  rate: number;
  start: string;
  months: number;
}

export interface BankZeile {
  company: number;
  name: string;
  own: boolean;
  deposit_spread: number;
  deposit_rate: number;
  loan_discount: number;
  max_debt_ratio: number;
  deposits_usd: number;
  deposit_target_usd: number;
  capacity_usd: number;
  equity_usd: number;
  cash_usd: number;
  reserve_usd: number;
  room_usd: number;
  loans_given_usd: number;
  loans: BankKredit[];
  interest_year_usd: number;
  write_offs_year_usd: number;
  result_year_usd: number;
}

/** Finances → bank (K4). */
export interface Bank {
  enabled: boolean;
  base_rate: number;
  banks: BankZeile[];
  subsidiaries: boolean;
}

/** A child of the person (PE2). */
export interface PersonKind {
  name: string;
  born: string;
  /** Age today, or at death. */
  age: number;
  died: string | null;
  /** The manager card and, while employed, the position in words. */
  manager: number | null;
  position: Meldung | null;
  /** Years until the manager card. */
  card_in: number | null;
}

/** A company the person holds shares of (PE2) with its money (PE3). */
export interface PersonAnteil {
  index?: number;
  company: string;
  share: number;
  controlled: boolean;
  person_ceo: boolean;
  ceo: string | null;
  value_usd?: number;
  cost_basis_usd?: number;
  /** The most capital the company can pay back now (only as sole owner). */
  withdraw_max_usd?: number;
  cash_usd?: number;
  loans?: PersonDarlehen[];
  /** What investors pay for the whole share at once (PE5). */
  investors_bid_usd?: number;
  /** The person can choose it as main company; it is the main company now. */
  selectable?: boolean;
  main?: boolean;
}

/** A loan of the person to a company (PE3). */
export interface PersonDarlehen {
  /** Position among the company's loans (`RepayLoan`). */
  index: number;
  balance_usd: number;
  rate: number;
  instalment_usd: number;
}

/** A level of lifestyle with its cost today and its effects (PE3). */
export interface Lebensstil {
  key: string;
  cost_usd: number;
  interest: number;
  salary_demand: number;
  education: number;
  mortality: number;
}

/** The person's money (PE3). */
export interface PersonGeld {
  cash_usd: number;
  shares_usd: number;
  loans_usd: number;
  wealth_usd: number;
  /** Wealth on the first day of each month, the earliest first. */
  history: [string, number][];
  /** Income and spending of the last twelve months (texts `privat.<key>`). */
  flows: [string, number][];
  lifestyle: string;
  lifestyle_next: string | null;
  lifestyle_change_from: string;
  lifestyles: Lebensstil[];
  short: boolean;
  salary_usd: number;
  salary_paid: boolean;
  salary_suggestion_usd: number;
  salary_max_usd: number | null;
  income_tax: number;
  savings_rate: number;
  loan_max_rate: number;
  loan_max_years: number;
}

/** What the founding dialog needs (PE3). */
export interface Gruendung {
  forms: { key: string; cost_usd: number }[];
  country: string;
  capital_suggestion_usd: number;
  cost_share: number;
  cost_min_usd: number;
}

/** The player as a person (PE2). */
export interface Person {
  name: string;
  born: string;
  age: number;
  /** Country of residence (ISO). */
  home: string;
  married: boolean;
  children: PersonKind[];
  children_possible: boolean;
  holdings: PersonAnteil[];
  /** The chronicle, the latest first. */
  history: { date: string; text: Meldung }[];
  money?: PersonGeld | null;
  /** While the person has no company yet: the founding. */
  founding?: Gruendung | null;
  /** Heir, estate and tax (PE6). */
  succession?: Nachfolge | null;
}

/** Who inherits and what it costs (PE6). */
export interface Nachfolge {
  /** The chosen child (index), if any; otherwise the rule decides. */
  chosen: number | null;
  /** The heir by rule today; null: a nephew or niece. */
  heir: string | null;
  choices: { index: number; name: string; age: number }[];
  estate_usd: number;
  tax_rate: number;
  tax_usd: number;
  death_chance_year: number;
  generation: number;
  ancestors: { name: string; born: string; until: string; died: boolean }[];
}
