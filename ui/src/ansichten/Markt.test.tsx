import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { vorschauKern } from "../kern/vorschau";
import { t } from "../texte";
import { MarktAnsicht } from "./Markt";

async function ansicht() {
  const kern = vorschauKern(0);
  const uebersicht = await kern.neuesSpiel({
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
  render(<MarktAnsicht kern={kern} uebersicht={uebersicht} onGeaendert={() => {}} />);
  const tabelle = await screen.findByRole("table", { name: /Markt/ });
  const produkte = () =>
    within(tabelle)
      .queryAllByRole("button")
      .map((b) => b.textContent);
  return { tabelle, produkte };
}

describe("Marktliste", () => {
  afterEach(cleanup);

  it("filtert nach Warengruppe und sucht nach dem Namen", async () => {
    const { produkte } = await ansicht();
    const alle = produkte();
    expect(alle.length).toBeGreaterThan(10);

    const gruppe = screen.getByLabelText("Warengruppe") as HTMLSelectElement;
    const eisen = [...gruppe.options].find((o) => o.value === "eisen_stahl");
    expect(eisen?.textContent).toMatch(/^Eisen und Stahl \(\d+\)$/);
    fireEvent.change(gruppe, { target: { value: "eisen_stahl" } });
    const stahl = produkte();
    expect(stahl.length).toBeGreaterThan(0);
    expect(stahl.length).toBeLessThan(alle.length);
    expect(stahl).toContain(t("produkt.stahl"));
    expect(stahl).not.toContain(t("produkt.getreide"));

    fireEvent.change(gruppe, { target: { value: "alle" } });
    fireEvent.change(screen.getByLabelText("Produkt suchen"), { target: { value: "stahl" } });
    const gefunden = produkte();
    expect(gefunden.length).toBeGreaterThan(0);
    expect(gefunden.every((p) => p?.toLowerCase().includes("stahl"))).toBe(true);

    fireEvent.change(screen.getByLabelText("Produkt suchen"), { target: { value: "xyz" } });
    expect(produkte()).toEqual([]);
    expect(screen.getByText("Keine Produkte für diese Auswahl.")).toBeTruthy();
  });
});
