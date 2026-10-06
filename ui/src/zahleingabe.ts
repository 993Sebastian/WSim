// Number fields that set thousands separators while one types ("200.000.000"; docs/
// BEDIENUNG.md, rule 5). Only the text of a field is shaped here: the number it means is
// read with `zahlLesen`, as before.
import { zahlTeile, type Zahlteile } from "./format";

/** What a number field accepts. */
export interface Zahlart {
  /** Whole numbers only (counts, years): no decimal comma. */
  ganzzahlig?: boolean;
  /** A leading minus sign. */
  negativ?: boolean;
  /**
   * Thousands separators (the default). Off for small numbers such as percentages and
   * for numbers that name something (a year, the seed).
   */
  gruppieren?: boolean;
}

/** The text of a field and where its caret stands. */
export interface Feldstand {
  text: string;
  cursor: number;
}

/** What the browser reports before an edit (`beforeinput`). */
export interface Eingabeart {
  /** The selection the edit replaces, [start, end). */
  auswahl?: readonly [number, number];
  /** `InputEvent.inputType`, e.g. "deleteContentForward". */
  typ?: string;
}

/**
 * The field after an edit: `alt` is the text before, `neu` and `cursor` what the browser
 * made of it. Digits are grouped in thousands, the caret stays behind the same digits.
 * A typed point only separates thousands (the field sets them anyway), unless it cannot:
 * after a lone zero and in fields without separators it is the decimal comma. Text
 * pasted or filled in at once is read as a whole, like `zahlLesen` ("1.5" is 1,5).
 */
export function zahlEingeben(
  alt: string,
  neu: string,
  cursor: number,
  art: Zahlart = {},
  eingabe: Eingabeart = {},
): Feldstand {
  const [p, s] = bereich(alt, neu, cursor, eingabe.auswahl);
  let vorne = alt.slice(0, p);
  let hinten = alt.slice(alt.length - s);
  const weg = alt.slice(p, alt.length - s);
  const dazu = neu.slice(p, neu.length - s);
  // Deleting a thousands separator alone would change nothing: the digit beside it goes.
  if (dazu === "" && weg === ".") {
    if (eingabe.typ === "deleteContentForward") hinten = hinten.slice(1);
    else vorne = vorne.slice(0, -1);
  }
  return zusammensetzen(vorne, einfuegung(dazu, vorne, hinten, art), hinten, art);
}

/**
 * The text a field shows once one leaves it: without leading zeros or a lone comma
 * ("05" → "5", ",5" → "0,5", "12," → "12"). Text that is no number stays as it is.
 */
export function zahlGlaetten(text: string, art: Zahlart = {}): string {
  const z = zahlTeile(text);
  if (!z) return text;
  const komma = z.komma && z.bruch !== "";
  const ganz = z.ganz.replace(/^0+(?=\d)/, "") || "0";
  return ausgeben({ ...z, ganz, komma, bruch: komma ? z.bruch : "" }, 0, art).text;
}

/**
 * Which part of `alt` an edit replaced: `alt[p, alt.length - s)` became
 * `neu[p, neu.length - s)`.
 */
function bereich(
  alt: string,
  neu: string,
  cursor: number,
  auswahl?: readonly [number, number],
): [number, number] {
  // The selection before the edit tells what a replaced text was (a field filled in
  // whole may begin like the old text: "1.852,15" → "1.5").
  if (auswahl) {
    const [a, b] = auswahl;
    const s = alt.length - b;
    if (
      a >= 0 &&
      a <= b &&
      s >= 0 &&
      cursor >= a &&
      cursor === neu.length - s &&
      neu.startsWith(alt.slice(0, a)) &&
      neu.endsWith(alt.slice(b))
    )
      return [a, s];
  }
  // Otherwise: behind the caret nothing changed, in front of it what both begin with.
  let s = neu.length - cursor;
  if (s < 0 || s > alt.length || !alt.endsWith(neu.slice(cursor))) {
    s = 0;
    while (s < alt.length && s < neu.length && alt.at(-1 - s) === neu.at(-1 - s)) s++;
  }
  let p = 0;
  const bis = Math.min(alt.length, neu.length) - s;
  while (p < bis && alt[p] === neu[p]) p++;
  return [p, s];
}

/** What the inserted text means: digits, at most one comma and a leading minus. */
function einfuegung(dazu: string, vorne: string, hinten: string, art: Zahlart): string {
  if (dazu === "") return "";
  if (dazu.length === 1) {
    if (dazu >= "0" && dazu <= "9") return dazu;
    if (dazu === "-") return art.negativ ? "-" : "";
    if (dazu !== "," && dazu !== ".") return "";
    if (art.ganzzahlig || (vorne + hinten).includes(",")) return "";
    const ganzVorne = vorne.replace(/\D/g, "");
    if (dazu === "." && art.gruppieren !== false && /[1-9]/.test(ganzVorne)) return "";
    return ganzVorne === "" ? "0," : ",";
  }
  const z = zahlTeile(dazu.replace(/\u2212/g, "-").replace(/[^\d.,-]/g, ""));
  if (!z) return dazu.replace(/\D/g, "");
  // A sign the field cannot take is no typo to drop: nothing is inserted.
  if (z.minus && !art.negativ) return "";
  const vorzeichen = z.minus ? "-" : "";
  // Whole numbers: the decimals are cut off, as the forms did before.
  if (art.ganzzahlig) return vorzeichen + z.ganz;
  return vorzeichen + z.ganz + (z.komma ? `,${z.bruch}` : "");
}

/** Puts old and inserted text together; the caret goes behind the same digits as before. */
function zusammensetzen(vorne: string, mitte: string, hinten: string, art: Zahlart): Feldstand {
  // The old text's points only grouped; its comma stays the decimal comma.
  const kommaAlt = (vorne + hinten).includes(",");
  const z: Zahlteile = { minus: false, ganz: "", komma: false, bruch: "" };
  let vorCursor = 0; // digits and comma in front of the caret
  const lesen = (text: string, eingefuegt: boolean, zaehlen: boolean) => {
    for (const c of text) {
      if (c >= "0" && c <= "9") {
        if (z.komma) z.bruch += c;
        else z.ganz += c;
      } else if (c === "," && !z.komma && !art.ganzzahlig && !(eingefuegt && kommaAlt)) {
        z.komma = true;
      } else {
        // The old minus stands in front; a typed one only where nothing precedes it.
        if (c === "-" && art.negativ && (!eingefuegt || (z.ganz === "" && !z.komma)))
          z.minus = true;
        continue;
      }
      if (zaehlen) vorCursor++;
    }
  };
  lesen(vorne, false, true);
  lesen(mitte, true, true);
  const ganzVorCursor = z.ganz.length;
  lesen(hinten, false, false);
  // Zeros in front of the caret that precede other digits go ("0" + "5" is "5"); those
  // behind it stay, so that a new first digit can still be typed ("|000" → "2|000").
  let nullen = 0;
  while (nullen < ganzVorCursor && z.ganz.length - nullen > 1 && z.ganz[nullen] === "0") nullen++;
  z.ganz = z.ganz.slice(nullen);
  return ausgeben(z, vorCursor - nullen, art);
}

/** The text of a number as the field shows it, with the caret behind `vorCursor` digits. */
function ausgeben(z: Zahlteile, vorCursor: number, art: Zahlart): Feldstand {
  // Leading zeros (left when a first digit was deleted) stay ungrouped: "0.005" would
  // read as a fraction.
  const gruppiert = art.gruppieren !== false && !/^0\d/.test(z.ganz);
  let text = z.minus ? "-" : "";
  let cursor = text.length;
  let gezaehlt = 0;
  const schreiben = (c: string, zaehlt: boolean) => {
    text += c;
    if (zaehlt && ++gezaehlt === vorCursor) cursor = text.length;
  };
  [...z.ganz].forEach((c, i) => {
    if (gruppiert && i > 0 && (z.ganz.length - i) % 3 === 0) schreiben(".", false);
    schreiben(c, true);
  });
  if (z.komma) {
    schreiben(",", true);
    for (const c of z.bruch) schreiben(c, true);
  }
  return { text, cursor };
}
