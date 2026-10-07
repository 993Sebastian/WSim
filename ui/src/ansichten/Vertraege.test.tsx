import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Befehl, Kern, Vertraege } from "../kern";
import { vorschauKern } from "../kern/vorschau";
import { Befehle, useBefehl } from "./gemeinsam";
import { VertraegeAnsicht } from "./Vertraege";

function Huelle({ kern }: { kern: Kern }) {
  const { senden, meldung } = useBefehl(
    kern,
    () => {},
    () => {},
  );
  return (
    <Befehle senden={senden} meldung={meldung}>
      <VertraegeAnsicht kern={kern} stand="1915-01-01" />
    </Befehle>
  );
}

async function zeige(aendern: (d: Vertraege) => void = () => {}) {
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
    vertraege: async () => {
      const d = await vorschau.vertraege();
      aendern(d);
      return d;
    },
    befehl: async (b) => {
      gesendet.push(b);
      return uebersicht;
    },
  };
  render(<Huelle kern={kern} />);
  await screen.findByRole("table", { name: "Lieferverträge" });
  return gesendet;
}

describe("Lieferverträge", () => {
  afterEach(cleanup);

  it("zeigt laufende Verträge und kündigt mit Strafe", async () => {
    const gesendet = await zeige();
    const tabelle = screen.getByRole("table", { name: "Lieferverträge" });
    const zeile = within(tabelle).getAllByRole("row")[1]!;
    expect(zeile.textContent).toContain("Draht");
    expect(zeile.textContent).toContain("Einkauf");
    expect(zeile.textContent).toContain("Läuft");
    // From the supplier's country to the workshop.
    expect(zeile.textContent).toContain("Vereinigte Staaten → Deutschland");
    fireEvent.click(within(zeile).getByRole("button", { name: /^Kündigen/ }));
    await screen.findByText("Erledigt.");
    expect(gesendet).toEqual([{ CancelContract: { contract: 3 } }]);
    expect(within(tabelle).getAllByText("Verfallen").length).toBeGreaterThan(0);
  });

  it("nimmt das Angebot einer Firma an", async () => {
    const gesendet = await zeige((d) => {
      const erster = d.contracts[0]!;
      d.contracts.unshift({
        ...erster,
        id: 7,
        status: "angeboten",
        start: null,
        end: null,
        answer: true,
        can_cancel: false,
        cancel_fee_usd: 0,
      });
    });
    const zeile = within(screen.getByRole("table", { name: "Lieferverträge" })).getAllByRole(
      "row",
    )[1]!;
    expect(zeile.textContent).toContain("Angeboten");
    expect(zeile.textContent).toContain("12 Monate");
    fireEvent.click(within(zeile).getByRole("button", { name: "Annehmen" }));
    await screen.findByText("Erledigt.");
    expect(gesendet).toEqual([{ AnswerContract: { contract: 7, accept: true } }]);
  });

  it("bietet einen neuen Vertrag mit dem Vorschlag des Lieferanten an", async () => {
    const gesendet = await zeige();
    const neu = screen.getByRole("region", { name: "Neuer Liefervertrag" });
    // The workshop sells nails and buys wire.
    const produkt = within(neu).getByLabelText("Produkt");
    expect(produkt).toHaveProperty("value", "naegel");
    fireEvent.change(produkt, { target: { value: "draht" } });
    const lieferant = await within(neu).findByLabelText("Lieferant");
    expect((lieferant as HTMLSelectElement).selectedOptions[0]!.textContent).toContain(
      "Morgan Iron and Steel Corporation (Vereinigte Staaten)",
    );
    expect(neu.textContent).toContain("Fracht und Zoll bis zu dir");
    fireEvent.click(within(neu).getByRole("button", { name: "Vertrag anbieten" }));
    await within(neu).findByText(/Die Firma nimmt an/);
    expect(gesendet).toHaveLength(1);
    const b = gesendet[0]!;
    if (!("ProposeContract" in b)) throw new Error("kein Vertragsangebot");
    expect(b.ProposeContract).toMatchObject({
      seller: 71,
      buyer: 0,
      product: "draht",
      months: 12,
      min_quality: 0,
      penalty: 0.2,
    });
    expect(b.ProposeContract.per_month).toBeGreaterThan(0);
    expect(b.ProposeContract.price).toBeGreaterThan(0);
  });
});
