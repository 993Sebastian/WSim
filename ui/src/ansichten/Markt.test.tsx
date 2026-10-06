import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Befehl, Kern } from "../kern";
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

describe("Produktnamen", () => {
  afterEach(cleanup);

  it("zeigt die Namen der Anbieter und benennt das eigene Produkt", async () => {
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
    const gesendet: Befehl[] = [];
    const kern: Kern = {
      ...vorschau,
      befehl: async (b) => {
        gesendet.push(b);
        return uebersicht;
      },
    };
    render(<MarktAnsicht kern={kern} uebersicht={uebersicht} onGeaendert={() => {}} />);
    const tabelle = await screen.findByRole("table", { name: /Markt/ });
    const moebel = within(tabelle)
      .getAllByRole("button")
      .find((b) => b.textContent === t("produkt.moebel"));
    fireEvent.click(moebel as HTMLElement);
    const anbieter = await screen.findByRole("table", { name: "Anbieter im Vormonat" });
    expect(within(anbieter).getByText("Zylvia Extra")).toBeTruthy();

    const formular = screen.getByRole("form", { name: "Dein Produktname" });
    expect(within(formular).getByText(/heißt bisher einfach „Möbel“/)).toBeTruthy();
    const feld = within(formular).getByLabelText("Produktname") as HTMLInputElement;
    // The first suggestion is filled in; another one is a click away.
    expect(feld.value).toBe("Valessa");
    fireEvent.click(within(formular).getByRole("button", { name: "Lavessa Classic" }));
    expect(feld.value).toBe("Lavessa Classic");
    fireEvent.click(within(formular).getByRole("button", { name: "Namen speichern" }));
    expect(
      await within(formular).findByText("Dein Möbel heißt jetzt „Lavessa Classic“."),
    ).toBeTruthy();
    expect(gesendet).toEqual([{ NameProduct: { product: "moebel", name: "Lavessa Classic" } }]);
    // A saved name can be removed again.
    fireEvent.click(within(formular).getByRole("button", { name: "Namen entfernen" }));
    expect(await within(formular).findByText(/trägt keinen eigenen Namen mehr/)).toBeTruthy();
    expect(gesendet.at(-1)).toEqual({ NameProduct: { product: "moebel", name: null } });
  });
});
