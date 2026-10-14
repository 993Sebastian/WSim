import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
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

  it("zeigt Niveau, Wirkung und Aufwand und setzt den Forscheranteil", async () => {
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
      screen.getByText(
        /für das Produkt bis zu \+30 Qualität, −20 % Arbeit, −15 % Vorprodukte, \+25 % Ausstoß je Stück/,
      ),
    ).toBeTruthy();
    // Only the own products at first: the workshop makes nails.
    expect(
      (screen.getByRole("checkbox", { name: /Nur eigene Produkte/ }) as HTMLInputElement).checked,
    ).toBe(true);
    const karte = screen.getByRole("article", { name: "Nägel" });
    expect(within(karte).getByText("Dein Niveau")).toBeTruthy();
    expect(within(karte).getAllByText("noch keine")).toHaveLength(2);
    expect(within(karte).getByText("Noch forscht niemand daran.")).toBeTruthy();
    expect(within(karte).getByText(/mit einem voll besetzten Labor in Deutschland/)).toBeTruthy();
    fireEvent.click(within(karte).getByRole("button", { name: "Weiterentwickeln" }));
    expect(
      await within(karte).findByText("Nägel: 50 % Weiterentwicklung, 50 % Fertigungsforschung."),
    ).toBeTruthy();
    expect(gesendet).toEqual([
      {
        SetResearchShares: { site: 99, shares: [["naegel", 0.5]], process: [["naegel", 0.5]] },
      },
    ]);

    // All products the company may make, not only its own, with a search by name.
    fireEvent.click(screen.getByRole("checkbox", { name: /Nur eigene Produkte/ }));
    expect(screen.getAllByRole("article").length).toBeGreaterThan(10);
    fireEvent.change(screen.getByLabelText("Produkt suchen"), { target: { value: "draht" } });
    const treffer = screen.getAllByRole("article").map((a) => a.getAttribute("aria-label"));
    expect(treffer.length).toBeGreaterThan(0);
    expect(treffer.every((n) => n?.toLowerCase().includes("draht"))).toBe(true);
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

describe("Laborpersonal", () => {
  afterEach(cleanup);

  it("stellt Leute je Rolle ein und entlässt sie", async () => {
    const { kern, uebersicht, gesendet } = await kernMitLabor();
    const mitPersonal: Kern = {
      ...kern,
      forschung: async () => {
        const d = await kern.forschung();
        d.centers[0]!.posts = 4;
        d.centers[0]!.staff = [
          { role: "entwickler", count: 2, research_weight: 1 },
          { role: "laborpersonal", count: 0, research_weight: 0.5 },
          { role: "produktmanager", count: 0, research_weight: 0 },
        ];
        return d;
      },
    };
    render(
      <ForschungAnsicht
        kern={mitPersonal}
        uebersicht={uebersicht}
        onGeaendert={() => {}}
        onStandorte={() => {}}
      />,
    );
    fireEvent.click(await screen.findByRole("button", { name: /^Forschungszentren/ }));
    const block = await screen.findByRole("region", { name: "Laborpersonal" });
    expect(within(block).getByText(/2 von 4 Arbeitsplätzen besetzt/)).toBeTruthy();
    expect(within(block).getByText(/noch ohne Wirkung/)).toBeTruthy();
    fireEvent.click(within(block).getByRole("button", { name: "Einstellen: Laborpersonal" }));
    await within(block).findByText("Eine Person ist eingestellt.");
    fireEvent.click(within(block).getByRole("button", { name: "Entlassen: Entwickler" }));
    await within(block).findByText("Eine Person ist entlassen.");
    expect(
      (within(block).getByRole("button", { name: "Entlassen: Laborpersonal" }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
    expect(gesendet).toEqual([
      { HireLabStaff: { site: 99, role: "Technician", count: 1 } },
      { DismissLabStaff: { site: 99, role: "Developer", count: 1 } },
    ]);
  });
});

describe("Projekt je Labor", () => {
  afterEach(cleanup);

  it("gibt einem Labor ein eigenes Projekt", async () => {
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
    const wahl = (await screen.findByLabelText("Projekt von Labor 1")) as HTMLSelectElement;
    expect(wahl.value).toBe("");
    fireEvent.change(wahl, { target: { value: "p:naegel" } });
    expect(await screen.findByText("Das Projekt des Labors ist gesetzt.")).toBeTruthy();
    expect(gesendet).toEqual([
      { SetLabWork: { site: 99, slot: 0, work: { Development: "naegel" } } },
    ]);
    fireEvent.change(wahl, { target: { value: "" } });
    await vi.waitFor(() => expect(gesendet).toHaveLength(2));
    expect(gesendet[1]).toEqual({ SetLabWork: { site: 99, slot: 0, work: null } });
  });
});

describe("Produkte erforschen", () => {
  afterEach(cleanup);

  it("zeigt erforschbare Produkte mit Aufwand und startet die Forschung im Zentrum", async () => {
    const { kern, uebersicht, gesendet } = await kernMitLabor();
    render(
      <ForschungAnsicht
        kern={kern}
        uebersicht={uebersicht}
        onGeaendert={() => {}}
        onStandorte={() => {}}
      />,
    );
    const tabelle = await screen.findByRole("table", { name: "Produkte" });
    const zeilen = within(tabelle).getAllByRole("row");
    expect(zeilen.length).toBeGreaterThan(2);
    expect(within(tabelle).queryAllByText("erforscht")).toHaveLength(0);
    // The first product opens its details: inputs, effort and a center to work on it.
    const erstes = within(zeilen[1]!).getAllByRole("button")[0]!;
    const name = erstes.textContent ?? "";
    fireEvent.click(erstes);
    const detail = screen.getByRole("region", { name });
    expect(within(detail).getByText("Vorprodukte")).toBeTruthy();
    fireEvent.click(within(detail).getByRole("button", { name: "Erforschen" }));
    expect(await within(detail).findByText(`Das Zentrum erforscht jetzt ${name}.`)).toBeTruthy();
    expect(gesendet).toHaveLength(1);
    expect(gesendet[0]).toMatchObject({ SetProductResearch: { site: 99 } });

    // Known products: the player may make them.
    fireEvent.change(screen.getByLabelText("Zeigen"), { target: { value: "bekannt" } });
    expect(
      within(screen.getByRole("table", { name: "Produkte" })).getAllByText("erforscht").length,
    ).toBeGreaterThan(0);
  });
});

describe("Patente", () => {
  afterEach(cleanup);

  it("meldet einen eigenen Anspruch in den gewählten Ländern an und zeigt fremde Patente", async () => {
    const { kern: basis, uebersicht, gesendet } = await kernMitLabor();
    const kern: Kern = {
      ...basis,
      forschung: async () => {
        const d = await basis.forschung();
        d.patent_countries = [
          { country: "USA", cost_usd: 30_000, suggested: true },
          { country: "DEU", cost_usd: 25_000, suggested: true },
          { country: "FRA", cost_usd: 24_000, suggested: false },
        ];
        d.technologies[0]!.patent = {
          holder: "Test AG",
          own: true,
          deadline: "1915-01-01",
          until: null,
          countries: [],
          free_for_player: true,
          blocks_player_in: [],
        };
        d.technologies[1]!.patent = {
          holder: "Konkurrenz AG",
          own: false,
          deadline: null,
          until: "1930-01-01",
          countries: ["DEU", "USA"],
          free_for_player: false,
          blocks_player_in: ["DEU"],
        };
        return d;
      },
    };
    render(
      <ForschungAnsicht
        kern={kern}
        uebersicht={uebersicht}
        onGeaendert={() => {}}
        onStandorte={() => {}}
      />,
    );
    fireEvent.click(await screen.findByRole("button", { name: /Patente/ }));
    const daten = await kern.forschung();
    const eigene = daten.technologies[0]!.key;
    const form = screen.getByRole("form", { name: /anmelden$/ });
    // The suggestion is chosen: the player's site countries and the largest markets.
    const knopf = within(form).getByRole("button", { name: /In 2 Ländern anmelden/ });
    fireEvent.click(within(form).getByRole("checkbox", { name: /Frankreich/ }));
    expect(knopf.textContent).toMatch(/In 3 Ländern anmelden/);
    fireEvent.click(knopf);
    expect(await within(form).findByText(/in 3 Ländern patentiert/)).toBeTruthy();
    expect(gesendet).toEqual([
      { FilePatent: { technology: eigene, countries: ["USA", "DEU", "FRA"] } },
    ]);
    // All countries at once, and none again.
    const zweite = screen.getByRole("form", { name: /anmelden$/ });
    fireEvent.click(within(zweite).getByRole("button", { name: /Auswahl aufheben/ }));
    const kaestchen = () => within(zweite).getAllByRole("checkbox") as HTMLInputElement[];
    expect(kaestchen().some((c) => c.checked)).toBe(false);
    fireEvent.click(within(zweite).getByRole("button", { name: /Alle \d+ Länder auswählen/ }));
    expect(kaestchen().every((c) => c.checked)).toBe(true);
    expect(screen.getByText(/Patentiert von Konkurrenz AG/)).toBeTruthy();
    expect(screen.getByText(/Deine Anlagen in Deutschland brauchen eine Lizenz/)).toBeTruthy();
  });
});
