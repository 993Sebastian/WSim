import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Befehl, Kern } from "../kern";
import { vorschauKern } from "../kern/vorschau";
import { hatText } from "../texte";
import { OrganisationAnsicht } from "./Organisation";

/** The preview core (a CEO with one quarterly review, beispiel.json), sending to a list. */
async function kernMitVorstand() {
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

async function ruecksprache() {
  const daten = await kernMitVorstand();
  render(
    <OrganisationAnsicht kern={daten.kern} uebersicht={daten.uebersicht} onGeaendert={() => {}} />,
  );
  fireEvent.click(await screen.findByRole("button", { name: "Rücksprache" }));
  await screen.findByRole("form", { name: "Strategieauftrag an den Vorstand" });
  return daten;
}

describe("Vorstand und Rücksprache", () => {
  afterEach(cleanup);

  it("zeigt im Organigramm den Vorstand mit CEO und Ressorts", async () => {
    const { kern, uebersicht } = await kernMitVorstand();
    render(<OrganisationAnsicht kern={kern} uebersicht={uebersicht} onGeaendert={() => {}} />);
    const vorstand = await screen.findByRole("region", { name: "Vorstand" });
    const karte = within(vorstand).getByRole("article", { name: "Vorstand" });
    const zeilen = within(karte).getAllByRole("row");
    expect(zeilen[1]!.textContent).toMatch(/^CEO/);
    expect(within(karte).getByText("Ressort Finanzen")).toBeTruthy();
    // The personnel member has no topics but can be hired for what it does.
    expect(within(karte).getByText("Schätzt Bewerber und Manager genauer ein.")).toBeTruthy();
    expect(
      within(karte).getByRole("button", { name: "Besetzen Ressort Personal (Vorstand)" }),
    ).toBeTruthy();
    expect(within(karte).getByText(/Entscheidest du selbst: .*Kredite, Kaufangebote/)).toBeTruthy();
  });

  it("zeigt Bericht, Ziele, Chancen, Risiken und Anträge der letzten Rücksprache", async () => {
    await ruecksprache();
    expect(screen.getByText(/Nächste Rücksprache mit .+: 01\.07\.1914/)).toBeTruthy();
    const bericht = screen.getByRole("article", { name: "Rücksprache 01.01.1914 – 31.03.1914" });
    const zahlen = within(bericht).getByRole("table", { name: /^Umsatz und Ergebnis/ });
    const zeilen = within(zahlen).getAllByRole("row");
    // Header, company, Europe, overhead.
    expect(zeilen).toHaveLength(4);
    expect(zeilen[1]!.textContent).toMatch(/^Firma/);
    expect(zeilen[3]!.textContent).toMatch(/^Gemeinkosten der Firma/);
    const ziele = within(bericht).getByRole("table", { name: /^Ziele / });
    expect(within(ziele).getAllByRole("row")).toHaveLength(4);
    expect(within(ziele).getByText("Umsatzrendite")).toBeTruthy();
    expect(within(ziele).getAllByText("verfehlt").length).toBeGreaterThan(0);
    const chancen = within(bericht).getByRole("region", { name: "Chancen" });
    expect(within(chancen).getByText(/^Nägel: .+ Marge/)).toBeTruthy();
    expect(
      within(chancen).getAllByText(/^Antrag: Engpässe · .+ – wartet auf deine Antwort$/),
    ).toHaveLength(3);
    const risiken = within(bericht).getByRole("region", { name: "Risiken" });
    expect(within(risiken).getByText("Ziel verfehlt: Umsatzrendite")).toBeTruthy();
    expect(within(bericht).getByRole("button", { name: "Zu den Anträgen (3)" })).toBeTruthy();
  });

  it("legt den Strategieauftrag fest und prüft die Grenzen", async () => {
    const { gesendet } = await ruecksprache();
    const form = screen.getByRole("form", { name: "Strategieauftrag an den Vorstand" });
    // As in the example: growth, quarterly, Russia blocked.
    expect((within(form).getByLabelText("Leitlinie") as HTMLSelectElement).value).toBe("wachstum");
    expect(within(form).getByRole("list", { name: "Gesperrte Länder" }).textContent).toMatch(
      /Russland/,
    );
    fireEvent.change(within(form).getByLabelText("Leitlinie"), {
      target: { value: "marktfuehrung" },
    });
    fireEvent.change(within(form).getByLabelText("Warengruppe"), {
      target: { value: "metallwaren" },
    });
    fireEvent.change(within(form).getByLabelText("Kredite höchstens"), {
      target: { value: "150" },
    });
    fireEvent.click(within(form).getByRole("button", { name: "Auftrag übernehmen" }));
    expect(await within(form).findByText(/Kredite zwischen 0 und 100 %/)).toBeTruthy();
    expect(gesendet).toEqual([]);
    fireEvent.change(within(form).getByLabelText("Kredite höchstens"), {
      target: { value: "30" },
    });
    // Russia free again, Brazil blocked.
    fireEvent.click(within(form).getByRole("button", { name: "Entfernen: Russland" }));
    fireEvent.change(within(form).getByLabelText("Gesperrte Länder"), {
      target: { value: "BRA" },
    });
    fireEvent.change(within(form).getByLabelText("Umsatzrendite"), { target: { value: "" } });
    fireEvent.change(within(form).getByLabelText("Rücksprache"), {
      target: { value: "jaehrlich" },
    });
    fireEvent.click(within(form).getByRole("button", { name: "Auftrag übernehmen" }));
    await within(form).findByText("Der Strategieauftrag ist übernommen.");
    expect(gesendet).toEqual([
      {
        SetMandate: {
          mandate: {
            guideline: { Leadership: "metallwaren" },
            goals: { growth: 0.1, margin: null, equity_ratio: null, rank: 50 },
            max_debt: 0.3,
            blocked_countries: ["BRA"],
            blocked_groups: [],
            review: "Yearly",
          },
        },
      },
    ]);
  });

  it("hat Texte für Leitlinien, Takte, Ziele und Gründe", () => {
    const fehlend = [
      ...["wachstum", "ertrag", "sicherheit", "marktfuehrung"].flatMap((k) => [
        `auftrag.leitlinie.${k}`,
        `auftrag.leitlinie_hilfe.${k}`,
      ]),
      ...["monatlich", "quartalsweise", "halbjaehrlich", "jaehrlich"].map(
        (k) => `auftrag.ruecksprache.${k}`,
      ),
      ...["wachstum", "rendite", "eigenkapitalquote", "rang"].map((k) => `ziel.${k}`),
      ...["verlust", "ziel", "reserve", "verschuldung"].map((k) => `ruecksprache.risiko.${k}`),
      ...["verschuldung", "antrag"].map((k) => `anliegen.grund.${k}`),
      "option.gegenangebot",
      "thema.antwort",
      "leitung.vorstand",
      "ebene.vorstand",
      "bericht.halt.ruecksprache",
    ].filter((k) => !hatText(k));
    expect(fehlend).toEqual([]);
  });
});
