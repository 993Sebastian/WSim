import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Anleihen, Befehl, Kern, Uebersicht } from "../kern";
import beispielJson from "../kern/beispiel.json";
import { vorschauKern } from "../kern/vorschau";
import { AnleihenTeil } from "./Anleihen";
import { Befehle, useBefehl } from "./gemeinsam";

const beispiel = beispielJson as unknown as { finanzen: { bonds: Anleihen } };

function Huelle({ kern, a }: { kern: Kern; a: Anleihen }) {
  const { senden, meldung } = useBefehl(
    kern,
    () => {},
    () => {},
  );
  return (
    <Befehle senden={senden} meldung={meldung}>
      <AnleihenTeil a={a} />
    </Befehle>
  );
}

function zeige(aendern: (a: Anleihen) => void = () => {}) {
  const a = structuredClone(beispiel.finanzen.bonds);
  aendern(a);
  const gesendet: Befehl[] = [];
  const kern: Kern = {
    ...vorschauKern(0),
    befehl: async (b) => {
      gesendet.push(b);
      return {} as Uebersicht;
    },
  };
  render(<Huelle kern={kern} a={a} />);
  return gesendet;
}

/** A company large enough for bonds, with two offers and one bond outstanding. */
function gross(a: Anleihen) {
  a.equity_usd = 50_000_000;
  a.grade = "a";
  a.max_usd = 20_000_000;
  a.quotes = [
    { amount_usd: 10_000_000, grade: "a", coupon: 0.031 },
    { amount_usd: 20_000_000, grade: "bbb", coupon: 0.038 },
  ];
  a.bonds = [
    {
      index: 0,
      principal_usd: 5_000_000,
      coupon: 0.029,
      issued: "1950-03-01",
      maturity: "1960-03-01",
      grade: "aa",
      redeem_usd: 5_100_000,
    },
  ];
}

describe("Anleihen", () => {
  afterEach(cleanup);

  it("erklärt, warum eine kleine Firma keine Anleihe ausgibt", () => {
    zeige();
    expect(screen.getByText(/erst von Firmen mit mindestens/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Anleihe ausgeben" })).toBeNull();
  });

  it("gibt eine Anleihe zum gewählten Betrag aus", async () => {
    const gesendet = zeige(gross);
    expect(screen.getByText("A", { selector: "strong" })).toBeTruthy();
    fireEvent.change(screen.getByLabelText("Betrag"), { target: { value: "1" } });
    fireEvent.change(screen.getByLabelText(/Laufzeit/), { target: { value: "15" } });
    fireEvent.click(screen.getByRole("button", { name: "Anleihe ausgeben" }));
    await screen.findByText("Erledigt.");
    expect(gesendet).toEqual([{ IssueBond: { amount: 200_000_000_000, years: 15 } }]);
  });

  it("kauft eine Anleihe zurück", async () => {
    const gesendet = zeige(gross);
    const liste = screen.getByRole("table", { name: "Ausgegebene Anleihen" });
    const zeile = within(liste).getAllByRole("row")[1]!;
    expect(zeile.textContent).toContain("AA");
    fireEvent.click(within(zeile).getByRole("button", { name: /^Zurückkaufen/ }));
    await screen.findByText("Erledigt.");
    expect(gesendet).toEqual([{ RedeemBond: { bond: 0 } }]);
  });
});
