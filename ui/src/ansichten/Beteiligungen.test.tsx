import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Befehl, Kern, StartUp, StartUps } from "../kern";
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
  value_usd: 240_000,
  success_value_usd: 21_000_000,
  own_share: 0,
  own_book_usd: 0,
  own_pledge_usd: 0,
  own_grants_usd: 0,
  invest_mode: "anteile",
  invest_max_usd: 288_000,
  sale_value_usd: 192_000,
  own_value_usd: 0,
  own_offer_usd: null,
  majority: false,
  pace: "normal",
  integration_usd: null,
  blocked: false,
  subsidiary: false,
  parent: null,
  expected_return: null,
  exit: null,
  exit_company: null,
  origin: null,
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
    portfolio_book_usd: 0,
    portfolio_value_usd: 0,
    holdings: 0,
    cash_usd: 1_000_000,
    blocking: 0.25,
    majority: 0.5,
    buy_premium: 0.2,
    sale_discount: 0.2,
    grant_effect: 0.3,
    paces: ["normal", "zuegig", "gruendlich"],
    offers_settle: "1914-09-01",
    company_premium_max: 0.2,
    spin_offs: [
      {
        site: 4,
        country: "DEU",
        kind: "technologie",
        target: "kompressionskuehlschrank",
        level: null,
        progress: 0.45,
        lead: 2,
        months: 42,
        rivals: 3,
        phase: "prototyp",
        value_usd: 600_000,
        sale_value_usd: 480_000,
        reason: null,
      },
      {
        site: 5,
        country: "DEU",
        kind: "verbesserung",
        target: "naegel",
        level: 2,
        progress: 0.04,
        lead: null,
        months: null,
        rivals: 0,
        phase: null,
        value_usd: null,
        sale_value_usd: null,
        reason: "fortschritt",
      },
    ],
    spin_off_min: 0.1,
  };
}

async function zeige(geschaetzt: boolean, aendern: (d: StartUps) => void = () => {}) {
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
    startups: async () => {
      const d = daten(geschaetzt);
      aendern(d);
      return d;
    },
    befehl: async (b) => {
      gesendet.push(b);
      return uebersicht;
    },
  };
  render(<BeteiligungenAnsicht kern={kern} uebersicht={uebersicht} onGeaendert={() => {}} />);
  await screen.findByRole("heading", { name: "Erfinder und Gründungen" });
  return gesendet;
}

/** Opens the detail of the first running start-up. */
function oeffne(): HTMLElement {
  fireEvent.click(
    screen.getByRole("button", { name: "Ada Muster: beteiligen, fördern, verkaufen" }),
  );
  return screen.getByRole("region", { name: "Ada Muster" });
}

describe("Beteiligungen", () => {
  afterEach(cleanup);

  it("gründet ein Forschungsprojekt aus und zeigt, was noch fehlt", async () => {
    const gesendet = await zeige(false);
    fireEvent.click(screen.getByRole("button", { name: /^Ausgründen/ }));
    expect(screen.getByText(/Möglich ab 10 % Fortschritt/)).toBeTruthy();
    const projekt = screen.getByRole("form", { name: "Projekt Kompressionskühlschrank" });
    expect(projekt.textContent).toContain("Forschungszentrum in Deutschland, 45 % fertig");
    expect(projekt.textContent).toContain("Beginnt als Prototyp");
    // Two years ahead, 42 months to go: history is faster.
    expect(projekt.textContent).toContain("erfunden wird die Technologie sonst in 2 Jahren");
    expect(within(projekt).getByText(/Achtung: Die Technologie dürfte erfunden sein/)).toBeTruthy();
    expect(
      within(projekt).getByText(/Achtung: 3 andere Firmen forschen am selben Ziel/),
    ).toBeTruthy();
    const zu_frueh = screen.getByRole("form", { name: "Projekt Nägel: Stufe 2" });
    expect(zu_frueh.textContent).toContain("Noch zu früh: Ausgründen geht ab 10 % Fortschritt.");
    expect(within(zu_frueh).queryByRole("button", { name: "Ausgründen" })).toBeNull();

    fireEvent.change(within(projekt).getByLabelText(/An Investoren verkaufen/), {
      target: { value: "100" },
    });
    fireEvent.click(within(projekt).getByRole("button", { name: "Ausgründen" }));
    expect(within(projekt).getByText("Gib einen Anteil von 0 bis unter 100 % an.")).toBeTruthy();
    fireEvent.change(within(projekt).getByLabelText(/An Investoren verkaufen/), {
      target: { value: "30" },
    });
    fireEvent.click(within(projekt).getByRole("button", { name: "Ausgründen" }));
    expect(
      await within(projekt).findByText("Kompressionskühlschrank ist jetzt ein eigenes Start-up."),
    ).toBeTruthy();
    expect(gesendet).toEqual([{ SpinOff: { site: 4, sell: 0.3 } }]);
  });

  it("zeigt die Herkunft einer Ausgründung", async () => {
    await zeige(false, (d) => {
      d.active = [{ ...LAUFEND, origin: "Test AG", subsidiary: true }];
    });
    const zeile = within(screen.getByRole("table", { name: "Laufend" })).getAllByRole("row")[1]!;
    expect(within(zeile).getByText("Ausgründung")).toBeTruthy();
    expect(oeffne().textContent).toContain("Ausgründung von Test AG");
  });

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
    // The table shows the player's part, the detail all owners.
    expect(zeile.textContent).toContain("–Handeln");
    expect(oeffne().textContent).toContain("Gründer 50 %, Investoren 50 %");
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

  it("zeigt den Ausgang eines erfolgreichen Start-ups", async () => {
    await zeige(false, (d) => {
      d.closed = [{ ...BEENDET, status: "erfolg", exit: "boerse", exit_company: "Beispiel Werke" }];
    });
    fireEvent.click(screen.getByRole("button", { name: "Beendet" }));
    const zeile = within(screen.getByRole("table", { name: "Beendet" })).getAllByRole("row")[1]!;
    expect(zeile.textContent).toContain("Erfolg");
    expect(zeile.textContent).toContain("– Börsengang, daraus wurde Beispiel Werke");
  });

  it("kauft zwischen den Runden Anteile und gibt Fördergeld", async () => {
    const gesendet = await zeige(true);
    expect(screen.getByText(/Du hältst noch keine Beteiligungen/)).toBeTruthy();
    const detail = oeffne();
    expect(detail.textContent).toContain("Ada Muster – Kompressionskühlschrank");
    expect(detail.textContent).toContain("Ab 25 % kann kein anderer");
    // Neither stake nor majority: no sale, no steering, no integration.
    expect(within(detail).queryByRole("form", { name: "An Investoren verkaufen" })).toBeNull();
    expect(within(detail).queryByRole("group", { name: "Lenken" })).toBeNull();
    expect(within(detail).queryByRole("group", { name: "Eingliedern" })).toBeNull();

    const kauf = within(detail).getByRole("form", { name: "Anteile kaufen" });
    expect(kauf.textContent).toContain("mit 20 % Aufschlag");
    fireEvent.change(within(kauf).getByLabelText(/Betrag/), { target: { value: "100000" } });
    fireEvent.click(within(kauf).getByRole("button", { name: "Anteile kaufen" }));
    expect(await within(kauf).findByRole("status")).toBeTruthy();
    expect(within(kauf).getByRole("status").textContent).toMatch(
      /Du beteiligst dich mit .* an Ada Muster\./,
    );

    const foerdern = within(detail).getByRole("form", { name: "Fördergeld geben" });
    fireEvent.change(within(foerdern).getByLabelText(/Betrag/), { target: { value: "5000" } });
    fireEvent.click(within(foerdern).getByRole("button", { name: "Fördern" }));
    await within(foerdern).findByRole("status");
    expect(gesendet).toEqual([
      { InvestInVenture: { venture: 3, amount: 100_000 * 10_000 } },
      { GrantVenture: { venture: 3, amount: 5_000 * 10_000 } },
    ]);
  });

  it("sagt in einer offenen Runde höchstens den offenen Rest zu", async () => {
    await zeige(true, (d) => {
      d.active = [
        {
          ...LAUFEND,
          raised_usd: 40_000,
          round_until: "1915-03-01",
          phase_until: null,
          invest_mode: "runde",
          invest_max_usd: 80_000,
          expected_return: 2.4,
        },
      ];
    });
    const detail = oeffne();
    expect(detail.textContent).toContain("im Mittel das 2,4-Fache des Einsatzes");
    const zusage = within(detail).getByRole("form", { name: "In der Runde zusagen" });
    expect(zusage.textContent).toContain("Höchstens");
    expect(within(zusage).getByRole("button", { name: "Zusagen" })).toBeTruthy();
  });

  it("verkauft, lenkt und gliedert als Mehrheitseigner ein", async () => {
    const gesendet = await zeige(true, (d) => {
      d.active = [
        {
          ...LAUFEND,
          owners: [
            { holder: "firma", company: "Test AG", share: 0.6 },
            { holder: "gruender", company: null, share: 0.4 },
          ],
          own_share: 0.6,
          own_book_usd: 100_000,
          majority: true,
          integration_usd: 115_200,
        },
      ];
      d.holdings = 1;
      d.portfolio_book_usd = 100_000;
      d.portfolio_value_usd = 144_000;
    });
    expect(screen.getByText(/Deine Beteiligungen \(1\): Buchwert/)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: /^Deine/ }));
    expect(within(screen.getByRole("table", { name: "Laufend" })).getByText("60 %")).toBeTruthy();
    const detail = oeffne();
    expect(detail.textContent).toContain("Test AG 60 %, Gründer 40 %");

    const verkauf = within(detail).getByRole("form", { name: "An Investoren verkaufen" });
    fireEvent.change(within(verkauf).getByLabelText(/Teil deines Anteils/), {
      target: { value: "50" },
    });
    fireEvent.click(within(verkauf).getByRole("button", { name: "Verkaufen" }));
    await within(verkauf).findByRole("status");

    const lenken = within(detail).getByRole("group", { name: "Lenken" });
    fireEvent.change(within(lenken).getByLabelText("Tempo"), { target: { value: "zuegig" } });
    expect(await within(lenken).findByText("Tempo „Zügig“ gilt ab sofort.")).toBeTruthy();

    const eingliedern = within(detail).getByRole("group", { name: "Eingliedern" });
    fireEvent.click(within(eingliedern).getByRole("button", { name: /eingliedern/ }));
    expect(
      await within(eingliedern).findByText("Ada Muster ist jetzt deine Tochterfirma."),
    ).toBeTruthy();

    expect(gesendet).toEqual([
      { SellVentureStake: { venture: 3, share: 0.3 } },
      { SteerVenture: { venture: 3, pace: "Fast" } },
      { IntegrateVenture: { venture: 3 } },
    ]);
  });

  it("bietet den ganzen Anteil den Firmen an und zieht das Angebot zurück", async () => {
    const mit = (angebot: number | null) => (d: StartUps) => {
      d.active = [
        {
          ...LAUFEND,
          owners: [
            { holder: "firma", company: "Test AG", share: 0.3 },
            { holder: "gruender", company: null, share: 0.7 },
          ],
          own_share: 0.3,
          own_book_usd: 50_000,
          own_value_usd: 72_000,
          own_offer_usd: angebot,
        },
      ];
    };
    const gesendet = await zeige(true, mit(null));
    let detail = oeffne();
    const firmen = within(detail).getByRole("form", { name: "An Firmen verkaufen" });
    expect(firmen.textContent).toContain("(30 %, heute");
    expect(firmen.textContent).toContain("Am 01.09.1914 kauft das beste Gebot");
    expect(firmen.textContent).toContain("bis zu 20 % über dem Wert");
    fireEvent.change(within(firmen).getByLabelText(/Mindestpreis/), {
      target: { value: "80000" },
    });
    fireEvent.click(within(firmen).getByRole("button", { name: "Anbieten" }));
    expect((await within(firmen).findByRole("status")).textContent).toContain(
      "das Ergebnis kommt am 01.09.1914",
    );
    expect(gesendet).toEqual([{ OfferVentureStake: { venture: 3, minimum: 80_000 * 10_000 } }]);

    cleanup();
    const gesendet2 = await zeige(true, mit(80_000));
    detail = oeffne();
    const angebot = within(detail).getByRole("group", { name: "An Firmen verkaufen" });
    expect(angebot.textContent).toContain("die Firmen bieten bis zum 01.09.1914");
    fireEvent.click(within(angebot).getByRole("button", { name: "Angebot zurückziehen" }));
    expect(await within(angebot).findByText("Angebot zurückgezogen.")).toBeTruthy();
    expect(gesendet2).toEqual([{ OfferVentureStake: { venture: 3, minimum: null } }]);
  });
});
