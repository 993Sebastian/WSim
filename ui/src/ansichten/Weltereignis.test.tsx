import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Meldung } from "../kern";
import { folgenVon, WeltereignisDialog } from "./Weltereignis";

const euro: Meldung = {
  kind: "world_event",
  group: "welt",
  key: "meldung.waehrungsreform",
  params: {
    ereignis: { type: "text_key", value: "meldung.waehrungsreform.titel" },
    art: { type: "text_key", value: "ereignisart.waehrung" },
    land: { type: "country", value: "DEU" },
    monat: { type: "text_key", value: "monat.1" },
    jahr: { type: "integer", value: 1999 },
    alt: { type: "text_key", value: "waehrung.d_mark" },
    symbol_alt: { type: "text", value: "DM" },
    neu: { type: "text_key", value: "waehrung.euro" },
    symbol_neu: { type: "text", value: "€" },
    faktor: { type: "number", value: 1.95583 },
    laender: { type: "countries", value: ["DEU"] },
  },
  target: null,
};

describe("Weltereignis", () => {
  afterEach(cleanup);

  it("erzählt eine Währungsumstellung mit dem amtlichen Kurs", () => {
    render(
      <WeltereignisDialog
        meldung={euro}
        nummer={1}
        anzahl={1}
        onWeiter={() => {}}
        onAlle={() => {}}
      />,
    );
    expect(screen.getByRole("dialog", { name: "Währungsumstellung" })).toBeTruthy();
    expect(
      screen.getByText(
        "Ab Januar 1999 rechnet Deutschland in Euro statt in D-Mark: 1 € = 1,95583 DM.",
      ),
    ).toBeTruthy();
    expect(screen.getByText(/ändert dein Vermögen nicht/)).toBeTruthy();
    expect(screen.queryByText(/späteren Ausbaustufe/)).toBeNull();
  });

  it("schreibt riesige Umstellungskurse in Worten", () => {
    const mark: Meldung = {
      ...euro,
      params: {
        ...euro.params,
        monat: { type: "text_key", value: "monat.12" },
        jahr: { type: "integer", value: 1923 },
        alt: { type: "text_key", value: "waehrung.mark" },
        symbol_alt: { type: "text", value: "M" },
        neu: { type: "text_key", value: "waehrung.reichsmark" },
        symbol_neu: { type: "text", value: "RM" },
        faktor: { type: "number", value: 1e12 },
      },
    };
    render(
      <WeltereignisDialog
        meldung={mark}
        nummer={1}
        anzahl={2}
        onWeiter={() => {}}
        onAlle={() => {}}
      />,
    );
    expect(screen.getByText(/: 1 RM = 1 Bio\. M\.$/)).toBeTruthy();
  });

  it("nennt die Folgen eines Krieges", () => {
    const krieg: Meldung = {
      kind: "world_event",
      group: "welt",
      key: "meldung.weltereignis",
      params: {
        ereignis: { type: "text_key", value: "ereignis.erster_weltkrieg" },
        beschreibung: { type: "text_key", value: "ereignis.erster_weltkrieg.text" },
        art: { type: "text_key", value: "ereignisart.krieg" },
        datum: { type: "date", value: "1914-07-28" },
        laender: { type: "countries", value: ["DEU", "FRA"] },
      },
      target: null,
    };
    const sperre: Meldung = {
      kind: "info",
      group: "welt",
      key: "meldung.folge.handelssperre",
      params: {
        ereignis: { type: "text_key", value: "ereignis.erster_weltkrieg" },
        laender: { type: "countries", value: ["FRA"] },
        gegen: { type: "countries", value: ["DEU"] },
        bis: { type: "date", value: "1919-07-12" },
      },
      target: null,
    };
    const nachfrage: Meldung = {
      ...sperre,
      key: "meldung.folge.nachfrage_offen",
      params: {
        ereignis: sperre.params.ereignis!,
        laender: { type: "countries", value: ["DEU"] },
        gruppen: { type: "text_keys", value: ["warengruppe.fahrzeuge", "warengruppe.elektro"] },
        konsum: { type: "number", value: 50 },
        staat: { type: "number", value: 100 },
      },
    };
    const anderes: Meldung = {
      ...sperre,
      params: { ...sperre.params, ereignis: { type: "text_key", value: "ereignis.pandemie" } },
    };
    const folgen = folgenVon([krieg, sperre, nachfrage, anderes], krieg);
    expect(folgen).toEqual([sperre, nachfrage]);
    render(
      <WeltereignisDialog
        meldung={krieg}
        folgen={folgen}
        nummer={1}
        anzahl={1}
        onWeiter={() => {}}
        onAlle={() => {}}
      />,
    );
    expect(screen.getByRole("heading", { name: "Folgen" })).toBeTruthy();
    expect(screen.getByText(/Handelssperre zwischen Frankreich und Deutschland bis/)).toBeTruthy();
    expect(
      screen.getByText(/Nachfrage nach Fahrzeuge, Elektrotechnik in Deutschland/),
    ).toBeTruthy();
    expect(screen.getByText(/gelten für dich und alle KI-Firmen/)).toBeTruthy();
  });
});
