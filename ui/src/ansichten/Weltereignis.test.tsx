import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Meldung } from "../kern";
import { WeltereignisDialog } from "./Weltereignis";

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
});
