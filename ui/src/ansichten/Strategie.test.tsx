import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Befehl, Kern } from "../kern";
import { vorschauKern } from "../kern/vorschau";
import { hatText } from "../texte";
import { OrganisationAnsicht } from "./Organisation";

/** The preview core (strategies on every level, beispiel.json), sending to a list. */
async function kernMitStrategie() {
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
  return { kern, uebersicht: { ...uebersicht, concerns_open: 0 }, gesendet };
}

async function strategieOeffnen() {
  const daten = await kernMitStrategie();
  render(
    <OrganisationAnsicht kern={daten.kern} uebersicht={daten.uebersicht} onGeaendert={() => {}} />,
  );
  fireEvent.click(await screen.findByRole("button", { name: "Strategie" }));
  await screen.findByRole("table", { name: "Wo gilt was" });
  return daten;
}

describe("Strategie", () => {
  afterEach(cleanup);

  it("zeigt je Einheit, welche Vorgabe gilt und woher sie stammt", async () => {
    await strategieOeffnen();
    const tabelle = screen.getByRole("table", { name: "Wo gilt was" });
    const zeilen = within(tabelle).getAllByRole("row");
    // Header, company, Europe, Germany, the works.
    expect(zeilen).toHaveLength(5);
    const werk = zeilen[4]!;
    expect(within(werk).getByText("Werk · Deutschland")).toBeTruthy();
    // The price comes from Europe, stock and wages are the works' own, the budget from
    // Germany, the reserve from the company.
    const zellen = within(werk).getAllByRole("cell");
    expect(zellen[1]!.textContent).toBe("Premiumvon Europa");
    expect(zellen[2]!.textContent).toBe(
      "Vorprodukte 10–30 Tage, Fertigwaren 14 Tagehier festgelegt",
    );
    expect(zellen[3]!.textContent).toBe("Lohnaufschlag 5 % bis 25 %hier festgelegt");
    expect(zellen[4]!.textContent).toBe("Eigene Ware zuerstStandard");
    expect(zellen[5]!.textContent).toMatch(/im Jahrvon Deutschland$/);
    expect(zellen[6]!.textContent).toMatch(/^1 Monat \(≈ .+\)von Firma$/);
  });

  it("setzt und entfernt Vorgaben auf einer Ebene", async () => {
    const { kern, gesendet } = await strategieOeffnen();
    const site = (await kern.strategie()).units.find((u) => u.level === "standort")!.site!;
    // Germany: a fighting price.
    fireEvent.click(screen.getByRole("button", { name: "Bearbeiten: Deutschland" }));
    const bereich = screen.getByRole("region", { name: "Vorgaben: Deutschland" });
    const preis = within(bereich).getByRole("form", { name: "Preis: Deutschland" });
    expect(within(preis).getByText(/Gilt: Premium/)).toBeTruthy();
    fireEvent.change(within(preis).getByLabelText("Preisstrategie"), {
      target: { value: "kampfpreis" },
    });
    fireEvent.click(within(preis).getByRole("button", { name: "Hier festlegen" }));
    await within(preis).findByText("Vorgabe „Preis“ für Deutschland festgelegt.");
    // Its own investment budget can be removed: then the company's holds.
    const budget = within(bereich).getByRole("form", { name: "Investitionsbudget: Deutschland" });
    fireEvent.click(within(budget).getByRole("button", { name: "Vorgabe hier entfernen" }));
    await within(budget).findByText(/entfernt – es gilt wieder die Ebene darüber/);
    // The works: a minimum margin, and who carries it out.
    fireEvent.click(screen.getByRole("button", { name: "Bearbeiten: Werk · Deutschland" }));
    const werk = screen.getByRole("region", { name: "Vorgaben: Werk · Deutschland" });
    const werkPreis = within(werk).getByRole("form", { name: /^Preis:/ });
    expect(within(werkPreis).getByText(/Umgesetzt von: Werksleitung/)).toBeTruthy();
    fireEvent.change(within(werkPreis).getByLabelText("Preisstrategie"), {
      target: { value: "mindestmarge" },
    });
    fireEvent.change(within(werkPreis).getByLabelText("Marge auf die Vollkosten"), {
      target: { value: "25" },
    });
    fireEvent.click(within(werkPreis).getByRole("button", { name: "Hier festlegen" }));
    await within(werkPreis).findByText(/festgelegt/);
    expect(gesendet).toEqual([
      {
        SetStrategy: { scope: { Country: "DEU" }, field: "Price", value: { Price: "Fight" } },
      },
      { SetStrategy: { scope: { Country: "DEU" }, field: "Investment", value: null } },
      {
        SetStrategy: {
          scope: { Site: site },
          field: "Price",
          value: { Price: { MinMargin: 0.25 } },
        },
      },
    ]);
  });

  it("prüft die Grenzen vor dem Senden", async () => {
    const { gesendet } = await strategieOeffnen();
    const bereich = screen.getByRole("region", { name: "Vorgaben: Firma" });
    const lager = within(bereich).getByRole("form", { name: "Lager: Firma" });
    fireEvent.change(within(lager).getByLabelText("Vorprodukte mindestens"), {
      target: { value: "40" },
    });
    fireEvent.click(within(lager).getByRole("button", { name: "Hier festlegen" }));
    expect(await within(lager).findByText(/nicht größer als die Höchstreichweite/)).toBeTruthy();
    const reserve = within(bereich).getByRole("form", { name: "Liquiditätsreserve: Firma" });
    expect(within(reserve).getByText(/Laufende Kosten aller Standorte/)).toBeTruthy();
    fireEvent.change(within(reserve).getByLabelText("Reserve"), { target: { value: "99" } });
    fireEvent.click(within(reserve).getByRole("button", { name: "Hier festlegen" }));
    expect(await within(reserve).findByText(/zwischen 0 und 24 Monaten/)).toBeTruthy();
    // The budget in the shown currency.
    const budget = within(bereich).getByRole("form", { name: "Investitionsbudget: Firma" });
    fireEvent.change(within(budget).getByLabelText("Budget im Jahr"), {
      target: { value: "1.000.000" },
    });
    fireEvent.click(within(budget).getByRole("button", { name: "Hier festlegen" }));
    await within(budget).findByText(/festgelegt/);
    expect(gesendet).toEqual([
      {
        SetStrategy: {
          scope: "Company",
          field: "Investment",
          value: { Investment: 1_000_000 * 10_000 },
        },
      },
    ]);
  });

  it("zeigt die Verkaufswege und legt Regeln fest", async () => {
    const { gesendet } = await strategieOeffnen();
    const wege = screen.getByRole("region", { name: "Verkaufswege" });
    const regeln = within(wege).getByRole("table", { name: "Regeln" });
    const zeilen = within(regeln).getAllByRole("row");
    expect(zeilen[1]!.textContent).toBe("Andere Firmenganze FirmagesperrtEntfernen");
    expect(zeilen[2]!.textContent).toMatch(
      /^KI-HändlerNägelerlaubt, ab 1\.700 USD\/t, höchstens 500 t im MonatEntfernen$/,
    );
    fireEvent.click(
      within(regeln).getByRole("button", { name: "Entfernen: Andere Firmen, ganze Firma" }),
    );
    const neu = within(wege).getByRole("form", { name: "Regel festlegen" });
    fireEvent.change(within(neu).getByLabelText("Land"), { target: { value: "DEU" } });
    fireEvent.change(within(neu).getByLabelText("Produkt"), { target: { value: "naegel" } });
    fireEvent.change(within(neu).getByLabelText("Mindestpreis"), {
      target: { value: "1.800" },
    });
    fireEvent.click(within(neu).getByRole("button", { name: "Festlegen" }));
    await within(wege).findAllByText(/Regel für|Vorschau/);
    expect(gesendet).toEqual([
      { SetSalesPolicy: { buyer: "Companies", scope: "Company", rule: null } },
      {
        SetSalesPolicy: {
          buyer: "Traders",
          scope: { ProductInCountry: ["naegel", "DEU"] },
          rule: { allowed: true, min_price: 1_800 * 10_000, max_per_month: null },
        },
      },
    ]);
  });

  it("hat Texte für alle Felder und Einstellungen", () => {
    const felder = ["preis", "lager", "personal", "eigenfertigung", "investition", "reserve"];
    const fehlend = [
      ...felder.map((f) => `strategie.feld.${f}`),
      ...felder.map((f) => `strategie.feldhilfe.${f}`),
      ...["marktpreis", "premium", "kampfpreis", "mindestmarge"].map((k) => `strategie.preis.${k}`),
      ...["eigene", "preis", "zukauf"].map((k) => `strategie.versorgung.${k}`),
      "ebene.firma",
    ].filter((k) => !hatText(k));
    expect(fehlend).toEqual([]);
  });
});
