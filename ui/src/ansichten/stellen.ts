// Names of positions and units (MA1–MA3), shared by the organisation chart and the
// concerns.
import type { Einheit, Stellenangabe } from "../kern";
import { landName } from "../format";
import { t } from "../texte";

/**
 * "Werksleitung" for the head of a works, "Landesleitung" and "Kontinentvorstand" for the
 * heads of countries and continents, else the function ("Produktion").
 */
export function stellenName(rolle: string, artText: string): string {
  if (rolle !== "leitung") return t(`bereich.${rolle}`);
  return t(`leitung.${artText.replace("standorttyp.", "").replace("ebene.", "")}`);
}

/** A unit as commands name it, from its key (`standort:3`, `land:DEU`, `kontinent:europa`). */
export function einheitAus(schluessel: string): Einheit {
  const [ebene, wert = ""] = schluessel.split(":");
  if (ebene === "land") return { Country: wert };
  if (ebene === "kontinent") return { Continent: wert };
  return { Site: Number(wert) };
}

/** The position as the core reads it. */
export function stellenangabe(einheit: string, rolle: string): Stellenangabe {
  return { unit: einheitAus(einheit), role: rolle === "leitung" ? "Head" : { Specialist: rolle } };
}

/** Where a unit is: "Werk · Deutschland", "Land · Deutschland", "Kontinent · Europa". */
export function einheitName(e: {
  kind_text: string;
  country: string | null;
  continent: string | null;
}): string {
  const ort = e.country ? landName(e.country) : t(`kontinent.${e.continent ?? ""}`);
  return `${t(e.kind_text)} · ${ort}`;
}
