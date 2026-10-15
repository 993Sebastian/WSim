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
  /** The build-up of the markets: start year, share of the full size then, years to full. */
  market_build_up?: { from_year: number; start_share: number; years: number } | null;
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
  /** Mandatory research (FW1): a new company knows only the basic product per branch. */
  research_mandatory?: boolean;
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
  /** The player's name for the site. */
  name?: string | null;
  country: string;
  /** Text key of the site type, e.g. standorttyp.werk. */
  kind: string;
  deposit: string | null;
  workers: number;
  facilities: Anlage[];
  stock: Lagerbestand[];
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
  /** Units made and sold, by continent, country, site and product. */
  quantities?: Mengenknoten | null;
  /** Every company the person controls: result and cash of the round. */
  companies?: FirmaInRunde[];
}

/** A company of the person in the round report. */
export interface FirmaInRunde {
  company: number;
  name: string;
  main: boolean;
  revenue_usd: number;
  result_usd: number;
  cash_before_usd: number;
  cash_after_usd: number;
}

/** Units of a product made and sold in the round. */
export interface Mengenzeile {
  product: string;
  unit: string;
  made: number;
  sold: number;
}

/** A level of the round's quantities: `firma`, `kontinent`, `land`, `standort`. */
export interface Mengenknoten {
  level: "firma" | "kontinent" | "land" | "standort";
  key: string;
  kind_text: string | null;
  site_name: string | null;
  country: string | null;
  products: Mengenzeile[];
  children: Mengenknoten[];
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
  /** Corruption and rule of law 0–1 (BP4). */
  corruption?: number;
  rule_of_law?: number;
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
  /** Corruption and rule of law 0–1 (BP4). */
  corruption?: number;
  rule_of_law?: number;
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
  /** Military expenditure as a share of GDP and war intensity 0–1 (H3). */
  military_share?: number;
  war_intensity?: number;
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
  /** The owner's research level of the product against the leader (0–1). */
  development_share?: number;
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
  /** Standing after a breakdown or for an overhaul until the day (BP2). */
  down_until?: string | null;
  down_overhaul?: boolean;
  overhaul_cost_usd?: number | null;
  failures_year?: number;
  /** Share of the output lost as scrap (BP4). */
  scrap?: number;
  /** How the quality of the next batch comes about with today's inputs (#191). */
  quality?: Qualitaetsteile | null;
}

/** Quality in points: recipe, inputs, automation, condition, research, training. */
export interface Qualitaetsteile {
  /** Average quality of the inputs in stock (null without inputs or stock). */
  input_quality: number | null;
  base: number;
  inputs: number;
  automation: number;
  condition: number;
  research: number;
  training: number;
  /** Points from the trim level (MN2). */
  trim?: number;
  total: number;
}

export interface Angebot {
  product: string;
  /** `aufschlag`: production cost plus `markup` as the margin. */
  mode: "markt" | "marktbezug" | "fest" | "aufschlag";
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
  /** The company's sales offices that offer the product; supplied from here by need. */
  outlets?: { site: number; name: string | null; country: string; supplied_here: boolean }[];
  /** Trim level of an end product made here (MN2); absent where there is none to choose. */
  trim?: "einfach" | "standard" | "premium" | null;
}

export interface Einkauf {
  product: string;
  target: number;
  max_price_usd: number;
  min_quality: number;
  bought_month: number;
  bought_last_month: number;
  /** One-off purchase still to buy and its price limit. */
  once?: number;
  once_max_price_usd?: number;
}

export interface Versorgung {
  product: string;
  need_per_day: number;
  stock: number;
  days: number | null;
  own: boolean;
  ordered: boolean;
  market_price_usd: number;
  /** Quality of the stock, if there is any. */
  quality?: number | null;
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
  /** Learning factor on the hours (1 = no experience). */
  experience?: number;
  experience_units?: number;
  learning_saving_usd?: number;
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

/** Market price of a product over the last twelve closed months. */
export interface Preisverlauf {
  months: Marktmonat[];
  year_ago_usd: number | null;
  /** Last closed month against a year before (0.1 = 10 % dearer). */
  change: number | null;
  low_usd: number | null;
  high_usd: number | null;
}

/** A product in stock at a site. */
export interface Lagerbestand {
  product: string;
  quantity: number;
  /** Book value: production costs of own output, purchase price of bought goods. */
  value_usd: number;
  /** Book value per unit. */
  unit_cost_usd?: number;
  quality?: number;
  /** Quantity at today's market price in the site's country. */
  market_value_usd?: number;
  need_per_day?: number;
  output_per_day?: number;
  /** Days the stock lasts at the planned use. */
  days?: number | null;
}

export interface StandortDetail {
  /** Branch of the works (FW1; key), none before its first product. */
  branch?: string | null;
  /** Products a unit may be bought for here (FW1); empty without mandatory research. */
  buy_products?: string[];
  index: number;
  /** The player's own name for the site, if any. */
  name?: string | null;
  country: string;
  /** Key of the country's continent, to group the sites. */
  continent?: string;
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
  stock: Lagerbestand[];
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
  /** Maintenance level of the site (BP2). */
  maintenance?: {
    level: number;
    min: number;
    max: number;
    failures: number;
    accidents?: number;
    accident_rate?: number;
    /** Condition for automatic overhauls: own setting and the one that holds. */
    overhaul_below?: number | null;
    overhaul_effective?: number | null;
  } | null;
  rival_premium_max: number;
  staff: Personal[];
  wage_cost_per_day_usd: number;
  unit_costs: Stueckkosten[];
  last_month: StandortErgebnis | null;
  /** Market and reference price in the site's country, by product. */
  prices: Record<string, { market_usd: number; reference_usd: number }>;
  /** The plot the site stands on (none for extraction sites). */
  plot: StandortGrundstueck | null;
  /** Running and proposed supply contracts of the site. */
  contracts?: Vertrag[];
  /** Market prices of the last twelve months in the site's country. */
  /** Products the site draws from other own sites, and the suggested reach (days). */
  supplies?: InternerBezug[];
  supply_days_default?: number;
  /** What closing the site brings and costs today. */
  closing?: {
    facilities_usd: number;
    building_usd: number;
    land_usd: number;
    stock_usd: number;
    proceeds_usd: number;
    book_usd: number;
    severance_usd: number;
    result_usd: number;
    /** Why the site cannot be closed now. */
    blocker: Meldung | null;
  } | null;
  price_trends?: Record<string, Preisverlauf>;
  /** Sites of the same type in the company and its group a facility can move to. */
  move_targets?: Verlegeziel[];
  /** The site's store (values at reference prices); null when stocks are unlimited. */
  storage?: LagerKapazitaet | null;
}

/** Capacity and fill of a site's store, what a step of expansion brings and costs. */
export interface LagerKapazitaet {
  /** Room and fill in m³ of storage volume; the stock's value at reference prices. */
  capacity_m3: number;
  fill_m3: number;
  value_usd: number;
  share: number;
  full: boolean;
  /** The site's own products fill the store: the facilities stand. */
  stops?: boolean;
  /** Goods lie in outside storage at a higher rate (BP3). */
  external?: boolean;
  hard_capacity_m3?: number;
  steps: number;
  step_capacity_m3: number;
  step_cost_usd: number;
  cost_per_day_usd: number;
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
  /** The player's running machine tenders (MAA). */
  tenders?: MaschinenAusschreibung[];
  /** Facilities are asked for in tenders instead of bought directly. */
  machine_tenders_enabled?: boolean;
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
    market_size_usd: number;
    sold_last_month: number;
    imported_last_month: number;
    exported_last_month: number;
    sellers: number;
    own_price_usd: number | null;
    /** Really offered here: sold last or this month, or goods in stock. */
    own_active?: boolean;
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
  /** The continents with their countries, for choosing a market. */
  regions?: { continent: string; countries: string[] }[];
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
    /** Development generation of the seller for the product (M37). */
    level: number;
    /** Research level of the seller for the product against the leader (0–1, Q1). */
    research_share?: number;
    /** The seller's own name for the product (M42). */
    product_name?: string | null;
    /** Quality of its stock at its selling sites (null without stock). */
    quality?: number | null;
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
  /** Raw materials: swing of the world's output from the commodity cycle (0.1 = +10 %). */
  supply_cycle?: number | null;
  /** The player's reputation here, 0–1; 0.5 is the middle (MN2). */
  own_reputation?: number;
  /** Whether the reputation counts in the choice of seller. */
  reputation_counts?: boolean;
  /** The player's sales channels for the goods group and their mean reach (MN3). */
  own_channels?: string[];
  own_reach?: number | null;
  /** The used market of a durable (MN4). */
  used?: { stock: number; taken_per_month: number } | null;
  /** The open lot of the government with the player's bid (MN2). */
  tender?: Ausschreibungslos | null;
  /** What households weigh in choosing a seller (text `erklaerung.faktor.<key>`). */
  choice_factors?: string[];
}

/** An open lot of a government (MN2). */
export interface Ausschreibungslos {
  id: number;
  per_day: number;
  deadline: string;
  bids: number;
  own_price_usd: number | null;
  own_site: number | null;
  sites: { site: number; kind_text: string; name: string | null }[];
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
  /** Share of output gained per run (manufacturing research, Q6). */
  output_gained?: number;
}

/** A product the player may develop or has developed (M37). */
export interface Weiterentwicklung {
  product: string;
  /** The player makes it in a facility of its own. */
  own: boolean;
  /** Generation reached (point thresholds for successor models and start-ups). */
  level: number;
  own_level: number;
  best_rival: number;
  /** All points of the player and of the leader. */
  total_points: number;
  leader_points: number;
  /** The player's level (0–1 of the leader's points), the best rival's, the maturity. */
  share: number;
  best_rival_share: number;
  maturity: number;
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
  /** What the lead (100 %) brings at today's maturity. */
  lead_effect: Entwicklungswirkung;
  sites: number[];
  /** Manufacturing research (Q6): own points, share of the leader's, maturity, centers. */
  process_points?: number;
  process_share?: number;
  process_maturity?: number;
  process_sites?: number[];
}

/** A product to research (Q6); the technologies stay behind it. */
export interface Produktforschung {
  product: string;
  kind: string;
  branch: string;
  status: "bekannt" | "in_arbeit" | "erforschbar" | "gesperrt";
  /** The product exists in the world (it can be bought). */
  in_world: boolean;
  needed: number;
  collected: number;
  /** Inputs that do not exist in the world yet. */
  missing_inputs: string[];
  inputs: string[];
  historical_year: number | null;
  field: string | null;
  sites: number[];
  points_per_day: number;
  days: number | null;
  one_lab: {
    country: string;
    researchers: number;
    points_per_day: number;
    days: number;
    cost_usd: number;
  } | null;
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
    /** Patent or own patent claim (P7). */
    patent?: Patent | null;
  }[];
  /** All products with their research state (Q6). */
  products?: Produktforschung[];
  /** Products to develop (M37) and the highest level. */
  developments: Weiterentwicklung[];
  development_levels: number;
  /** What the lead brings at full maturity. */
  development_max: Entwicklungswirkung;
  /** Kind of every product: rohstoff, halbzeug, komponente, endprodukt, energie. */
  product_kinds?: Record<string, string>;
  centers: {
    site: number;
    /** The player's name for the center. */
    name?: string | null;
    country: string;
    researchers: number;
    project: string | null;
    /** Product developed instead of a technology (M37). */
    development: string | null;
    /** Shares of the researchers per developed product; the rest works on `project`. */
    shares?: [string, number][];
    /** Shares on manufacturing research per product (Q6). */
    process_shares?: [string, number][];
    /** Product it researches (Q6); `project` is the technology it works on for it. */
    research_product?: string | null;
    ready: boolean;
    building_until: string | null;
    /** The staff by role (PD1); empty where the data have no roles. */
    staff?: { role: string; count: number; research_weight: number }[];
    /** Posts for people in the laboratories (PD1). */
    posts?: number;
    labs: {
      slot: number;
      count: number;
      utilization: number;
      ready: string;
      /** The laboratory's own project (LB1); without one it follows the center. */
      work?: { kind: "technologie" | "produkt" | "entwicklung"; key: string; technology?: string };
      /** Share of the center's researchers working here. */
      share?: number;
    }[];
  }[];
  laboratory: string | null;
  laboratory_usd: number;
  laboratory_posts: number;
  units: Record<string, string>;
  /** Countries to file patents in, largest markets first (P7). */
  patent_countries?: { country: string; cost_usd: number; suggested: boolean }[];
  /** The development projects PM0–PM7 (PD2); absent where the data have none. */
  projects?: Entwicklungsprojekte;
}

/** A patent or an open claim on a technology (P7). */
export interface Patent {
  holder: string;
  own: boolean;
  /** Last day to file further countries (own claims only). */
  deadline: string | null;
  until: string | null;
  countries: string[];
  free_for_player: boolean;
  blocks_player_in: string[];
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
    /** The player's own name for the site, if any. */
    site_name?: string | null;
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
  site_name?: string | null;
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
  /** The player's own name for the site, if any. */
  site_name?: string | null;
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
  /** Industry experience as a level, the best first (MG5); the key is the branch. */
  branches?: Faehigkeit[];
  /** Estimates of targets judged so far, and the hit rate (null before the first; ZA3). */
  judged: number;
  hit_rate: number | null;
  /** Age today, the age at which he plans to retire and the day (PE1). */
  age?: number | null;
  retires_at?: number | null;
  retirement?: string | null;
  /** A child of the person (PE2). */
  family?: boolean;
  /** His style (text `stil.<key>`) and favourite reasons (MG7). */
  style?: string | null;
  reasons?: string[];
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
  /** The player's own name of the site (sites). */
  site_name?: string | null;
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

/** A decision file with its review as the chronicle shows it (MG3). */
export interface ChronikEintrag {
  /** What the manager's way was forecast to bring more per year, if he objected (MG7). */
  objection_usd?: number | null;
  created: string;
  reviewed: string | null;
  due: string;
  topic: string;
  site: number;
  site_name: string | null;
  country: string;
  product: string | null;
  unit: string;
  manager_id: number;
  manager: string;
  recommended: string;
  chosen: string;
  against: boolean;
  delegated: boolean;
  forecast_usd: number | null;
  actual_usd: number | null;
  alternative_usd: number | null;
  unserved: number | null;
  hit: boolean | null;
}

/** What a manager remembers of an earlier decision he was right about (MG3). */
export interface Erinnerung {
  decided: string;
  reviewed: string;
  kind: string;
  actual_usd: number;
  alternative_usd: number | null;
}

export interface Organisation {
  /** False without manager data. */
  enabled: boolean;
  /** The decision files with their reviews, the latest first (MG3). */
  chronicle?: ChronikEintrag[];
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
  /** On the market less than a month (MG1). */
  new?: boolean;
  /** Days until AI companies may take him; null once they may (MG1). */
  ai_days?: number | null;
  /** Managers of other companies (MG1): the company, his satisfaction level (0 unhappy, 1
   * mixed, 2 happy) and the day the pause after the last offer to him ends. */
  company?: string | null;
  satisfaction?: number | null;
  blocked_until?: string | null;
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
  /** Running cost per year (MG4). */
  yearly_usd: number;
  /** Risk 0–1 (MG4). */
  risk: number;
  /** Months until the effect shows; 0 for doing nothing (MG4). */
  months: number;
  /** The manager who stands for the option where two contradict (MG4). */
  advocate: AnliegenFuersprecher | null;
}

/** Who stands for an option and from which topic (MG4). */
export interface AnliegenFuersprecher {
  manager: string;
  position: AnliegenStelle;
  /** Text `thema.<topic>`. */
  topic: string;
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
  /** The player's own name of the site (sites). */
  site_name?: string | null;
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
  /** The player's own name for the site, if any. */
  site_name?: string | null;
  country: string | null;
  product: string | null;
  option: AnliegenOption;
}

export interface Anliegen {
  id: number;
  /** The site the decision is about, if any. */
  site: number | null;
  site_kind_text: string | null;
  site_name?: string | null;
  site_country: string | null;
  /** The position that asks the player. */
  asker: AnliegenStelle;
  /** The positions on the way, the first first; empty where it asks itself (MA3). */
  path: AnliegenWeg[];
  /** The sites' parts of a strategic concern (MA3). */
  parts: AnliegenTeil[];
  /** The manager who asks. */
  manager: string;
  /** Recommendations of the manager who asks, judged so far and the hit rate (MG2). */
  manager_judged?: number;
  manager_hit_rate?: number | null;
  /** An earlier decision on the topic in which the manager was right (MG3). */
  memory?: Erinnerung | null;
  /** The latest files on the same topic at the same site (MG3). */
  earlier?: ChronikEintrag[];
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
  /** How sure the proposing manager is of his figures, 0–1 (strategy proposals). */
  certainty?: number | null;
  /** A proposal not yet put off can be postponed once. */
  can_postpone?: boolean;
  /** The style of the asking manager and the reason he favours in his words (MG7). */
  style?: string | null;
  favorite?: Meldung | null;
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
  /** The player's own name of the site (sites). */
  site_name?: string | null;
  country: string | null;
  continent: string;
  /** The branch of the works the position is at (MG5). */
  branch?: string | null;
  /** Free candidates, those of the site's continent first. */
  candidates: Kandidat[];
  /** The company's managers on other positions. */
  own: Kandidat[];
  /** Managers of other companies the player could poach (MG1). */
  others?: Kandidat[];
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
  | { Overhaul: number }
  | { Environment: boolean };

export type Vorgabefeld =
  | "Price"
  | "Stock"
  | "Wages"
  | "Supply"
  | "Investment"
  | "Reserve"
  | "Training"
  | "Environment"
  | "Overhaul";

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
    | "umwelt"
    | "ueberholung";
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
  /** The player's own name for the site, if any. */
  site_name?: string | null;
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
  /** The release cells, sorted by scope and branch (MG10). */
  release?: {
    scope: "Company" | { Continent: string } | { Country: string };
    branch: string | null;
    stage: "Free" | "Ask" | "Hold" | "Locked";
  }[];
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
  site_name?: string | null;
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
  release: FreigabeZelle[];
  review: "monatlich" | "quartalsweise" | "halbjaehrlich" | "jaehrlich";
}

/** The stage of a branch in a scope of the release matrix (MG10). */
export type Freigabestufe = "frei" | "rueckfrage" | "halten" | "gesperrt";

/** One cell of the branch release matrix as the view shows it (MG10). */
export interface FreigabeZelle {
  scope: "konzern" | "kontinent" | "land";
  /** Key of the continent or country; null for the company. */
  key: string | null;
  /** Key of the branch; null for all other branches. */
  branch: string | null;
  stage: Freigabestufe;
}

/** The mandate to the board and the CEO's strategy reviews (MA5). */
export interface Ruecksprachen {
  enabled: boolean;
  mandate: AuftragSicht;
  guidelines: { key: string; aggressiveness: number }[];
  intervals: string[];
  groups: string[];
  countries: string[];
  /** Branches and continents of the release matrix (MG10). */
  branches: string[];
  continents: string[];
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
  site_name?: string | null;
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
  /** The player's own name for the site, if any. */
  site_name?: string | null;
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
  own_site_name?: string | null;
  partner: string;
  partner_country: string;
  per_day: number;
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
  /** Standing with suppliers and customers (LV2). */
  partnerships?: Partnerschaft[];
}

/** The standing with a business partner (LV2). */
export interface Partnerschaft {
  company: string;
  role: "lieferant" | "kunde";
  /** 0–1. */
  grade: number;
  /** The buyer's share of the supplier's turnover in the last 12 months. */
  turnover_share: number;
  /** Discount the partnership brings on an AI supplier's price. */
  discount: number;
}

export interface Vertragspartner {
  site: number;
  company: string;
  country: string;
  kind_text: string;
  free_per_day: number;
  suggested_price_usd: number;
  delivery_cost_usd: number;
  /** Quality of the seller's stock (buying only). */
  quality?: number | null;
  /** Standing of the seller's company with the own one (buying only). */
  partnership?: Partnerschaft | null;
}

/** A site of the company or group seen from another, with the way between them. */
export interface InternerStandort {
  site: number;
  name: string | null;
  kind_text: string;
  country: string;
  company: string;
  own_company: boolean;
  stock: number;
  output_per_day: number;
  /** What it can give now. */
  spare: number;
  /** Production cost per unit of its stock (the price within the group). */
  unit_value_usd: number | null;
  /** Freight per unit and days of the way (0 within a country), customs rate. */
  freight_per_unit_usd: number;
  travel_days: number;
  customs_rate: number;
}

/** A product a site draws from another site of the company or group. */
export interface InternerBezug {
  product: string;
  source: InternerStandort;
  mode: "fest" | "bedarf";
  per_day: number | null;
  days: number | null;
  demand_per_day: number;
  wanted_today: number;
  in_transit: number;
  delivered_month: number;
  delivered_last_month: number;
}

/** A source of a product for a site (#191). */
export interface Bezugsquelle {
  kind: "markt" | "import" | "staat" | "vertrag";
  site: number | null;
  company: string | null;
  country: string;
  quality: number;
  /** Price delivered to the site; of it freight and customs (paid by the seller). */
  price_usd: number;
  delivery_cost_usd: number;
  /** In stock now, or free a month for contracts; null: unlimited. */
  available: number | null;
}

export interface Bezugsquellen {
  site: number;
  product: string;
  unit: string;
  country: string;
  own_quality: number | null;
  /** What the site uses a month, and the defaults of a new contract. */
  need_per_day: number;
  contract_months: number;
  contract_penalty: number;
  /** Cheapest first. */
  sources: Bezugsquelle[];
}

export interface Vertragspartnerliste {
  site: number;
  product: string;
  unit: string;
  role: "verkauf" | "einkauf";
  own_per_day: number;
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
  /** The player's own name for the site, if any. */
  site_name?: string | null;
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
  /** The player's own name of a site. */
  site_name?: string | null;
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
  /** Make or buy per input of the own sites (BP4). */
  make_or_buy?: MakeOrBuyZeile[];
  /** Purchasing controlling (EC1). */
  purchasing?: Zukauf;
}

export interface ZukaufZahlen {
  quantity: number;
  quality: number;
  price_usd: number;
  spent_usd: number;
}

export interface ZukaufZeile {
  site: number;
  site_name: string | null;
  country: string;
  product: string;
  source: "markt" | "vertrag";
  month: ZukaufZahlen;
  year: ZukaufZahlen;
  own_usd: number | null;
  own_vs_price: number | null;
}

/** What the company bought in the last month and the last 12 months (EC1). */
export interface Zukauf {
  rows: ZukaufZeile[];
  month_usd: number;
  year_usd: number;
}

export interface MakeOrBuyZeile {
  site: number;
  country: string;
  product: string;
  own_usd: number | null;
  buy_usd: number | null;
  supply: number;
  advice: "halten" | "puffer" | "zukauf_pruefen";
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
  /** At a private school now. */
  private_school?: boolean;
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
  /** The person's yearly salary from the company, the suggestion and the most it may be
   * with co-owners; whether a salary may be set (PE3). */
  salary_usd?: number;
  salary_suggestion_usd?: number;
  salary_max_usd?: number | null;
  salary_possible?: boolean;
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
  /** Paid as owner of the main company led by a manager, not as CEO. */
  salary_as_owner?: boolean;
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
  /** Partner search, private schools and health care; none without them in the data. */
  private_life?: {
    seeking_partner: boolean;
    partner_cost_usd: number;
    partner_chance: number;
    school_cost_usd: number;
    school_education: number;
    health_care: boolean;
    care_cost_usd: number;
    care_mortality: number;
  } | null;
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

/** A site a facility can be moved to (free within the company, sold within the group). */
export interface Verlegeziel {
  site: number;
  name: string | null;
  country: string;
  company: string;
  same_company: boolean;
}

export interface MaschinenAngebot {
  id: number;
  builder: string;
  builder_country: string;
  received: string;
  valid_until: string;
  price_usd: number;
  catalog_price_usd: number;
  build_days: number;
  catalog_build_days: number;
  ready: string;
  factors: { output: number; labor: number; inputs: number; scrap: number; area: number };
  area_unit_m2: number;
  area_total_m2: number;
  free_before_m2: number | null;
  free_after_m2: number | null;
  materials: { product: string; quantity: number; price_usd: number }[];
  material_usd: number;
  unit_cost_usd: number;
  catalog_unit_cost_usd: number;
}

export interface MaschinenAusschreibung {
  id: number;
  site: number;
  kind: "bauen" | "umbauen";
  product: string;
  size: string;
  count: number;
  slot: number | null;
  start: string;
  day: number;
  days: number;
  ends: string;
  offers: MaschinenAngebot[];
}

/** A project leader (PD2). */
export interface Projektleiter {
  id: number;
  name: string;
  home: string;
  age: number;
  /** `optimistisch`, `vorsichtig`, `sachlich` … */
  character: string;
  experience: number;
  finished: number;
  aborted: number;
  /** Works for the player's company (else on the market). */
  hired: boolean;
  /** Leads a running project. */
  busy: boolean;
  /** Mean value of the after-calculations of the projects; none before the first (PD6). */
  balance: number | null;
  balance_count: number;
}

/** Staffing of a role in the current phase of a project (PD2). */
export interface Projektrolle {
  role: string;
  need: number;
  assigned: number;
  /** People of the role in the laboratory not working on any project. */
  free: number;
}

/** What the rest of the project takes with `delta` people per role more or fewer than needed. */
export interface Projektszenario {
  delta: number;
  days: number;
  risk: number;
  penalty: number;
}

/** The figures a milestone report shows (PD3). */
export interface Projektkennzahlen {
  unit_cost: number;
  price: number;
  volume_year: number;
  revenue_year: number;
  lifecycle_revenue: number;
  contribution_year: number;
  plants: number;
  investment: number;
  project_cost: number;
  project_days: number;
  people: number;
  payback_years: number;
  quality: number;
}

/** A target profile (PM1) or solution path (PM2) with its factors (PD3). */
export interface Projektwahl {
  key: string;
  cost: number;
  time: number;
  risk: number;
  quality: number;
  preferred: boolean;
}

/** Decision keys of a milestone (PD3). */
export type Projektentscheidung = "weiter" | "auflagen" | "schleife" | "pause" | "abbruch";

/** A report at a milestone (PD3). */
export interface Meilensteinbericht {
  milestone: number;
  date: string;
  kpis: Projektkennzahlen;
  baseline: Projektkennzahlen | null;
  /** Relative deviation from the baseline. */
  deviation: Projektkennzahlen | null;
  /** `gruen`, `gelb` or `rot`: schedule, budget, quality, risk. */
  lights: string[];
  recommendation: Projektentscheidung;
  causes: string[];
  /** The voices of the gate council at PM3, PM4 and PM7 (PD7). */
  council: Gremiumstimme[];
  /** The council turned the leader's decision back a step. */
  council_changed: boolean;
  /** Decision taken and whether the player took it. */
  decided: [Projektentscheidung, boolean] | null;
}

/** One voice of the gate council (PD7). */
export interface Gremiumstimme {
  role: "finanzen" | "technik" | "markt";
  stance: "dafuer" | "bedenken" | "dagegen";
  /** The voice stands against the other two. */
  dissent: boolean;
}

/** A decision a project waits for (PD3). */
export interface Projektentscheid {
  milestone: number;
  deadline: string;
  recommendation: Projektentscheidung;
  choices: Projektwahl[];
  /** `profil`, `weg` or empty. */
  choice_kind: string;
  /** What the leader proposes with this milestone (PD7). */
  proposal: "fast_track" | "pilot" | "";
}

/** One option of a project event (PD4). */
export interface Ereignisoption {
  key: string;
  /** Extra cost as a share of the phase's cost. */
  cost: number;
  /** Share of the phase lost; negative: gained. */
  delay: number;
  quality: number;
}

/** An event a project waits for to be decided (PD4). */
export interface Projektereignis {
  key: string;
  deadline: string;
  /** Index of the option the leader recommends. */
  recommended: number;
  options: Ereignisoption[];
}

/** One development project of the company (PD2). */
export interface Projekt {
  id: number;
  product: string;
  site: number;
  site_name: string | null;
  lead: string;
  lead_id: number;
  /** Head start from partial knowledge and known components (PD8). */
  head_start: number;
  platform: number;
  license_from: string | null;
  partner: string | null;
  can_partner: boolean;
  /** Phases 3 and 4 run overlapped (PD7). */
  fast_track: boolean;
  /** A pilot series: planned or done (PD7). */
  pilot: "geplant" | "fertig" | "";
  /** What the suggestion promised, frozen at the start (PD6). */
  promise: Projektkennzahlen | null;
  tempo: string;
  trim: string;
  /** `laufend`, `freigegeben` or `abgebrochen`. */
  status: "laufend" | "freigegeben" | "abgebrochen";
  /** Milestone reached last (0 = started) and the next one. */
  milestone: number;
  next_milestone: number | null;
  /** Share of the duration done, 0–1. */
  progress: number;
  started: string;
  finished: string | null;
  days_left: number;
  risk: number;
  penalty: number;
  spent_usd: number;
  budget_usd: number;
  auto: boolean;
  roles: Projektrolle[];
  scenarios: Projektszenario[];
  profile: string;
  path: string;
  conditions: number;
  loops: number;
  paused: boolean;
  /** `alles`, `standard` or `leiter`. */
  delegation: string;
  pending: Projektentscheid | null;
  reports: Meilensteinbericht[];
  event: Projektereignis | null;
}

/** A product a project can start for, at normal tempo and standard trim (PD2). */
export interface Projektangebot {
  product: string;
  effort: number;
  cost_usd: number;
  days: number;
}

/** Know-how of a released project of another company for sale (PD8). */
export interface Lizenzangebot {
  project: number;
  product: string;
  seller: string;
  price_usd: number;
  fee_usd: number;
  fee_years: number;
  from_milestone: number;
}

/** An AI company that can be asked to be the partner of a project (PD8). */
export interface Partnerfirma {
  company: number;
  name: string;
}

/** A product suggestion of the strategy (PD6). */
export interface Produktvorschlag {
  id: number;
  product: string;
  site: number;
  created: string;
  expires: string;
  status: "offen" | "gestartet" | "abgelehnt" | "verfallen";
  /** `branche` or `luecke`. */
  reason: string;
  forecast: Projektkennzahlen;
  /** Value of the after-calculation (0–1) and whether it was a hit. */
  review: [number, boolean] | null;
}

/** An approach the analysis of an improvement project found (PD5). */
export interface Ansatz {
  gain: number;
  cost: number;
}

/** An improvement project WM0–WM3 (PD5). */
export interface Verbesserung {
  id: number;
  product: string;
  target: string;
  site: number;
  site_name: string | null;
  lead: string;
  /** 0–2 while running, 3 once released. */
  phase: number;
  progress: number;
  status: "laufend" | "freigegeben" | "abgebrochen";
  spent_usd: number;
  started: string;
  finished: string | null;
  approaches: Ansatz[];
  chosen: number | null;
  result: number | null;
  expected: number | null;
  /** `ansatz` or `freigabe`: what the project waits for. */
  pending: "ansatz" | "freigabe" | null;
  deadline: string | null;
}

/** A step a company can take in an improvement (PD5). */
export interface Verbesserungsangebot {
  product: string;
  target: string;
  stages: number;
  effort: number;
  cost_usd: number;
  days: number;
}

/** The development projects of the player's company (PD2). */
export interface Entwicklungsprojekte {
  tempos: string[];
  trims: string[];
  salary_usd_year: number;
  delegation: string;
  player_at: number[];
  leads: Projektleiter[];
  projects: Projekt[];
  offers: Projektangebot[];
  improvements: Verbesserung[];
  improvement_offers: Verbesserungsangebot[];
  improvement_stages: number;
  improvement_targets: string[];
  suggestions: Produktvorschlag[];
  license_offers: Lizenzangebot[];
  partners: Partnerfirma[];
  /** Product and share kept from aborted projects. */
  knowledge: [string, number][];
  suggestion_hit_rate: number | null;
}
