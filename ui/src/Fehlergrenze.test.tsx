import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Fehlergrenze } from "./Fehlergrenze";

let kaputt = true;
function Ansicht() {
  if (kaputt) throw new Error("Wert fehlt");
  return <p>Ansicht da</p>;
}

describe("Fehlergrenze", () => {
  afterEach(cleanup);

  it("zeigt statt eines leeren Fensters den Fehler und geht danach weiter", () => {
    const stumm = vi.spyOn(console, "error").mockImplementation(() => {});
    kaputt = true;
    render(
      <Fehlergrenze>
        <Ansicht />
      </Fehlergrenze>,
    );
    expect(screen.getByRole("heading", { name: "Da ist etwas schiefgelaufen" })).toBeTruthy();
    expect(screen.getByText(/Wert fehlt/)).toBeTruthy();
    kaputt = false;
    fireEvent.click(screen.getByRole("button", { name: "Weiter" }));
    expect(screen.getByText("Ansicht da")).toBeTruthy();
    stumm.mockRestore();
  });
});
