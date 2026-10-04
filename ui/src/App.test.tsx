import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { App } from "./App";
import type { Kern } from "./kern";
import { vorschauKern } from "./kern/vorschau";

const kernMit = (info: Kern["info"], echt = true): Kern => ({ ...vorschauKern(0), echt, info });

describe("Hauptmenü", () => {
  afterEach(cleanup);

  it("zeigt Titel und Version des Kerns", async () => {
    render(<App kern={kernMit(async () => ({ version: "0.1.0" }))} />);
    expect(screen.getByRole("heading", { name: "WSim" })).toBeTruthy();
    expect(await screen.findByText("Simulationskern 0.1.0 bereit")).toBeTruthy();
  });

  it("kennzeichnet die Browser-Vorschau", async () => {
    render(<App kern={kernMit(async () => ({ version: "vorschau" }), false)} />);
    expect(await screen.findByText(/Vorschau im Browser/)).toBeTruthy();
  });

  it("meldet einen nicht erreichbaren Kern und sperrt das neue Spiel", async () => {
    render(<App kern={kernMit(() => Promise.reject(new Error("weg")))} />);
    expect(await screen.findByText(/nicht erreichbar: Error: weg/)).toBeTruthy();
    expect(
      (screen.getByRole("button", { name: "Neues Spiel" }) as HTMLButtonElement).disabled,
    ).toBe(true);
  });
});

describe("Spielablauf", () => {
  afterEach(cleanup);

  it("startet ein Spiel, beendet eine Runde und speichert", async () => {
    render(<App kern={vorschauKern(0)} />);
    fireEvent.click(await screen.findByRole("button", { name: "Neues Spiel" }));
    fireEvent.change(await screen.findByLabelText("Name der Firma"), {
      target: { value: "Test AG" },
    });
    expect((screen.getByLabelText("Startland (Firmensitz)") as HTMLSelectElement).value).toBe(
      "DEU",
    );
    fireEvent.click(screen.getByRole("button", { name: "Spiel starten" }));

    expect(await screen.findByText("Test AG")).toBeTruthy();
    expect(screen.getByText("01.01.1914")).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Wettbewerb" })).toBeTruthy();

    fireEvent.click(screen.getByRole("button", { name: "Runde beenden" }));
    const ereignis = await screen.findByRole("dialog", { name: "Erster Weltkrieg" });
    expect(within(ereignis).getByText(/Österreich-Ungarn erklärt Serbien den Krieg/)).toBeTruthy();
    expect(within(ereignis).getByText(/Deutschland/)).toBeTruthy();
    fireEvent.click(within(ereignis).getByRole("button", { name: "Weiter" }));

    const bericht = await screen.findByRole("dialog", { name: "Rundenbericht" });
    expect(within(bericht).getByRole("heading", { name: "Weltgeschehen" })).toBeTruthy();
    expect(within(bericht).getByRole("heading", { name: "Finanzergebnis" })).toBeTruthy();
    fireEvent.click(within(bericht).getByRole("button", { name: "Weiter" }));
    expect(await screen.findByText("01.08.1914")).toBeTruthy();

    fireEvent.click(screen.getByRole("button", { name: "Speichern" }));
    const dialog = await screen.findByRole("dialog", { name: "Spiel speichern" });
    fireEvent.change(within(dialog).getByLabelText("Name des Spielstands"), {
      target: { value: "Probe" },
    });
    fireEvent.click(within(dialog).getByRole("button", { name: "Speichern" }));
    expect(await within(dialog).findByText("Gespeichert: Probe")).toBeTruthy();
  });
});
