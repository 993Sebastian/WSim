// Plays the requests of `crates/wsim-web/examples/zustand.rs` in the WebAssembly module
// and prints the state hash; CI compares it with the native result (the browser
// version computes the same game as the desktop app).
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const datei = join(dirname(fileURLToPath(import.meta.url)), "../public/wsim_web.wasm");
let exporte;
const { instance } = await WebAssembly.instantiate(readFileSync(datei), {
  env: {
    wsim_fortschritt() {},
    wsim_absturz(ptr, len) {
      const text = new TextDecoder().decode(new Uint8Array(exporte.memory.buffer, ptr, len));
      console.error(`Absturz: ${text}`);
    },
  },
});
exporte = instance.exports;
exporte.wsim_start();

function anfrage(text) {
  const bytes = new TextEncoder().encode(text);
  const ptr = exporte.wsim_alloc(bytes.length);
  new Uint8Array(exporte.memory.buffer, ptr, bytes.length).set(bytes);
  const gepackt = exporte.wsim_anfrage(ptr, bytes.length);
  const p = Number(gepackt >> 32n);
  const l = Number(gepackt & 0xffffffffn);
  const antwort = new TextDecoder().decode(new Uint8Array(exporte.memory.buffer, p, l));
  exporte.wsim_free(p, l);
  if (!antwort.startsWith('{"ok"')) throw new Error(antwort);
  return antwort;
}

anfrage(
  JSON.stringify({
    op: "neues_spiel",
    args: {
      einstellungen: {
        seed: 7,
        start_year: 1900,
        country: "DEU",
        capital_usd: 100000.0,
        start_form: "werkstatt",
        company_name: "Probe",
        companies: 20,
        difficulty: "mittel",
        research_factor: 1.0,
      },
    },
  }),
);
anfrage('{"op": "runde_beenden", "args": {"laenge": "monat"}}');
anfrage('{"op": "runde_beenden", "args": {"laenge": "monat"}}');
console.log(anfrage('{"op": "zustands_hash"}'));
