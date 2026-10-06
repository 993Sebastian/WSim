import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Befehl, Forschung, Kern } from "../kern";
import { vorschauKern } from "../kern/vorschau";
import { ForschungAnsicht } from "./Forschung";

/** The preview core with a research center that has a laboratory ready (M37). */
async function kernMitLabor() {
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
    forschung: async () => {
      const d: Forschung = await vorschau.forschung();
      d.centers = [
        {
          site: 99,
          country: "DEU",
          researchers: 20,
          project: null,
          development: null,
          ready: true,
          building_until: null,
          labs: [{ slot: 0, count: 1, utilization: 1, ready: "1914-01-01" }],
        },
      ];
      return d;
    },
    befehl: async (b) => {
      gesendet.push(b);
      return uebersicht;
    },
  };
  return { kern, uebersicht, gesendet };
}

describe("Weiterentwicklung", () => {
  afterEach(cleanup);

  it("zeigt Stufe, Wirkung und Aufwand und startet die Weiterentwicklung", async () => {
    const { kern, uebersicht, gesendet } = await kernMitLabor();
    render(
      <ForschungAnsicht
        kern={kern}
        uebersicht={uebersicht}
        onGeaendert={() => {}}
        onStandorte={() => {}}
      />,
    );
    fireEvent.click(await screen.findByRole("button", { name: "Weiterentwicklung" }));
    expect(
      screen.getByText(/für das Produkt \+4 Qualität, −3 % Arbeit, −2 % Vorprodukte je Stück/),
    ).toBeTruthy();
    // Only the own products at first: the workshop makes nails.
    expect(
      (screen.getByRole("checkbox", { name: /Nur eigene Produkte/ }) as HTMLInputElement).checked,
    ).toBe(true);
    const karte = screen.getByRole("article", { name: "Nägel" });
    expect(within(karte).getByText("Stufe 0 von 5")).toBeTruthy();
    expect(within(karte).getByText("noch keine")).toBeTruthy();
    expect(within(karte).getByText("Der beste Wettbewerber ist auf Stufe 0.")).toBeTruthy();
    expect(within(karte).getByText(/mit einem voll besetzten Labor in Deutschland/)).toBeTruthy();
    fireEvent.click(within(karte).getByRole("button", { name: "Weiterentwickeln" }));
    expect(
      await within(karte).findByText("Das Zentrum entwickelt jetzt Nägel weiter, Stufe für Stufe."),
    ).toBeTruthy();
    expect(gesendet).toEqual([{ SetDevelopment: { site: 99, product: "naegel" } }]);

    // All products the company may make, not only its own.
    fireEvent.click(screen.getByRole("checkbox", { name: /Nur eigene Produkte/ }));
    expect(screen.getAllByRole("article").length).toBeGreaterThan(10);
  });

  it("lässt das Forschungszentrum ein Produkt statt einer Technologie bearbeiten", async () => {
    const { kern, uebersicht, gesendet } = await kernMitLabor();
    render(
      <ForschungAnsicht
        kern={kern}
        uebersicht={uebersicht}
        onGeaendert={() => {}}
        onStandorte={() => {}}
      />,
    );
    fireEvent.click(await screen.findByRole("button", { name: /^Forschungszentren/ }));
    const formular = await screen.findByRole("form", { name: "Forschungsprojekt in Deutschland" });
    const projekt = within(formular).getByLabelText("Projekt") as HTMLSelectElement;
    fireEvent.change(projekt, { target: { value: "p:naegel" } });
    fireEvent.click(within(formular).getByRole("button", { name: "Übernehmen" }));
    expect(await screen.findByText(/Das Zentrum entwickelt jetzt Nägel weiter/)).toBeTruthy();
    // No project stops both kinds of work.
    fireEvent.change(projekt, { target: { value: "" } });
    fireEvent.click(within(formular).getByRole("button", { name: "Übernehmen" }));
    await screen.findByText(/angehalten/i);
    expect(gesendet).toEqual([
      { SetDevelopment: { site: 99, product: "naegel" } },
      { SetResearch: { site: 99, technology: null } },
      { SetDevelopment: { site: 99, product: null } },
    ]);
  });
});
