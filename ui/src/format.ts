// Display formats (German). Only formatting, no game values are computed here.
import type { Geldanzeige, Meldung, Parameter } from "./kern";
import { t } from "./texte";

const zahl = (stellen: number) =>
  new Intl.NumberFormat("de-DE", { maximumFractionDigits: stellen, minimumFractionDigits: 0 });

export function formatZahl(wert: number, stellen = 0): string {
  return zahl(stellen).format(wert);
}

/** Areas in hectares; below one hectare in square meters, so that the few hundred square
 * meters of a small facility do not read as "0 ha". */
export function formatFlaeche(ha: number): string {
  if (ha > 0 && ha < 1) return `${zahl(0).format(ha * 10_000)} m²`;
  return `${zahl(ha < 100 ? 2 : 0).format(ha)} ha`;
}

/** Quantities of goods: small ones with three significant digits ("0,021 t"), large
 * ones whole. */
export function formatMenge(wert: number): string {
  if (Math.abs(wert) >= 100) return zahl(0).format(wert);
  return new Intl.NumberFormat("de-DE", { maximumSignificantDigits: 3 }).format(wert);
}

/** The game's own unit: US dollars of the base year's purchasing power. */
const SPIELDOLLAR: Geldanzeige = { currency: "us_dollar", symbol: "USD", factor: 1 };
let anzeige = SPIELDOLLAR;

/**
 * How amounts are shown (M21): the currency and the factor from game dollars, both
 * from the core. Null shows game dollars (outside a game). The game screen sets it
 * while drawing, so every amount of a screen is shown the same way.
 */
export function setzeGeldanzeige(a: Geldanzeige | null): void {
  anzeige = a ?? SPIELDOLLAR;
}

/** Game dollars in the shown currency. */
export function inAnzeige(usd: number): number {
  return usd * anzeige.factor;
}

/** An amount in the shown currency in game dollars (for commands). */
export function ausAnzeige(betrag: number): number {
  return betrag / anzeige.factor;
}

/** Symbol of the shown currency (units of input fields, e.g. "€/t"). */
export function geldEinheit(): string {
  return anzeige.symbol;
}

/**
 * Part of the key of forms that hold amounts: they start afresh when the way of
 * showing changes (another currency, or the prices of another month).
 */
export function geldSchluessel(): string {
  return `${anzeige.currency}@${anzeige.factor}`;
}

/** German short scales; amounts in hyperinflation reach Billiarden, the forint of 1946
 * was worth 400 Quadrilliarden pengő. */
const GROSS: [number, string][] = [
  [1e27, "Quadrilliarden"],
  [1e24, "Quadrillionen"],
  [1e21, "Trilliarden"],
  [1e18, "Trill."],
  [1e15, "Brd."],
  [1e12, "Bio."],
  [1e9, "Mrd."],
  [1e6, "Mio."],
];

/** Large numbers in German short scales ("4,2 Bio."); null below a million. */
function kurz(wert: number): string | null {
  const betrag = Math.abs(wert);
  for (const [ab, name] of GROSS) {
    if (betrag >= ab) return `${zahl(2).format(wert / ab)} ${name}`;
  }
  return null;
}

/** A number of any size: from a million in short scales, else as a quantity. */
export function formatZahlKurz(wert: number): string {
  return kurz(wert) ?? formatMenge(wert);
}

/**
 * Amounts (given in game dollars) in the shown currency: whole units from 100, from a
 * million shortened (Mio., Mrd., …); below one unit three significant digits, so that
 * small prices in a strong currency do not read as zero.
 */
export function formatGeld(usd: number): string {
  // Below half the core's resolution (a hundredth of a cent) it is rounding noise.
  const wert = Math.abs(usd) < 5e-5 ? 0 : inAnzeige(usd);
  const betrag = Math.abs(wert);
  const zeichen = anzeige.symbol;
  const gross = kurz(wert);
  if (gross) return `${gross} ${zeichen}`;
  if (betrag > 0 && betrag < 1) {
    const klein = new Intl.NumberFormat("de-DE", { maximumSignificantDigits: 3 });
    return `${klein.format(wert)} ${zeichen}`;
  }
  return `${zahl(betrag < 100 ? 2 : 0).format(wert)} ${zeichen}`;
}

export function formatProzent(anteil: number): string {
  return `${zahl(0).format(anteil * 100)} %`;
}

/** Amount (in game dollars) per unit, e.g. "1.234 USD/t". */
export function formatPreis(usd: number, einheit: string): string {
  return `${formatGeld(usd)}/${einheit}`;
}

/** A typed number in parts, its digits as typed: "1.800,50" is 1800 and 50. */
export interface Zahlteile {
  minus: boolean;
  /** Digits before the decimal separator (may be empty: ",5"). */
  ganz: string;
  /** Has a decimal separator (possibly without decimals: "12,"). */
  komma: boolean;
  bruch: string;
}

/** Reads typed text into its parts by the rules of `zahlLesen`; null if it is no number. */
export function zahlTeile(text: string): Zahlteile | null {
  const s = text
    .trim()
    .replace(/\s|\u00a0/g, "")
    .replace(/%$/, "");
  if (s === "") return null;
  let normal: string;
  if (s.includes(",")) normal = s.replace(/\./g, "").replace(",", ".");
  else if (/^-?[1-9]\d{0,2}(\.\d{3})+$/.test(s)) normal = s.replace(/\./g, "");
  else normal = s;
  if (!/^-?\d*\.?\d+$/.test(normal) && !/^-?\d+\.?$/.test(normal)) return null;
  const [, minus, ganz = "", komma, bruch = ""] = /^(-?)(\d*)(\.?)(\d*)$/.exec(normal) ?? [];
  return { minus: minus === "-", ganz, komma: komma === ".", bruch };
}

/**
 * Reads a number as typed in German: "1.800" is 1800, "1,5" is 1.5, "1.800,50" is
 * 1800.5. A point is a decimal point only where it cannot be a thousands separator
 * ("1.5", "12.75", "0.005"). Empty or invalid input gives null.
 */
export function zahlLesen(text: string): number | null {
  const z = zahlTeile(text);
  if (!z) return null;
  const wert = Number(`${z.minus ? "-" : ""}${z.ganz}${z.komma ? "." : ""}${z.bruch}`);
  return Number.isFinite(wert) ? wert : null;
}

/** A number for an input field as the field shows it, with thousands separators ("1.800,5"). */
export function zahlFeld(wert: number, stellen = 2): string {
  return new Intl.NumberFormat("de-DE", {
    maximumFractionDigits: stellen,
    useGrouping: true,
  }).format(wert);
}

/** Decimals of an amount in a field: two, small amounts three significant digits. */
function geldStellen(wert: number): number {
  if (wert === 0 || !Number.isFinite(wert)) return 2;
  return Math.min(20, Math.max(2, 2 - Math.floor(Math.log10(Math.abs(wert)))));
}

/**
 * An amount (in game dollars) for an input field in the shown currency: two decimals,
 * small amounts with three significant digits (a price of 0,00512 £ stays as it is).
 */
export function geldFeld(usd: number): string {
  const wert = inAnzeige(usd);
  return zahlFeld(wert, geldStellen(wert));
}

/** An amount in the shown currency rounded as a field shows it. */
export function geldRunden(wert: number): number {
  const faktor = 10 ** geldStellen(wert);
  return Math.round(wert * faktor) / faktor;
}

/** 1900-01-31 → 31.01.1900 */
export function formatDatum(iso: string): string {
  const [jahr, monat, tag] = iso.split("-");
  return `${tag}.${monat}.${jahr}`;
}

export function landName(schluessel: string): string {
  return t(`land.${schluessel}`);
}

const sechsStellen = new Intl.NumberFormat("de-DE", { maximumSignificantDigits: 6 });

function parameterText(p: Parameter): string | number {
  switch (p.type) {
    case "text":
      return p.value;
    case "integer":
      return p.value;
    case "number":
      // Up to six significant digits: rates fixed by law read 1,95583 or 1.936,27.
      return kurz(p.value) ?? sechsStellen.format(p.value);
    case "money":
      return formatGeld(p.value);
    case "date":
      return formatDatum(p.value);
    case "country":
      return landName(p.value);
    case "text_key":
      return t(p.value);
    case "countries":
      return p.value.map(landName).join(", ");
  }
}

/** One parameter of a message as display text (empty if the message has none). */
export function parameterAnzeige(m: Meldung, name: string): string {
  const p = m.params[name];
  return p === undefined ? "" : String(parameterText(p));
}

/** The text of a message from the core, with its parameters formatted. */
export function meldungText(m: Meldung): string {
  const parameter = Object.fromEntries(
    Object.entries(m.params).map(([name, p]) => [name, parameterText(p)]),
  );
  return t(m.key, parameter);
}
