import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Controlling, Kern } from "../kern";
import { vorschauKern } from "../kern/vorschau";
import { ControllingAnsicht } from "./Controlling";

async function zeige(aendern: (d: Controlling) => void = () => {}) {
  const vorschau = vorschauKern(0);
  await vorschau.neuesSpiel({
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
  const zeitraeume: string[] = [];
  const kern: Kern = {
    ...vorschau,
    controlling: async (z) => {
      zeitraeume.push(z);
      const d = await vorschau.controlling(z);
      aendern(d);
      return d;
    },
  };
  render(<ControllingAnsicht kern={kern} stand="1914-10-01" />);
  await screen.findByRole("table", { name: "Ergebnis je Ebene" });
  return zeitraeume;
}

describe("Controlling", () => {
  afterEach(cleanup);

  it("klappt vom Konzern bis zum Produkt auf", async () => {
    await zeige();
    const tabelle = screen.getByRole("table", { name: "Ergebnis je Ebene" });
    // The group and its companies are open.
    expect(within(tabelle).getByText("Konzern")).toBeTruthy();
    expect(within(tabelle).getByText("Zentrale und Firma")).toBeTruthy();
    fireEvent.click(within(tabelle).getByRole("button", { name: "Europa aufklappen" }));
    fireEvent.click(within(tabelle).getByRole("button", { name: "Deutschland aufklappen" }));
    fireEvent.click(
      within(tabelle).getByRole("button", { name: /^Werk · Deutschland aufklappen/ }),
    );
    fireEvent.click(within(tabelle).getByRole("button", { name: "Nägel" }));
    const details = screen.getByRole("region", { name: "Einzelheiten: Nägel" });
    expect(within(details).getByText("Deckungsbeitrag I")).toBeTruthy();
    expect(within(details).getByRole("table", { name: "Kostenarten" })).toBeTruthy();
  });

  it("wechselt den Zeitraum", async () => {
    const zeitraeume = await zeige();
    fireEvent.click(screen.getByRole("button", { name: "Vormonat" }));
    await screen.findByRole("table", { name: "Ergebnis je Ebene" });
    expect(zeitraeume).toContain("monat");
  });

  it("zeigt die Abweichung zur Vorperiode", async () => {
    await zeige((d) => {
      if (d.root) d.root.previous_result_usd = d.root.result_usd - 1000;
    });
    const tabelle = screen.getByRole("table", { name: "Ergebnis je Ebene" });
    const zeile = within(tabelle).getAllByRole("row")[1]!;
    expect(zeile.textContent).toContain("+1.000");
  });
});
