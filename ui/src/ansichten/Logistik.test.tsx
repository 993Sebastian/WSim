import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Befehl, Kern, Logistik } from "../kern";
import { vorschauKern } from "../kern/vorschau";
import { Befehle, useBefehl } from "./gemeinsam";
import { LogistikAnsicht } from "./Logistik";

function Huelle({ kern }: { kern: Kern }) {
  const { senden, meldung } = useBefehl(
    kern,
    () => {},
    () => {},
  );
  return (
    <Befehle senden={senden} meldung={meldung}>
      <LogistikAnsicht kern={kern} stand="1915-01-01" />
    </Befehle>
  );
}

async function zeige(aendern: (d: Logistik) => void = () => {}) {
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
    logistik: async () => {
      const d = await vorschau.logistik();
      aendern(d);
      return d;
    },
    befehl: async (b) => {
      gesendet.push(b);
      return uebersicht;
    },
  };
  render(<Huelle kern={kern} />);
  await screen.findByRole("table", { name: "Fahrzeuge kaufen" });
  return gesendet;
}

describe("Logistik", () => {
  afterEach(cleanup);

  it("zeigt die Flotte und verkauft ein Fahrzeug", async () => {
    const gesendet = await zeige();
    const flotte = screen.getByRole("table", { name: "Eigene Flotte" });
    const zeile = within(flotte).getAllByRole("row")[1]!;
    expect(zeile.textContent).toContain("Fuhrwerk");
    expect(zeile.textContent).toContain("Gelände");
    // Carts run dearer than the railway.
    expect(zeile.textContent).toContain("bleibt stehen");
    fireEvent.click(within(zeile).getByRole("button", { name: "Verkaufen" }));
    await screen.findByText("Erledigt.");
    expect(gesendet).toEqual([{ SellVehicles: { vehicle: "fuhrwerk", count: 1 } }]);
  });

  it("kauft Fahrzeuge in gewählter Zahl", async () => {
    const gesendet = await zeige();
    const kauf = screen.getByRole("table", { name: "Fahrzeuge kaufen" });
    const zeile = within(kauf)
      .getAllByRole("row")
      .find((z) => z.textContent?.includes("Eisenbahn"))!;
    fireEvent.change(within(zeile).getByLabelText("Anzahl"), { target: { value: "3" } });
    fireEvent.click(within(zeile).getByRole("button", { name: "Kaufen" }));
    await screen.findByText("Erledigt.");
    expect(gesendet).toEqual([{ BuyVehicles: { vehicle: "eisenbahn", count: 3 } }]);
  });

  it("wählt den Weg der Ladungen", async () => {
    const gesendet = await zeige((d) => {
      d.mode = "markt";
      d.carry_for_others = false;
    });
    const uebernehmen = screen.getByRole<HTMLButtonElement>("button", { name: "Übernehmen" });
    expect(uebernehmen.disabled).toBe(true);
    fireEvent.click(screen.getByRole("radio", { name: /Staatlicher Transport/ }));
    fireEvent.click(screen.getByRole("checkbox", { name: /für andere fahren/ }));
    fireEvent.click(uebernehmen);
    await screen.findByText("Erledigt.");
    expect(gesendet).toEqual([{ SetLogistics: { mode: "State", carry_for_others: true } }]);
  });

  it("sagt, wenn es keine eigene Logistik gibt", async () => {
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
      logistik: async () => ({ ...(await vorschau.logistik()), enabled: false }),
    };
    render(<Huelle kern={kern} />);
    await screen.findByText(/alle Ladungen über den Frachtmarkt/);
  });
});
