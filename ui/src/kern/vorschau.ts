// Browser preview: answers with example views of a real game. It makes no game
// decisions of its own; every round returns the same example report.
import beispielJson from "./beispiel.json";
import { KernFehler } from "./fehler";
import type { Kern } from "./index";
import type { Befehl } from "./befehle";
import type {
  AnliegenListe,
  Angebote,
  Finanzen,
  Firmen,
  Firmendetail,
  Forschung,
  Landdetail,
  Managermarkt,
  Markt,
  Ketten,
  Organisation,
  ProduktMarkt,
  Produktion,
  WeltMarkt,
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
  produktion: Produktion;
  markt: Markt;
  produktmaerkte: Record<string, ProduktMarkt>;
  weltmarkt: WeltMarkt;
  ketten: Ketten;
  angebote: Angebote;
  firmen: Firmen;
  firma: Firmendetail;
  forschung: Forschung;
  finanzen: Finanzen;
  organisation: Organisation;
  anliegen: AnliegenListe;
  managermarkt: Managermarkt;
};

/** Commands the preview received (for the UI tests). */
export const vorschauBefehle: Befehl[] = [];
declare global {
  interface Window {
    __wsimBefehle?: Befehl[];
  }
}
if (typeof window !== "undefined") window.__wsimBefehle = vorschauBefehle;

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
    rundeBeenden: async (laenge, fortschritt, bis = "runde") => {
      if (!spiel) throw keinSpiel();
      const total = TAGE[laenge];
      // Several rounds: two in the preview, each with its progress.
      for (let runde = 0; runde < (bis === "runde" ? 1 : 2); runde++) {
        for (let done = 1; done <= total; done++) {
          await new Promise((r) => setTimeout(r, verzoegerungMs));
          fortschritt({ done, total });
        }
      }
      spiel = mitName(beispiel.uebersicht);
      const bericht = kopie(beispiel.bericht);
      return bis === "runde"
        ? bericht
        : { ...bericht, rounds: 2, stop: bis === "jahresende" ? "jahresende" : "weltereignis" };
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
    produktion: async () => {
      if (!spiel) throw keinSpiel();
      return kopie(beispiel.produktion);
    },
    markt: async (land) => {
      if (!spiel) throw keinSpiel();
      return { ...kopie(beispiel.markt), country: land };
    },
    produktmarkt: async (land, produkt) => {
      if (!spiel) throw keinSpiel();
      const markt = beispiel.produktmaerkte[produkt];
      if (!markt) {
        throw new KernFehler({
          kind: "error",
          group: "allgemein",
          key: "vorschau.produkt_fehlt",
          params: {},
          target: null,
        });
      }
      const c = { ...kopie(markt), country: land };
      for (const s of c.sellers) if (s.own) s.company = firma;
      return c;
    },
    ketten: async () => {
      if (!spiel) throw keinSpiel();
      return kopie(beispiel.ketten);
    },
    angebote: async () => {
      if (!spiel) throw keinSpiel();
      return kopie(beispiel.angebote);
    },
    firmen: async () => {
      if (!spiel) throw keinSpiel();
      const f = kopie(beispiel.firmen);
      for (const c of f.companies) if (c.player) c.name = firma;
      return f;
    },
    firma: async (index) => {
      if (!spiel) throw keinSpiel();
      // The preview knows one company in detail; the others show its sites.
      const d = kopie(beispiel.firma);
      const zeile = beispiel.firmen.companies.find((c) => c.index === index);
      if (zeile) d.company = { ...kopie(zeile), name: zeile.player ? firma : zeile.name };
      return d;
    },
    weltmarkt: async (produkt) => {
      if (!spiel) throw keinSpiel();
      return { ...kopie(beispiel.weltmarkt), product: produkt };
    },
    forschung: async () => {
      if (!spiel) throw keinSpiel();
      return kopie(beispiel.forschung);
    },
    finanzen: async () => {
      if (!spiel) throw keinSpiel();
      return kopie(beispiel.finanzen);
    },
    organisation: async () => {
      if (!spiel) throw keinSpiel();
      return kopie(beispiel.organisation);
    },
    anliegen: async () => {
      if (!spiel) throw keinSpiel();
      return kopie(beispiel.anliegen);
    },
    managermarkt: async (standort, stelle) => {
      if (!spiel) throw keinSpiel();
      // The preview knows the market of one position; it shows it for every position.
      return { ...kopie(beispiel.managermarkt), site: standort, role: stelle };
    },
    befehl: async (befehl) => {
      if (!spiel) throw keinSpiel();
      vorschauBefehle.push(kopie(befehl));
      throw new KernFehler({
        kind: "error",
        group: "allgemein",
        key: "vorschau.befehl",
        params: {},
        target: null,
      });
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
