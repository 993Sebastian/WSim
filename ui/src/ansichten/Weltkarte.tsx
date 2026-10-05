import { useEffect, useMemo, useState } from "react";
import { formatGeld, formatPreis, formatZahl, landName } from "../format";
import type { KartenLand, Kern, Lagerstaette, WeltMarkt, Weltkarte } from "../kern";
import welt from "../karte/welt.json";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { fehlerText } from "./fehler";
import { Laenderdetail } from "./Laenderdetail";

const pfade = welt as Record<string, string>;

export type Ebene = "absatz" | "lohn" | "bip" | "bevoelkerung" | "rohstoffe" | "standorte";
const EBENEN: Ebene[] = ["absatz", "lohn", "bip", "bevoelkerung", "rohstoffe", "standorte"];
const STUFEN = 5;
/** Price against the reference price: limits of the classes of the sales layer. */
const PREISSTUFEN = [0.9, 1.0, 1.1, 1.3];

const wert: Record<"lohn" | "bip" | "bevoelkerung", (l: KartenLand) => number> = {
  lohn: (l) => l.wage_usd,
  bip: (l) => l.gdp_per_capita_usd,
  bevoelkerung: (l) => l.population,
};
const anzeige: Record<"lohn" | "bip" | "bevoelkerung", (v: number) => string> = {
  lohn: (v) => formatPreis(v, "h"),
  bip: (v) => formatGeld(v),
  bevoelkerung: (v) => formatZahl(v),
};

/** Class limits by quantiles, so every class holds about as many countries. */
function grenzen(werte: number[]): number[] {
  const sortiert = [...werte].sort((a, b) => a - b);
  return Array.from(
    { length: STUFEN - 1 },
    (_, i) => sortiert[Math.floor(((i + 1) * sortiert.length) / STUFEN)] ?? 0,
  );
}

function stufe(v: number, g: number[]): number {
  let s = 0;
  while (s < g.length && v >= g[s]!) s++;
  return s;
}

// Equirectangular map: x = lon + 180, y = 90 − lat (see tools/daten/karte.py).
const punkt = (l: { lat: number; lon: number }) => ({ x: l.lon + 180, y: 90 - l.lat });

export function WeltkarteAnsicht({
  kern,
  datum,
  onGruenden,
}: {
  kern: Kern;
  datum: string;
  /** Opens the form to found a site in this country. */
  onGruenden: (land: string) => void;
}) {
  const [karte, setKarte] = useState<Weltkarte | null>(null);
  const [fehler, setFehler] = useState<string | null>(null);
  const [ebene, setEbene] = useState<Ebene>("absatz");
  const [rohstoff, setRohstoff] = useState<string>("");
  const [produkt, setProdukt] = useState<string>("naegel");
  const [markt, setMarkt] = useState<WeltMarkt | null>(null);
  const [land, setLand] = useState<string | null>(null);

  useEffect(() => {
    if (ebene !== "absatz" || !produkt) return;
    let aktiv = true;
    kern
      .weltmarkt(produkt)
      .then((m) => aktiv && setMarkt(m))
      .catch((e: unknown) => aktiv && setFehler(fehlerText(e)));
    return () => {
      aktiv = false;
    };
  }, [kern, datum, ebene, produkt]);

  useEffect(() => {
    let aktiv = true;
    kern
      .weltkarte()
      .then((k) => {
        if (!aktiv) return;
        setKarte(k);
        setRohstoff((r) => r || k.resources[0] || "");
      })
      .catch((e: unknown) => aktiv && setFehler(fehlerText(e)));
    return () => {
      aktiv = false;
    };
  }, [kern, datum]);

  const klassen = useMemo(() => {
    if (!karte || !(ebene in wert)) return null;
    const f = wert[ebene as keyof typeof wert];
    const g = grenzen(karte.countries.map(f));
    return { g, je: new Map(karte.countries.map((l) => [l.key, stufe(f(l), g)])) };
  }, [karte, ebene]);

  if (!karte) return <FehlerText fehler={fehler} />;

  const absatz =
    ebene === "absatz" && markt?.product === produkt
      ? new Map(markt.countries.map((m) => [m.country, m]))
      : null;
  const einheit = markt ? t(`einheit.${markt.unit}`) : "";
  const titel = (l: KartenLand) => {
    const name = landName(l.key);
    if (ebene === "absatz") {
      const m = absatz?.get(l.key);
      if (!m) return `${name}: ${t("karte.absatz_keine")}`;
      return `${name}: ${t("karte.absatz_titel", {
        preis: `${formatGeld(m.price_usd)}/${einheit}`,
        anteil: formatZahl((m.price_usd / Math.max(m.reference_usd, 1e-9)) * 100),
        nachfrage: `${formatZahl(m.demand_last_month)} ${einheit}`,
        anbieter: m.sellers,
      })}`;
    }
    if (ebene in wert) {
      const k = ebene as keyof typeof wert;
      return `${name}: ${anzeige[k](wert[k](l))}`;
    }
    if (ebene === "standorte")
      return `${name}: ${t("karte.standorte_titel", { eigene: l.own_sites, fremde: l.other_sites })}`;
    const n = karte.deposits.filter((d) => d.country === l.key && d.resource === rohstoff).length;
    return `${name}: ${t("karte.lagerstaetten_titel", { anzahl: n })}`;
  };
  const fuellung = (key: string) => {
    if (ebene === "absatz") {
      const m = absatz?.get(key);
      if (!m || m.reference_usd <= 0 || m.demand_last_month <= 0) return "var(--land)";
      return `var(--skala-${stufe(m.price_usd / m.reference_usd, PREISSTUFEN)})`;
    }
    return klassen ? `var(--skala-${klassen.je.get(key) ?? 0})` : "var(--land)";
  };
  const groessteNachfrage = Math.max(
    1e-9,
    ...(markt?.countries.map((m) => m.demand_last_month) ?? [0]),
  );
  const laender = new Map(karte.countries.map((l) => [l.key, l]));
  const lager = karte.deposits.filter((d) => d.resource === rohstoff);
  const groesste = Math.max(1, ...lager.map((d) => d.max_output_per_year));

  return (
    <div className="weltkarte" id="weltkarte">
      <div className="karten-spalte">
        <div className="karten-steuerung" role="radiogroup" aria-label={t("karte.ebene")}>
          {EBENEN.map((e) => (
            <button
              key={e}
              type="button"
              role="radio"
              aria-checked={ebene === e}
              onClick={() => setEbene(e)}
            >
              {t(`karte.ebene.${e}`)}
            </button>
          ))}
          {ebene === "absatz" && (
            <label>
              {t("karte.produkt")}
              <select
                id="absatz_produkt"
                value={produkt}
                onChange={(e) => setProdukt(e.target.value)}
              >
                {[...karte.products]
                  .sort((a, b) => t(`produkt.${a}`).localeCompare(t(`produkt.${b}`), "de"))
                  .map((p) => (
                    <option key={p} value={p}>
                      {t(`produkt.${p}`)}
                    </option>
                  ))}
              </select>
            </label>
          )}
          {ebene === "rohstoffe" && (
            <label>
              {t("karte.rohstoff")}
              <select id="rohstoff" value={rohstoff} onChange={(e) => setRohstoff(e.target.value)}>
                {karte.resources.map((r) => (
                  <option key={r} value={r}>
                    {t(`produkt.${r}`)}
                  </option>
                ))}
              </select>
            </label>
          )}
        </div>
        <div className="karten-flaeche">
          <svg
            viewBox="0 4 360 146"
            className="kartenbild"
            role="group"
            aria-label={t("karte.titel")}
          >
            <rect x="0" y="4" width="360" height="146" className="meer" />
            {karte.countries.map((l) => {
              const d = pfade[l.key];
              const p = punkt(l);
              const props = {
                role: "button",
                tabIndex: 0,
                "aria-label": landName(l.key),
                "aria-pressed": land === l.key,
                className: `land${land === l.key ? " gewaehlt" : ""}`,
                style: { fill: fuellung(l.key) },
                onClick: () => setLand(l.key),
                onKeyDown: (e: React.KeyboardEvent) =>
                  (e.key === "Enter" || e.key === " ") && setLand(l.key),
              };
              return d ? (
                <path key={l.key} d={d} {...props}>
                  <title>{titel(l)}</title>
                </path>
              ) : (
                <circle key={l.key} cx={p.x} cy={p.y} r={0.9} {...props}>
                  <title>{titel(l)}</title>
                </circle>
              );
            })}
            {absatz &&
              karte.countries
                .filter((l) => (absatz.get(l.key)?.demand_last_month ?? 0) > 0)
                .map((l) => {
                  const m = absatz.get(l.key)!;
                  const p = punkt(l);
                  return (
                    <circle
                      key={l.key}
                      cx={p.x}
                      cy={p.y}
                      r={0.5 + 3 * Math.sqrt(m.demand_last_month / groessteNachfrage)}
                      className={`nachfrage${m.own_sellers > 0 ? " eigen" : ""}`}
                      aria-hidden="true"
                    />
                  );
                })}
            {ebene === "standorte" &&
              karte.countries
                .filter((l) => l.own_sites + l.other_sites > 0)
                .map((l) => {
                  const p = punkt(l);
                  return (
                    <g key={l.key} className="symbol" aria-hidden="true">
                      {l.other_sites > 0 && (
                        <circle
                          cx={p.x}
                          cy={p.y}
                          r={0.8 + Math.sqrt(l.other_sites) * 0.6}
                          className="fremd"
                        />
                      )}
                      {l.own_sites > 0 && (
                        <circle
                          cx={p.x}
                          cy={p.y}
                          r={0.6 + Math.sqrt(l.own_sites) * 0.6}
                          className="eigen"
                        />
                      )}
                    </g>
                  );
                })}
            {ebene === "rohstoffe" &&
              lager.map((d: Lagerstaette, i) => {
                const l = laender.get(d.country);
                if (!l) return null;
                const p = punkt(l);
                const versatz =
                  lager.slice(0, i).filter((x) => x.country === d.country).length * 1.6;
                const r = 0.6 + 2.2 * Math.sqrt(d.max_output_per_year / groesste);
                const art = d.undiscovered
                  ? "unentdeckt"
                  : d.free_concessions > 0
                    ? "frei"
                    : "vergeben";
                return (
                  <circle key={d.key} cx={p.x + versatz} cy={p.y} r={r} className={`lager ${art}`}>
                    <title>
                      {t(`lagerstaette.${d.key}`)} –{" "}
                      {t(`karte.lager.${art}`, { frei: d.free_concessions, gesamt: d.concessions })}
                    </title>
                  </circle>
                );
              })}
          </svg>
          <Legende ebene={ebene} klassen={klassen?.g ?? null} />
        </div>
        <p className="hinweis-links">
          {ebene === "absatz" ? t("karte.absatz_hinweis") : t("karte.hinweis")}
        </p>
      </div>
      {land && (
        <Laenderdetail
          kern={kern}
          schluessel={land}
          datum={karte.date}
          onSchliessen={() => setLand(null)}
          onGruenden={() => onGruenden(land)}
        />
      )}
    </div>
  );
}

function Legende({ ebene, klassen }: { ebene: Ebene; klassen: number[] | null }) {
  if (ebene === "absatz") {
    const texte = [
      `< ${formatZahl(PREISSTUFEN[0]! * 100)} %`,
      ...PREISSTUFEN.slice(1).map(
        (g, i) => `${formatZahl(PREISSTUFEN[i]! * 100)} – ${formatZahl(g * 100)} %`,
      ),
      `≥ ${formatZahl(PREISSTUFEN.at(-1)! * 100)} %`,
    ];
    return (
      <ul className="legende" aria-label={t("karte.legende")}>
        {texte.map((text, i) => (
          <li key={i}>
            <span className="farbe" style={{ background: `var(--skala-${i})` }} />
            {text}
          </li>
        ))}
        <li>
          <span className="punkt nachfrage" />
          {t("karte.legende.nachfrage")}
        </li>
      </ul>
    );
  }
  if (klassen && ebene in anzeige) {
    const f = anzeige[ebene as keyof typeof anzeige];
    return (
      <ul className="legende" aria-label={t("karte.legende")}>
        {Array.from({ length: STUFEN }, (_, i) => (
          <li key={i}>
            <span className="farbe" style={{ background: `var(--skala-${i})` }} />
            {i === 0
              ? `< ${f(klassen[0]!)}`
              : i === STUFEN - 1
                ? `≥ ${f(klassen[i - 1]!)}`
                : `${f(klassen[i - 1]!)} – ${f(klassen[i]!)}`}
          </li>
        ))}
      </ul>
    );
  }
  const eintraege =
    ebene === "standorte"
      ? [
          ["eigen", t("karte.eigene_standorte")],
          ["fremd", t("karte.fremde_standorte")],
        ]
      : [
          ["frei", t("karte.legende.frei")],
          ["vergeben", t("karte.legende.vergeben")],
          ["unentdeckt", t("karte.legende.unentdeckt")],
        ];
  return (
    <ul className="legende" aria-label={t("karte.legende")}>
      {eintraege.map(([k, text]) => (
        <li key={k}>
          <span className={`punkt ${k}`} />
          {text}
        </li>
      ))}
    </ul>
  );
}
