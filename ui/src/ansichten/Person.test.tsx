import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { App } from "../App";
import type { Befehl, Kern, Meldung, NeuesSpiel } from "../kern";
import { vorschauKern } from "../kern/vorschau";
import { PersonAnsicht } from "./Person";

describe("Person (PE2)", () => {
  afterEach(cleanup);

  it("nimmt Name, Geburtsjahr und Familie beim neuen Spiel auf und zeigt die Person", async () => {
    const vorschau = vorschauKern(0);
    const gesendet: NeuesSpiel[] = [];
    const kern: Kern = {
      ...vorschau,
      neuesSpiel: async (e) => {
        gesendet.push(e);
        return vorschau.neuesSpiel(e);
      },
      // The example person has no children yet: two born before the start.
      person: async () => {
        const p = await vorschau.person();
        const kind = (name: string, born: string, age: number, card_in: number) => ({
          name,
          born,
          age,
          died: null,
          manager: null,
          position: null,
          card_in,
        });
        p.children = [
          kind("Clara Albrecht", "1910-03-02", 4, 21),
          kind("Paul Albrecht", "1913-11-20", 0, 25),
        ];
        const geburt = (name: string, date: string): { date: string; text: Meldung } => ({
          date,
          text: {
            group: "allgemein",
            key: "person.ereignis.geburt",
            kind: "info",
            params: { name: { type: "text", value: name } },
            target: null,
          },
        });
        p.history = [
          ...p.history,
          geburt("Paul Albrecht", "1913-11-20"),
          geburt("Clara Albrecht", "1910-03-02"),
        ];
        return p;
      },
    };
    render(<App kern={kern} />);
    fireEvent.click(await screen.findByRole("button", { name: "Neues Spiel" }));
    const person = await screen.findByRole("group", { name: "Person" });
    // The year of birth defaults to the start year minus 30; the hint names the range.
    expect((within(person).getByLabelText(/Geburtsjahr/) as HTMLInputElement).placeholder).toBe(
      "1870",
    );
    expect(within(person).getByText(/1840 bis 1882/)).toBeTruthy();
    fireEvent.change(within(person).getByLabelText("Dein Name"), {
      target: { value: "Clara Weber" },
    });
    fireEvent.change(within(person).getByLabelText(/Geburtsjahr/), {
      target: { value: "1865" },
    });
    fireEvent.click(within(person).getByLabelText("Verheiratet"));
    fireEvent.change(within(person).getByLabelText("Kinder"), { target: { value: "2" } });
    fireEvent.click(screen.getByRole("button", { name: "Spiel starten" }));
    // Without a company only some views are open, and the founding dialog asks first.
    const gruendung = await screen.findByRole("dialog", { name: "Firma gründen" });
    expect(screen.getByRole("button", { name: "Standorte" })).toHaveProperty("disabled", true);
    fireEvent.change(await within(gruendung).findByLabelText("Name der Firma"), {
      target: { value: "Test AG" },
    });
    // The dialog suggests 90 % of the account as capital (in the shown currency).
    expect((within(gruendung).getByLabelText(/^Einlage/) as HTMLInputElement).value).toMatch(
      /^\d{1,3}(\.\d{3})*(,\d+)?$/,
    );
    fireEvent.click(within(gruendung).getByRole("button", { name: "Firma gründen" }));
    await screen.findByText("Test AG");
    expect(gesendet[0]).toMatchObject({
      person_name: "Clara Weber",
      birth_year: 1865,
      married: false,
      children: 2,
      found_at_start: false,
    });
    expect(screen.getByRole("button", { name: "Standorte" })).toHaveProperty("disabled", false);

    // The header opens the view of the person (the preview shows its example person).
    fireEvent.click(screen.getByRole("button", { name: /^Person: / }));
    const steckbrief = await screen.findByRole("region", { name: "Steckbrief" });
    expect(within(steckbrief).getByRole("heading", { name: "Jürgen Albrecht" })).toBeTruthy();
    expect(within(steckbrief).getByText(/10\.07\.1884 \(30 Jahre\)/)).toBeTruthy();
    const familie = screen.getByRole("region", { name: "Familie" });
    const zeilen = within(familie).getAllByRole("row");
    expect(zeilen[1]!.textContent).toMatch(/^Clara Albrecht02\.03\.19104in 21 Jahren$/);
    expect(within(familie).getByText(/Weitere Kinder sind möglich/)).toBeTruthy();
    const rollen = screen.getByRole("region", { name: "Rollen" });
    expect(within(rollen).getByText("CEO – du führst die Firma selbst")).toBeTruthy();
    // Wealth, lifestyle with its effects, the salary (PE3).
    const vermoegen = screen.getByRole("region", { name: "Vermögen" });
    expect(within(vermoegen).getByText("Vermögen gesamt")).toBeTruthy();
    const stil = screen.getByRole("region", { name: "Lebensstil" });
    expect(within(stil).getAllByRole("row")).toHaveLength(5);
    expect(within(stil).getByText("Luxuriös")).toBeTruthy();
    // The salary sits in the company's row of the roles.
    expect(within(rollen).getByRole("form", { name: "Gehalt von Neue Firma" })).toBeTruthy();
    const lebenslauf = screen.getByRole("region", { name: "Lebenslauf" });
    expect(within(lebenslauf).getByText("Beginn mit der Firma Neue Firma")).toBeTruthy();
    expect(within(lebenslauf).getByText("Geburt von Paul Albrecht")).toBeTruthy();
  });

  it("verkauft Anteile an Anleger und bietet die Gründung weiterer Firmen an (PE5)", async () => {
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
      found_at_start: true,
    });
    const gesendet: Befehl[] = [];
    const kern: Kern = {
      ...vorschau,
      befehl: async (b) => {
        gesendet.push(b);
        return vorschau.uebersicht();
      },
    };
    let gruenden = 0;
    render(<PersonAnsicht kern={kern} stand="a" onGruenden={() => (gruenden += 1)} />);
    const handel = await screen.findByRole("region", { name: "Anteile an Neue Firma" });
    expect(within(handel).getByText(/^Das ist deine Hauptfirma/)).toBeTruthy();
    expect(within(handel).queryByRole("button", { name: "Als Hauptfirma führen" })).toBeNull();
    expect(within(handel).getByText(/Anleger zahlen dafür sofort/)).toBeTruthy();
    fireEvent.change(within(handel).getByLabelText(/^Anteil verkaufen/), {
      target: { value: "25" },
    });
    expect(within(handel).getByText(/^Erlös etwa /)).toBeTruthy();
    fireEvent.click(within(handel).getByRole("button", { name: "An Anleger verkaufen" }));
    expect(await within(handel).findByText("Anteile verkauft.")).toBeTruthy();
    expect(gesendet).toEqual([{ SellStake: { company: 0, share: 0.25 } }]);
    // A further company can be founded at any time.
    const gruendung = screen.getByRole("region", { name: "Firma gründen" });
    fireEvent.click(within(gruendung).getByRole("button"));
    expect(gruenden).toBe(1);
  });

  it("zeigt Nachlass und Erben und übergibt erst nach Rückfrage (PE6)", async () => {
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
      found_at_start: true,
    });
    const gesendet: Befehl[] = [];
    const kern: Kern = {
      ...vorschau,
      befehl: async (b) => {
        gesendet.push(b);
        return vorschau.uebersicht();
      },
    };
    render(<PersonAnsicht kern={kern} stand="a" />);
    const nachfolge = await screen.findByRole("region", { name: "Nachfolge" });
    expect(within(nachfolge).getByText(/^Generation 1 · Sterberisiko/)).toBeTruthy();
    expect(within(nachfolge).getByText(/^Erbe nach heutigem Stand: ein Neffe/)).toBeTruthy();
    expect(within(nachfolge).getByText(/^Nachlass .*Steuer etwa/)).toBeTruthy();
    fireEvent.click(within(nachfolge).getByRole("button", { name: "Jetzt übergeben" }));
    expect(gesendet).toEqual([]);
    fireEvent.click(within(nachfolge).getByRole("button", { name: "Ja, übergeben" }));
    expect(await within(nachfolge).findByText(/^Übergeben/)).toBeTruthy();
    expect(gesendet).toEqual([{ HandOver: {} }]);
  });

  it("zeigt Zeit und Gesundheit und sendet die Zeitverteilung (PV1b)", async () => {
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
      found_at_start: true,
    });
    const gesendet: Befehl[] = [];
    const kern: Kern = {
      ...vorschau,
      person: async () => {
        const p = await vorschau.person();
        p.wellbeing = {
          background: "tueftler",
          research: 0.03,
          loyalty: 0,
          loan_rate: 0,
          experience: 0.2,
          time: [60, 15, 10, 15],
          min_firm: 30,
          delay_days: 0,
          health: 72,
          mortality_factor: 1,
          ill_until: null,
        };
        return p;
      },
      befehl: async (b) => {
        gesendet.push(b);
        return vorschau.uebersicht();
      },
    };
    render(<PersonAnsicht kern={kern} stand="a" />);
    const zeit = await screen.findByRole("region", { name: "Zeit und Gesundheit" });
    expect(within(zeit).getByText("Tüftler")).toBeTruthy();
    expect(within(zeit).getByText(/72 \/ 100/)).toBeTruthy();
    fireEvent.change(within(zeit).getByLabelText(/^Erholung/), { target: { value: "20" } });
    expect(within(zeit).getByText(/Summe 105/)).toBeTruthy();
    expect(within(zeit).getByRole("button", { name: "Zeit verteilen" })).toHaveProperty(
      "disabled",
      true,
    );
    fireEvent.change(within(zeit).getByLabelText(/^Firma/), { target: { value: "55" } });
    fireEvent.click(within(zeit).getByRole("button", { name: "Zeit verteilen" }));
    await within(zeit).findByRole("status");
    expect(gesendet).toEqual([{ SetTimeBudget: { firm: 55, family: 15, network: 10, rest: 20 } }]);
  });

  it("zeigt die Anlagen und sendet Käufe, Verkäufe und den Sparplan (PV2)", async () => {
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
      found_at_start: true,
    });
    const gesendet: Befehl[] = [];
    const kern: Kern = {
      ...vorschau,
      person: async () => {
        const p = await vorschau.person();
        p.assets = {
          deposits: [{ principal_usd: 5000, rate: 0.02, until: "1916-01-01", paid_usd: 10 }],
          bonds: [],
          gold_ounces: 2,
          gold_price_usd: 1000,
          gold_value_usd: 2000,
          gold_cost_usd: 1900,
          gold_fee: 0.005,
          gold_spread: 0.01,
          funds: [
            {
              key: "gemanagt",
              open: true,
              from: 1900,
              units: 3,
              nav: 100,
              value_usd: 300,
              cost_usd: 290,
              running_cost: 0.015,
              load: 0.05,
              fee: 0.005,
            },
            {
              key: "index",
              open: false,
              from: 1990,
              units: 0,
              nav: 100,
              value_usd: 0,
              cost_usd: 0,
              running_cost: 0.002,
              load: 0,
              fee: 0.005,
            },
          ],
          plan: null,
          deposit_rate: 0.02,
          bond_coupon: 0.025,
          deposit_months: [3, 120],
          bond_months: [12, 360],
          deposit_penalty: 0.5,
          bond_spread: 0.01,
        };
        return p;
      },
      befehl: async (b) => {
        gesendet.push(b);
        return vorschau.uebersicht();
      },
    };
    render(<PersonAnsicht kern={kern} stand="a" />);
    const anlagen = await screen.findByRole("region", { name: "Anlagen" });
    expect(within(anlagen).getByText(/ab 1990 erhältlich/)).toBeTruthy();
    fireEvent.change(within(anlagen).getByLabelText(/^Betrag/), { target: { value: "1000" } });
    fireEvent.click(within(anlagen).getByRole("button", { name: "Gold kaufen" }));
    await within(anlagen).findByRole("status");
    expect(gesendet[0]).toMatchObject({
      Invest: { action: { BuyGold: { amount: expect.any(Number) } } },
    });
    fireEvent.click(within(anlagen).getByRole("button", { name: "Kündigen" }));
    fireEvent.click(within(anlagen).getByRole("button", { name: /^Sparplan/ }));
    await within(anlagen).findByRole("status");
    expect(gesendet).toContainEqual({ Invest: { action: { BreakDeposit: { index: 0 } } } });
    expect(gesendet[2]).toMatchObject({
      Invest: { action: { SetPlan: { fund: "Managed", amount: expect.any(Number) } } },
    });
  });
});
