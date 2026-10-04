// Browser preview: answers with example views of a real game. It makes no game
// decisions of its own; every round returns the same example report.
import beispielJson from "./beispiel.json";
import { KernFehler } from "./fehler";
import type { Kern } from "./index";
import type {
  Landdetail,
  Optionen,
  Rundenbericht,
  Rundenlaenge,
  Spielstand,
  Uebersicht,
  Weltkarte,
} from "./typen";

const beispiel = beispielJson as unknown as {
  optionen: Optionen;
  uebersicht_start: Uebersicht;
  bericht: Rundenbericht;
  uebersicht: Uebersicht;
  weltkarte: Weltkarte;
  laender: Record<string, Landdetail>;
};

const TAGE: Record<Rundenlaenge, number> = { tag: 1, woche: 7, monat: 31, quartal: 92 };

const kopie = <T>(x: T): T => JSON.parse(JSON.stringify(x)) as T;

export function vorschauKern(verzoegerungMs = 15): Kern {
  let spiel: Uebersicht | null = null;
  let firma = "";
  const staende = new Map<string, { stand: Spielstand; uebersicht: Uebersicht }>();
  const keinSpiel = () =>
    new KernFehler({
      kind: "error",
      group: "allgemein",
      key: "fehler.sitzung.kein_spiel",
      params: {},
      target: null,
    });
  const mitName = (u: Uebersicht): Uebersicht => {
    const c = kopie(u);
    c.company.name = firma;
    return c;
  };

  return {
    echt: false,
    info: async () => ({ version: "vorschau" }),
    optionen: async () => kopie(beispiel.optionen),
    neuesSpiel: async (einstellungen) => {
      firma = einstellungen.company_name.trim() || "Neue Firma";
      spiel = mitName(beispiel.uebersicht_start);
      return kopie(spiel);
    },
    uebersicht: async () => {
      if (!spiel) throw keinSpiel();
      return kopie(spiel);
    },
    rundeBeenden: async (laenge, fortschritt) => {
      if (!spiel) throw keinSpiel();
      const total = TAGE[laenge];
      for (let done = 1; done <= total; done++) {
        await new Promise((r) => setTimeout(r, verzoegerungMs));
        fortschritt({ done, total });
      }
      spiel = mitName(beispiel.uebersicht);
      return kopie(beispiel.bericht);
    },
    weltkarte: async () => {
      if (!spiel) throw keinSpiel();
      return kopie(beispiel.weltkarte);
    },
    land: async (schluessel) => {
      if (!spiel) throw keinSpiel();
      const land = beispiel.laender[schluessel];
      if (!land) {
        throw new KernFehler({
          kind: "error",
          group: "allgemein",
          key: "vorschau.land_fehlt",
          params: {},
          target: null,
        });
      }
      const c = kopie(land);
      for (const f of c.companies) if (f.own) f.name = firma;
      return c;
    },
    speichern: async (name) => {
      if (!spiel) throw keinSpiel();
      const stand = { name: name.trim(), date: spiel.date, company: spiel.company.name };
      if (!stand.name) {
        throw new KernFehler({
          kind: "error",
          group: "allgemein",
          key: "fehler.sitzung.name_ungueltig",
          params: {},
          target: null,
        });
      }
      staende.set(stand.name, { stand, uebersicht: kopie(spiel) });
      return stand;
    },
    spielstaende: async () => [...staende.values()].map((s) => s.stand),
    laden: async (name) => {
      const s = staende.get(name);
      if (!s) {
        throw new KernFehler({
          kind: "error",
          group: "allgemein",
          key: "fehler.sitzung.laden",
          params: { fehler: { type: "text", value: name } },
          target: null,
        });
      }
      spiel = kopie(s.uebersicht);
      firma = spiel.company.name;
      return kopie(spiel);
    },
  };
}
