// Access to the simulation core. In the desktop app via Tauri IPC, in the browser
// preview (development, UI tests) a stand-in that answers with example views of a
// real game (beispiel.json, written by `wsim beispielsichten`).
import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { KernFehler } from "./fehler";
import type {
  Fortschritt,
  Landdetail,
  NeuesSpiel,
  Optionen,
  Rundenbericht,
  Rundenlaenge,
  Spielstand,
  Uebersicht,
  Weltkarte,
} from "./typen";
import { vorschauKern } from "./vorschau";

export type * from "./typen";
export { KernFehler } from "./fehler";

export interface KernInfo {
  version: string;
}

export interface Kern {
  /** False for the browser preview without simulation core. */
  readonly echt: boolean;
  info(): Promise<KernInfo>;
  optionen(): Promise<Optionen>;
  neuesSpiel(einstellungen: NeuesSpiel): Promise<Uebersicht>;
  uebersicht(): Promise<Uebersicht>;
  rundeBeenden(laenge: Rundenlaenge, fortschritt: (f: Fortschritt) => void): Promise<Rundenbericht>;
  weltkarte(): Promise<Weltkarte>;
  land(schluessel: string): Promise<Landdetail>;
  speichern(name: string): Promise<Spielstand>;
  spielstaende(): Promise<Spielstand[]>;
  laden(name: string): Promise<Uebersicht>;
}

async function aufruf<T>(befehl: string, argumente?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(befehl, argumente);
  } catch (antwort) {
    throw new KernFehler(antwort);
  }
}

const tauriKern: Kern = {
  echt: true,
  info: () => aufruf("kern_info"),
  optionen: () => aufruf("optionen"),
  neuesSpiel: (einstellungen) => aufruf("neues_spiel", { einstellungen }),
  uebersicht: () => aufruf("uebersicht"),
  rundeBeenden: async (laenge, fortschritt) => {
    const abmelden = await listen<Fortschritt>("fortschritt", (e) => fortschritt(e.payload));
    try {
      return await aufruf<Rundenbericht>("runde_beenden", { laenge });
    } finally {
      abmelden();
    }
  },
  weltkarte: () => aufruf("weltkarte"),
  land: (schluessel) => aufruf("land", { schluessel }),
  speichern: (name) => aufruf("speichern", { name }),
  spielstaende: () => aufruf("spielstaende"),
  laden: (name) => aufruf("laden", { name }),
};

export function verbindeKern(): Kern {
  return isTauri() ? tauriKern : vorschauKern();
}
