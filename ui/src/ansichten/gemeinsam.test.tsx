import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { afterEach, describe, expect, it } from "vitest";
import type { Zahlart } from "../zahleingabe";
import { ZahlFeld } from "./gemeinsam";

/** A number field whose parent keeps the text and shows the number it was given. */
function Formular({ anfang = "", ...art }: Zahlart & { anfang?: string }) {
  const [text, setText] = useState(anfang);
  const [zahl, setZahl] = useState<number | null>(null);
  return (
    <>
      <ZahlFeld
        name="Betrag"
        wert={text}
        onWert={(t, z) => {
          setText(t);
          setZahl(z);
        }}
        {...art}
      />
      <output>{String(zahl)}</output>
    </>
  );
}

const wertSetzen = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!;

/** One edit as a browser makes it: beforeinput, the new text with its caret, input. */
function bearbeiten(feld: HTMLInputElement, dazu: string, typ = "insertText") {
  const a = feld.selectionStart ?? 0;
  const b = feld.selectionEnd ?? 0;
  const [von, bis] =
    a !== b
      ? [a, b]
      : typ === "deleteContentBackward"
        ? [a - 1, a]
        : typ === "deleteContentForward"
          ? [a, a + 1]
          : [a, a];
  act(() => {
    feld.dispatchEvent(
      new InputEvent("beforeinput", { bubbles: true, inputType: typ, data: dazu }),
    );
    // The prototype's setter: React must see the change as a typed one.
    wertSetzen.call(feld, feld.value.slice(0, von) + dazu + feld.value.slice(bis));
    feld.setSelectionRange(von + dazu.length, von + dazu.length);
    feld.dispatchEvent(new InputEvent("input", { bubbles: true, inputType: typ, data: dazu }));
  });
}

function tippen(feld: HTMLInputElement, text: string) {
  for (const z of text) bearbeiten(feld, z);
}

function feldMitCursor(feld: HTMLInputElement): string {
  const c = feld.selectionStart ?? 0;
  return `${feld.value.slice(0, c)}|${feld.value.slice(c)}`;
}

function zeigen(art: Zahlart & { anfang?: string } = {}) {
  render(<Formular {...art} />);
  const feld = screen.getByLabelText<HTMLInputElement>("Betrag");
  feld.focus();
  feld.setSelectionRange(feld.value.length, feld.value.length);
  return feld;
}

describe("ZahlFeld", () => {
  afterEach(cleanup);

  it("setzt Tausenderpunkte beim Tippen und meldet die Zahl", () => {
    const feld = zeigen();
    tippen(feld, "200000000");
    expect(feldMitCursor(feld)).toBe("200.000.000|");
    expect(screen.getByRole("status").textContent).toBe("200000000");
    expect(feld.getAttribute("inputmode")).toBe("decimal");
  });

  it("hält den Cursor hinter der getippten Ziffer", () => {
    const feld = zeigen({ anfang: "1.234" });
    feld.setSelectionRange(4, 4);
    tippen(feld, "5");
    expect(feldMitCursor(feld)).toBe("12.35|4");
    // Backspace behind a thousands separator deletes the digit in front of it.
    feld.setSelectionRange(3, 3);
    bearbeiten(feld, "", "deleteContentBackward");
    expect(feldMitCursor(feld)).toBe("1|.354");
    expect(screen.getByRole("status").textContent).toBe("1354");
  });

  it("liest ausgefüllte Zahlen als Ganzes, auch mit Komma", () => {
    const feld = zeigen({ anfang: "1.852,15" });
    feld.setSelectionRange(0, feld.value.length);
    bearbeiten(feld, "1.5");
    expect(feldMitCursor(feld)).toBe("1,5|");
    feld.setSelectionRange(0, feld.value.length);
    bearbeiten(feld, "2.400,50");
    expect(feld.value).toBe("2.400,50");
    expect(screen.getByRole("status").textContent).toBe("2400.5");
    // A change without the browser's report of the edit (set by a script).
    fireEvent.change(feld, { target: { value: "5000000" } });
    expect(feld.value).toBe("5.000.000");
    expect(screen.getByRole("status").textContent).toBe("5000000");
  });

  it("räumt beim Verlassen auf", () => {
    const feld = zeigen({ anfang: "1.000" });
    feld.setSelectionRange(1, 1);
    bearbeiten(feld, "", "deleteContentBackward");
    expect(feldMitCursor(feld)).toBe("|000");
    fireEvent.blur(feld);
    expect(feld.value).toBe("0");
    expect(screen.getByRole("status").textContent).toBe("0");
  });

  it("nimmt in Feldern für ganze Zahlen kein Komma", () => {
    const feld = zeigen({ ganzzahlig: true });
    tippen(feld, "12,5");
    expect(feld.value).toBe("125");
    expect(feld.getAttribute("inputmode")).toBe("numeric");
  });

  it("zeigt ungültige Eingaben an", () => {
    const feld = zeigen({ negativ: true });
    tippen(feld, "-");
    expect(feld.getAttribute("aria-invalid")).toBe("true");
    expect(screen.getByText(/Bitte eine Zahl eingeben/)).toBeTruthy();
    tippen(feld, "5");
    expect(feld.value).toBe("-5");
    expect(feld.getAttribute("aria-invalid")).toBeNull();
  });
});
