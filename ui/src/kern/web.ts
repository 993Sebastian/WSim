// Browser version: the simulation core runs as WebAssembly in a web worker
// (kern.worker.ts). Requests and answers are the same as in the desktop app.
import { KernFehler } from "./fehler";
import type { Kern } from "./index";
import type { Fortschritt } from "./typen";

interface Offen {
  aufloesen: (wert: unknown) => void;
  ablehnen: (fehler: unknown) => void;
}

type Nachricht =
  { typ: "fortschritt"; done: number; total: number } | { id: number; ok?: unknown; err?: unknown };

export function webKern(): Kern {
  const worker = new Worker(new URL("./kern.worker.ts", import.meta.url), { type: "module" });
  // Next to the page, wherever the page is published (e.g. GitHub Pages under /WSim/).
  worker.postMessage({ typ: "start", wasm: new URL("wsim_web.wasm", document.baseURI).href });
  let naechste = 1;
  const offen = new Map<number, Offen>();
  let fortschritt: ((f: Fortschritt) => void) | null = null;

  worker.onmessage = (ereignis: MessageEvent<Nachricht>) => {
    const d = ereignis.data;
    if ("typ" in d) {
      fortschritt?.({ done: d.done, total: d.total });
      return;
    }
    const o = offen.get(d.id);
    if (!o) return;
    offen.delete(d.id);
    if (d.err !== undefined) o.ablehnen(new KernFehler(d.err));
    else o.aufloesen(d.ok);
  };

  function aufruf<T>(op: string, args?: Record<string, unknown>): Promise<T> {
    return new Promise<T>((aufloesen, ablehnen) => {
      const id = naechste++;
      offen.set(id, { aufloesen: aufloesen as (wert: unknown) => void, ablehnen });
      worker.postMessage({ id, op, args });
    });
  }

  return {
    echt: true,
    info: () => aufruf("kern_info"),
    optionen: () => aufruf("optionen"),
    neuesSpiel: (einstellungen) => aufruf("neues_spiel", { einstellungen }),
    uebersicht: () => aufruf("uebersicht"),
    rundeBeenden: async (laenge, melden, bis = "runde") => {
      fortschritt = melden;
      try {
        return await aufruf("runde_beenden", { laenge, bis });
      } finally {
        fortschritt = null;
      }
    },
    weltkarte: () => aufruf("weltkarte"),
    land: (schluessel) => aufruf("land", { schluessel }),
    produktion: () => aufruf("produktion"),
    markt: (land) => aufruf("markt", { land }),
    produktmarkt: (land, produkt) => aufruf("produktmarkt", { land, produkt }),
    ketten: () => aufruf("ketten"),
    angebote: () => aufruf("angebote"),
    firmen: () => aufruf("firmen"),
    firma: (index) => aufruf("firma", { index }),
    weltmarkt: (produkt) => aufruf("weltmarkt", { produkt }),
    forschung: () => aufruf("forschung"),
    finanzen: () => aufruf("finanzen"),
    befehl: (befehl) => aufruf("befehl", { befehl }),
    speichern: (name) => aufruf("speichern", { name }),
    spielstaende: () => aufruf("spielstaende"),
    laden: (name) => aufruf("laden", { name }),
    spielstandDatei: (name) => aufruf("datei", { name }),
    spielstandEinlesen: (name, bytes) => aufruf("einlesen", { name, bytes }),
  };
}
