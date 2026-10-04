// Display formats (German). Only formatting, no game values are computed here.
import type { Meldung, Parameter } from "./kern";
import { t } from "./texte";

const zahl = (stellen: number) =>
  new Intl.NumberFormat("de-DE", { maximumFractionDigits: stellen, minimumFractionDigits: 0 });

export function formatZahl(wert: number, stellen = 0): string {
  return zahl(stellen).format(wert);
}

/** Amounts in USD: whole dollars, from a million shortened (Mio., Mrd.). */
export function formatGeld(usd: number): string {
  const betrag = Math.abs(usd);
  if (betrag >= 1e9) return `${zahl(2).format(usd / 1e9)} Mrd. USD`;
  if (betrag >= 1e6) return `${zahl(2).format(usd / 1e6)} Mio. USD`;
  return `${zahl(betrag < 100 ? 2 : 0).format(usd)} USD`;
}

export function formatProzent(anteil: number): string {
  return `${zahl(0).format(anteil * 100)} %`;
}

/** 1900-01-31 → 31.01.1900 */
export function formatDatum(iso: string): string {
  const [jahr, monat, tag] = iso.split("-");
  return `${tag}.${monat}.${jahr}`;
}

export function landName(schluessel: string): string {
  return t(`land.${schluessel}`);
}

function parameterText(p: Parameter): string | number {
  switch (p.type) {
    case "text":
      return p.value;
    case "integer":
      return p.value;
    case "number":
      return formatZahl(p.value, 2);
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
