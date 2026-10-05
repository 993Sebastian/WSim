import { describe, expect, it } from "vitest";
import { hatText } from "./texte";

// The interface's sources as text, to find the keys they ask for.
const quellen = import.meta.glob<string>(["./**/*.ts", "./**/*.tsx", "!./**/*.test.*"], {
  eager: true,
  query: "?raw",
  import: "default",
});

describe("Textschlüssel", () => {
  // Keys built at run time (`produkt.${p}`) are not seen here.
  it("gibt es für jeden festen Schlüssel der Oberfläche", () => {
    const fehlend: string[] = [];
    for (const [datei, quelle] of Object.entries(quellen)) {
      const code = quelle.replace(/\/\*[\s\S]*?\*\//g, "").replace(/^\s*\/\/.*$/gm, "");
      for (const [, schluessel] of code.matchAll(/"([a-z][a-z0-9_]*(?:\.[a-z0-9_]+)+)"/g)) {
        // File names are no keys.
        if (/\.(wasm|js|ts|json|yaml|wsim)$/.test(schluessel!)) continue;
        if (!hatText(schluessel!)) fehlend.push(`${datei}: ${schluessel}`);
      }
    }
    expect(fehlend).toEqual([]);
  });
});
