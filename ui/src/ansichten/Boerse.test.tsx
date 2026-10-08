import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Befehl, Boerse, Kern } from "../kern";
import { vorschauKern } from "../kern/vorschau";
import { BoerseAnsicht } from "./Boerse";
import { Befehle, useBefehl } from "./gemeinsam";

function Huelle({ kern }: { kern: Kern }) {
  const { senden, meldung } = useBefehl(
    kern,
    () => {},
    () => {},
  );
  return (
    <Befehle senden={senden} meldung={meldung}>
      <BoerseAnsicht kern={kern} stand="1915-01-01" />
    </Befehle>
  );
}

async function zeige(aendern: (d: Boerse) => void = () => {}) {
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
    boerse: async () => {
      const d = await vorschau.boerse();
      aendern(d);
      return d;
    },
    befehl: async (b) => {
      gesendet.push(b);
      return uebersicht;
    },
  };
  render(<Huelle kern={kern} />);
  await screen.findByRole("heading", { name: "Deine Firma an der Börse" });
  return gesendet;
}

describe("Börse", () => {
  afterEach(cleanup);

  it("zeigt den Index und die notierten Firmen und kauft Aktien", async () => {
    const gesendet = await zeige();
    expect(screen.getByText(/^Börsenindex:/)).toBeTruthy();
    const tabelle = screen.getByRole("table", { name: "Börsennotierte Firmen" });
    const zeile = within(tabelle).getAllByRole("row")[1]!;
    const kauf = within(zeile).getByRole("combobox", { name: /kaufen$/ });
    fireEvent.change(kauf, { target: { value: "1" } });
    fireEvent.click(within(zeile).getByRole("button", { name: /^Kaufen:/ }));
    await screen.findByText("Erledigt.");
    expect(gesendet).toEqual([
      { BuyShares: { company: expect.any(Number) as number, share: 0.05 } },
    ]);
  });

  it("zeigt zuerst die größten Firmen, auf Wunsch alle", async () => {
    await zeige();
    const tabelle = screen.getByRole("table", { name: "Börsennotierte Firmen" });
    expect(within(tabelle).getAllByRole("row")).toHaveLength(21);
    fireEvent.click(screen.getByRole("button", { name: /^Alle \d+ Firmen zeigen$/ }));
    expect(within(tabelle).getAllByRole("row").length).toBeGreaterThan(21);
  });

  it("verkauft gehaltene Aktien und zeigt sie im Depot", async () => {
    const gesendet = await zeige((d) => {
      const c = d.companies[0]!;
      c.held = 0.1;
      c.held_value_usd = 1_000_000;
      c.held_cost_usd = 900_000;
      c.sell = [{ share: 0.1, usd: 950_000 }];
      d.portfolio_cost_usd = 900_000;
      d.portfolio_value_usd = 1_000_000;
    });
    expect(screen.getByText(/^Aktien anderer Firmen im Besitz deiner Firma/)).toBeTruthy();
    const tabelle = screen.getByRole("table", { name: "Börsennotierte Firmen" });
    const zeile = within(tabelle).getAllByRole("row")[1]!;
    fireEvent.click(within(zeile).getByRole("button", { name: /^Verkaufen:/ }));
    await screen.findByText("Erledigt.");
    expect(gesendet).toEqual([
      { SellShares: { company: expect.any(Number) as number, share: 0.1 } },
    ]);
  });

  it("verlangt Eigenkapital für den Börsengang", async () => {
    await zeige((d) => {
      d.own.equity_usd = 1000;
    });
    expect(screen.getByText(/braucht deine Firma mindestens/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "An die Börse gehen" })).toBeNull();
  });

  it("geht an die Börse und legt die Dividende fest", async () => {
    const gesendet = await zeige((d) => {
      d.own.equity_usd = d.own.equity_min_usd * 2;
    });
    fireEvent.change(screen.getByLabelText("Neue Aktien"), { target: { value: "3" } });
    expect(screen.getByText(/dein Anteil danach 80 %/)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "An die Börse gehen" }));
    await screen.findByText("Erledigt.");
    expect(gesendet).toEqual([{ GoPublic: { share: 0.2 } }]);
    cleanup();

    const danach = await zeige((d) => {
      d.own.listed = true;
    });
    expect(screen.getByRole("heading", { name: "Kapitalerhöhung" })).toBeTruthy();
    fireEvent.change(screen.getByLabelText(/Ausschüttungsquote/), { target: { value: "30" } });
    fireEvent.click(screen.getByRole("button", { name: "Festlegen" }));
    await screen.findByText("Erledigt.");
    expect(danach).toEqual([{ SetDividend: { payout: 0.3 } }]);
  });

  it("bietet für eine Firma und kauft eigene Aktien zurück", async () => {
    const gesendet = await zeige((d) => {
      d.own.listed = true;
      d.own.player_stake = 0.45;
      d.own.owners = [
        { kind: "anleger", name: null, share: 0.55 },
        { kind: "du", name: null, share: 0.45 },
      ];
      d.own.buyback = [{ share: 0.05, usd: 50_000 }];
    });
    expect(screen.getByText(/Du hältst nur 45 % deiner Firma/)).toBeTruthy();
    const eigner = screen.getByRole("table", { name: "Eigner deiner Firma" });
    expect(within(eigner).getByText("Anleger (Streubesitz)")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Aktien zurückkaufen" }));
    await screen.findByText("Erledigt.");
    const tabelle = screen.getByRole("table", { name: "Börsennotierte Firmen" });
    const zeile = within(tabelle).getAllByRole("row")[2]!;
    fireEvent.click(within(zeile).getAllByRole("button")[0]!);
    fireEvent.click(await screen.findByRole("button", { name: /^Übernehmen für/ }));
    await screen.findAllByText("Erledigt.");
    expect(gesendet[0]).toEqual({ BuyBackShares: { share: 0.05 } });
    expect(gesendet[1]).toEqual({ TakeOver: { company: expect.any(Number) as number } });
  });

  it("warnt, bevor neue Aktien die Mehrheit kosten", async () => {
    await zeige((d) => {
      d.own.equity_usd = d.own.equity_min_usd * 2;
      d.own.issue = d.own.issue.map((x) => ({ ...x, loses_majority: x.share > 0.3 }));
    });
    fireEvent.change(screen.getByLabelText("Neue Aktien"), { target: { value: "7" } });
    expect(screen.getByRole("alert").textContent).toContain("nicht mehr die Mehrheit");
  });
});
