import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Anliegen, AnliegenListe, Befehl, Kern } from "../kern";
import { vorschauKern } from "../kern/vorschau";
import { hatText } from "../texte";
import { OrganisationAnsicht, trefferquote } from "./Organisation";

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
    // Asked by the country's head, on its way from the works (MA3).
    expect(within(karte).getByText(/Landesleitung · Land · Deutschland/)).toBeTruthy();
    expect(
      within(karte).getByText(
        "Weg: Werksleitung (Werk · Deutschland, empfiehlt Ausbauen) → Landesleitung (Land · Deutschland, empfiehlt Ausbauen)",
      ),
    ).toBeTruthy();
    expect(within(karte).getByText(/Dafür braucht es einen Kredit/)).toBeTruthy();
    expect(within(karte).getByText(/1 × Nagelmaschine bauen \(sehr klein\)/)).toBeTruthy();
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
    const site = (await kern.organisation()).continents[0]!.countries[0]!.sites[0]!.site!;
    const position = { unit: { Site: site }, role: "Head" };
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

  it("setzt und entfernt Budget-Vorgaben für Stellentypen", async () => {
    const { kern, uebersicht, gesendet } = await kernMitAnliegen();
    render(
      <OrganisationAnsicht
        kern={kern}
        uebersicht={{ ...uebersicht, concerns_open: 0 }}
        onGeaendert={() => {}}
      />,
    );
    fireEvent.click(await screen.findByText(/Budget-Vorgaben für Stellentypen \(1\)/));
    expect(screen.getByText(/Werk: Produktion · ganze Firma: 3 % \/ 8 %/)).toBeTruthy();
    const formular = screen.getByRole("form", { name: "Budget-Vorgabe für einen Stellentyp" });
    fireEvent.change(within(formular).getByLabelText("Stellentyp"), {
      target: { value: String(1) },
    });
    const typ = (within(formular).getByLabelText("Stellentyp") as HTMLSelectElement)
      .selectedOptions[0]!.textContent;
    fireEvent.change(within(formular).getByLabelText("Gilt für"), {
      target: { value: "kontinent:europa" },
    });
    fireEvent.change(within(formular).getByLabelText("Je Entscheidung"), {
      target: { value: "4" },
    });
    fireEvent.change(within(formular).getByLabelText("Im Jahr"), { target: { value: "9" } });
    fireEvent.click(within(formular).getByRole("button", { name: "Vorgabe setzen" }));
    await screen.findByText(`Vorgabe gesetzt: ${typ} – Europa.`);
    fireEvent.click(
      screen.getByRole("button", { name: "Entfernen: Werk: Produktion, ganze Firma" }),
    );
    expect(gesendet[0]).toMatchObject({
      SetBudgetRule: { scope: { Continent: "europa" }, shares: [0.04, 0.09] },
    });
    expect(gesendet[1]).toEqual({
      SetBudgetRule: {
        kind: { level: { Site: "Factory" }, role: { Specialist: "produktion" } },
        scope: "Company",
        shares: null,
      },
    });
  });

  it("zeigt Gehalt, Marktwert und Zufriedenheit, passt das Gehalt an und lässt die Leitung einstellen", async () => {
    const { kern, uebersicht, gesendet } = await kernMitAnliegen();
    // An offer of another company to the head of the works (MA6).
    const mitAngebot: Kern = {
      ...kern,
      organisation: async () => {
        const o = await kern.organisation();
        const leitung = o.continents[0]!.countries[0]!.sites[0]!.positions[0]!;
        leitung.holder = {
          ...leitung.holder!,
          satisfaction: 0,
          offer: { company: "Rivale AG", salary_usd: 99_000, until: "1915-03-31" },
        };
        return o;
      },
    };
    render(
      <OrganisationAnsicht
        kern={mitAngebot}
        uebersicht={{ ...uebersicht, concerns_open: 0 }}
        onGeaendert={() => {}}
      />,
    );
    // In the table and in the position's details.
    expect(await screen.findAllByText("unzufrieden")).toHaveLength(2);
    expect(
      screen.getByText(/Angebot von Rivale AG: 99\.000 .* – Antwort bis 31\.03\.1915/),
    ).toBeTruthy();
    const zeile = await screen.findByText(/Werksleitung/, { selector: "summary strong" });
    fireEvent.click(zeile);
    const details = within(zeile.closest("details")!);
    expect(details.getByText("Gehalt und Zufriedenheit")).toBeTruthy();
    expect(details.getByText("Marktwert (seine Forderung heute)")).toBeTruthy();
    const o = await kern.organisation();
    const h = o.continents[0]!.countries[0]!.sites[0]!.positions[0]!.holder!;
    const formular = details.getByRole("form", {
      name: `Gehalt von ${h.manager.name} anpassen`,
    });
    // Lower than now is refused before sending.
    fireEvent.change(within(formular).getByLabelText("Neues Gehalt im Jahr"), {
      target: { value: "1" },
    });
    fireEvent.click(within(formular).getByRole("button", { name: "Gehalt anpassen" }));
    expect(await within(formular).findByText(/muss über dem bisherigen/)).toBeTruthy();
    expect(gesendet).toEqual([]);
    const neu = Math.round(h.salary_usd * 1.5);
    fireEvent.change(within(formular).getByLabelText("Neues Gehalt im Jahr"), {
      target: { value: String(neu) },
    });
    fireEvent.click(within(formular).getByRole("button", { name: "Gehalt anpassen" }));
    await screen.findByText(new RegExp(`${h.manager.name} bekommt jetzt`));
    fireEvent.click(details.getByLabelText(/Besetzt freie Fachstellen selbst/));
    await screen.findByText("Werksleitung besetzt ihre freien Fachstellen ab jetzt selbst.");
    const site = o.continents[0]!.countries[0]!.sites[0]!.site!;
    expect(gesendet).toEqual([
      { RaiseSalary: { manager: h.manager.id, salary: neu * 10_000 } },
      { SetHiringByHead: { position: { unit: { Site: site }, role: "Head" }, enabled: true } },
    ]);
  });

  it("warnt im Managermarkt vor Gehältern über der Kasse (N37)", async () => {
    const { kern, uebersicht } = await kernMitAnliegen();
    render(
      <OrganisationAnsicht
        kern={kern}
        uebersicht={{ ...uebersicht, concerns_open: 0 }}
        onGeaendert={() => {}}
      />,
    );
    fireEvent.click(
      await screen.findByRole("button", { name: "Besetzen Produktion (Werk · Deutschland)" }),
    );
    expect(await screen.findByText(/Mehrere Forderungen sind höher als deine Kasse/)).toBeTruthy();
    // The candidates of the unit's continent are listed first.
    expect(screen.getAllByText("über der Kasse").length).toBeGreaterThan(0);
    expect(screen.getByText(/Manager anderer Firmen wirbst du unter „Wettbewerb“ ab/)).toBeTruthy();
  });

  it("zeigt den Hauptsitz mit Steuer und Lohn und verlegt ihn", async () => {
    const { kern, uebersicht, gesendet } = await kernMitAnliegen();
    render(
      <OrganisationAnsicht
        kern={kern}
        uebersicht={{ ...uebersicht, concerns_open: 0 }}
        onGeaendert={() => {}}
      />,
    );
    const karte = await screen.findByRole("article", { name: "Hauptsitz" });
    expect(within(karte).getByText(/Hauptsitz: Berlin, Deutschland/)).toBeTruthy();
    expect(within(karte).getByText("Gewinnsteuer")).toBeTruthy();
    const formular = within(karte).getByRole("form", { name: "Hauptsitz verlegen" });
    expect(within(formular).getByText(/Kosten jetzt 250\.000 .*6 Monate/)).toBeTruthy();
    const knopf = within(formular).getByRole("button", { name: "Hauptsitz verlegen" });
    expect((knopf as HTMLButtonElement).disabled).toBe(true);
    fireEvent.change(within(formular).getByLabelText(/Neues Land/), {
      target: { value: "FRA" },
    });
    // The capital is preselected; another city can be chosen (W2).
    const stadt = within(formular).getByLabelText(/^Stadt/) as HTMLSelectElement;
    expect(stadt.value).toBe("paris");
    fireEvent.change(stadt, { target: { value: "lyon" } });
    fireEvent.click(knopf);
    await screen.findByText("Der Umzug nach Lyon, Frankreich beginnt; er dauert 6 Monate.");
    expect(gesendet).toEqual([{ SetHeadquarters: { country: "FRA", city: "lyon" } }]);
  });

  it("zeigt die Städte des Sitzlands und zieht innerhalb des Landes um (W2)", async () => {
    const { kern, uebersicht, gesendet } = await kernMitAnliegen();
    render(
      <OrganisationAnsicht
        kern={kern}
        uebersicht={{ ...uebersicht, concerns_open: 0 }}
        onGeaendert={() => {}}
      />,
    );
    const karte = await screen.findByRole("article", { name: "Hauptsitz" });
    const tabelle = within(karte).getByRole("table", { name: "Städte in Deutschland" });
    const zeilen = within(tabelle).getAllByRole("row");
    expect(zeilen).toHaveLength(6);
    const berlin = zeilen[1] as HTMLElement;
    expect(within(berlin).getByText("Berlin")).toBeTruthy();
    expect(within(berlin).getByText("Sitz")).toBeTruthy();
    expect(
      within(karte).getByText(/Umzug in eine andere Stadt des Landes: .*3 Monate/),
    ).toBeTruthy();
    fireEvent.click(within(tabelle).getByRole("button", { name: "Nach Hamburg ziehen" }));
    await screen.findByText("Der Umzug nach Hamburg beginnt; er dauert 3 Monate.");
    expect(gesendet).toEqual([{ SetHeadquarters: { country: "DEU", city: "hamburg" } }]);
  });

  it("zeigt die Zentralabteilungen und stellt Angestellte ein", async () => {
    const { kern, uebersicht, gesendet } = await kernMitAnliegen();
    render(
      <OrganisationAnsicht
        kern={kern}
        uebersicht={{ ...uebersicht, concerns_open: 0 }}
        onGeaendert={() => {}}
      />,
    );
    const karte = await screen.findByRole("article", { name: "Zentralabteilungen" });
    expect(within(karte).getByText(/Noch keine Angestellten/)).toBeTruthy();
    const tabelle = within(karte).getByRole("table", { name: "Zentralabteilungen" });
    // Header and the five departments of the data, none headed yet.
    expect(within(tabelle).getAllByRole("row")).toHaveLength(6);
    expect(within(tabelle).getAllByText(/Leitung frei/)).toHaveLength(5);
    expect(within(tabelle).getByText(/Leitung frei – im Vorstand „Recht“ besetzen/)).toBeTruthy();
    const formular = within(tabelle).getByRole("form", {
      name: "Angestellte der Abteilung Finanzen",
    });
    const knopf = within(formular).getByRole("button", { name: "Festlegen" });
    expect((knopf as HTMLButtonElement).disabled).toBe(true);
    fireEvent.change(within(formular).getByRole("textbox"), { target: { value: "3" } });
    fireEvent.click(knopf);
    await screen.findByText("Finanzen: jetzt 3 Angestellte.");
    expect(gesendet).toEqual([{ StaffDepartment: { department: "Finance", staff: 3 } }]);
  });

  it("zeigt die Trefferquote einer Leitung", () => {
    expect(trefferquote(null, 0)).toBe("noch nichts bewertet");
    expect(trefferquote(0.625, 8)).toMatch(/^63\s?% \(8 bewertet\)$/);
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
      "gegenangebot",
      "gehen_lassen",
      "umschulden",
    ];
    const gruende: Anliegen["reason"][] = [
      "entscheidung",
      "jahr",
      "immer",
      "kredit",
      "reserve",
      "investition",
      "verschuldung",
      "antrag",
      "abwerbung",
      "freigabe",
      "beteiligung",
    ];
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
      ...[0, 1, 2].map((k) => `organisation.zufriedenheit.${k}`),
      "thema.abwerbung",
      ...["lizenz", "umschuldung", "gehaltsrunde"].map((k) => `thema.${k}`),
      ...["strategie", "finanzen", "personal", "recht", "marketing"].flatMap((k) => [
        `abteilung.${k}`,
        `abteilung.wirkung.${k}`,
      ]),
    ].filter((k) => !hatText(k));
    expect(fehlend).toEqual([]);
  });
});
