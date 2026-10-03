import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { App } from "./App";
import type { Kern } from "./kern";

const kernMit = (info: Kern["info"], echt = true): Kern => ({ echt, info });

describe("Startbildschirm", () => {
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

  it("meldet einen nicht erreichbaren Kern", async () => {
    render(<App kern={kernMit(() => Promise.reject(new Error("weg")))} />);
    expect(await screen.findByText(/nicht erreichbar: Error: weg/)).toBeTruthy();
  });
});
