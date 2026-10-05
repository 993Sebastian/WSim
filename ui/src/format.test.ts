import { describe, expect, it } from "vitest";
import { formatDatum, formatGeld, formatMenge, meldungText, zahlFeld, zahlLesen } from "./format";

describe("Formate", () => {
  it("kürzt große Beträge", () => {
    expect(formatGeld(1234)).toBe("1.234 USD");
    expect(formatGeld(12.5)).toBe("12,5 USD");
    expect(formatGeld(2_500_000)).toBe("2,5 Mio. USD");
    expect(formatGeld(-3_100_000_000)).toBe("-3,1 Mrd. USD");
  });

  it("zeigt kleine Mengen mit drei gültigen Stellen", () => {
    expect(formatMenge(0.0213)).toBe("0,0213");
    expect(formatMenge(8.333)).toBe("8,33");
    expect(formatMenge(57)).toBe("57");
    expect(formatMenge(1234.6)).toBe("1.235");
    expect(formatMenge(0)).toBe("0");
  });

  it("liest Zahlen, wie man sie deutsch eintippt", () => {
    expect(zahlLesen("1.800")).toBe(1800);
    expect(zahlLesen("1.800.000")).toBe(1_800_000);
    expect(zahlLesen("1,5")).toBe(1.5);
    expect(zahlLesen("1.800,50")).toBe(1800.5);
    expect(zahlLesen(" 2 400 ")).toBe(2400);
    expect(zahlLesen("12,5 %")).toBe(12.5);
    // A point that cannot separate thousands is a decimal point.
    expect(zahlLesen("1.5")).toBe(1.5);
    expect(zahlLesen("12.75")).toBe(12.75);
    expect(zahlLesen(",5")).toBe(0.5);
    expect(zahlLesen("-3")).toBe(-3);
    expect(zahlLesen("")).toBeNull();
    expect(zahlLesen("abc")).toBeNull();
    expect(zahlLesen("1,2,3")).toBeNull();
    expect(zahlLesen("1.800.5")).toBeNull();
  });

  it("schreibt Zahlen für Eingabefelder ohne Tausenderpunkt", () => {
    expect(zahlFeld(1800.5)).toBe("1800,5");
    expect(zahlFeld(0.125, 3)).toBe("0,125");
    expect(zahlLesen(zahlFeld(1234567.89))).toBe(1234567.89);
  });

  it("schreibt Daten deutsch", () => {
    expect(formatDatum("1900-01-31")).toBe("31.01.1900");
  });

  it("setzt Meldungsparameter nach ihrem Typ ein", () => {
    const text = meldungText({
      kind: "warning",
      group: "warnung",
      key: "fehler.spielstart.startform",
      params: { betrag: { type: "money", value: 60000 } },
      target: null,
    });
    expect(text).toContain("60.000 USD");
  });

  it("zeigt Länderlisten mit Namen", () => {
    const text = meldungText({
      kind: "world_event",
      group: "welt",
      key: "weltereignis.laender",
      params: { laender: { type: "countries", value: ["DEU", "FRA"] } },
      target: null,
    });
    expect(text).toBe("Betroffene Länder");
  });
});
