import { describe, expect, it } from "vitest";
import { zahlLesen } from "./format";
import { zahlEingeben, zahlGlaetten, type Eingabeart, type Zahlart } from "./zahleingabe";

// Fields are written with their caret "|" or a selection "[…]".
function zerlegen(feld: string): { text: string; a: number; b: number } {
  const a = feld.search(/[|[]/);
  const text = feld.replace(/[|[\]]/g, "");
  return { text, a, b: feld.includes("[") ? feld.indexOf("]") - 1 : a };
}

function zeigen(r: { text: string; cursor: number }): string {
  return `${r.text.slice(0, r.cursor)}|${r.text.slice(r.cursor)}`;
}

/** Types or pastes `dazu` at the caret or over the selection, as a browser would. */
function eingeben(feld: string, dazu: string, art?: Zahlart, mitAuswahl = true): string {
  const { text, a, b } = zerlegen(feld);
  const neu = text.slice(0, a) + dazu + text.slice(b);
  const eingabe: Eingabeart = mitAuswahl ? { auswahl: [a, b], typ: "insertText" } : {};
  return zeigen(zahlEingeben(text, neu, a + dazu.length, art, eingabe));
}

/** Types each character in turn. */
function tippen(feld: string, zeichen: string, art?: Zahlart): string {
  return [...zeichen].reduce((f, z) => eingeben(f, z, art), feld);
}

/** Backspace ("zurueck") or Delete ("vor") at the caret or on the selection. */
function loeschen(feld: string, richtung: "zurueck" | "vor", art?: Zahlart): string {
  const { text, a, b } = zerlegen(feld);
  const [von, bis] = a < b ? [a, b] : richtung === "zurueck" ? [a - 1, a] : [a, a + 1];
  const typ = richtung === "zurueck" ? "deleteContentBackward" : "deleteContentForward";
  const neu = text.slice(0, von) + text.slice(bis);
  return zeigen(zahlEingeben(text, neu, von, art, { auswahl: [a, b], typ }));
}

const ganz: Zahlart = { ganzzahlig: true };
const prozent: Zahlart = { gruppieren: false };

describe("Zahlenfelder beim Tippen", () => {
  it("setzt Tausenderpunkte, während man tippt", () => {
    expect(tippen("|", "200000000")).toBe("200.000.000|");
    expect(tippen("|", "1234")).toBe("1.234|");
    expect(tippen("|", "1852,15")).toBe("1.852,15|");
    // The decimals are not grouped.
    expect(tippen("|", "0,000125")).toBe("0,000125|");
  });

  it("lässt den Cursor hinter derselben Ziffer", () => {
    expect(tippen("1.23|4", "5")).toBe("12.35|4");
    expect(tippen("|123", "4")).toBe("4|.123");
    expect(tippen("123|", "4")).toBe("1.234|");
    expect(tippen("1.|234", "9")).toBe("19|.234");
    expect(tippen("12.345|,5", "6")).toBe("123.456|,5");
    // Without the browser's selection: from the caret alone.
    expect(eingeben("1.23|4", "5", undefined, false)).toBe("12.35|4");
  });

  it("löscht über den Tausenderpunkt hinweg die Ziffer daneben", () => {
    expect(loeschen("1.|234", "zurueck")).toBe("|234");
    expect(loeschen("12.|345", "zurueck")).toBe("1|.345");
    expect(loeschen("1|.234", "vor")).toBe("1|34");
    expect(loeschen("1.234.5|67", "zurueck")).toBe("123.4|67");
    expect(loeschen("1.234|", "zurueck")).toBe("123|");
    expect(loeschen("1.[23]4", "zurueck")).toBe("1|4");
  });

  it("behält Nullen hinter dem Cursor, damit eine neue erste Ziffer davor passt", () => {
    expect(loeschen("1|.000.000", "zurueck")).toBe("|000000");
    expect(tippen("|000000", "2")).toBe("2|.000.000");
    // Zeros in front of the caret go.
    expect(tippen("0|", "5")).toBe("5|");
    expect(tippen("|5", "0")).toBe("|5");
    expect(tippen("0|", "0")).toBe("0|");
    expect(tippen("0,|5", "1")).toBe("0,1|5");
  });

  it("nimmt das Komma als Dezimaltrennzeichen, nur einmal", () => {
    expect(tippen("12|", ",5")).toBe("12,5|");
    expect(tippen("|", ",5")).toBe("0,5|");
    expect(tippen("1.2|34", ",")).toBe("12,|34");
    expect(tippen("1.234,5|", ",")).toBe("1.234,5|");
    expect(tippen("1.2|34,5", ",")).toBe("1.2|34,5");
  });

  it("liest einen getippten Punkt als Tausenderpunkt, nach einer Null als Komma", () => {
    // Typing the separators oneself gives the same as typing the digits.
    expect(tippen("|", "1.800.000")).toBe("1.800.000|");
    expect(tippen("|", "2.400,50")).toBe("2.400,50|");
    // "0." can only begin a fraction (small prices in a strong currency).
    expect(tippen("|", "0.005")).toBe("0,005|");
    expect(tippen("|", ".5")).toBe("0,5|");
  });

  it("nimmt nur Ziffern, ohne Minus, wo keine negativen Zahlen erlaubt sind", () => {
    expect(tippen("12|", "a")).toBe("12|");
    expect(tippen("|12", "-")).toBe("|12");
    expect(tippen("1.2|34", " ")).toBe("1.2|34");
    expect(eingeben("|", "-5")).toBe("|");
  });

  it("erlaubt ein Minus vorn, wo negative Zahlen erlaubt sind", () => {
    const negativ: Zahlart = { negativ: true };
    expect(tippen("|", "-1234", negativ)).toBe("-1.234|");
    expect(tippen("-1.2|34", "-", negativ)).toBe("-1.2|34");
    expect(tippen("1.2|34", "-", negativ)).toBe("1.2|34");
    expect(tippen("|5.000", "-", negativ)).toBe("-|5.000");
    expect(loeschen("-|5.000", "zurueck", negativ)).toBe("|5.000");
    expect(eingeben("|", "-2.500,5", negativ)).toBe("-2.500,5|");
  });

  it("nimmt in Feldern für ganze Zahlen kein Komma", () => {
    expect(tippen("|", "1234", ganz)).toBe("1.234|");
    expect(tippen("12|", ",5", ganz)).toBe("125|");
    expect(tippen("12|", ".", ganz)).toBe("12|");
    // Pasted decimals are cut off, as the forms rounded down before.
    expect(eingeben("[8]", "8,5", ganz)).toBe("8|");
  });

  it("schreibt in Prozentfeldern ohne Tausenderpunkte, mit Komma oder Punkt", () => {
    expect(tippen("|", "12,5", prozent)).toBe("12,5|");
    expect(tippen("|", "12.5", prozent)).toBe("12,5|");
    expect(tippen("|", "1000", prozent)).toBe("1000|");
    expect(tippen("|", "0.25", prozent)).toBe("0,25|");
  });
});

describe("Zahlenfelder beim Einfügen und Ausfüllen", () => {
  it("liest eingefügte Ziffern und deutsch geschriebene Zahlen", () => {
    expect(eingeben("|", "200000000")).toBe("200.000.000|");
    expect(eingeben("[1.852,15]", "200000000")).toBe("200.000.000|");
    expect(eingeben("[37.200]", "2.400,50")).toBe("2.400,50|");
    expect(eingeben("|", "1.800.000")).toBe("1.800.000|");
    expect(eingeben("|", "12,5 %")).toBe("12,5|");
    expect(eingeben("|", "2 400")).toBe("2.400|");
    expect(eingeben("|", "1.800 €")).toBe("1.800|");
    expect(eingeben("1|000", "234")).toBe("1.234|.000");
  });

  it("liest einen Punkt, der kein Tausenderpunkt sein kann, als Komma (wie zahlLesen)", () => {
    // Filled in whole: it does not matter that the old text began with "1.".
    expect(eingeben("[1.852,15]", "1.5")).toBe("1,5|");
    expect(eingeben("[2.037,37]", "2.500")).toBe("2.500|");
    expect(eingeben("|", "12.75")).toBe("12,75|");
    expect(eingeben("|", "0.005")).toBe("0,005|");
  });

  it("ergibt dieselbe Zahl wie zahlLesen auf dem ganzen Text", () => {
    for (const text of ["200000000", "2.500", "2.400,50", "12,5", "1.5", "0.005", "50000"]) {
      const feld = eingeben("[1.852,15]", text).replace("|", "");
      expect(zahlLesen(feld)).toBe(zahlLesen(text));
    }
  });

  it("leert das Feld", () => {
    expect(loeschen("[1.852,15]", "vor")).toBe("|");
  });
});

describe("Zahlenfelder bei zufälligen Eingaben", () => {
  /** Random edits (typing, Backspace, Delete, over selections) from a fixed seed. */
  function zufallsEingaben(art: Zahlart, muster: RegExp) {
    let zufall = 7;
    const wuerfeln = (n: number) => {
      zufall = (zufall * 1103515245 + 12345) % 2 ** 31;
      return zufall % n;
    };
    let text = "";
    for (let i = 0; i < 3000; i++) {
      const a = wuerfeln(text.length + 1);
      const b = wuerfeln(4) === 0 ? a + wuerfeln(text.length - a + 1) : a;
      const aktion = wuerfeln(15);
      let von = a;
      let bis = b;
      let dazu = "";
      let typ = "insertText";
      if (aktion < 13) dazu = "0123456789,.-"[aktion]!;
      else if (aktion === 13) {
        typ = "deleteContentBackward";
        if (a === b) von = Math.max(0, a - 1);
      } else {
        typ = "deleteContentForward";
        if (a === b) bis = Math.min(text.length, a + 1);
      }
      const neu = text.slice(0, von) + dazu + text.slice(bis);
      const r = zahlEingeben(text, neu, von + dazu.length, art, { auswahl: [a, b], typ });
      expect(r.text).toMatch(muster);
      expect(r.cursor).toBeGreaterThanOrEqual(0);
      expect(r.cursor).toBeLessThanOrEqual(r.text.length);
      // The field never shows text that `zahlLesen` would read otherwise than its digits.
      if (/\d/.test(r.text))
        expect(zahlLesen(r.text)).toBe(Number(r.text.replace(/\./g, "").replace(",", ".")));
      text = r.text;
    }
  }

  it("bleibt lesbar und eindeutig", () => {
    zufallsEingaben({}, /^(0|[1-9]\d{0,2}(\.\d{3})*|0\d+)?(,\d*)?$/);
    zufallsEingaben(ganz, /^(0|[1-9]\d{0,2}(\.\d{3})*|0\d+)?$/);
    zufallsEingaben(prozent, /^\d*(,\d*)?$/);
    zufallsEingaben({ negativ: true }, /^-?(0|[1-9]\d{0,2}(\.\d{3})*|0\d+)?(,\d*)?$/);
  });
});

describe("Zahlenfelder beim Verlassen", () => {
  it("räumt Zwischenstände auf, ohne die Zahl zu ändern", () => {
    expect(zahlGlaetten("000000")).toBe("0");
    expect(zahlGlaetten("0001234")).toBe("1.234");
    expect(zahlGlaetten(",5")).toBe("0,5");
    expect(zahlGlaetten("12,")).toBe("12");
    expect(zahlGlaetten("1.852,15")).toBe("1.852,15");
    expect(zahlGlaetten("12,50", prozent)).toBe("12,50");
    expect(zahlGlaetten("")).toBe("");
    expect(zahlGlaetten("-")).toBe("-");
  });
});
