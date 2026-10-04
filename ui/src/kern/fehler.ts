import type { Meldung } from "./typen";

/** A refused request: a message with text key, or a plain text (e.g. data errors). */
export class KernFehler extends Error {
  readonly meldung: Meldung | null;

  constructor(antwort: unknown) {
    const meldung = istMeldung(antwort) ? antwort : null;
    super(meldung ? meldung.key : String(antwort));
    this.meldung = meldung;
  }
}

function istMeldung(x: unknown): x is Meldung {
  return typeof x === "object" && x !== null && "key" in x && "kind" in x;
}
