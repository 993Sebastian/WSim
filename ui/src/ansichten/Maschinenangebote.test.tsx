import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { MaschinenAngebot, Produktion } from "../kern";
import { Befehle } from "./gemeinsam";
import { abweichung, MaschinenAusschreibungen } from "./Maschinenangebote";

const angebot = (teil: Partial<MaschinenAngebot> = {}): MaschinenAngebot => ({
  id: 1,
  builder: "arg_pampa_ingenier_a",
  builder_country: "ARG",
  received: "1907-07-10",
  valid_until: "1907-07-24",
  price_usd: 900_000,
  catalog_price_usd: 1_000_000,
  build_days: 45,
  catalog_build_days: 60,
  ready: "1907-08-31",
  factors: { output: 1.19, labor: 0.95, inputs: 1, scrap: 1.1, area: 1.2 },
  area_unit_m2: 240,
  area_total_m2: 480,
  free_before_m2: 1000,
  free_after_m2: 520,
  materials: [{ product: "eisen", quantity: 1.5, price_usd: 80 }],
  material_usd: 120,
  unit_cost_usd: 300,
  catalog_unit_cost_usd: 330,
  ...teil,
});

const produktion = (angebote: MaschinenAngebot[]) =>
  ({
    units: { eisen: "t" },
    tenders: [
      {
        id: 7,
        site: 0,
        kind: "bauen",
        product: "eisen",
        size: "mittel",
        count: 2,
        slot: null,
        start: "1907-07-01",
        day: 18,
        days: 28,
        ends: "1907-07-28",
        offers: angebote,
      },
    ],
  }) as unknown as Produktion;

describe("Maschinenangebote (MAA)", () => {
  afterEach(cleanup);

  it("formatiert Abweichungen mit Vorzeichen", () => {
    expect(abweichung(1.19)).toBe("+19 %");
    expect(abweichung(0.93)).toBe("−7 %");
    expect(abweichung(1)).toBe("±0 %");
  });

  it("zeigt Abweichungen, Fläche, Bauzeit und Kosten und nimmt ein Angebot an", () => {
    const senden = vi.fn(async () => ({ fehler: false }));
    render(
      <Befehle senden={senden as never} meldung={null}>
        <MaschinenAusschreibungen site={0} produktion={produktion([angebot()])} />
      </Befehle>,
    );
    expect(screen.getByText(/Tag 18 von 28/)).toBeTruthy();
    expect(screen.getByText("+19 %")).toBeTruthy();
    expect(screen.getByText("45 Tage")).toBeTruthy();
    expect(screen.getByText(/gesamt 480 m²/)).toBeTruthy();
    expect(screen.getByText("1.000 m² → 520 m²")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Annehmen" }));
    expect(senden).toHaveBeenCalledWith("angebot/7/1", expect.any(String), [
      { AcceptMachineOffer: { tender: 7, offer: 1 } },
    ]);
  });

  it("sperrt das Annehmen, wenn die Fläche nicht reicht", () => {
    const senden = vi.fn(async () => ({ fehler: false }));
    render(
      <Befehle senden={senden as never} meldung={null}>
        <MaschinenAusschreibungen
          site={0}
          produktion={produktion([angebot({ free_after_m2: -80 })])}
        />
      </Befehle>,
    );
    expect((screen.getByRole("button", { name: "Annehmen" }) as HTMLButtonElement).disabled).toBe(
      true,
    );
    expect(screen.getByText(/→ reicht nicht/)).toBeTruthy();
  });
});
