import { describe, expect, it } from "vitest";
import type { Rundenbericht } from "../kern";
import { hatText } from "../texte";
import { naechster, PFADE, umsatz } from "./Einfuehrung";
import { ANSICHTEN } from "./Spiel";

// The views' sources, to find the marks the introduction points at.
const quellen = Object.values(
  import.meta.glob<string>(["./*.tsx", "!./*.test.*"], {
    eager: true,
    query: "?raw",
    import: "default",
  }),
).join("\n");

const schritte = Object.values(PFADE).flat();

function bericht(umsatzUsd: number): Rundenbericht {
  return {
    products: [{ product: "naegel", revenue_usd: umsatzUsd, margin_usd: 0 }],
  } as Rundenbericht;
}

describe("Einführung", () => {
  it("hat für jeden Schritt Titel und Text", () => {
    const keys = new Set(schritte.map((s) => s.key));
    keys.delete("bericht");
    for (const k of ["runde_nochmal", "bericht_verkauft", "bericht_nichts"]) keys.add(k);
    for (const k of keys) {
      expect(hatText(`einfuehrung.${k}.titel`), k).toBe(true);
      expect(hatText(`einfuehrung.${k}.text`), k).toBe(true);
    }
  });

  it("zeigt nur auf Stellen, die die Ansichten markieren", () => {
    const ids = new Set(
      schritte.flatMap((s) => [
        ...(s.ziele ?? []),
        ...(s.fertig ?? []).map((f) => f.slice(f.startsWith("!") ? 1 : 0)),
      ]),
    );
    for (const id of ids) {
      const [art, rest] = [id.split("-")[0], id.slice(id.indexOf("-") + 1)];
      const da =
        art === "reiter"
          ? (ANSICHTEN as string[]).includes(rest)
          : art === "bereich"
            ? quellen.includes(`key: "${rest}"`) || quellen.includes(`"${rest}"`)
            : quellen.includes(`data-tour="${id}"`) ||
              quellen.includes(`tour="${id}"`) ||
              // Chosen in an expression: data-tour={o ? "preis" : undefined}
              new RegExp(`data-tour=\\{[^}]*"${id}"`).test(quellen);
      expect(da, id).toBe(true);
    }
    expect(quellen).toContain("data-tour={`reiter-${a}`}");
    expect(quellen).toContain("data-tour={`bereich-${b.key}`}");
  });

  it("geht nach einem Bericht ohne Verkauf zurück zur Runde", () => {
    const pfad = PFADE.werkstatt;
    const nr = pfad.findIndex((s) => s.key === "bericht");
    expect(naechster("werkstatt", nr, bericht(0))).toBe(pfad.findIndex((s) => s.key === "runde"));
    expect(naechster("werkstatt", nr, bericht(674))).toBe(nr + 1);
    expect(naechster("handel", 0, null)).toBe(1);
    expect(umsatz(null)).toBeNull();
    expect(umsatz(bericht(674))).toBe(674);
  });
});
