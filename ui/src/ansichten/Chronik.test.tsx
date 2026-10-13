import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { ChronikEintrag, Kern } from "../kern";
import { vorschauKern } from "../kern/vorschau";
import { ChronikAnsicht } from "./Chronik";

const eintrag = (aendern: Partial<ChronikEintrag>): ChronikEintrag => ({
  created: "1914-03-01",
  reviewed: "1914-09-01",
  due: "1914-09-01",
  topic: "ausbau",
  site: 3,
  site_name: "Werk Nord",
  country: "DEU",
  product: "naegel",
  unit: "standort:3",
  manager_id: 7,
  manager: "Anna Kovalenko",
  recommended: "bauen",
  chosen: "beibehalten",
  against: true,
  delegated: false,
  forecast_usd: null,
  actual_usd: 1_200,
  alternative_usd: 5_400,
  unserved: 600,
  hit: true,
  ...aendern,
});

async function zeige(eintraege: ChronikEintrag[]) {
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
  });
  const kern: Kern = {
    ...vorschau,
    organisation: async () => ({ ...(await vorschau.organisation()), chronicle: eintraege }),
  };
  render(<ChronikAnsicht kern={kern} stand="1914-10-01" />);
  await screen.findByRole("table", { name: "Chronik der Entscheidungen" });
}

describe("Chronik", () => {
  afterEach(cleanup);

  it("zeigt Entscheidungen mit Urteil und filtert gegen die Empfehlung", async () => {
    await zeige([
      eintrag({}),
      eintrag({
        created: "1914-05-01",
        topic: "produktion",
        recommended: "anpassen",
        chosen: "anpassen",
        against: false,
        forecast_usd: 3_000,
        actual_usd: 3_300,
        alternative_usd: 0,
        unserved: null,
        manager_id: 8,
        manager: "Erik Hansen",
        hit: false,
      }),
    ]);
    const tabelle = screen.getByRole("table", { name: "Chronik der Entscheidungen" });
    expect(within(tabelle).getByText("So lassen statt Ausbauen")).toBeTruthy();
    expect(within(tabelle).getByText("Empfehlung war richtig")).toBeTruthy();
    expect(within(tabelle).getByText("Empfehlung war nicht besser")).toBeTruthy();
    // Only the decision against the advice.
    fireEvent.click(screen.getByLabelText("Gegen die Empfehlung entschieden"));
    expect(within(tabelle).queryByText("Erik Hansen")).toBeNull();
    expect(within(tabelle).getByText("Anna Kovalenko")).toBeTruthy();
  });
});
