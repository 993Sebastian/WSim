import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { App } from "../App";
import type { Kern, NeuesSpiel } from "../kern";
import { vorschauKern } from "../kern/vorschau";

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
    fireEvent.change(await screen.findByLabelText("Name der Firma"), {
      target: { value: "Test AG" },
    });
    const person = screen.getByRole("group", { name: "Person" });
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
    await screen.findByText("Test AG");
    expect(gesendet[0]).toMatchObject({
      person_name: "Clara Weber",
      birth_year: 1865,
      married: false,
      children: 2,
    });

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
    const lebenslauf = screen.getByRole("region", { name: "Lebenslauf" });
    expect(within(lebenslauf).getByText("Beginn mit der Firma Neue Firma")).toBeTruthy();
    expect(within(lebenslauf).getByText("Geburt von Paul Albrecht")).toBeTruthy();
  });
});
