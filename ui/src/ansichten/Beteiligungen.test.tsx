import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Kern, StartUp, StartUps } from "../kern";
import { vorschauKern } from "../kern/vorschau";
import { BeteiligungenAnsicht } from "./Beteiligungen";

const LAUFEND: StartUp = {
  id: 3,
  name: "Ada Muster",
  inventor: true,
  country: "SWE",
  kind: "technologie",
  target: "kompressionskuehlschrank",
  level: null,
  lead: 4,
  founded: "1914-08-01",
  phase: "prototyp",
  phase_number: 2,
  phases: 3,
  capital_usd: 120_000,
  raised_usd: 120_000,
  round_until: null,
  phase_until: "1916-02-01",
  chance: 0.34,
  chance_level: "mittel",
  owners: [
    { holder: "gruender", company: null, share: 0.5 },
    { holder: "investoren", company: null, share: 0.5 },
  ],
  status: "aktiv",
  ended: null,
};

const BEENDET: StartUp = {
  ...LAUFEND,
  id: 1,
  name: "Otto Beispiel",
  inventor: false,
  kind: "verbesserung",
  target: "naegel",
  level: 2,
  lead: 0,
  phase: null,
  chance: null,
  chance_level: null,
  status: "ueberholt",
  ended: "1914-06-01",
};

function daten(geschaetzt: boolean): StartUps {
  return {
    label: "erfinder",
    per_year: 12,
    estimated: geschaetzt,
    active: [LAUFEND],
    closed: [BEENDET],
    founded: 9,
    succeeded: 0,
    failed: 1,
    keep_years: 10,
  };
}

async function zeige(geschaetzt: boolean) {
  const vorschau = vorschauKern(0);
  const uebersicht = await vorschau.neuesSpiel({
    seed: 1,
    start_year: 1914,
    country: "DEU",
    capital_usd: 100_000,
    start_form: "werkstatt",
    company_name: "Test AG",
    companies: 100,
    difficulty: "mittel",
    research_factor: 1,
  });
  const kern: Kern = { ...vorschau, startups: async () => daten(geschaetzt) };
  render(<BeteiligungenAnsicht kern={kern} uebersicht={uebersicht} />);
  return screen.findByRole("heading", { name: "Erfinder und Gründungen" });
}

describe("Beteiligungen", () => {
  afterEach(cleanup);

  it("zeigt laufende Start-ups mit Phase, Finanzierung, Stufe und Eignern", async () => {
    await zeige(false);
    expect(screen.getByText(/Rund 12 neue im Jahr · 9 gegründet seit Spielbeginn/)).toBeTruthy();
    expect(screen.getByText(/Eine Strategieabteilung \(Organisation\) schätzt sie genauer/));
    const tabelle = screen.getByRole("table", { name: "Laufend" });
    const zeile = within(tabelle).getAllByRole("row")[1]!;
    expect(within(zeile).getByText("Ada Muster")).toBeTruthy();
    expect(within(zeile).getByText("Erfinder")).toBeTruthy();
    expect(within(zeile).getByText("Schweden")).toBeTruthy();
    expect(zeile.textContent).toContain("(neu, 4 Jahre vor der Zeit)");
    expect(zeile.textContent).toContain("Prototyp (2 von 3)");
    expect(zeile.textContent).toContain("finanziert, Entscheidung am 01.02.1916");
    // Without a strategy department only the level.
    expect(within(zeile).getByText("mittel")).toBeTruthy();
    expect(zeile.textContent).toContain("Gründer 50 %, Investoren 50 %");
  });

  it("zeigt mit Strategieabteilung die geschätzte Chance", async () => {
    await zeige(true);
    const tabelle = screen.getByRole("table", { name: "Laufend" });
    expect(within(tabelle).getByText("34 %")).toBeTruthy();
    expect(screen.getByText(/schätzt deine Strategieabteilung/)).toBeTruthy();
  });

  it("zeigt beendete mit Ergebnis", async () => {
    await zeige(false);
    fireEvent.click(screen.getByRole("button", { name: "Beendet" }));
    const tabelle = screen.getByRole("table", { name: "Beendet" });
    const zeile = within(tabelle).getAllByRole("row")[1]!;
    expect(zeile.textContent).toContain("Nägel: Stufe 2");
    expect(within(zeile).getByText("Überholt")).toBeTruthy();
    expect(zeile.textContent).toContain("01.06.1914");
  });
});
