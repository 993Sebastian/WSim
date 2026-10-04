import { describe, expect, it } from "vitest";
import { formatDatum, formatGeld, meldungText } from "./format";

describe("Formate", () => {
  it("kürzt große Beträge", () => {
    expect(formatGeld(1234)).toBe("1.234 USD");
    expect(formatGeld(12.5)).toBe("12,5 USD");
    expect(formatGeld(2_500_000)).toBe("2,5 Mio. USD");
    expect(formatGeld(-3_100_000_000)).toBe("-3,1 Mrd. USD");
  });

  it("schreibt Daten deutsch", () => {
    expect(formatDatum("1900-01-31")).toBe("31.01.1900");
  });

  it("setzt Meldungsparameter nach ihrem Typ ein", () => {
    const text = meldungText({
      kind: "warning",
      key: "fehler.spielstart.startform",
      params: { betrag: { type: "money", value: 60000 } },
      target: null,
    });
    expect(text).toContain("60.000 USD");
  });
});
