import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Befehl, Kern, Konzern } from "../kern";
import { vorschauKern } from "../kern/vorschau";
import { Befehle, useBefehl } from "./gemeinsam";
import { TochterfirmenAnsicht } from "./Tochterfirmen";

function Huelle({ kern }: { kern: Kern }) {
  const { senden, meldung } = useBefehl(
    kern,
    () => {},
    () => {},
  );
  return (
    <Befehle senden={senden} meldung={meldung}>
      <TochterfirmenAnsicht kern={kern} stand="1915-01-01" />
    </Befehle>
  );
}

async function zeige(aendern: (d: Konzern) => void = () => {}) {
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
    konzern: async () => {
      const d = await vorschau.konzern();
      aendern(d);
      return d;
    },
    befehl: async (b) => {
      gesendet.push(b);
      return uebersicht;
    },
  };
  render(<Huelle kern={kern} />);
  await screen.findByRole("table", { name: "Aktiva" });
  return gesendet;
}

describe("Tochterfirmen", () => {
  afterEach(cleanup);

  it("zeigt die Tochter und legt Kapital ein", async () => {
    const gesendet = await zeige();
    const liste = screen.getByRole("table", { name: "Tochterfirmen" });
    const zeile = within(liste).getAllByRole("row")[1]!;
    expect(zeile.textContent).toContain("Spedition");
    fireEvent.change(within(zeile).getByLabelText(/Betrag/), { target: { value: "5000" } });
    fireEvent.click(within(zeile).getByRole("button", { name: "Kapital einlegen" }));
    await screen.findByText("Erledigt.");
    const tochter = gesendet[0] as { MoveCapital: { company: number; amount: number } };
    expect(tochter.MoveCapital.amount).toBeGreaterThan(0);
  });

  it("gründet eine Logistik-Tochter", async () => {
    const gesendet = await zeige();
    fireEvent.change(screen.getByLabelText("Name"), { target: { value: "Reederei Nord" } });
    fireEvent.click(screen.getByRole("radio", { name: /^Logistik/ }));
    fireEvent.click(screen.getByRole("button", { name: "Gründen" }));
    await screen.findByText("Erledigt.");
    expect(gesendet).toEqual([
      {
        FoundSubsidiary: {
          name: "Reederei Nord",
          country: "DEU",
          capital: expect.any(Number) as number,
          focus: "Logistics",
        },
      },
    ]);
  });

  it("überträgt einen Standort an die Tochter", async () => {
    const gesendet = await zeige();
    const tabelle = screen.getByRole("table", { name: "Standorte im Konzern übertragen" });
    const zeile = within(tabelle).getAllByRole("row")[1]!;
    fireEvent.click(within(zeile).getByRole("button", { name: "Übertragen" }));
    await screen.findByText("Erledigt.");
    expect(gesendet).toHaveLength(1);
    expect(Object.keys(gesendet[0]!)).toEqual(["TransferSite"]);
  });

  it("sagt, wenn es keine Tochterfirmen gibt", async () => {
    await zeige((d) => {
      d.subsidiaries = [];
      d.sites = [];
    });
    expect(screen.getByText("Du hast noch keine Tochterfirmen.")).toBeTruthy();
  });
});
