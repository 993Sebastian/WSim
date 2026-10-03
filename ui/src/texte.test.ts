import { describe, expect, it } from "vitest";
import { erstelleTextkatalog, formatiere, t } from "./texte";

describe("Texte", () => {
  it("liefert Texte aus data/texte/de", () => {
    expect(t("app.untertitel")).toBe("Wirtschaftssimulation 1900–2100");
  });

  it("setzt Platzhalter ein", () => {
    expect(t("kern.bereit", { version: "1.2.3" })).toBe("Simulationskern 1.2.3 bereit");
  });

  it("lässt unbekannte Platzhalter sichtbar stehen", () => {
    expect(formatiere("Preis {betrag} in {waehrung}", { betrag: 5 })).toBe("Preis 5 in {waehrung}");
  });

  it("zeigt bei fehlendem Text den Schlüssel", () => {
    expect(t("gibt.es.nicht")).toBe("gibt.es.nicht");
  });

  it("weist Einträge ab, die kein Text sind", () => {
    expect(() => erstelleTextkatalog({ "a.b": 3 })).toThrow("„a.b“ ist kein Text");
  });
});
