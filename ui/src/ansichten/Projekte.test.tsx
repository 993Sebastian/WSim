import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Befehl, Entwicklungsprojekte, Forschung, Kern } from "../kern";
import { vorschauKern } from "../kern/vorschau";
import { ForschungAnsicht } from "./Forschung";

const projekte: Entwicklungsprojekte = {
  tempos: ["gruendlich", "normal", "beschleunigt"],
  trims: ["basis", "standard", "premium"],
  salary_usd_year: 90_000,
  delegation: "standard",
  player_at: [2, 4, 6, 7],
  leads: [
    {
      id: 0,
      name: "Anna Weber",
      home: "DEU",
      age: 45,
      character: "vorsichtig",
      experience: 0.6,
      finished: 2,
      aborted: 0,
      hired: true,
      busy: true,
    },
    {
      id: 1,
      name: "Carl Brandt",
      home: "DEU",
      age: 39,
      character: "optimistisch",
      experience: 0.3,
      finished: 0,
      aborted: 0,
      hired: true,
      busy: false,
    },
    {
      id: 2,
      name: "Dora Keller",
      home: "DEU",
      age: 52,
      character: "sachlich",
      experience: 0.8,
      finished: 5,
      aborted: 1,
      hired: false,
      busy: false,
    },
  ],
  projects: [
    {
      id: 0,
      product: "naegel",
      site: 99,
      site_name: null,
      lead: "Anna Weber",
      lead_id: 0,
      tempo: "normal",
      trim: "standard",
      status: "laufend",
      milestone: 2,
      next_milestone: 3,
      progress: 0.3,
      started: "1914-02-01",
      finished: null,
      days_left: 120,
      risk: 0.1,
      penalty: 0.05,
      spent_usd: 20_000,
      budget_usd: 100_000,
      auto: true,
      roles: [
        { role: "produktmanager", need: 2, assigned: 0, free: 0 },
        { role: "entwickler", need: 6, assigned: 4, free: 2 },
        { role: "fertigungsplaner", need: 1, assigned: 1, free: 0 },
        { role: "laborpersonal", need: 2, assigned: 2, free: 0 },
      ],
      scenarios: [
        { delta: -1, days: 160, risk: 0.2, penalty: 0.1 },
        { delta: 0, days: 120, risk: 0.1, penalty: 0 },
        { delta: 1, days: 100, risk: 0.08, penalty: 0 },
      ],
      profile: "ausgewogen",
      path: "",
      conditions: 0,
      loops: 0,
      paused: false,
      delegation: "standard",
      pending: {
        milestone: 2,
        deadline: "1914-06-08",
        recommendation: "weiter",
        choice_kind: "weg",
        choices: [
          { key: "bewaehrt", cost: 0.95, time: 0.95, risk: 0.7, quality: -0.03, preferred: false },
          { key: "standard", cost: 1, time: 1, risk: 1, quality: 0, preferred: true },
          { key: "innovativ", cost: 1.2, time: 1.1, risk: 1.5, quality: 0.1, preferred: false },
        ],
      },
      reports: [
        {
          milestone: 2,
          date: "1914-06-01",
          kpis: {
            unit_cost: 10,
            price: 14,
            volume_year: 5000,
            revenue_year: 70000,
            lifecycle_revenue: 500000,
            contribution_year: 20000,
            plants: 1,
            investment: 30000,
            project_cost: 100000,
            project_days: 300,
            people: 6,
            payback_years: 3,
            quality: 1,
          },
          baseline: null,
          deviation: null,
          lights: ["gruen", "gelb", "gruen", "rot"],
          recommendation: "weiter",
          causes: ["verzug"],
          decided: null,
        },
      ],
    },
  ],
  offers: [{ product: "schrauben", effort: 5000, cost_usd: 500_000, days: 250 }],
};

async function kernMitProjekten() {
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
      d.projects = projekte;
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

describe("Entwicklungsprojekte", () => {
  afterEach(cleanup);

  it("zeigt Projekt, Besetzung und Szenarien und gibt Befehle", async () => {
    const { kern, uebersicht, gesendet } = await kernMitProjekten();
    render(
      <ForschungAnsicht
        kern={kern}
        uebersicht={uebersicht}
        onGeaendert={() => {}}
        onStandorte={() => {}}
      />,
    );
    fireEvent.click(await screen.findByRole("button", { name: /Entwicklungsprojekte/ }));
    const karte = screen.getByRole("article", { name: "Nägel" });
    expect(within(karte).getByText(/Meilenstein M2 erreicht, weiter zu M3/)).toBeTruthy();
    expect(within(karte).getByText("Anna Weber")).toBeTruthy();
    expect(within(karte).getAllByText("120 Tage").length).toBeGreaterThan(0);
    // Szenarien: nach Bedarf, mehr und weniger.
    expect(within(karte).getByText("nach Bedarf")).toBeTruthy();
    expect(within(karte).getByText("1 mehr je Rolle")).toBeTruthy();
    expect(within(karte).getByText("1 weniger je Rolle")).toBeTruthy();
    // Eine Person mehr bei den Entwicklern, Befehl mit der neuen Zahl.
    fireEvent.click(within(karte).getByRole("button", { name: "Eine Person mehr: Entwickler" }));
    expect(gesendet).toContainEqual({
      AssignProjectStaff: { project: 0, role: "Developer", count: 5 },
    });
    // Keine freien Produktmanager: das Plus ist gesperrt.
    expect(
      (
        within(karte).getByRole("button", {
          name: "Eine Person mehr: Produktmanager",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
  });

  it("stellt Projektleiter ein, entlässt freie und startet ein Projekt", async () => {
    const { kern, uebersicht, gesendet } = await kernMitProjekten();
    render(
      <ForschungAnsicht
        kern={kern}
        uebersicht={uebersicht}
        onGeaendert={() => {}}
        onStandorte={() => {}}
      />,
    );
    fireEvent.click(await screen.findByRole("button", { name: /Entwicklungsprojekte/ }));
    fireEvent.click(screen.getByRole("button", { name: "Einstellen: Dora Keller" }));
    expect(gesendet).toContainEqual({ HireProjectLead: { lead: 2 } });
    // Wer ein Projekt leitet, kann nicht gehen.
    expect(
      (screen.getByRole("button", { name: "Entlassen: Anna Weber" }) as HTMLButtonElement).disabled,
    ).toBe(true);
    fireEvent.click(screen.getByRole("button", { name: "Entlassen: Carl Brandt" }));
    expect(gesendet).toContainEqual({ DismissProjectLead: { lead: 1 } });
    // Projekt starten: Schrauben mit dem freien Projektleiter.
    fireEvent.click(screen.getByRole("button", { name: "Projekt starten" }));
    expect(gesendet).toContainEqual({
      StartProject: { site: 99, product: "schrauben", lead: 1, tempo: 1, trim: 1 },
    });
  });

  it("zeigt den Bericht und sendet die Entscheidung mit dem Lösungsweg", async () => {
    const { kern, uebersicht, gesendet } = await kernMitProjekten();
    render(
      <ForschungAnsicht
        kern={kern}
        uebersicht={uebersicht}
        onGeaendert={() => {}}
        onStandorte={() => {}}
      />,
    );
    fireEvent.click(await screen.findByRole("button", { name: /Entwicklungsprojekte/ }));
    const gruppe = screen.getByRole("group", { name: "Entscheidung am Meilenstein" });
    expect(within(gruppe).getByText("Das Projekt liegt hinter dem Plan.")).toBeTruthy();
    fireEvent.click(within(gruppe).getByRole("radio", { name: /Innovativ/ }));
    fireEvent.click(within(gruppe).getByRole("button", { name: "Weiter" }));
    expect(gesendet).toContainEqual({
      DecideMilestone: { project: 0, decision: "Continue", choice: 2 },
    });
  });
});
