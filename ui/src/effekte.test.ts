import { describe, expect, it } from "vitest";

// Every source file of the interface (tests aside).
const quellen = import.meta.glob<string>(["./**/*.{ts,tsx}", "!./**/*.test.{ts,tsx}"], {
  eager: true,
  query: "?raw",
  import: "default",
});

describe("Effekte", () => {
  it("geben nichts außer einer Aufräumfunktion zurück", () => {
    // `useEffect(() => f())` hands f's result to React as the cleanup; newer WebView2
    // returns a promise from window.scrollTo, and the interface vanished.
    const ausdruck = /use(Layout)?Effect\(\s*\(\)\s*=>\s*(?![\s{]|\(\)\s*=>)/;
    const treffer = Object.entries(quellen)
      .filter(([, text]) => ausdruck.test(text))
      .map(([datei]) => datei);
    expect(treffer).toEqual([]);
  });
});
