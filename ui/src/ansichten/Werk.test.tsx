import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { StandortDetail } from "../kern";
import { Befehle } from "./gemeinsam";
import { Ausstattung } from "./Werk";

describe("Ausstattung (MN2)", () => {
  afterEach(cleanup);

  it("stellt die Ausstattung eines Endprodukts um", async () => {
    const senden = vi.fn(async () => ({ fehler: false }));
    render(
      <Befehle senden={senden as never} meldung={null}>
        <Ausstattung
          s={{ index: 3 } as StandortDetail}
          produkt="fahrrad"
          name="Fahrrad"
          stufe="standard"
        />
      </Befehle>,
    );
    const auswahl = screen.getByLabelText("Ausstattung") as HTMLSelectElement;
    expect(auswahl.value).toBe("standard");
    expect([...auswahl.options].map((o) => o.text)).toEqual(["einfach", "Standard", "Premium"]);
    fireEvent.change(auswahl, { target: { value: "premium" } });
    expect(senden).toHaveBeenCalledWith(
      "ausstattung/3/fahrrad",
      "Fahrrad wird jetzt in der Ausstattung „Premium“ gefertigt.",
      [{ SetTrim: { site: 3, product: "fahrrad", trim: "Premium" } }],
    );
  });
});
