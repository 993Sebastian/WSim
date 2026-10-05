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
export type MeldungsGruppe = "welt" | "wettbewerb" | "warnung" | "forschung" | "allgemein";

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
}

export interface Spielstand {
  name: string;
  date: string;
  company: string;
}

export type Rundenlaenge = "tag" | "woche" | "monat" | "quartal";

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
}

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
}
