// Web worker of the browser version: runs the simulation core (WebAssembly module
// `wsim_web.wasm`, crate `wsim-web`) away from the page, so that the page stays
// responsive during a round, and keeps the saves in the browser's storage (IndexedDB).

interface Exporte {
  memory: WebAssembly.Memory;
  wsim_start(): void;
  wsim_alloc(len: number): number;
  wsim_free(ptr: number, len: number): void;
  wsim_anfrage(ptr: number, len: number): bigint;
  wsim_spielstand(ptr: number, len: number): bigint;
  wsim_spielstand_einlegen(namePtr: number, nameLen: number, ptr: number, len: number): void;
}

interface Anfrage {
  id: number;
  op: string;
  args?: Record<string, unknown>;
}

// The worker's own scope; typed as a worker from the page's point of view.
const scope = self as unknown as Worker;
const enc = new TextEncoder();
const dec = new TextDecoder();

let wasmUrl = "";
let modul: Exporte | null = null;
let absturz: string | null = null;

function schreiben(e: Exporte, bytes: Uint8Array): [number, number] {
  const ptr = e.wsim_alloc(bytes.length);
  new Uint8Array(e.memory.buffer, ptr, bytes.length).set(bytes);
  return [ptr, bytes.length];
}

function lesen(e: Exporte, gepackt: bigint): Uint8Array {
  const ptr = Number(gepackt >> 32n);
  const len = Number(gepackt & 0xffffffffn);
  const kopie = new Uint8Array(e.memory.buffer, ptr, len).slice();
  e.wsim_free(ptr, len);
  return kopie;
}

// --- Saves in IndexedDB ---

const DATENBANK = "wsim";
const ABLAGE = "spielstaende";

function datenbank(): Promise<IDBDatabase> {
  return new Promise((aufloesen, ablehnen) => {
    const anfrage = indexedDB.open(DATENBANK, 1);
    anfrage.onupgradeneeded = () => anfrage.result.createObjectStore(ABLAGE);
    anfrage.onsuccess = () => aufloesen(anfrage.result);
    anfrage.onerror = () => ablehnen(anfrage.error);
  });
}

async function alleSpielstaende(): Promise<[string, Uint8Array][]> {
  const db = await datenbank();
  return new Promise((aufloesen, ablehnen) => {
    const liste: [string, Uint8Array][] = [];
    const zeiger = db.transaction(ABLAGE).objectStore(ABLAGE).openCursor();
    zeiger.onsuccess = () => {
      const c = zeiger.result;
      if (!c) return aufloesen(liste);
      liste.push([String(c.key), new Uint8Array(c.value as ArrayBuffer)]);
      c.continue();
    };
    zeiger.onerror = () => ablehnen(zeiger.error);
  });
}

async function sichern(name: string, bytes: Uint8Array): Promise<void> {
  const db = await datenbank();
  return new Promise((aufloesen, ablehnen) => {
    const t = db.transaction(ABLAGE, "readwrite");
    t.objectStore(ABLAGE).put(bytes.buffer, name);
    t.oncomplete = () => aufloesen();
    t.onerror = () => ablehnen(t.error);
  });
}

// --- The module ---

async function starten(): Promise<Exporte> {
  if (modul) return modul;
  const importe = {
    env: {
      wsim_fortschritt: (done: number, total: number) =>
        scope.postMessage({ typ: "fortschritt", done, total }),
      wsim_absturz: (ptr: number, len: number) => {
        absturz = modul ? dec.decode(new Uint8Array(modul.memory.buffer, ptr, len)) : "Absturz";
      },
    },
  };
  const antwort = await fetch(wasmUrl);
  if (!antwort.ok) throw new Error(`Simulationskern nicht geladen (${antwort.status})`);
  const { instance } = await WebAssembly.instantiate(await antwort.arrayBuffer(), importe);
  const e = instance.exports as unknown as Exporte;
  modul = e;
  e.wsim_start();
  for (const [name, bytes] of await alleSpielstaende()) {
    const [np, nl] = schreiben(e, enc.encode(name));
    const [dp, dl] = schreiben(e, bytes);
    e.wsim_spielstand_einlegen(np, nl, dp, dl);
  }
  return e;
}

function rufe(e: Exporte, op: string, args?: Record<string, unknown>) {
  const [ptr, len] = schreiben(e, enc.encode(JSON.stringify({ op, args: args ?? {} })));
  return JSON.parse(dec.decode(lesen(e, e.wsim_anfrage(ptr, len)))) as {
    ok?: unknown;
    err?: unknown;
  };
}

/** Writes the saves the last request wrote (manual save, autosave) to IndexedDB. */
async function geschriebeneSichern(e: Exporte) {
  const namen = (rufe(e, "geschrieben").ok as string[] | undefined) ?? [];
  for (const name of namen) {
    const [ptr, len] = schreiben(e, enc.encode(name));
    await sichern(name, lesen(e, e.wsim_spielstand(ptr, len)));
  }
}

scope.onmessage = async (ereignis: MessageEvent) => {
  const daten = ereignis.data as Anfrage | { typ: "start"; wasm: string };
  if ("typ" in daten) {
    wasmUrl = daten.wasm;
    return;
  }
  const { id, op, args } = daten;
  try {
    const e = await starten();
    if (op === "datei") {
      // The bytes of a save, to download it.
      const [ptr, len] = schreiben(e, enc.encode(String(args?.name)));
      const bytes = lesen(e, e.wsim_spielstand(ptr, len));
      scope.postMessage(
        bytes.length > 0
          ? { id, ok: bytes }
          : { id, err: `Spielstand „${String(args?.name)}“ nicht gefunden.` },
      );
      return;
    }
    if (op === "einlesen") {
      // A save from a file: into the module and the browser's storage.
      const name = String(args?.name);
      const bytes = args?.bytes as Uint8Array;
      const [np, nl] = schreiben(e, enc.encode(name));
      const [dp, dl] = schreiben(e, bytes);
      e.wsim_spielstand_einlegen(np, nl, dp, dl);
      await sichern(name, bytes.slice());
      scope.postMessage({ id, ...rufe(e, "spielstaende") });
      return;
    }
    const antwort = rufe(e, op, args);
    if (op === "speichern" || op === "runde_beenden") await geschriebeneSichern(e);
    scope.postMessage({ id, ...antwort });
  } catch (fehler) {
    // A trap (panic) leaves the module unusable: the next request starts afresh and
    // reloads the saves; the running game is lost.
    modul = null;
    scope.postMessage({ id, err: absturz ?? String(fehler) });
    absturz = null;
  }
};
