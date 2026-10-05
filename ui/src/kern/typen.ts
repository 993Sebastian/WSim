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
}

export interface Ursache {
  key: string;
  detail: string | null;
}

export interface AnlageDetail {
  index: number;
  facility: string;
  count: number;
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
  wage_premium: number;
  wage_premium_max: number;
  rival_premium_max: number;
  staff: Personal[];
  wage_cost_per_day_usd: number;
  unit_costs: Stueckkosten[];
  last_month: StandortErgebnis | null;
  /** Market and reference price in the site's country, by product. */
  prices: Record<string, { market_usd: number; reference_usd: number }>;
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
  centers: {
    site: number;
    country: string;
    researchers: number;
    project: string | null;
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
  /** Base value G = max(earnings value, liquidation) + under construction + stocks. */
  base_usd: number;
  earnings_years: number;
}

/** The object of an offer. */
export interface Gegenstandssicht {
  kind: "standort" | "lizenz";
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
}

export interface Firmen {
  companies: Firmenzeile[];
}

export interface FremderStandort {
  site: number;
  site_type: string;
  country: string;
  facilities: [string, number][];
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
}

export interface Lizenzmoeglichkeit {
  technology: string;
  value_usd: number;
  open_offer: number | null;
  blocked_until: string | null;
}

export interface Firmendetail {
  company: Firmenzeile;
  sites: FremderStandort[];
  licenses: Lizenzmoeglichkeit[];
  cash_usd: number;
  min_age_months: number;
}
