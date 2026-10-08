import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { App } from "../App";
import type { Befehl, Kern, NeuesSpiel } from "../kern";
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
    expect(screen.getByRole("form", { name: "Dein Gehalt als CEO" })).toBeTruthy();
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
});
