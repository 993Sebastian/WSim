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
}

export interface NeuesSpiel {
  seed: number;
  start_year: number;
  country: string;
  capital_usd: number;
  start_form: string;
  company_name: string;
  companies: number;
  difficulty: string;
  research_factor: number;
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
  company: Firma;
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
  | { type: "countries"; value: string[] };

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
}

export interface Stelleninhaber {
  manager: Manager;
  salary_usd: number;
  since: string;
  /** What a dismissal costs now. */
  severance_usd: number;
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
}

export interface StandortOrganisation {
  site: number;
  /** Text key of the site type. */
  kind_text: string;
  country: string;
  positions: Stelle[];
  /** Next check of the positions; null without a manager there. */
  next_check: string | null;
  /** Topics nobody takes care of here: the player decides them. */
  own_topics: string[];
}

export interface Organisation {
  /** False without manager data. */
  enabled: boolean;
  continents: {
    continent: string;
    countries: { country: string; sites: StandortOrganisation[] }[];
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
}

export interface Kandidat {
  manager: Manager;
  /** Salary per year for the position. */
  demand_usd: number;
  /** An own manager's position now. */
  current: { site: number; role: string } | null;
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
export interface Anliegen {
  id: number;
  site: number;
  /** `leitung` or the function of a specialist position. */
  role: string;
  /** Text key of the site type. */
  kind_text: string;
  country: string;
  /** The manager who asks. */
  manager: string;
  /** Topic (text `thema.<topic>`). */
  topic: string;
  product: string | null;
  /** Why the position asks (text `anliegen.grund.<reason>`). */
  reason: "entscheidung" | "jahr" | "immer" | "kredit";
  options: AnliegenOption[];
  recommended: number;
  /** Why the position recommends its option. */
  because: Meldung;
  per_decision_usd: number;
  left_usd: number;
  /** Mean monthly result of the site in the last closed months. */
  site_result_usd: number;
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
  site: number;
  role: string;
  kind_text: string;
  country: string;
  continent: string;
  /** Free candidates, those of the site's continent first. */
  candidates: Kandidat[];
  /** The company's managers on other positions. */
  own: Kandidat[];
}
