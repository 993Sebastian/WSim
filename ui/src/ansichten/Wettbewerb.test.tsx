import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Befehl, Firmendetail, Kern } from "../kern";
import { vorschauKern } from "../kern/vorschau";
import { WettbewerbAnsicht } from "./Wettbewerb";

/** The preview core with one of its companies insolvent and in auction (M38). */
async function kernMitVersteigerung() {
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
    firmen: async () => {
      const f = await vorschau.firmen();
      const pleite = f.companies.find((c) => !c.player);
      if (pleite) pleite.auction_until = "1914-02-15";
      return f;
    },
    firma: async (index) => {
      const d: Firmendetail = await vorschau.firma(index);
      d.company.auction_until = "1914-02-15";
      d.company.departments = ["finanzen", "marketing"];
      d.company.central_staff = 3;
      d.company.moving_to = "AUT";
      d.company.moving_until = "1914-07-01";
      d.areas = [];
      d.licenses = [];
      for (const s of d.sites) {
        s.min_bid_usd = Math.round(s.value.base_usd / 2);
        s.needed = true;
        s.blocked = null;
      }
      return d;
    },
    befehl: async (b) => {
      gesendet.push(b);
      return uebersicht;
    },
  };
  return { kern, uebersicht, gesendet };
}

describe("Versteigerung", () => {
  afterEach(cleanup);

  it("zeigt die insolvente Firma, das Mindestgebot und schickt ein Gebot", async () => {
    const { kern, uebersicht, gesendet } = await kernMitVersteigerung();
    render(
      <WettbewerbAnsicht kern={kern} uebersicht={uebersicht} onGeaendert={() => {}} offene={0} />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Firmen" }));
    const liste = await screen.findByRole("table", { name: "Firmen" });
    const marke = await within(liste).findByText("insolvent – Versteigerung bis 15.02.1914");
    const zeile = marke.closest("tr");
    expect(zeile).not.toBeNull();
    fireEvent.click(within(zeile as HTMLElement).getAllByRole("button")[0] as HTMLElement);

    expect(
      await screen.findByText(/Ihre Standorte werden bis 15.02.1914 versteigert/),
    ).toBeTruthy();
    // Its central departments and a move of its headquarters (ZA4).
    expect(
      screen.getByText(
        "Zentrale: Finanzen, Marketing (3 Angestellte). Verlegt den Hauptsitz nach Österreich (ab 01.07.1914).",
      ),
    ).toBeTruthy();
    const standort = (await screen.findAllByRole("article"))[0] as HTMLElement;
    expect(within(standort).getByText("Mindestgebot:")).toBeTruthy();
    // An insolvent company needs nothing any more: no hint during the auction.
    expect(within(standort).queryByText(/braucht diesen Standort selbst/)).toBeNull();
    const preis = within(standort).getByLabelText("Preis") as HTMLInputElement;
    const vorschlag = Number(preis.value.replace(/\./g, ""));
    expect(vorschlag).toBeGreaterThan(0);
    fireEvent.click(within(standort).getByRole("button", { name: "Bieten" }));
    expect(
      await within(standort).findByText(
        "Gebot abgegeben – der Zuschlag fällt am Ende der Versteigerung.",
      ),
    ).toBeTruthy();
    expect(gesendet).toHaveLength(1);
    const gebot = gesendet[0] as { MakeOffer: { object: { Site: number }; price: number } };
    expect(gebot.MakeOffer.object.Site).toBeGreaterThanOrEqual(0);
    // Hundredths of a cent: the shown price in game dollars times 10 000.
    expect(gebot.MakeOffer.price).toBe(vorschlag * 10_000);
  });
});
