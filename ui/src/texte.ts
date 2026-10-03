// Central access to all display texts. The texts themselves live in data/texte/de/.
import uiTexte from "../../data/texte/de/ui.yaml";

export type TextParameter = Record<string, string | number>;

export function erstelleTextkatalog(quelle: Record<string, unknown>): Map<string, string> {
  const katalog = new Map<string, string>();
  for (const [schluessel, wert] of Object.entries(quelle)) {
    if (typeof wert !== "string") {
      throw new Error(`Text „${schluessel}“ ist kein Text, sondern ${typeof wert}.`);
    }
    katalog.set(schluessel, wert);
  }
  return katalog;
}

const katalog = erstelleTextkatalog(uiTexte);

/** Formats a text with {placeholders}. Unknown keys show the key itself so gaps stay visible. */
export function formatiere(vorlage: string, parameter: TextParameter = {}): string {
  return vorlage.replace(/\{(\w+)\}/g, (ganz, name: string) =>
    name in parameter ? String(parameter[name]) : ganz,
  );
}

export function t(schluessel: string, parameter?: TextParameter): string {
  const vorlage = katalog.get(schluessel);
  if (vorlage === undefined) {
    console.warn(`Fehlender Text: ${schluessel}`);
    return schluessel;
  }
  return formatiere(vorlage, parameter);
}
