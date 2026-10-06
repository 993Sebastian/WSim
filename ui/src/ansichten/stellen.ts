// Names of positions (MA1, MA2), shared by the organisation chart and the concerns.
import type { Stellenangabe } from "../kern";
import { t } from "../texte";

/** "Werksleitung" for the head of a works, else the function ("Produktion"). */
export function stellenName(rolle: string, artText: string): string {
  if (rolle !== "leitung") return t(`bereich.${rolle}`);
  return t(`leitung.${artText.replace("standorttyp.", "")}`);
}

/** The position as the core reads it. */
export function stellenangabe(site: number, rolle: string): Stellenangabe {
  return { site, role: rolle === "leitung" ? "Head" : { Specialist: rolle } };
}
