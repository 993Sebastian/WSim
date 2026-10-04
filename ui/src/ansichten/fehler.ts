import { KernFehler } from "../kern";
import { meldungText } from "../format";

/** Text for a refused request: the core's message, or the plain error. */
export function fehlerText(fehler: unknown): string {
  if (fehler instanceof KernFehler && fehler.meldung) return meldungText(fehler.meldung);
  return fehler instanceof Error ? fehler.message : String(fehler);
}
