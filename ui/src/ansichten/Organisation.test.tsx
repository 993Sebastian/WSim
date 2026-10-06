import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Anliegen, AnliegenListe, Befehl, Kern } from "../kern";
import { vorschauKern } from "../kern/vorschau";
import { hatText } from "../texte";
import { OrganisationAnsicht } from "./Organisation";

/** The preview core whose example concern stands twice (as from two works), sending to a list. */
async function kernMitAnliegen() {
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
    anliegen: async () => {
      const a: AnliegenListe = await vorschau.anliegen();
      const gruppe = a.open[0]!;
      gruppe.concerns.push({ ...gruppe.concerns[0]!, id: 99 });
      return a;
    },
    befehl: async (b) => {
      gesendet.push(b);
      return uebersicht;
    },
  };
  return { kern, uebersicht: { ...uebersicht, concerns_open: 2 }, gesendet };
}

describe("Anliegen", () => {
  afterEach(cleanup);

  it("zeigt Optionen mit Schritten, Empfehlung und Frist und schickt die Antworten", async () => {
    const { kern, uebersicht, gesendet } = await kernMitAnliegen();
    render(<OrganisationAnsicht kern={kern} uebersicht={uebersicht} onGeaendert={() => {}} />);
    // With open concerns the inbox comes first.
    const gruppe = await screen.findByRole("region", { name: /2 gleiche Anliegen: Ausbau/ });
    const karten = within(gruppe).getAllByRole("article");
    expect(karten).toHaveLength(2);
    const karte = karten[0]!;
    expect(within(karte).getByText(/Antwort bis 30\.01\.1915/)).toBeTruthy();
    expect(within(karte).getByText(/Diese Stelle soll bei jeder Ausgabe fragen/)).toBeTruthy();
    expect(within(karte).getByText(/1 × Nagelmaschine bauen \(klein\)/)).toBeTruthy();
    expect(within(karte).getByText(/Bringt nach meiner Schätzung/)).toBeTruthy();

    fireEvent.click(within(karte).getByRole("button", { name: "Entscheide selbst" }));
    expect(await screen.findByText(/Die Stelle setzt ihre Empfehlung um: Ausbauen/)).toBeTruthy();
    fireEvent.click(within(karte).getByRole("button", { name: /^Umsetzen: So lassen/ }));
    fireEvent.click(within(karte).getByRole("button", { name: "Nicht mehr fragen" }));
    fireEvent.click(within(karte).getByRole("button", { name: "Ablehnen" }));
    fireEvent.click(within(gruppe).getByRole("button", { name: "Empfehlung für alle übernehmen" }));
    await screen.findByText("2 Anliegen entschieden.");
    const id = (await kern.anliegen()).open[0]!.concerns[0]!.id;
    expect(gesendet).toEqual([
      { AnswerConcern: { concern: id, answer: "Delegate" } },
      { AnswerConcern: { concern: id, answer: { Choose: 1 } } },
      { AnswerConcern: { concern: id, answer: "NeverAsk" } },
      { AnswerConcern: { concern: id, answer: "Decline" } },
      { AnswerConcern: { concern: id, answer: "Delegate" } },
      { AnswerConcern: { concern: 99, answer: "Delegate" } },
    ]);
  });

  it("zeigt Budget und eigene Entscheidungen einer Stelle und ändert das Budget", async () => {
    const { kern, uebersicht, gesendet } = await kernMitAnliegen();
    render(
      <OrganisationAnsicht
        kern={kern}
        uebersicht={{ ...uebersicht, concerns_open: 0 }}
        onGeaendert={() => {}}
      />,
    );
    const details = await screen.findByText(/Werksleitung/, { selector: "summary strong" });
    fireEvent.click(details);
    const formular = await screen.findByRole("form", { name: "Budget: Werksleitung" });
    expect(screen.getByText(/Ausbau · Nägel: Ausbauen/)).toBeTruthy();
    fireEvent.change(within(formular).getByLabelText("Je Entscheidung"), {
      target: { value: "3" },
    });
    fireEvent.change(within(formular).getByLabelText("Im Jahr"), { target: { value: "8" } });
    fireEvent.click(within(formular).getByRole("button", { name: "Budget übernehmen" }));
    await screen.findByText("Das Budget von Werksleitung ist geändert.");
    fireEvent.click(within(formular).getByRole("button", { name: /Standard wiederherstellen/ }));
    const site = (await kern.organisation()).continents[0]!.countries[0]!.sites[0]!.site;
    const position = { site, role: "Head" };
    expect(gesendet).toEqual([
      { SetBudget: { position, shares: [0.03, 0.08] } },
      { SetBudget: { position, shares: null } },
    ]);
    // More per decision than per year is refused before sending.
    fireEvent.change(within(formular).getByLabelText("Je Entscheidung"), {
      target: { value: "20" },
    });
    fireEvent.click(within(formular).getByRole("button", { name: "Budget übernehmen" }));
    expect(await within(formular).findByText(/je Entscheidung darf nicht mehr/)).toBeTruthy();
    expect(gesendet).toHaveLength(2);
  });

  it("hat Texte für alle Optionen, Gründe und Ausgänge", () => {
    const optionen = [
      "beibehalten",
      "anpassen",
      "liefern",
      "benennen",
      "kredit",
      "tilgen",
      "stilllegen",
      "verkaufen",
      "anfahren",
      "bauen",
      "anbieten",
      "annehmen",
      "ablehnen",
      "forschen",
      "weiterentwickeln",
    ];
    const gruende: Anliegen["reason"][] = ["entscheidung", "jahr", "immer", "kredit"];
    const ausgaenge: Anliegen["status"][] = [
      "offen",
      "gewaehlt",
      "delegiert",
      "nicht_mehr_fragen",
      "abgelehnt",
      "abgelaufen",
      "erledigt",
    ];
    const fehlend = [
      ...optionen.map((k) => `option.${k}`),
      ...gruende.map((k) => `anliegen.grund.${k}`),
      ...ausgaenge.map((k) => `anliegen.status.${k}`),
      ...["alle", "wichtige", "nie"].map((k) => `spiel.anhalten_${k}`),
    ].filter((k) => !hatText(k));
    expect(fehlend).toEqual([]);
  });
});
