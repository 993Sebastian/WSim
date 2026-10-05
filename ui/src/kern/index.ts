// Access to the simulation core. In the desktop app via Tauri IPC; in the browser
// version (build mode "web") the core runs as WebAssembly in a worker; in the browser
// preview (development, UI tests) a stand-in answers with example views of a real game
// (beispiel.json, written by `wsim beispielsichten`).
import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Befehl } from "./befehle";
import { KernFehler } from "./fehler";
import type {
  Finanzen,
  Forschung,
  Markt,
  Ketten,
  ProduktMarkt,
  Produktion,
  WeltMarkt,
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
import { webKern } from "./web";

export type * from "./typen";
export type { Befehl, Preisart } from "./befehle";
export { geld } from "./befehle";
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
  produktion(): Promise<Produktion>;
  markt(land: string): Promise<Markt>;
  produktmarkt(land: string, produkt: string): Promise<ProduktMarkt>;
  /** Production chains of the end products (M25). */
  ketten(): Promise<Ketten>;
  weltmarkt(produkt: string): Promise<WeltMarkt>;
  forschung(): Promise<Forschung>;
  finanzen(): Promise<Finanzen>;
  /** Carries out a decision; answers with the new overview. */
  befehl(befehl: Befehl): Promise<Uebersicht>;
  speichern(name: string): Promise<Spielstand>;
  spielstaende(): Promise<Spielstand[]>;
  laden(name: string): Promise<Uebersicht>;
  /** Browser version: a save as file contents, to move it to another device. */
  spielstandDatei?(name: string): Promise<Uint8Array>;
  /** Browser version: adds a save from a file. */
  spielstandEinlesen?(name: string, bytes: Uint8Array): Promise<Spielstand[]>;
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
  produktion: () => aufruf("produktion"),
  markt: (land) => aufruf("markt", { land }),
  produktmarkt: (land, produkt) => aufruf("produktmarkt", { land, produkt }),
  ketten: () => aufruf("ketten"),
  weltmarkt: (produkt) => aufruf("weltmarkt", { produkt }),
  forschung: () => aufruf("forschung"),
  finanzen: () => aufruf("finanzen"),
  befehl: (befehl) => aufruf("befehl", { befehl }),
  speichern: (name) => aufruf("speichern", { name }),
  spielstaende: () => aufruf("spielstaende"),
  laden: (name) => aufruf("laden", { name }),
};

export function verbindeKern(): Kern {
  if (isTauri()) return tauriKern;
  return import.meta.env.VITE_KERN === "web" ? webKern() : vorschauKern();
}
