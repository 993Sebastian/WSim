import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { vorschauKern } from "../kern/vorschau";
import { Laenderdetail } from "./Laenderdetail";

describe("Länderdetail: Regulierung (H2)", () => {
  afterEach(cleanup);

  it("zeigt die geltenden Regeln eines Landes", async () => {
    const kern = vorschauKern(0);
    await kern.neuesSpiel({
      seed: 1,
      start_year: 1914,
      country: "DEU",
      capital_usd: 100_000,
      start_form: "werkstatt",
      company_name: "Test AG",
      companies: 100,
      difficulty: "mittel",
      research_factor: 1,
      found_at_start: true,
    });
    render(
      <Laenderdetail kern={kern} schluessel="GBR" datum="1914-07-01" onSchliessen={() => {}} />,
    );
    const titel = await screen.findByRole("heading", { name: "Regulierung und Umwelt" });
    const liste = titel.nextElementSibling as HTMLElement;
    expect(
      within(liste).getByText(
        /^Fabrikgesetz \(Factory and Workshop Act\) \(seit .*1901\): Löhne 1 % höher$/,
      ),
    ).toBeTruthy();
  });
});

describe("Länderdetail: Rüstung (H3)", () => {
  afterEach(cleanup);

  it("zeigt Militärausgaben und Kriegswirtschaft", async () => {
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
      found_at_start: true,
    });
    const kern = {
      ...vorschau,
      land: async (k: string) => ({
        ...(await vorschau.land(k)),
        military_share: 0.42,
        war_intensity: 1,
      }),
    };
    render(
      <Laenderdetail kern={kern} schluessel="GBR" datum="1916-07-01" onSchliessen={() => {}} />,
    );
    await screen.findByRole("heading", { name: "Rüstung" });
    expect(
      screen.getByText(/Militärausgaben: 42\s?% des BIP\. Kriegswirtschaft zu 100\s?%/),
    ).toBeTruthy();
  });
});
