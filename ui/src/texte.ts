// Central access to all display texts. The texts themselves live in data/texte/de/:
// the interface texts (ui.yaml) and the names and messages of the game data.
const dateien = import.meta.glob<Record<string, unknown>>("../../data/texte/de/*.yaml", {
  eager: true,
  import: "default",
});

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

const katalog = erstelleTextkatalog(Object.assign({}, ...Object.values(dateien)));

/** Formats a text with {placeholders}. Unknown keys show the key itself so gaps stay visible. */
export function formatiere(vorlage: string, parameter: TextParameter = {}): string {
  return vorlage.replace(/\{(\w+)\}/g, (ganz, name: string) =>
    name in parameter ? String(parameter[name]) : ganz,
  );
}

export function hatText(schluessel: string): boolean {
  return katalog.has(schluessel);
}

export function t(schluessel: string, parameter?: TextParameter): string {
  const vorlage = katalog.get(schluessel);
  if (vorlage === undefined) {
    console.warn(`Fehlender Text: ${schluessel}`);
    return schluessel;
  }
  return formatiere(vorlage, parameter);
}
