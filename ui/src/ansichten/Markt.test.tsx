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
    expect(within(anbieter).getByText("Belisa Classic")).toBeTruthy();

    const formular = screen.getByRole("form", { name: "Dein Produktname" });
    expect(within(formular).getByText(/heißt bisher einfach „Möbel“/)).toBeTruthy();
    const feld = within(formular).getByLabelText("Produktname") as HTMLInputElement;
    // The first suggestion is filled in; another one is a click away.
    expect(feld.value).toBe("Ulvira");
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

  it("bleibt beim Produkt, wenn Gebiet oder Land wechseln", async () => {
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
    const moebel = within(tabelle)
      .getAllByRole("button")
      .find((b) => b.textContent === t("produkt.moebel"));
    fireEvent.click(moebel as HTMLElement);
    await screen.findByRole("table", { name: "Anbieter im Vormonat" });
    // A continent shows the product's countries side by side …
    fireEvent.change(screen.getByLabelText(t("markt.gebiet")), { target: { value: "europa" } });
    const titel = t("markt.produktmarkt_titel", {
      produkt: t("produkt.moebel"),
      land: t("kontinent.europa"),
    });
    expect(await screen.findByRole("heading", { name: titel })).toBeTruthy();
    // … and a country opens its market of the same product again.
    fireEvent.change(screen.getByLabelText(t("markt.land")), { target: { value: "FRA" } });
    expect(await screen.findByRole("table", { name: "Anbieter im Vormonat" })).toBeTruthy();
  });
});

describe("Ruf, Vertrieb, Gebrauchtmarkt und Ausschreibung (MN5)", () => {
  afterEach(cleanup);

  it("zeigt Ruf, Vertriebswege, Gebrauchtmarkt und die Faktoren der Anbieterwahl und bietet auf ein Los", async () => {
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
      produktmarkt: async (land, produkt) => {
        const m = await vorschau.produktmarkt(land, produkt);
        return {
          ...m,
          own_reputation: 0.7,
          reputation_counts: true,
          own_channels: ["kaufhaus"],
          own_reach: 0.8,
          used: { stock: 1200, taken_per_month: 300 },
          choice_factors: ["preis", "qualitaet", "marke", "praesenz", "ruf", "vertrieb"],
          tender: {
            id: 7,
            per_day: 40,
            deadline: "1914-02-01",
            bids: 2,
            own_price_usd: null,
            own_site: null,
            sites: [{ site: 3, kind_text: "standort.fabrik", name: null }],
          },
        };
      },
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
    expect(await screen.findByText("Dein Ruf")).toBeTruthy();
    expect(screen.getByText(/70 % \(gut\)/)).toBeTruthy();
    expect(screen.getByText(/Kaufhaus · Reichweite 80 %/)).toBeTruthy();
    expect(screen.getByText(/Bestand 1\.200 .*im Monat genutzt 300/)).toBeTruthy();
    // The explanation names every factor.
    const wie = screen.getByLabelText("Wie entsteht: So wählen die Käufer ihren Anbieter?");
    wie.closest("details")?.setAttribute("open", "");
    fireEvent(wie.closest("details") as Element, new Event("toggle"));
    for (const f of ["Preis", "Qualität", "Marke", "Präsenz", "Ruf", "Vertriebswege"]) {
      expect(screen.getAllByText(f).length).toBeGreaterThan(0);
    }
    // Bidding for the lot.
    const los = screen.getByRole("region", { name: "Ausschreibung des Staates" });
    expect(within(los).getByText(/bisher 2 Gebote/)).toBeTruthy();
    fireEvent.change(within(los).getByLabelText(/Gebot je/), { target: { value: "50" } });
    fireEvent.click(within(los).getByRole("button", { name: "Bieten" }));
    await within(los).findByText(/Gebot abgegeben/);
    expect(gesendet.at(-1)).toMatchObject({ BidTender: { tender: 7, site: 3 } });
  });
});
