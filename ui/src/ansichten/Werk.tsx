// The plant view (docs/BEDIENUNG.md): one site with its areas – facilities, purchasing,
// sales, staff and costs. Every decision shows the numbers it rests on next to it, and
// the answer of the core next to its button.
import { Fragment, useEffect, useId, useState, type FormEvent, type ReactNode } from "react";
import {
  ausAnzeige,
  formatDatum,
  formatFlaeche,
  formatGeld,
  formatMenge,
  formatPreis,
  formatProzent,
  formatZahl,
  geldEinheit,
  geldFeld,
  geldSchluessel,
  landName,
  meldungText,
  zahlFeld,
  zahlLesen,
} from "../format";
import type {
  AnlageDetail,
  Angebot,
  Befehl,
  Bezugsquelle,
  Bezugsquellen,
  InternerStandort,
  Kern,
  Preisverlauf,
  Qualitaetsteile,
  Produktion,
  StandortDetail,
  StandortGrundstueck,
  Stueckkosten,
  Uebersicht,
  Verlegeziel,
  Versorgung,
  Vertrag,
} from "../kern";
import { geld } from "../kern";
import { VertragAktionen } from "./Vertraege";
import { t } from "../texte";
import {
  Erklaerung,
  Rueckmeldung,
  SuchAuswahl,
  Unterreiter,
  useAktion,
  ZahlFeld,
} from "./gemeinsam";
import { fehlerText } from "./fehler";
import { Verlauf } from "./Grafik";
import { ForschungImWerk } from "./Forschung";
import { fortgeschrieben } from "./Markt";
import { standortName } from "./stellen";

/** "Werk · China (Firma)" for a site of the group. */
function internName(x: InternerStandort): string {
  const ort = standortName(x.kind_text, x.country, x.name);
  return x.own_company ? ort : `${ort} (${x.company})`;
}

/** Freight and days of the way, or "im Land". */
function wegText(x: InternerStandort, e: string): string {
  if (x.travel_days === 0) return t("werk.intern.im_land");
  return t("werk.intern.weg", {
    fracht: formatPreis(x.freight_per_unit_usd, e),
    tage: x.travel_days,
    zoll: formatProzent(x.customs_rate),
  });
}

type Bereich = "anlagen" | "forschung" | "einkauf" | "verkauf" | "lager" | "personal" | "kosten";

/** Unit of a product as text, e.g. "t" or "Stück". */
export function einheit(produktion: Produktion, produkt: string): string {
  return t(`einheit.${produktion.units[produkt] ?? "t"}`);
}

const produktName = (p: string) => t(`produkt.${p}`);

/** Labor group key "fachkraft.metall" as text. */
export function gruppenName(gruppe: string): string {
  const [stufe, richtung] = gruppe.split(".");
  return t(`qualifikation.${stufe}`) + (richtung ? ` (${t(`fachrichtung.${richtung}`)})` : "");
}

export function ursacheText(a: AnlageDetail): string {
  if (!a.cause) return "";
  const d = a.cause.detail;
  let detail = "";
  if (d && a.cause.key === "ursache.arbeitskraefte") detail = gruppenName(d);
  else if (d) detail = produktName(d);
  return t(a.cause.key, { detail });
}

/** Workers: whole persons from ten on, else one decimal (markets are scaled down). */
export function formatKoepfe(anzahl: number): string {
  return formatZahl(anzahl, anzahl >= 10 ? 0 : 1);
}

export function standortTitel(s: StandortDetail): string {
  return standortName(s.kind_text, s.country, s.name);
}

const BEREICHE: Bereich[] = [
  "anlagen",
  "forschung",
  "einkauf",
  "verkauf",
  "lager",
  "personal",
  "kosten",
];

export function Werk({
  kern,
  standort: s,
  produktion,
  bereich: start = null,
  onZurueck,
  onGeaendert,
}: {
  kern: Kern;
  standort: StandortDetail;
  produktion: Produktion;
  /** Area to show first. */
  bereich?: string | null;
  onZurueck: () => void;
  /** The overview after a decision taken in an own view of the site (research). */
  onGeaendert?: (u: Uebersicht) => void;
}) {
  const [bereich, setBereich] = useState<Bereich>(
    BEREICHE.includes(start as Bereich)
      ? (start as Bereich)
      : s.kind === "ResearchCenter"
        ? "forschung"
        : "anlagen",
  );
  const forschung = s.kind === "ResearchCenter";
  const engpaesse = s.slots.filter((a) => a.cause && a.cause.key !== "ursache.im_bau").length;
  const knapp = s.inputs.filter((v) => !v.own && v.days !== null && v.days < 7).length;
  const fehlend = s.staff.filter((l) => l.employed + 0.05 < l.needed).length;
  const bereiche: { key: Bereich; text: string; zaehler?: number }[] = forschung
    ? [
        { key: "forschung", text: t("werk.forschung") },
        { key: "anlagen", text: t("werk.anlagen"), zaehler: engpaesse },
        { key: "personal", text: t("werk.personal"), zaehler: fehlend },
      ]
    : [
        { key: "anlagen", text: t("werk.anlagen"), zaehler: engpaesse },
        { key: "einkauf", text: t("werk.einkauf"), zaehler: knapp },
        { key: "verkauf", text: t("werk.verkauf") },
        { key: "lager", text: t("werk.lager") },
        { key: "personal", text: t("werk.personal"), zaehler: fehlend },
        { key: "kosten", text: t("werk.kosten") },
      ];
  const titel = standortTitel(s);
  return (
    <section className="werk" aria-label={titel} data-tour="werk">
      <div className="werk-kopf">
        <button type="button" className="schlicht" onClick={onZurueck}>
          ← {t("werk.alle_standorte")}
        </button>
        <h2>
          {titel}
          {s.name && <small>{t(s.kind_text)}</small>}
          {s.deposit && <small>{t(`lagerstaette.${s.deposit}`)}</small>}
        </h2>
        <Umbenennen key={s.name ?? ""} s={s} />
        <Schliessen s={s} titel={titel} onGeschlossen={onZurueck} />
      </div>
      <WerkKennzahlen s={s} />
      <Unterreiter
        name={t("werk.bereiche")}
        bereiche={bereiche}
        aktiv={bereich}
        onWahl={setBereich}
      />
      {bereich === "anlagen" && <Anlagen s={s} produktion={produktion} />}
      {bereich === "forschung" && forschung && (
        <ForschungImWerk kern={kern} site={s.index} onGeaendert={onGeaendert} />
      )}
      {bereich === "einkauf" && <Einkauf kern={kern} s={s} produktion={produktion} />}
      {bereich === "verkauf" && <Verkauf s={s} produktion={produktion} />}
      {bereich === "lager" && (
        <div className="karten">
          <p className="erklaerung">{t("werk.lager_hinweis")}</p>
          <LagerPlatz s={s} />
          <LagerTabelle s={s} produktion={produktion} />
          <Umlagern kern={kern} s={s} produktion={produktion} />
        </div>
      )}
      {bereich === "personal" && <Personal key={s.wage_premium} s={s} />}
      {bereich === "kosten" && <Kosten s={s} produktion={produktion} />}
    </section>
  );
}

/** The site's own name (RenameSite); empty keeps kind and country. */
function Umbenennen({ s }: { s: StandortDetail }) {
  const [name, setName] = useState(s.name ?? "");
  const { los, antwort } = useAktion(`umbenennen/${s.index}`);
  const absenden = (ev: FormEvent) => {
    ev.preventDefault();
    const neu = name.trim();
    void los(
      [{ RenameSite: { site: s.index, name: neu === "" ? null : neu } }],
      neu === "" ? t("werk.name_entfernt") : t("werk.name_gesetzt", { name: neu }),
    );
  };
  return (
    <details className="umbenennen">
      <summary>{t("werk.umbenennen")}</summary>
      <form className="formular-zeile" aria-label={t("werk.umbenennen")} onSubmit={absenden}>
        <label>
          {t("werk.name")}
          <input
            type="text"
            value={name}
            maxLength={40}
            placeholder={`${t(s.kind_text)} ${landName(s.country)}`}
            onChange={(e) => setName(e.target.value)}
          />
        </label>
        <p className="feld-hilfe">{t("werk.name_hilfe")}</p>
        <div className="knopfreihe links">
          <button type="submit">{t("werk.name_speichern")}</button>
        </div>
        <Rueckmeldung meldung={antwort} />
      </form>
    </details>
  );
}

/** Capacity and fill of the store, its costs and the expansion (BuildStorage). */
function LagerPlatz({ s }: { s: StandortDetail }) {
  const [stufen, setStufen] = useState(1);
  const { los, antwort } = useAktion(`lagerausbau/${s.index}`);
  const l = s.storage;
  if (!l) return null;
  return (
    <div className="karte">
      <h3>{t("werk.lagerplatz.titel")}</h3>
      <p className="erklaerung">{t("werk.lagerplatz.erklaerung")}</p>
      <progress max={1} value={Math.min(1, l.share)} aria-label={t("werk.lagerplatz.fuellung")} />
      <dl className="kennzahlen">
        <div>
          <dt>{t("werk.lagerplatz.fuellung")}</dt>
          <dd className={l.full ? "negativ" : undefined}>
            {formatProzent(l.share)} ({formatZahl(l.fill_m3)} m³)
          </dd>
        </div>
        <div>
          <dt>{t("werk.lagerplatz.kapazitaet")}</dt>
          <dd>{formatZahl(l.capacity_m3)} m³</dd>
        </div>
        <div>
          <dt>{t("werk.lagerplatz.warenwert")}</dt>
          <dd>{formatGeld(l.value_usd)}</dd>
        </div>
        <div>
          <dt>{t("werk.lagerplatz.kosten")}</dt>
          <dd>{formatGeld(l.cost_per_day_usd)}</dd>
        </div>
        <div>
          <dt>{t("werk.lagerplatz.stufen")}</dt>
          <dd>{l.steps}</dd>
        </div>
      </dl>
      {l.stops ? (
        <p className="warnung">{t("werk.lagerplatz.voll")}</p>
      ) : (
        l.full && <p className="hinweis">{t("werk.lagerplatz.voll_vorprodukte")}</p>
      )}
      {!l.full && l.external && <p className="hinweis">{t("werk.lagerplatz.fremdlager")}</p>}
      <form
        className="formular-zeile"
        aria-label={t("werk.lagerplatz.ausbauen")}
        onSubmit={(e) => {
          e.preventDefault();
          void los(
            [{ BuildStorage: { site: s.index, steps: stufen } }],
            t("werk.lagerplatz.ausgebaut", { stufen }),
          );
        }}
      >
        <label>
          {t("werk.lagerplatz.stufen_neu")}
          <input
            type="number"
            min={1}
            max={100}
            value={stufen}
            onChange={(e) => setStufen(Math.max(1, Math.round(Number(e.target.value) || 1)))}
          />
        </label>
        <p className="feld-hilfe">
          {t("werk.lagerplatz.ausbau_hilfe", {
            kapazitaet: `${formatZahl(l.step_capacity_m3 * stufen)} m³`,
            kosten: formatGeld(l.step_cost_usd * stufen),
          })}
        </p>
        <div className="knopfreihe links">
          <button type="submit">{t("werk.lagerplatz.ausbauen")}</button>
        </div>
        <Rueckmeldung meldung={antwort} />
      </form>
    </div>
  );
}

/** Closing the site (CloseSite): what it brings and costs, confirmed twice. */
function Schliessen({
  s,
  titel,
  onGeschlossen,
}: {
  s: StandortDetail;
  titel: string;
  onGeschlossen: () => void;
}) {
  const [sicher, setSicher] = useState(false);
  const { los, antwort } = useAktion(`schliessen/${s.index}`);
  const c = s.closing;
  if (!c) return null;
  const zeilen: [string, number][] = [
    [t("werk.schliessen.anlagen"), c.facilities_usd],
    [t("werk.schliessen.gebaeude"), c.building_usd],
    [t("werk.schliessen.grundstueck"), c.land_usd],
    [t("werk.schliessen.lager"), c.stock_usd],
  ];
  return (
    <details className="umbenennen">
      <summary>{t("werk.schliessen.titel")}</summary>
      <form
        aria-label={t("werk.schliessen.titel")}
        onSubmit={(e) => {
          e.preventDefault();
          void los(
            [{ CloseSite: { site: s.index } }],
            t("werk.schliessen.erledigt", { standort: titel }),
          ).then((ok) => ok && onGeschlossen());
        }}
      >
        <p className="erklaerung">{t("werk.schliessen.erklaerung")}</p>
        <table className="tabelle">
          <tbody>
            {zeilen.map(([text, wert]) => (
              <tr key={text}>
                <th scope="row">{text}</th>
                <td className="zahl">{formatGeld(wert)}</td>
              </tr>
            ))}
            <tr>
              <th scope="row">{t("werk.schliessen.erloes")}</th>
              <td className="zahl">{formatGeld(c.proceeds_usd)}</td>
            </tr>
            <tr>
              <th scope="row">{t("werk.schliessen.abfindung")}</th>
              <td className="zahl negativ">{formatGeld(-c.severance_usd)}</td>
            </tr>
            <tr>
              <th scope="row">{t("werk.schliessen.buchwert")}</th>
              <td className="zahl">{formatGeld(c.book_usd)}</td>
            </tr>
            <tr>
              <th scope="row">{t("werk.schliessen.ergebnis")}</th>
              <td className={c.result_usd < 0 ? "zahl negativ" : "zahl"}>
                {formatGeld(c.result_usd)}
              </td>
            </tr>
          </tbody>
        </table>
        {c.blocker ? (
          <p className="warnung">{meldungText(c.blocker)}</p>
        ) : (
          <>
            <label>
              <input
                type="checkbox"
                checked={sicher}
                onChange={(e) => setSicher(e.target.checked)}
              />
              {t("werk.schliessen.bestaetigen", { standort: titel })}
            </label>
            <div className="knopfreihe links">
              <button type="submit" className="gefahr" disabled={!sicher}>
                {t("werk.schliessen.knopf")}
              </button>
            </div>
          </>
        )}
        <Rueckmeldung meldung={antwort} />
      </form>
    </details>
  );
}

function WerkKennzahlen({ s }: { s: StandortDetail }) {
  const benoetigt = s.staff.reduce((summe, l) => summe + l.needed, 0);
  const m = s.last_month;
  return (
    <dl className="kennzahlen">
      <div>
        <dt>{t("uebersicht.umsatz_monat")}</dt>
        <dd>{m ? formatGeld(m.revenue_usd) : "–"}</dd>
      </div>
      <div>
        <dt>{t("uebersicht.ergebnis_monat")}</dt>
        <dd className={m && m.result_usd < 0 ? "negativ" : ""}>
          {m ? formatGeld(m.result_usd) : t("uebersicht.noch_kein_monat")}
        </dd>
      </div>
      <div>
        <dt>{t("werk.beschaeftigte")}</dt>
        <dd>
          {formatKoepfe(s.workers)}
          {benoetigt > s.workers + 0.05 && (
            <small className="warntext">
              {" "}
              {t("werk.von_benoetigt", { anzahl: formatKoepfe(benoetigt) })}
            </small>
          )}
        </dd>
      </div>
      <div>
        <dt>{t("werk.lohnkosten_tag")}</dt>
        <dd>{formatGeld(s.wage_cost_per_day_usd)}</dd>
      </div>
    </dl>
  );
}

// --- Facilities ---

function Anlagen({ s, produktion }: { s: StandortDetail; produktion: Produktion }) {
  return (
    <div className="karten" data-tour="werk-anlagen">
      {s.slots.length === 0 && <p className="gedaempft">{t("werk.keine_anlagen")}</p>}
      {s.slots.map((a) => (
        // New values from the core start the form afresh.
        <AnlageKarte
          key={`${a.index}/${a.recipe}/${a.utilization}`}
          a={a}
          s={s}
          produktion={produktion}
        />
      ))}
      {s.kind === "Extraction" && <Erschliessung s={s} produktion={produktion} />}
      {s.plot && <GrundstueckKarte s={s} plot={s.plot} produktion={produktion} />}
      <AnlageBauen s={s} produktion={produktion} />
    </div>
  );
}

function mengen(produktion: Produktion, liste: [string, number][]): string {
  return liste
    .map(([p, q]) => `${formatMenge(q)} ${einheit(produktion, p)} ${produktName(p)}`)
    .join(" + ");
}

function AnlageKarte({
  a,
  s,
  produktion,
}: {
  a: AnlageDetail;
  s: StandortDetail;
  produktion: Produktion;
}) {
  const name = t(`anlage.${a.facility}`);
  const { los, antwort } = useAktion(`anlage/${s.index}/${a.index}`);
  const [rezept, setRezept] = useState(a.recipe ?? "");
  const [auslastung, setAuslastung] = useState(zahlFeld(a.utilization * 100, 0));
  const [fehler, setFehler] = useState<string | null>(null);
  const rezepte = produktion.recipes.filter((r) => r.facility === a.facility);
  const labor = s.kind === "ResearchCenter";
  const id = useId();
  const imBau = a.ready > produktion.date;
  const ursache = ursacheText(a);

  const absenden = (e: FormEvent) => {
    e.preventDefault();
    const prozent = zahlLesen(auslastung);
    if (prozent === null || prozent < 0 || prozent > 100) {
      setFehler(t("werk.auslastung_bereich"));
      return;
    }
    setFehler(null);
    void los(
      [
        {
          SetProduction: {
            site: s.index,
            slot: a.index,
            recipe: labor ? a.recipe : rezept || null,
            utilization: prozent / 100,
          },
        },
      ],
      t("werk.anlage_geaendert"),
    );
  };

  return (
    <article className="karte" aria-label={name} data-tour="anlage">
      <h3>
        {a.count > 1 ? `${a.count} × ` : ""}
        {name}
        {a.size !== "mittel" && ` (${t(`anlagengroesse.${a.size}`)})`}
        {imBau && <small>{t("uebersicht.im_bau", { datum: formatDatum(a.ready) })}</small>}
        {a.operation !== "laeuft" && a.operation_date && (
          <small>
            {t(a.operation === "stillgelegt" ? "werk.stillgelegt_seit" : "werk.wiederanlauf_bis", {
              datum: formatDatum(a.operation_date),
            })}
          </small>
        )}
      </h3>
      {a.product && (
        <p className="rezeptzeile">
          {a.inputs_per_day.length > 0 && `${mengen(produktion, a.inputs_per_day)} → `}
          <strong>
            {formatMenge(a.planned_per_day)} {einheit(produktion, a.product)}{" "}
            {produktName(a.product)}
          </strong>{" "}
          {t("werk.je_tag_geplant")}
        </p>
      )}
      <dl className="werte">
        {a.product && !imBau && (
          <>
            <dt>{t("werk.gestern")}</dt>
            <dd>
              {t("werk.ist_von_plan", {
                ist: formatMenge(a.made_per_day),
                plan: formatMenge(a.planned_per_day),
                einheit: einheit(produktion, a.product),
              })}
            </dd>
          </>
        )}
        <dt>{a.cause ? t("produktion.ursache") : t("werk.betrieb")}</dt>
        <dd className={a.cause ? "warntext" : "erfolgstext"}>
          {ursache || (a.recipe || labor ? t("produktion.laeuft") : "")}
          {a.cause && <small className="feld-hilfe">{t(`${a.cause.key}.hilfe`)}</small>}
        </dd>
        <dt>{t("werk.zustand")}</dt>
        <dd>{formatProzent(a.condition)}</dd>
        {a.product && (a.development_share ?? 0) > 0 && (
          <>
            <dt>{t("werk.entwicklung")}</dt>
            <dd>
              {t("werk.entwicklung_niveau", { niveau: formatProzent(a.development_share ?? 0) })}
            </dd>
          </>
        )}
      </dl>
      {a.quality && !labor && <QualitaetTeile q={a.quality} />}
      <form
        className="formular-zeile"
        aria-label={t("werk.anlage_steuern", { anlage: name })}
        onSubmit={absenden}
      >
        {!labor && (
          <div className="feld">
            <label htmlFor={`${id}-rezept`}>{t("werk.verfahren")}</label>
            <select id={`${id}-rezept`} value={rezept} onChange={(e) => setRezept(e.target.value)}>
              <option value="">{t("produktion.kein_rezept")}</option>
              {rezepte.map((r) => (
                <option key={r.key} value={r.key}>
                  {t(`rezept.${r.key}`)}
                </option>
              ))}
            </select>
          </div>
        )}
        <ZahlFeld
          name={t("werk.auslastung")}
          einheit="%"
          wert={auslastung}
          onWert={setAuslastung}
          gruppieren={false}
        />
        <button type="submit">{t("werk.uebernehmen")}</button>
        {fehler && <p className="fehlertext">{fehler}</p>}
        <Rueckmeldung meldung={antwort} />
      </form>
      {!imBau && <AnlageWartung a={a} s={s} name={name} />}
      {!imBau && <AnlageUmwelt a={a} s={s} name={name} />}
      {!imBau && <AnlageAbbau a={a} s={s} name={name} />}
    </article>
  );
}

/** Breakdowns and the overhaul of a facility (BP2). */
function AnlageWartung({ a, s, name }: { a: AnlageDetail; s: StandortDetail; name: string }) {
  const { los, antwort } = useAktion(`wartung/${s.index}/${a.index}`);
  const kosten = a.overhaul_cost_usd ?? null;
  if (kosten === null) return null;
  return (
    <section className="anlage-wartung" aria-label={t("werk.wartung_fuer", { anlage: name })}>
      <dl className="werte">
        <dt>{t("werk.ausfaelle_jahr")}</dt>
        <dd>{formatZahl(a.failures_year ?? 0, 1)}</dd>
        {(a.scrap ?? 0) > 0 && (
          <>
            <dt title={t("werk.ausschuss_hilfe")}>{t("werk.ausschuss")}</dt>
            <dd>{formatProzent(a.scrap ?? 0)}</dd>
          </>
        )}
        {a.down_until && (
          <>
            <dt>{t(a.down_overhaul ? "werk.ueberholung_bis" : "werk.ausfall_bis")}</dt>
            <dd className="negativ">{formatDatum(a.down_until)}</dd>
          </>
        )}
      </dl>
      {!a.down_until && (
        <button
          type="button"
          onClick={() =>
            void los(
              [{ Overhaul: { site: s.index, slot: a.index } }],
              t("werk.ueberholt", { anlage: name }),
            )
          }
        >
          {t("werk.ueberholen", { kosten: formatGeld(kosten) })}
        </button>
      )}
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

/** Emissions and retrofitting against pollutants (H2); only for facilities that emit. */
function AnlageUmwelt({ a, s, name }: { a: AnlageDetail; s: StandortDetail; name: string }) {
  const { los, antwort } = useAktion(`umwelt/${s.index}/${a.index}`);
  const co2 = a.co2_t_per_day ?? 0;
  const schadstoff = a.pollutant_kg_per_day ?? 0;
  const kosten = a.retrofit_cost_usd ?? null;
  if (co2 <= 0 && schadstoff <= 0 && kosten === null) return null;
  const stufe = a.retrofit ?? 0;
  const pflicht = a.retrofit_required ?? 0;
  return (
    <section className="anlage-umwelt" aria-label={t("werk.umwelt_fuer", { anlage: name })}>
      <dl className="werte">
        <dt>{t("werk.emissionen")}</dt>
        <dd>
          {t("werk.emissionen_wert", {
            co2: formatZahl(co2, 1),
            schadstoff: formatZahl(schadstoff, 1),
          })}
        </dd>
        <dt>{t("werk.nachruestung")}</dt>
        <dd className={stufe < pflicht ? "negativ" : undefined}>
          {t(pflicht > 0 ? "werk.nachruestung_pflicht" : "werk.nachruestung_stufe", {
            stufe,
            pflicht,
          })}
        </dd>
      </dl>
      {kosten !== null && (
        <button
          type="button"
          onClick={() =>
            void los(
              [{ Retrofit: { site: s.index, slot: a.index } }],
              t("werk.nachgeruestet", { anlage: name }),
            )
          }
        >
          {t("werk.nachruesten", { kosten: formatGeld(kosten) })}
        </button>
      )}
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

/**
 * Shutting down, starting up again and selling (M22): with what each costs or brings,
 * so that the player sees whether a facility is worth keeping.
 */
function AnlageAbbau({ a, s, name }: { a: AnlageDetail; s: StandortDetail; name: string }) {
  const { los, antwort } = useAktion(`abbau/${s.index}/${a.index}`);
  const [einheiten, setEinheiten] = useState(zahlFeld(a.count, 0));
  const [frage, setFrage] = useState(false);
  const id = useId();
  const k = Math.floor(zahlLesen(einheiten) ?? 0);
  const gueltig = k >= 1 && k <= a.count;
  const anteil = gueltig ? k / a.count : 0;
  const erloes = a.sale_value_usd * anteil;
  const buchwert = a.book_value_usd * anteil;
  const befehl = (art: "MothballFacility" | "SellFacility") =>
    art === "MothballFacility"
      ? { MothballFacility: { site: s.index, slot: a.index, count: k } }
      : { SellFacility: { site: s.index, slot: a.index, count: k } };
  return (
    <details className="anlage-abbau">
      <summary>{t("werk.stilllegen_verkaufen")}</summary>
      <dl className="werte">
        <dt>{t("werk.wartung_monat")}</dt>
        <dd>
          {formatGeld(a.maintenance_month_usd)}
          <small className="feld-hilfe">
            {t("werk.wartung_stillgelegt", {
              betrag: formatGeld(a.maintenance_mothballed_month_usd),
            })}
          </small>
        </dd>
        <dt>{t("werk.restbuchwert")}</dt>
        <dd>{formatGeld(a.book_value_usd)}</dd>
        <dt>{t("werk.verkaufserloes")}</dt>
        <dd>{formatGeld(a.sale_value_usd)}</dd>
      </dl>
      {a.count > 1 && (
        <ZahlFeld
          name={t("werk.einheiten")}
          einheit={t("werk.von_einheiten", { anzahl: a.count })}
          ganzzahlig
          wert={einheiten}
          onWert={(w) => {
            setEinheiten(w);
            setFrage(false);
          }}
        />
      )}
      {!gueltig && <p className="fehlertext">{t("werk.einheiten_bereich", { anzahl: a.count })}</p>}
      {frage ? (
        <div className="bestaetigung" role="group" aria-labelledby={`${id}-frage`}>
          <p id={`${id}-frage`}>
            {t(buchwert > erloes ? "werk.verkaufen_frage_verlust" : "werk.verkaufen_frage", {
              anzahl: k,
              anlage: name,
              erloes: formatGeld(erloes),
              verlust: formatGeld(Math.abs(buchwert - erloes)),
            })}
          </p>
          <div className="knopfreihe links">
            <button
              type="button"
              className="gefahr"
              onClick={() => {
                setFrage(false);
                void los(
                  [befehl("SellFacility")],
                  t("werk.verkauft_meldung", {
                    anzahl: k,
                    anlage: name,
                    erloes: formatGeld(erloes),
                  }),
                );
              }}
            >
              {t("werk.verkaufen_ja")}
            </button>
            <button type="button" className="schlicht" onClick={() => setFrage(false)}>
              {t("werk.abbrechen")}
            </button>
          </div>
        </div>
      ) : (
        <div className="knopfreihe links">
          {a.operation === "laeuft" && (
            <button
              type="button"
              disabled={!gueltig}
              onClick={() =>
                void los(
                  [befehl("MothballFacility")],
                  t("werk.stillgelegt_meldung", { anzahl: k, anlage: name }),
                )
              }
            >
              {t("werk.stilllegen")}
            </button>
          )}
          {a.operation === "stillgelegt" && (
            <button
              type="button"
              onClick={() =>
                void los(
                  [{ RestartFacility: { site: s.index, slot: a.index } }],
                  t("werk.angefahren_meldung", { anlage: name, tage: a.restart_days }),
                )
              }
            >
              {t("werk.wieder_anfahren", { kosten: formatGeld(a.restart_cost_usd) })}
            </button>
          )}
          <button
            type="button"
            className="schlicht"
            disabled={!gueltig}
            onClick={() => setFrage(true)}
          >
            {t("werk.verkaufen")}
          </button>
        </div>
      )}
      <p className="erklaerung">
        {t("werk.abbau_erklaerung", {
          tage: a.restart_days,
          kosten: formatGeld(a.restart_cost_usd),
        })}
      </p>
      <Rueckmeldung meldung={antwort} />
      {gueltig && (s.move_targets ?? []).length > 0 && a.cause?.key !== "ursache.im_bau" && (
        <AnlageVerlegen a={a} s={s} name={name} anzahl={k} buchwert={buchwert} />
      )}
    </details>
  );
}

/**
 * Moving units to another site of the same type: free within the company, sold at a
 * chosen price (the book value by default) to another company of the group.
 */
function AnlageVerlegen({
  a,
  s,
  name,
  anzahl,
  buchwert,
}: {
  a: AnlageDetail;
  s: StandortDetail;
  name: string;
  anzahl: number;
  buchwert: number;
}) {
  const { los, antwort } = useAktion(`verlegen/${s.index}/${a.index}`);
  const ziele = s.move_targets ?? [];
  const [ziel, setZiel] = useState(String(ziele[0]?.site ?? ""));
  const [preis, setPreis] = useState(geldFeld(buchwert));
  const z = ziele.find((x) => String(x.site) === ziel) ?? ziele[0];
  const p = zahlLesen(preis);
  const verkauf = z !== undefined && !z.same_company;
  const ortName = (x: Verlegeziel) => {
    const ort = standortName(s.kind_text, x.country, x.name);
    return x.same_company ? ort : `${ort} (${x.company})`;
  };
  return (
    <div className="anlage-verlegen">
      <h4>{t("werk.verlegen_titel")}</h4>
      <label>
        {t("werk.verlegen_ziel")}
        <select value={ziel} onChange={(ev) => setZiel(ev.target.value)}>
          {ziele.map((x) => (
            <option key={x.site} value={String(x.site)}>
              {ortName(x)}
            </option>
          ))}
        </select>
      </label>
      {verkauf && (
        <ZahlFeld
          name={t("werk.verlegen_preis")}
          einheit={geldEinheit()}
          wert={preis}
          onWert={setPreis}
        />
      )}
      <p className="erklaerung">
        {t(verkauf ? "werk.verlegen_verkauf_hilfe" : "werk.verlegen_hilfe", {
          tage: a.restart_days,
          buchwert: formatGeld(buchwert),
        })}
      </p>
      <div className="knopfreihe links">
        <button
          type="button"
          disabled={z === undefined || (verkauf && (p === null || p < 0))}
          onClick={() => {
            if (z === undefined) return;
            void los(
              [
                {
                  MoveFacility: {
                    site: s.index,
                    slot: a.index,
                    count: anzahl,
                    to: z.site,
                    ...(verkauf ? { price: geld(ausAnzeige(p ?? 0)) } : {}),
                  },
                },
              ],
              t("werk.verlegt_meldung", { anzahl, anlage: name, ziel: ortName(z) }),
            );
          }}
        >
          {t(verkauf ? "werk.verlegen_verkaufen" : "werk.verlegen_knopf")}
        </button>
      </div>
      <Rueckmeldung meldung={antwort} />
    </div>
  );
}

/** The plot of a site: room left for facilities, bought or leased (M35). */
function GrundstueckKarte({
  s,
  plot: g,
  produktion,
}: {
  s: StandortDetail;
  plot: StandortGrundstueck;
  produktion: Produktion;
}) {
  const { los, antwort } = useAktion(`grundstueck/${s.index}`);
  const anteil = g.area_ha > 0 ? Math.min(g.used_ha / g.area_ha, 1) : 1;
  // Full when not even one more unit of any facility of this site type fits.
  const voll = produktion.facilities
    .filter((f) => f.site_type === s.kind)
    .every((f) => (g.fits[f.key] ?? [0]).every((n) => n === 0));
  return (
    <section className="karte" aria-label={t("grundstueck.titel")} data-tour="werk-grundstueck">
      <h3>{t("grundstueck.titel")}</h3>
      <dl className="werte">
        <dt>{t("grundstueck.lage")}</dt>
        <dd>
          {t(`lage.${g.location}`)}
          <small className="feld-hilfe">{t(`grundstueck.lage_${g.location}`)}</small>
        </dd>
        <dt>{t("grundstueck.flaeche")}</dt>
        <dd>
          {t("grundstueck.belegt", {
            belegt: formatFlaeche(g.used_ha),
            gesamt: formatFlaeche(g.area_ha),
            anteil: formatProzent(anteil),
          })}
          <span className="fuellstand" aria-hidden="true">
            <span style={{ width: `${anteil * 100}%` }} />
          </span>
        </dd>
        <dt>{t("grundstueck.besitz")}</dt>
        <dd>
          {g.owned
            ? t("grundstueck.eigen", { wert: formatGeld(g.value_usd) })
            : t("grundstueck.gepachtet", {
                pacht: formatGeld(g.rent_usd_year ?? 0),
                wert: formatGeld(g.value_usd),
              })}
        </dd>
      </dl>
      {voll && <p className="warntext">{t("grundstueck.voll")}</p>}
      {!g.owned && (
        <div className="knopfreihe links">
          <button
            type="button"
            onClick={() =>
              void los(
                [{ BuyPlot: { site: s.index } }],
                t("grundstueck.gekauft_meldung", { preis: formatGeld(g.value_usd) }),
              )
            }
          >
            {t("grundstueck.kaufen_knopf", { preis: formatGeld(g.value_usd) })}
          </button>
        </div>
      )}
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

function AnlageBauen({ s, produktion }: { s: StandortDetail; produktion: Produktion }) {
  const baubar = produktion.facilities
    .filter((f) => f.site_type === s.kind)
    .sort((a, b) => a.investment_usd - b.investment_usd);
  const [anlage, setAnlage] = useState(baubar[0]?.key ?? "");
  const [anzahl, setAnzahl] = useState("1");
  const [groesse, setGroesse] = useState("Medium");
  const { los, antwort } = useAktion(`bauen/${s.index}`);
  const id = useId();
  if (baubar.length === 0) return null;
  const f = baubar.find((x) => x.key === anlage) ?? baubar[0]!;
  const rezepte = produktion.recipes.filter((r) => f.recipes.includes(r.key));
  const zahl = Math.floor(zahlLesen(anzahl) ?? 0);
  // The chosen size (M36); without sizes the data size.
  const nr = Math.max(
    0,
    f.sizes.findIndex((g) => g.size === groesse),
  );
  const g = f.sizes[nr] ?? {
    size: "Medium",
    key: "mittel",
    capacity: 1,
    investment_usd: f.investment_usd,
    build_days: f.build_days,
    area_ha: f.area_ha,
    labor_per_unit: 1,
  };
  // Units that still fit on the plot (none: no plot, no limit).
  const passen = s.plot ? (s.plot.fits[f.key]?.[nr] ?? 0) : null;
  const zuGross = passen !== null && zahl > passen;
  const keinePasst = (key: string) => (s.plot?.fits[key] ?? [0]).every((n) => n === 0);
  return (
    <form
      className="karte bauen"
      aria-label={t("produktion.bauen")}
      onSubmit={(e) => {
        e.preventDefault();
        if (zahl < 1) return;
        void los(
          [{ BuildFacility: { site: s.index, facility: f.key, count: zahl, size: g.size } }],
          t("werk.bau_begonnen", { anlage: t(`anlage.${f.key}`), tage: g.build_days }),
        );
      }}
    >
      <h3>{t("produktion.bauen")}</h3>
      <div className="formular-zeile">
        <div className="feld">
          <label htmlFor={`${id}-anlage`}>{t("werk.anlage")}</label>
          <SuchAuswahl
            id={`${id}-anlage`}
            name={t("werk.anlage")}
            wert={f.key}
            optionen={baubar.map((x) => ({
              wert: x.key,
              text:
                `${t(`anlage.${x.key}`)} – ${formatGeld(x.investment_usd)}` +
                (s.plot && keinePasst(x.key) ? ` (${t("grundstueck.passt_nicht")})` : ""),
            }))}
            onWahl={setAnlage}
          />
        </div>
        {f.sizes.length > 0 && (
          <div className="feld">
            <label htmlFor={`${id}-groesse`}>{t("werk.groesse")}</label>
            <select
              id={`${id}-groesse`}
              value={g.size}
              onChange={(e) => setGroesse(e.target.value)}
            >
              {f.sizes.map((o, i) => (
                <option key={o.size} value={o.size}>
                  {t("werk.groesse_option", {
                    groesse: t(`anlagengroesse.${o.key}`),
                    leistung: formatZahl(o.capacity, 2),
                    betrag: formatGeld(o.investment_usd),
                  })}
                  {s.plot &&
                    (s.plot.fits[f.key]?.[i] ?? 0) === 0 &&
                    ` (${t("grundstueck.passt_nicht")})`}
                </option>
              ))}
            </select>
          </div>
        )}
        <ZahlFeld name={t("produktion.anzahl")} ganzzahlig wert={anzahl} onWert={setAnzahl} />
      </div>
      <p className="erklaerung">{t("werk.groesse_hinweis")}</p>
      <dl className="werte">
        <dt>{t("werk.investition")}</dt>
        <dd>
          {formatGeld(g.investment_usd * Math.max(zahl, 1))}
          {zahl > 1 && (
            <small className="gedaempft">
              {" "}
              ({t("werk.je_anlage", { betrag: formatGeld(g.investment_usd) })})
            </small>
          )}
        </dd>
        <dt>{t("werk.bauzeit")}</dt>
        <dd>{t("produktion.tage", { tage: g.build_days })}</dd>
        <dt>{t("werk.arbeit_je_stueck")}</dt>
        <dd>{formatProzent(g.labor_per_unit)}</dd>
        {passen !== null && (
          <>
            <dt>{t("grundstueck.flaechenbedarf")}</dt>
            <dd>
              {formatFlaeche(g.area_ha * Math.max(zahl, 1))}
              <small className="gedaempft">
                {" "}
                ({t("grundstueck.passen_noch", { anzahl: formatZahl(passen) })})
              </small>
            </dd>
          </>
        )}
        <dt>{t("werk.leistung")}</dt>
        <dd>
          {rezepte.length === 0 && <span className="gedaempft">{t("werk.kein_verfahren")}</span>}
          <ul className="schlicht-liste">
            {rezepte.map((r) => (
              <li key={r.key}>
                {r.inputs_per_day.length > 0 &&
                  `${mengen(
                    produktion,
                    r.inputs_per_day.map(([p, q]) => [p, q * g.capacity]),
                  )} → `}
                {formatMenge(r.output_per_day * g.capacity)} {einheit(produktion, r.product)}{" "}
                {produktName(r.product)} {t("werk.je_tag")}
              </li>
            ))}
          </ul>
        </dd>
      </dl>
      {zuGross && (
        <p className="warntext">
          {passen === 0
            ? t("grundstueck.kein_platz")
            : t("grundstueck.zu_viele", { anzahl: formatZahl(passen) })}
        </p>
      )}
      <div className="knopfreihe links">
        <button type="submit" className="haupt" disabled={zahl < 1 || zuGross}>
          {t("produktion.bauen_knopf")}
        </button>
      </div>
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

function Erschliessung({ s, produktion }: { s: StandortDetail; produktion: Produktion }) {
  const datum = produktion.date;
  const [lager, setLager] = useState(s.free_deposits[0]?.key ?? "");
  const gewaehlt = s.free_deposits.find((d) => d.key === lager);
  const { los, antwort } = useAktion(`erschliessen/${s.index}`);
  const id = useId();
  if (s.deposit) {
    return s.deposit_ready && s.deposit_ready > datum ? (
      <p className="erklaerung">
        {t("produktion.erschliessung_bis", { datum: formatDatum(s.deposit_ready) })}
      </p>
    ) : null;
  }
  if (s.free_deposits.length === 0)
    return (
      <p className="gedaempft">{t("produktion.keine_lagerstaette", { jahr: datum.slice(0, 4) })}</p>
    );
  return (
    <form
      className="karte"
      aria-label={t("produktion.erschliessen")}
      onSubmit={(e) => {
        e.preventDefault();
        void los([{ DevelopDeposit: { site: s.index, deposit: lager } }]);
      }}
    >
      <h3>{t("produktion.erschliessen")}</h3>
      <div className="feld">
        <label htmlFor={`${id}-lager`}>{t("werk.lagerstaette")}</label>
        <select id={`${id}-lager`} value={lager} onChange={(e) => setLager(e.target.value)}>
          {s.free_deposits.map((d) => (
            <option key={d.key} value={d.key}>
              {t(`lagerstaette.${d.key}`)} ({produktName(d.resource)}) – {formatGeld(d.cost_usd)},{" "}
              {t("produktion.bauzeit", { tage: d.days })},{" "}
              {t("produktion.foerderung_jahr", {
                menge: `${formatZahl(d.output_per_year)} ${einheit(produktion, d.resource)}`,
              })}
            </option>
          ))}
        </select>
      </div>
      {gewaehlt && (
        <dl className="werte">
          <dt>{t("produktion.erschliessung_kosten")}</dt>
          <dd>{formatGeld(gewaehlt.cost_usd)}</dd>
          <dt>{t("produktion.erschliessung_dauer")}</dt>
          <dd>{t("produktion.bauzeit", { tage: gewaehlt.days })}</dd>
          <dt>{t("produktion.erschliessung_hoechst")}</dt>
          <dd>
            {formatZahl(gewaehlt.output_per_year)} {einheit(produktion, gewaehlt.resource)}{" "}
            {produktName(gewaehlt.resource)}{" "}
            {t("produktion.erschliessung_je_tag", {
              menge: formatZahl(gewaehlt.output_per_year / 365),
              einheit: einheit(produktion, gewaehlt.resource),
            })}
          </dd>
        </dl>
      )}
      <p className="feld-hilfe">{t("produktion.erschliessung_hilfe")}</p>
      <div className="knopfreihe links">
        <button type="submit">{t("produktion.erschliessen_knopf")}</button>
      </div>
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

// --- Stock of the site ---

/** Scraps a stock position after a question: the goods and their book value are gone. */
function Verschrotten({ s, produkt }: { s: StandortDetail; produkt: string }) {
  const { los, antwort } = useAktion(`verschrotten/${s.index}/${produkt}`);
  const [fragen, setFragen] = useState(false);
  return (
    <>
      {fragen ? (
        <span className="knopfreihe links">
          <button
            type="button"
            className="gefahr"
            onClick={() =>
              void los(
                [{ ScrapStock: { site: s.index, product: produkt } }],
                t("werk.verschrottet", { produkt: produktName(produkt) }),
              ).then(() => setFragen(false))
            }
          >
            {t("werk.verschrotten_ja")}
          </button>
          <button type="button" className="schlicht" onClick={() => setFragen(false)}>
            {t("werk.abbrechen")}
          </button>
        </span>
      ) : (
        <button
          type="button"
          aria-label={t("werk.verschrotten_von", { produkt: produktName(produkt) })}
          onClick={() => setFragen(true)}
        >
          {t("werk.verschrotten")}
        </button>
      )}
      <Rueckmeldung meldung={antwort} />
    </>
  );
}

/** Everything in stock at a site: quantity, quality, use, output, reach and value. */
export function LagerTabelle({ s, produktion }: { s: StandortDetail; produktion: Produktion }) {
  if (s.stock.length === 0) return <p className="gedaempft">{t("werk.lager_leer")}</p>;
  const buchwert = s.stock.reduce((summe, l) => summe + l.value_usd, 0);
  const marktwert = s.stock.reduce((summe, l) => summe + (l.market_value_usd ?? 0), 0);
  const titel = t("werk.lager_von", { standort: standortTitel(s) });
  return (
    <div className="tabelle">
      <table className="mobil-karten" aria-label={titel}>
        <thead>
          <tr>
            <th>{t("vertrag.produkt")}</th>
            <th className="zahl">{t("werk.lager_menge")}</th>
            <th className="zahl">{t("werk.lager_qualitaet")}</th>
            <th className="zahl">{t("werk.lager_verbrauch")}</th>
            <th className="zahl">{t("werk.lager_erzeugung")}</th>
            <th className="zahl">{t("produktion.reichweite")}</th>
            <th className="zahl">{t("werk.lager_stueckkosten")}</th>
            <th className="zahl">{t("werk.lager_buchwert")}</th>
            <th className="zahl">{t("werk.lager_marktwert")}</th>
            <th>{t("vertrag.aktion")}</th>
          </tr>
        </thead>
        <tbody>
          {s.stock.map((l) => {
            const e = einheit(produktion, l.product);
            const tage = l.days ?? null;
            return (
              <tr key={l.product}>
                <td data-titel={t("vertrag.produkt")}>{produktName(l.product)}</td>
                <td className="zahl" data-titel={t("werk.lager_menge")}>
                  {formatMenge(l.quantity)} {e}
                </td>
                <td className="zahl" data-titel={t("werk.lager_qualitaet")}>
                  {l.quality === undefined ? "–" : formatZahl(l.quality, 0)}
                </td>
                <td className="zahl" data-titel={t("werk.lager_verbrauch")}>
                  {l.need_per_day ? `${formatMenge(l.need_per_day)} ${e}` : "–"}
                </td>
                <td className="zahl" data-titel={t("werk.lager_erzeugung")}>
                  {l.output_per_day ? `${formatMenge(l.output_per_day)} ${e}` : "–"}
                </td>
                <td
                  className={`zahl ${tage === null ? "" : `zustand-${reichweiteZustand(tage)}`}`}
                  data-titel={t("produktion.reichweite")}
                >
                  {tage === null ? "–" : t("produktion.tage", { tage: formatZahl(tage, 0) })}
                </td>
                <td className="zahl" data-titel={t("werk.lager_stueckkosten")}>
                  {l.unit_cost_usd === undefined ? "–" : formatPreis(l.unit_cost_usd, e)}
                </td>
                <td className="zahl" data-titel={t("werk.lager_buchwert")}>
                  {formatGeld(l.value_usd)}
                </td>
                <td className="zahl" data-titel={t("werk.lager_marktwert")}>
                  {l.market_value_usd === undefined ? "–" : formatGeld(l.market_value_usd)}
                </td>
                <td data-titel={t("vertrag.aktion")}>
                  <Verschrotten s={s} produkt={l.product} />
                </td>
              </tr>
            );
          })}
        </tbody>
        <tfoot>
          <tr>
            <th colSpan={7}>{t("werk.lager_summe")}</th>
            <td className="zahl" data-titel={t("werk.lager_buchwert")}>
              {formatGeld(buchwert)}
            </td>
            <td className="zahl" data-titel={t("werk.lager_marktwert")}>
              {formatGeld(marktwert)}
            </td>
            <td />
          </tr>
        </tfoot>
      </table>
    </div>
  );
}

// --- Supply contracts of the site (changed under Markt → Lieferverträge) ---

function WerkVertraege({ s, rolle }: { s: StandortDetail; rolle: Vertrag["role"] }) {
  const liste = (s.contracts ?? []).filter((v) => v.role === rolle);
  if (liste.length === 0) return null;
  return (
    <section className="karte" aria-label={t(`werk.vertraege.${rolle}`)}>
      <h3>{t(`werk.vertraege.${rolle}`)}</h3>
      <div className="tabelle">
        <table className="mobil-karten">
          <thead>
            <tr>
              <th>{t("vertrag.produkt")}</th>
              <th>{t("vertrag.partner")}</th>
              <th className="zahl">{t("vertrag.menge")}</th>
              <th className="zahl">{t("vertrag.preis")}</th>
              <th className="zahl">{t("vertrag.geliefert")}</th>
              <th>{t("vertrag.stand")}</th>
              <th>{t("vertrag.aktion")}</th>
            </tr>
          </thead>
          <tbody>
            {liste.map((v) => (
              <tr key={v.id}>
                <td data-titel={t("vertrag.produkt")}>{produktName(v.product)}</td>
                <td data-titel={t("vertrag.partner")}>
                  {v.partner} ({landName(v.partner_country)})
                </td>
                <td className="zahl" data-titel={t("vertrag.menge")}>
                  {formatMenge(v.per_day)} {t(`einheit.${v.unit}`)}
                </td>
                <td className="zahl" data-titel={t("vertrag.preis")}>
                  {formatPreis(v.price_usd, t(`einheit.${v.unit}`))}
                  <br />
                  <span className="gedaempft">
                    {t("vertrag.marktpreis", {
                      preis: formatPreis(v.market_price_usd, t(`einheit.${v.unit}`)),
                    })}
                  </span>
                </td>
                <td className="zahl" data-titel={t("vertrag.geliefert")}>
                  {formatMenge(v.delivered_month)} {t(`einheit.${v.unit}`)}
                </td>
                <td data-titel={t("vertrag.stand")}>
                  {t(`vertrag.status.${v.status}`)}
                  {v.end && (
                    <>
                      <br />
                      <span className="gedaempft">
                        {t("werk.vertraege.bis", { datum: formatDatum(v.end) })}
                      </span>
                    </>
                  )}
                </td>
                <td data-titel={t("vertrag.aktion")}>
                  <VertragAktionen v={v} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p className="erklaerung">{t("werk.vertraege.hinweis")}</p>
    </section>
  );
}

// --- Market price over twelve months, for buying and selling decisions ---

function PreisVerlauf({ v, einheit: e }: { v: Preisverlauf | undefined; einheit: string }) {
  const preise = v?.months.map((m) => m.price_usd) ?? [];
  if (!v || preise.filter((p) => p !== null).length < 2) return null;
  const werte = fortgeschrieben(preise, v.low_usd ?? 0);
  const letzter = werte.at(-1) ?? 0;
  return (
    <details className="preisverlauf">
      <summary>
        {t("werk.preisverlauf")}
        {v.change !== null && (
          <span className="gedaempft">
            {" "}
            {t("werk.preisverlauf_aenderung", {
              wert: `${v.change > 0 ? "+" : ""}${formatZahl(v.change * 100, 1)} %`,
            })}
          </span>
        )}
      </summary>
      <Verlauf
        name={t("werk.preisverlauf")}
        monate={v.months.map((m) => m.month)}
        werte={werte}
        bezug={v.year_ago_usd ?? undefined}
        format={(p) => formatPreis(p, e)}
      />
      <dl className="werte">
        <dt>{t("werk.preisverlauf_jetzt")}</dt>
        <dd>{formatPreis(letzter, e)}</dd>
        {v.year_ago_usd !== null && (
          <>
            <dt>{t("werk.preisverlauf_vorjahr")}</dt>
            <dd>{formatPreis(v.year_ago_usd, e)}</dd>
          </>
        )}
        {v.low_usd !== null && v.high_usd !== null && (
          <>
            <dt>{t("werk.preisverlauf_spanne")}</dt>
            <dd>
              {formatPreis(v.low_usd, e)} – {formatPreis(v.high_usd, e)}
            </dd>
          </>
        )}
      </dl>
      {v.year_ago_usd !== null && <p className="feld-hilfe">{t("werk.preisverlauf_hilfe")}</p>}
    </details>
  );
}

// --- Purchasing ---

function Einkauf({
  kern,
  s,
  produktion,
}: {
  kern: Kern;
  s: StandortDetail;
  produktion: Produktion;
}) {
  const bezogen = (s.supplies ?? [])
    .map((b) => b.product)
    .filter(
      (p) => !s.inputs.some((v) => v.product === p) && !s.orders.some((o) => o.product === p),
    );
  const zeilen: Versorgung[] = [
    ...s.inputs,
    ...[...s.orders.map((o) => o.product), ...bezogen]
      .filter((p) => !s.inputs.some((v) => v.product === p))
      .map((p) => ({
        product: p,
        need_per_day: 0,
        stock: s.stock.find((l) => l.product === p)?.quantity ?? 0,
        days: null,
        own: false,
        ordered: s.orders.some((o) => o.product === p),
        market_price_usd: s.prices[p]?.market_usd ?? 0,
      })),
  ];
  return (
    <div className="karten" data-tour="werk-einkauf">
      <p className="erklaerung">{t("werk.einkauf_hinweis")}</p>
      <WerkVertraege s={s} rolle="einkauf" />
      {zeilen.length === 0 && (
        <p className="gedaempft">
          {t(s.slots.length === 0 ? "werk.kein_einkauf" : "werk.kein_bedarf")}
        </p>
      )}
      {zeilen.map((v) => {
        const auftrag = s.orders.find((o) => o.product === v.product);
        return (
          <EinkaufKarte
            key={`${v.product}/${auftrag?.target}/${auftrag?.max_price_usd}/${auftrag?.min_quality}/${geldSchluessel()}`}
            kern={kern}
            v={v}
            s={s}
            produktion={produktion}
          />
        );
      })}
      <WeiteresProdukt
        tour="einkauf-neu"
        titel={t("produktion.neuer_einkauf")}
        produkte={produktion.products.filter((p) => !zeilen.some((v) => v.product === p))}
      >
        {(p) => (
          <EinkaufKarte
            key={p}
            kern={kern}
            v={{
              product: p,
              need_per_day: 0,
              stock: s.stock.find((l) => l.product === p)?.quantity ?? 0,
              days: null,
              own: false,
              ordered: false,
              market_price_usd: s.prices[p]?.market_usd ?? 0,
            }}
            s={s}
            produktion={produktion}
          />
        )}
      </WeiteresProdukt>
    </div>
  );
}

function reichweiteZustand(tage: number | null): "fehlt" | "knapp" | "gut" {
  const d = tage ?? Infinity;
  return d < 1 ? "fehlt" : d < 7 ? "knapp" : "gut";
}

function EinkaufKarte({
  kern,
  v,
  s,
  produktion,
}: {
  kern: Kern;
  v: Versorgung;
  s: StandortDetail;
  produktion: Produktion;
}) {
  const auftrag = s.orders.find((o) => o.product === v.product);
  const name = produktName(v.product);
  const e = einheit(produktion, v.product);
  const { los, antwort } = useAktion(`einkauf/${s.index}/${v.product}`);
  const vorschlagPreis = v.market_price_usd * 1.1;
  // Without a need here (trade) the player chooses the stock: the field starts empty.
  const [ziel, setZiel] = useState(
    auftrag
      ? zahlFeld(auftrag.target, 1)
      : v.need_per_day > 0
        ? zahlFeld(Math.ceil(v.need_per_day * 20), 1)
        : "",
  );
  const [preis, setPreis] = useState(geldFeld(auftrag ? auftrag.max_price_usd : vorschlagPreis));
  const [qualitaet, setQualitaet] = useState(zahlFeld(auftrag?.min_quality ?? 0, 0));
  const [fehler, setFehler] = useState<string | null>(null);
  const zustand = reichweiteZustand(v.days);

  const absenden = (ev: FormEvent) => {
    ev.preventDefault();
    const z = zahlLesen(ziel);
    const p = zahlLesen(preis);
    const q = zahlLesen(qualitaet);
    if (z === null || z <= 0 || p === null || p <= 0) {
      setFehler(t("werk.einkauf_werte"));
      return;
    }
    if (q === null || q < 0 || q > 100) {
      setFehler(t("werk.mindestqualitaet_bereich"));
      return;
    }
    setFehler(null);
    void los(
      [
        {
          SetPurchase: {
            site: s.index,
            product: v.product,
            target: z,
            max_price: geld(ausAnzeige(p)),
            min_quality: q,
          },
        },
      ],
      t("werk.einkauf_gesetzt", { produkt: name }),
    );
  };

  return (
    <article
      className="karte"
      aria-label={t("produktion.einkauf_von", { produkt: name })}
      data-tour={auftrag ? "einkauf-auftrag" : "einkauf-karte"}
    >
      <h3>
        {name}
        {v.own && <span className="marke">{t("produktion.eigen")}</span>}
      </h3>
      <dl className="werte">
        <dt>{t("produktion.bedarf")}</dt>
        <dd>{v.need_per_day > 0 ? `${formatMenge(v.need_per_day)} ${e}` : "–"}</dd>
        <dt>{t("uebersicht.lager")}</dt>
        <dd>
          {formatMenge(v.stock)} {e}
        </dd>
        <dt>{t("produktion.reichweite")}</dt>
        <dd className={`zustand-${zustand}`}>
          {v.days === null ? "–" : t("produktion.tage", { tage: formatZahl(v.days, 0) })}
        </dd>
        <dt>{t("werk.marktpreis")}</dt>
        <dd>{v.market_price_usd > 0 ? formatPreis(v.market_price_usd, e) : "–"}</dd>
        <dt>{t("werk.qualitaet_lager")}</dt>
        <dd>{v.quality == null ? "–" : formatZahl(v.quality, 0)}</dd>
        {auftrag && (
          <>
            <dt>{t("produktion.gekauft_vormonat")}</dt>
            <dd>
              {formatZahl(auftrag.bought_last_month, 1)} {e}
            </dd>
          </>
        )}
      </dl>
      <PreisVerlauf v={s.price_trends?.[v.product]} einheit={e} />
      {v.own && !auftrag && <p className="erklaerung">{t("werk.eigenes_vorprodukt")}</p>}
      <form
        className="formular-zeile"
        aria-label={t("werk.einkauf_festlegen", { produkt: name })}
        onSubmit={absenden}
      >
        <ZahlFeld
          name={t("werk.ziellager")}
          einheit={e}
          wert={ziel}
          onWert={setZiel}
          hilfe={t("werk.ziellager_hilfe")}
        />
        <ZahlFeld
          name={t("werk.hoechstpreis")}
          einheit={`${geldEinheit()}/${e}`}
          wert={preis}
          onWert={setPreis}
          hilfe={t("werk.hoechstpreis_hilfe")}
        />
        <ZahlFeld
          name={t("werk.mindestqualitaet")}
          wert={qualitaet}
          onWert={setQualitaet}
          hilfe={t("werk.mindestqualitaet_hilfe")}
          gruppieren={false}
        />
        <div className="knopfreihe links">
          <button type="submit">
            {auftrag ? t("werk.einkauf_aendern") : t("werk.einkauf_starten")}
          </button>
          {auftrag && (
            <button
              type="button"
              className="schlicht"
              onClick={() =>
                void los(
                  [
                    {
                      SetPurchase: {
                        site: s.index,
                        product: v.product,
                        target: 0,
                        max_price: 0,
                        min_quality: 0,
                      },
                    },
                  ],
                  t("werk.einkauf_gestoppt", { produkt: name }),
                )
              }
            >
              {t("produktion.stoppen")}
            </button>
          )}
        </div>
        {fehler && <p className="fehlertext">{fehler}</p>}
        <Rueckmeldung meldung={antwort} />
      </form>
      <InternerBezugKarte kern={kern} s={s} produkt={v.product} einheit={e} />
      <BezugsquellenVergleich
        kern={kern}
        s={s}
        produkt={v.product}
        einheit={e}
        onWahl={(quelle) => {
          setQualitaet(zahlFeld(Math.floor(quelle.quality), 0));
          setPreis(geldFeld(Math.max(quelle.price_usd, ausAnzeige(zahlLesen(preis) ?? 0))));
        }}
      />
      <Pauschalkauf
        key={`${auftrag?.once ?? 0}/${auftrag?.once_max_price_usd ?? 0}/${geldSchluessel()}`}
        v={v}
        s={s}
        einheit={e}
        vorschlagPreis={vorschlagPreis}
      />
    </article>
  );
}

/**
 * Sources of a product (#191): offers on the site's market, traders' imports, the state
 * market and other companies' sites worldwide by contract, with quality and the price
 * delivered. A market source sets the purchase's quality and price limit; a contract
 * source can be proposed here.
 */
function BezugsquellenVergleich({
  kern,
  s,
  produkt,
  einheit: e,
  onWahl,
}: {
  kern: Kern;
  s: StandortDetail;
  produkt: string;
  einheit: string;
  onWahl: (quelle: Bezugsquelle) => void;
}) {
  const [offen, setOffen] = useState(false);
  const [liste, setListe] = useState<Bezugsquellen | null>(null);
  const [fehler, setFehler] = useState<string | null>(null);
  const [vertrag, setVertrag] = useState<Bezugsquelle | null>(null);
  const [sortierung, setSortierung] = useState<"preis" | "qualitaet">("preis");
  useEffect(() => {
    if (!offen) return;
    let aktiv = true;
    kern
      .bezugsquellen(s.index, produkt)
      .then((l) => {
        if (!aktiv) return;
        setListe(l);
        setFehler(null);
      })
      .catch((err: unknown) => aktiv && setFehler(fehlerText(err)));
    return () => {
      aktiv = false;
    };
  }, [kern, s.index, produkt, offen, s.stock]);
  const quellen = [...(liste?.sources ?? [])];
  if (sortierung === "qualitaet") quellen.sort((a, b) => b.quality - a.quality);
  return (
    <details className="bezugsquellen" onToggle={(ev) => setOffen(ev.currentTarget.open)}>
      <summary>{t("werk.bezug.titel")}</summary>
      {fehler && <p className="fehlertext">{fehler}</p>}
      {liste && quellen.length === 0 && <p className="gedaempft">{t("werk.bezug.keine")}</p>}
      {liste && quellen.length > 0 && (
        <>
          <p className="feld-hilfe">{t("werk.bezug.hilfe", { monate: liste.contract_months })}</p>
          <fieldset className="umschalter">
            <legend className="unsichtbar">{t("werk.bezug.sortierung")}</legend>
            {(["preis", "qualitaet"] as const).map((k) => (
              <label key={k}>
                <input
                  type="radio"
                  name={`bezug-${s.index}-${produkt}`}
                  checked={sortierung === k}
                  onChange={() => setSortierung(k)}
                />
                {t(`werk.bezug.nach_${k}`)}
              </label>
            ))}
          </fieldset>
          <div className="tabelle">
            <table className="mobil-karten">
              <thead>
                <tr>
                  <th>{t("werk.bezug.quelle")}</th>
                  <th className="zahl">{t("werk.bezug.qualitaet")}</th>
                  <th className="zahl">{t("werk.bezug.preis")}</th>
                  <th className="zahl">{t("werk.bezug.transport")}</th>
                  <th className="zahl">{t("werk.bezug.verfuegbar")}</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                {quellen.map((q) => (
                  <tr key={`${q.kind}/${q.site ?? ""}`}>
                    <td data-titel={t("werk.bezug.quelle")}>
                      {q.company ?? t(`werk.bezug.art.${q.kind}`)}
                      <br />
                      <span className="gedaempft">
                        {q.company ? `${t(`werk.bezug.art.${q.kind}`)} · ` : ""}
                        {landName(q.country)}
                      </span>
                    </td>
                    <td className="zahl" data-titel={t("werk.bezug.qualitaet")}>
                      {formatZahl(q.quality, 0)}
                    </td>
                    <td className="zahl" data-titel={t("werk.bezug.preis")}>
                      {formatPreis(q.price_usd, e)}
                    </td>
                    <td className="zahl" data-titel={t("werk.bezug.transport")}>
                      {q.delivery_cost_usd > 0 ? formatPreis(q.delivery_cost_usd, e) : "–"}
                    </td>
                    <td className="zahl" data-titel={t("werk.bezug.verfuegbar")}>
                      {q.available === null
                        ? t("werk.bezug.unbegrenzt")
                        : `${formatMenge(q.available)} ${e}${q.kind === "vertrag" ? ` ${t("werk.bezug.je_tag")}` : ""}`}
                    </td>
                    <td>
                      {q.kind === "vertrag" ? (
                        <button type="button" className="schlicht" onClick={() => setVertrag(q)}>
                          {t("werk.bezug.vertrag")}
                        </button>
                      ) : (
                        <button type="button" className="schlicht" onClick={() => onWahl(q)}>
                          {t("werk.bezug.uebernehmen")}
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          {vertrag && (
            <BezugsVertrag
              key={vertrag.site ?? 0}
              s={s}
              produkt={produkt}
              einheit={e}
              liste={liste}
              quelle={vertrag}
              onFertig={() => setVertrag(null)}
            />
          )}
        </>
      )}
    </details>
  );
}

/** A supply contract with a source of the comparison (ProposeContract). */
function BezugsVertrag({
  s,
  produkt,
  einheit: e,
  liste,
  quelle,
  onFertig,
}: {
  s: StandortDetail;
  produkt: string;
  einheit: string;
  liste: Bezugsquellen;
  quelle: Bezugsquelle;
  onFertig: () => void;
}) {
  const { los, antwort } = useAktion(`bezugsvertrag/${s.index}/${produkt}`);
  const frei = quelle.available ?? 0;
  const [menge, setMenge] = useState(
    zahlFeld(Math.min(frei, liste.need_per_day > 0 ? liste.need_per_day : frei), 1),
  );
  const [preis, setPreis] = useState(geldFeld(quelle.price_usd));
  const [monate, setMonate] = useState(String(liste.contract_months));
  const [qualitaet, setQualitaet] = useState(
    zahlFeld(Math.max(0, Math.floor(quelle.quality) - 5), 0),
  );
  const [fehler, setFehler] = useState<string | null>(null);
  const absenden = (ev: FormEvent) => {
    ev.preventDefault();
    const m = zahlLesen(menge);
    const p = zahlLesen(preis);
    const n = zahlLesen(monate);
    const q = zahlLesen(qualitaet);
    if (m === null || m <= 0 || p === null || p <= 0 || n === null || n < 1) {
      setFehler(t("werk.einkauf_werte"));
      return;
    }
    if (q === null || q < 0 || q > 100) {
      setFehler(t("werk.mindestqualitaet_bereich"));
      return;
    }
    setFehler(null);
    void los(
      [
        {
          ProposeContract: {
            seller: quelle.site ?? 0,
            buyer: s.index,
            product: produkt,
            per_day: m,
            price: geld(ausAnzeige(p)),
            months: Math.round(n),
            min_quality: q,
            penalty: liste.contract_penalty,
          },
        },
      ],
      t("vertrag.abgeschlossen"),
    ).then((ok) => ok && onFertig());
  };
  return (
    <form
      className="formular-zeile"
      aria-label={t("werk.bezug.vertrag_mit", { firma: quelle.company ?? "" })}
      onSubmit={absenden}
    >
      <ZahlFeld
        name={t("vertrag.menge")}
        einheit={`${e}/${t("werk.bezug.tag")}`}
        wert={menge}
        onWert={setMenge}
      />
      <ZahlFeld
        name={t("vertrag.preis")}
        einheit={`${geldEinheit()}/${e}`}
        wert={preis}
        onWert={setPreis}
      />
      <ZahlFeld name={t("werk.bezug.monate")} wert={monate} onWert={setMonate} gruppieren={false} />
      <ZahlFeld
        name={t("werk.mindestqualitaet")}
        wert={qualitaet}
        onWert={setQualitaet}
        gruppieren={false}
      />
      <div className="knopfreihe links">
        <button type="submit">{t("werk.bezug.anbieten")}</button>
        <button type="button" className="schlicht" onClick={onFertig}>
          {t("werk.bezug.abbrechen")}
        </button>
      </div>
      {fehler && <p className="fehlertext">{fehler}</p>}
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

/** How the quality of the product made here comes about (docs/FORMELN.md). */
function QualitaetTeile({ q }: { q: Qualitaetsteile }) {
  const teile: [string, number][] = [
    ["grund", q.base],
    ["vorprodukte", q.inputs],
    ["automatisierung", q.automation],
    ["zustand", q.condition],
    ["forschung", q.research],
    ["schulung", q.training],
    ...(q.trim ? [["ausstattung", q.trim] as [string, number]] : []),
  ];
  const punkte = (x: number) => `${x > 0 ? "+" : ""}${formatZahl(x, 1)}`;
  return (
    <details className="qualitaet-teile">
      <summary>{t("werk.qualitaet.titel", { punkte: formatZahl(q.total, 0) })}</summary>
      <table>
        <tbody>
          {teile.map(([k, x]) => (
            <tr key={k}>
              <th scope="row">
                {t(`werk.qualitaet.${k}`)}
                {k === "vorprodukte" && q.input_quality !== null && (
                  <span className="gedaempft">
                    {" "}
                    ({t("werk.qualitaet.durchschnitt", { wert: formatZahl(q.input_quality, 0) })})
                  </span>
                )}
              </th>
              <td className="zahl">{k === "grund" ? formatZahl(x, 1) : punkte(x)}</td>
            </tr>
          ))}
          <tr>
            <th scope="row">{t("werk.qualitaet.gesamt")}</th>
            <td className="zahl">
              <strong>{formatZahl(q.total, 1)}</strong>
            </td>
          </tr>
        </tbody>
      </table>
      <p className="feld-hilfe">{t("werk.qualitaet.hilfe")}</p>
    </details>
  );
}

/**
 * Drawing a product from another site of the company or group (SetInternalSupply): the
 * running supply with what came this month and what is on the way, or a form to choose
 * a source and a fixed quantity a day or stock for a number of days of need.
 */
function InternerBezugKarte({
  kern,
  s,
  produkt,
  einheit: e,
}: {
  kern: Kern;
  s: StandortDetail;
  produkt: string;
  einheit: string;
}) {
  const bezug = (s.supplies ?? []).find((b) => b.product === produkt);
  const name = produktName(produkt);
  const { los, antwort } = useAktion(`intern/${s.index}/${produkt}`);
  const [offen, setOffen] = useState(false);
  const [quellen, setQuellen] = useState<InternerStandort[] | null>(null);
  const [fehler, setFehler] = useState<string | null>(null);
  const [quelle, setQuelle] = useState<number | null>(bezug?.source.site ?? null);
  const [art, setArt] = useState<"fest" | "bedarf">(bezug?.mode ?? "bedarf");
  const [menge, setMenge] = useState(zahlFeld(bezug?.per_day ?? 0, 1));
  const [tage, setTage] = useState(zahlFeld(bezug?.days ?? s.supply_days_default ?? 20, 0));
  const id = useId();
  useEffect(() => {
    if (!offen) return;
    let aktiv = true;
    kern
      .interneQuellen(s.index, produkt)
      .then((l) => {
        if (!aktiv) return;
        setQuellen(l);
        setFehler(null);
        setQuelle((q) => q ?? l[0]?.site ?? null);
      })
      .catch((err: unknown) => aktiv && setFehler(fehlerText(err)));
    return () => {
      aktiv = false;
    };
  }, [kern, s.index, produkt, offen]);
  const gewaehlt = quellen?.find((q) => q.site === quelle);
  const absenden = (ev: FormEvent) => {
    ev.preventDefault();
    const wert = zahlLesen(art === "fest" ? menge : tage);
    if (quelle === null || wert === null || wert <= 0) {
      setFehler(t("werk.intern.werte"));
      return;
    }
    setFehler(null);
    void los(
      [
        {
          SetInternalSupply: {
            site: s.index,
            product: produkt,
            source: quelle,
            mode: art === "fest" ? { Fixed: { per_day: wert } } : { Need: { days: wert } },
          },
        },
      ],
      t("werk.intern.gesetzt", { produkt: name }),
    );
  };
  return (
    <details
      className="intern"
      open={bezug !== undefined || undefined}
      onToggle={(ev) => setOffen(ev.currentTarget.open)}
    >
      <summary>{t("werk.intern.titel")}</summary>
      {bezug && (
        <dl className="werte" aria-label={t("werk.intern.laufend")}>
          <dt>{t("werk.intern.quelle")}</dt>
          <dd>
            {internName(bezug.source)}
            <br />
            <span className="gedaempft">{wegText(bezug.source, e)}</span>
          </dd>
          <dt>{t("werk.intern.art")}</dt>
          <dd>
            {bezug.mode === "fest"
              ? t("werk.intern.fest_wert", { menge: `${formatMenge(bezug.per_day ?? 0)} ${e}` })
              : t("werk.intern.bedarf_wert", {
                  tage: formatZahl(bezug.days ?? 0, 0),
                  bedarf: `${formatMenge(bezug.demand_per_day)} ${e}`,
                })}
          </dd>
          <dt>{t("werk.intern.geliefert")}</dt>
          <dd>
            {formatMenge(bezug.delivered_month)} {e}
            {bezug.in_transit > 0 && (
              <span className="gedaempft">
                {" "}
                · {t("werk.intern.unterwegs", { menge: `${formatMenge(bezug.in_transit)} ${e}` })}
              </span>
            )}
          </dd>
          <dt>{t("werk.intern.quelle_lager")}</dt>
          <dd className={bezug.source.spare <= 0 ? "warntext" : undefined}>
            {formatMenge(bezug.source.stock)} {e}
          </dd>
        </dl>
      )}
      {fehler && <p className="fehlertext">{fehler}</p>}
      {quellen && quellen.length === 0 && <p className="gedaempft">{t("werk.intern.keine")}</p>}
      {quellen && quellen.length > 0 && (
        <form
          className="formular-zeile"
          aria-label={t("werk.intern.festlegen", { produkt: name })}
          onSubmit={absenden}
        >
          <div className="feld">
            <label htmlFor={`${id}-quelle`}>{t("werk.intern.quelle")}</label>
            <select
              id={`${id}-quelle`}
              value={quelle ?? ""}
              onChange={(ev) => setQuelle(Number(ev.target.value))}
            >
              {quellen.map((q) => (
                <option key={q.site} value={q.site}>
                  {t("werk.intern.quelle_wahl", {
                    standort: internName(q),
                    lager: `${formatMenge(q.stock)} ${e}`,
                    leistung: `${formatMenge(q.output_per_day)} ${e}`,
                  })}
                </option>
              ))}
            </select>
            {gewaehlt && (
              <small className="feld-hilfe">
                {wegText(gewaehlt, e)}
                {!gewaehlt.own_company &&
                  gewaehlt.unit_value_usd !== null &&
                  ` · ${t("werk.intern.konzernpreis", { preis: formatPreis(gewaehlt.unit_value_usd, e) })}`}
              </small>
            )}
          </div>
          <fieldset className="auswahlgruppe">
            <legend>{t("werk.intern.art")}</legend>
            <label>
              <input
                type="radio"
                name={`${id}-art`}
                checked={art === "bedarf"}
                onChange={() => setArt("bedarf")}
              />
              {t("werk.intern.nach_bedarf")}
            </label>
            <label>
              <input
                type="radio"
                name={`${id}-art`}
                checked={art === "fest"}
                onChange={() => setArt("fest")}
              />
              {t("werk.intern.feste_menge")}
            </label>
          </fieldset>
          {art === "fest" ? (
            <ZahlFeld
              name={t("werk.intern.menge")}
              einheit={`${e}/${t("werk.bezug.tag")}`}
              wert={menge}
              onWert={setMenge}
            />
          ) : (
            <ZahlFeld
              name={t("werk.intern.reichweite")}
              einheit={t("werk.intern.tage")}
              wert={tage}
              onWert={setTage}
              hilfe={t("werk.intern.reichweite_hilfe")}
              gruppieren={false}
            />
          )}
          <div className="knopfreihe links">
            <button type="submit">
              {bezug ? t("werk.intern.aendern") : t("werk.intern.starten")}
            </button>
            {bezug && (
              <button
                type="button"
                className="schlicht"
                onClick={() =>
                  void los(
                    [
                      {
                        SetInternalSupply: {
                          site: s.index,
                          product: produkt,
                          source: null,
                          mode: { Need: { days: 1 } },
                        },
                      },
                    ],
                    t("werk.intern.beendet", { produkt: name }),
                  )
                }
              >
                {t("werk.intern.beenden")}
              </button>
            )}
          </div>
          <Rueckmeldung meldung={antwort} />
        </form>
      )}
    </details>
  );
}

/** Sends goods of the site's stock to another own site once (TransferGoods). */
function Umlagern({
  kern,
  s,
  produktion,
}: {
  kern: Kern;
  s: StandortDetail;
  produktion: Produktion;
}) {
  const waren = s.stock.filter((l) => l.quantity > 0).map((l) => l.product);
  const [produkt, setProdukt] = useState(waren[0] ?? "");
  const [ziele, setZiele] = useState<InternerStandort[] | null>(null);
  const [ziel, setZiel] = useState<number | null>(null);
  const [menge, setMenge] = useState("");
  const [fehler, setFehler] = useState<string | null>(null);
  const { los, antwort } = useAktion(`umlagern/${s.index}`);
  const id = useId();
  useEffect(() => {
    if (!produkt) return;
    let aktiv = true;
    kern
      .lieferziele(s.index, produkt)
      .then((l) => {
        if (!aktiv) return;
        setZiele(l);
        setZiel(l[0]?.site ?? null);
        setFehler(null);
      })
      .catch((err: unknown) => aktiv && setFehler(fehlerText(err)));
    return () => {
      aktiv = false;
    };
  }, [kern, s.index, produkt]);
  if (waren.length === 0) return null;
  const e = einheit(produktion, produkt);
  const lager = s.stock.find((l) => l.product === produkt)?.quantity ?? 0;
  const gewaehlt = ziele?.find((z) => z.site === ziel);
  const absenden = (ev: FormEvent) => {
    ev.preventDefault();
    const m = zahlLesen(menge);
    if (ziel === null || m === null || m <= 0 || m > lager + 1e-9) {
      setFehler(t("werk.umlagern.werte", { lager: `${formatMenge(lager)} ${e}` }));
      return;
    }
    setFehler(null);
    void los(
      [{ TransferGoods: { from: s.index, to: ziel, product: produkt, quantity: m } }],
      t("werk.umlagern.gesendet", { produkt: produktName(produkt) }),
    );
  };
  return (
    <section className="karte" aria-label={t("werk.umlagern.titel")}>
      <h3>{t("werk.umlagern.titel")}</h3>
      <p className="feld-hilfe">{t("werk.umlagern.hilfe")}</p>
      <form className="formular-zeile" aria-label={t("werk.umlagern.titel")} onSubmit={absenden}>
        <div className="feld">
          <label htmlFor={`${id}-ware`}>{t("werk.umlagern.ware")}</label>
          <select id={`${id}-ware`} value={produkt} onChange={(ev) => setProdukt(ev.target.value)}>
            {waren.map((p) => (
              <option key={p} value={p}>
                {produktName(p)}
              </option>
            ))}
          </select>
        </div>
        <div className="feld">
          <label htmlFor={`${id}-ziel`}>{t("werk.umlagern.ziel")}</label>
          <select
            id={`${id}-ziel`}
            value={ziel ?? ""}
            onChange={(ev) => setZiel(Number(ev.target.value))}
          >
            {(ziele ?? []).map((z) => (
              <option key={z.site} value={z.site}>
                {internName(z)}
              </option>
            ))}
          </select>
          {gewaehlt && <small className="feld-hilfe">{wegText(gewaehlt, e)}</small>}
        </div>
        <ZahlFeld
          name={t("werk.umlagern.menge")}
          einheit={e}
          wert={menge}
          onWert={setMenge}
          hilfe={t("werk.umlagern.im_lager", { lager: `${formatMenge(lager)} ${e}` })}
        />
        <div className="knopfreihe links">
          <button type="submit" disabled={!ziele || ziele.length === 0}>
            {t("werk.umlagern.senden")}
          </button>
        </div>
        {ziele && ziele.length === 0 && (
          <p className="gedaempft">{t("werk.umlagern.keine_ziele")}</p>
        )}
        {fehler && <p className="fehlertext">{fehler}</p>}
        <Rueckmeldung meldung={antwort} />
      </form>
    </section>
  );
}

/** Buys a quantity once on top of the stock target (BuyOnce). */
function Pauschalkauf({
  v,
  s,
  einheit: e,
  vorschlagPreis,
}: {
  v: Versorgung;
  s: StandortDetail;
  einheit: string;
  vorschlagPreis: number;
}) {
  const auftrag = s.orders.find((o) => o.product === v.product);
  const offen = auftrag?.once ?? 0;
  const name = produktName(v.product);
  const { los, antwort } = useAktion(`pauschal/${s.index}/${v.product}`);
  const [menge, setMenge] = useState("");
  const [preis, setPreis] = useState(
    geldFeld(offen > 0 ? (auftrag?.once_max_price_usd ?? vorschlagPreis) : vorschlagPreis),
  );
  const [fehler, setFehler] = useState<string | null>(null);
  const senden = (quantity: number, p: number, meldung: string) =>
    void los(
      [
        {
          BuyOnce: { site: s.index, product: v.product, quantity, max_price: geld(ausAnzeige(p)) },
        },
      ],
      meldung,
    );
  const absenden = (ev: FormEvent) => {
    ev.preventDefault();
    const m = zahlLesen(menge);
    const p = zahlLesen(preis);
    if (m === null || m <= 0 || p === null || p <= 0) {
      setFehler(t("werk.pauschal_werte"));
      return;
    }
    setFehler(null);
    senden(m, p, t("werk.pauschal_gesetzt", { produkt: name }));
  };
  return (
    <form
      className="formular-zeile pauschalkauf"
      aria-label={t("werk.pauschal_von", { produkt: name })}
      onSubmit={absenden}
    >
      <h4>{t("werk.pauschal_titel")}</h4>
      {offen > 0 && auftrag && (
        <p className="erklaerung">
          {t("werk.pauschal_offen", {
            menge: `${formatMenge(offen)} ${e}`,
            preis: formatPreis(auftrag.once_max_price_usd ?? 0, e),
          })}
        </p>
      )}
      <ZahlFeld
        name={t("werk.pauschal_menge")}
        einheit={e}
        wert={menge}
        onWert={setMenge}
        hilfe={t("werk.pauschal_hilfe")}
      />
      <ZahlFeld
        name={t("werk.pauschal_preis")}
        einheit={`${geldEinheit()}/${e}`}
        wert={preis}
        onWert={setPreis}
        hilfe={t("werk.hoechstpreis_hilfe")}
      />
      <div className="knopfreihe links">
        <button type="submit">{t("werk.pauschal_kaufen")}</button>
        {offen > 0 && (
          <button
            type="button"
            className="schlicht"
            onClick={() => senden(0, 0, t("werk.pauschal_gestoppt", { produkt: name }))}
          >
            {t("werk.pauschal_abbrechen")}
          </button>
        )}
      </div>
      {fehler && <p className="fehlertext">{fehler}</p>}
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

// --- Sales ---

function Verkauf({ s, produktion }: { s: StandortDetail; produktion: Produktion }) {
  const angebotbar = new Set([
    ...s.slots.map((a) => a.product).filter((p): p is string => !!p),
    ...s.stock.map((l) => l.product),
  ]);
  const ohneAngebot = [...angebotbar].filter((p) => !s.offers.some((o) => o.product === p));
  return (
    <div className="karten" data-tour="werk-verkauf">
      <p className="erklaerung">{t("werk.verkauf_hinweis")}</p>
      <WerkVertraege s={s} rolle="verkauf" />
      {s.offers.map((o) => (
        <AngebotKarte
          key={`${o.product}/${o.mode}/${o.price_usd}/${o.floor_usd}/${o.keep}/${geldSchluessel()}`}
          produkt={o.product}
          o={o}
          s={s}
          produktion={produktion}
        />
      ))}
      {ohneAngebot.map((p) => (
        <AngebotKarte
          key={`${p}/${geldSchluessel()}`}
          produkt={p}
          o={null}
          s={s}
          produktion={produktion}
        />
      ))}
      <WeiteresProdukt
        tour="angebot-neu"
        titel={t("produktion.neues_angebot")}
        produkte={produktion.products.filter(
          (p) => !s.offers.some((o) => o.product === p) && !ohneAngebot.includes(p),
        )}
      >
        {(p) => (
          <AngebotKarte
            key={`${p}/${geldSchluessel()}`}
            produkt={p}
            o={null}
            s={s}
            produktion={produktion}
          />
        )}
      </WeiteresProdukt>
    </div>
  );
}

function AngebotKarte({
  produkt,
  o,
  s,
  produktion,
}: {
  produkt: string;
  o: Angebot | null;
  s: StandortDetail;
  produktion: Produktion;
}) {
  const name = produktName(produkt);
  const e = einheit(produktion, produkt);
  const { los, antwort } = useAktion(`verkauf/${s.index}/${produkt}`);
  const preise = s.prices[produkt];
  const markt = o?.market_price_usd ?? preise?.market_usd ?? 0;
  const richt = o?.reference_usd ?? preise?.reference_usd ?? 0;
  const stueck = o?.unit_cost_usd ?? s.unit_costs.find((u) => u.product === produkt)?.total_usd;
  // Both automatic modes are shown as one: the market price with a deviation.
  const [art, setArt] = useState<"markt" | "fest" | "aufschlag">(
    o?.mode === "fest" || o?.mode === "aufschlag" ? o.mode : "markt",
  );
  const [abweichung, setAbweichung] = useState(
    zahlFeld((o && (o.mode === "markt" || o.mode === "marktbezug") ? o.markup : 0) * 100, 1),
  );
  const [marge, setMarge] = useState(zahlFeld((o?.mode === "aufschlag" ? o.markup : 0.2) * 100, 1));
  const preisFeld = geldFeld(o ? o.price_usd : markt);
  const [preis, setPreis] = useState(preisFeld);
  const [mindest, setMindest] = useState(geldFeld(o ? o.floor_usd : 0));
  const [behalten, setBehalten] = useState(zahlFeld(o ? o.keep : 0, 1));
  const [fehler, setFehler] = useState<string | null>(null);
  const id = useId();
  const lager = o?.stock ?? s.stock.find((l) => l.product === produkt)?.quantity ?? 0;
  const eigenbedarf = o?.used_here ?? s.inputs.some((v) => v.product === produkt);

  const absenden = (ev: FormEvent) => {
    ev.preventDefault();
    const p = art === "fest" ? zahlLesen(preis) : 1;
    const m = art === "markt" ? zahlLesen(mindest || "0") : 0;
    const k = zahlLesen(behalten || "0");
    const a = art === "aufschlag" ? zahlLesen(marge) : 0;
    const d = art === "markt" ? zahlLesen(abweichung || "0") : 0;
    if (p === null || p <= 0 || m === null || m < 0 || k === null || k < 0) {
      setFehler(t("werk.verkauf_werte"));
      return;
    }
    if (d === null || d < -90 || d > 200) {
      setFehler(t("werk.abweichung_bereich"));
      return;
    }
    if (a === null || a < -50 || a > 1000) {
      setFehler(t("werk.kostenaufschlag_marge_bereich"));
      return;
    }
    setFehler(null);
    const befehle: Befehl[] = [
      {
        SetSale: {
          site: s.index,
          product: produkt,
          mode:
            art === "aufschlag"
              ? { CostPlus: { margin: a / 100 } }
              : art === "fest"
                ? { Fixed: geld(ausAnzeige(p)) }
                : {
                    Relative: {
                      deviation: d / 100,
                      floor: geld(ausAnzeige(m)),
                    },
                  },
          keep: k,
        },
      },
    ];
    void los(befehle, t(o ? "werk.angebot_geaendert" : "werk.angebot_neu", { produkt: name }));
  };

  return (
    <article
      className="karte"
      aria-label={t("produktion.verkauf_von", { produkt: name })}
      data-tour={o ? "angebot-aktiv" : "angebot-karte"}
    >
      <h3>
        {name}
        {o?.product_name && <small>„{o.product_name}“</small>}
        {!o && <small>{t("werk.nicht_angeboten")}</small>}
      </h3>
      <dl className="werte">
        <dt>{t("uebersicht.lager")}</dt>
        <dd>
          {formatMenge(lager)} {e}
        </dd>
        {o && (
          <>
            <dt>{t("produktion.verkauft_vormonat")}</dt>
            <dd>
              {formatZahl(o.sold_last_month, 1)} {e}
            </dd>
            <dt>{t("produktion.verkauft_monat")}</dt>
            <dd>
              {formatZahl(o.sold_month, 1)} {e}
            </dd>
          </>
        )}
      </dl>
      <PreisVerlauf v={s.price_trends?.[produkt]} einheit={e} />
      <Preisvergleich
        einheit={e}
        eigen={o?.price_usd ?? null}
        markt={markt}
        richt={richt}
        stueck={stueck ?? null}
        teile={s.unit_costs.find((u) => u.product === produkt) ?? null}
        marge={o?.margin ?? null}
      />
      <form
        className="formular-spalten"
        aria-label={t("werk.preis_festlegen", { produkt: name })}
        onSubmit={absenden}
      >
        <fieldset className="auswahlgruppe">
          <legend>{t("werk.preisart")}</legend>
          <label>
            <input
              type="radio"
              name={`${id}-art`}
              checked={art === "markt"}
              onChange={() => setArt("markt")}
            />
            {t("werk.automatisch")}
          </label>
          <label>
            <input
              type="radio"
              name={`${id}-art`}
              checked={art === "fest"}
              onChange={() => setArt("fest")}
            />
            {t("werk.fest")}
          </label>
          <label>
            <input
              type="radio"
              name={`${id}-art`}
              checked={art === "aufschlag"}
              onChange={() => setArt("aufschlag")}
            />
            {t("werk.kostenaufschlag")}
          </label>
          <small className="feld-hilfe">
            {t(
              art === "markt"
                ? "werk.automatisch_hilfe"
                : art === "fest"
                  ? "werk.fest_hilfe"
                  : "werk.kostenaufschlag_hilfe",
            )}
          </small>
        </fieldset>
        {o?.mode === "aufschlag" && o.price_usd > markt && markt > 0 && (
          <p className="warntext" role="status">
            {t("werk.kostenaufschlag_warnung", {
              preis: formatPreis(o.price_usd, e),
              markt: formatPreis(markt, e),
            })}
          </p>
        )}
        <div className="formular-zeile" data-tour={o ? "preis" : undefined}>
          {art === "aufschlag" && (
            <ZahlFeld
              name={t("werk.kostenaufschlag_marge")}
              einheit="%"
              wert={marge}
              onWert={setMarge}
              hilfe={t("werk.kostenaufschlag_marge_hilfe")}
              gruppieren={false}
              negativ
            />
          )}
          {art === "fest" && (
            <ZahlFeld
              name={t("werk.preis")}
              einheit={`${geldEinheit()}/${e}`}
              wert={preis}
              onWert={setPreis}
            />
          )}
          {art === "markt" && (
            <ZahlFeld
              name={t("werk.abweichung")}
              einheit="%"
              wert={abweichung}
              onWert={setAbweichung}
              hilfe={t("werk.abweichung_hilfe")}
              gruppieren={false}
              negativ
            />
          )}
          {art === "markt" && (
            <ZahlFeld
              name={t("werk.mindestpreis")}
              einheit={`${geldEinheit()}/${e}`}
              wert={mindest}
              onWert={setMindest}
              hilfe={t("werk.mindestpreis_hilfe")}
            />
          )}
          {eigenbedarf && (
            <ZahlFeld
              name={t("werk.behalten")}
              einheit={e}
              wert={behalten}
              onWert={setBehalten}
              hilfe={t("werk.behalten_hilfe")}
            />
          )}
        </div>
        <div className="knopfreihe links">
          <button type="submit" className={o ? undefined : "haupt"}>
            {o ? t("werk.uebernehmen") : t("produktion.anbieten")}
          </button>
          {o && (
            <button
              type="button"
              className="schlicht"
              onClick={() =>
                void los(
                  [{ SetSale: { site: s.index, product: produkt, mode: null, keep: 0 } }],
                  t("werk.verkauf_beendet", { produkt: name }),
                )
              }
            >
              {t("werk.verkauf_beenden")}
            </button>
          )}
        </div>
        {fehler && <p className="fehlertext">{fehler}</p>}
        <Rueckmeldung meldung={antwort} />
        {/* Tells the introduction that a price was set. */}
        {antwort && !antwort.fehler && <span data-tour="preis-gesetzt" hidden />}
      </form>
      {o?.trim && <Ausstattung s={s} produkt={produkt} name={name} stufe={o.trim} />}
      {o && (o.outlets ?? []).length > 0 && (
        <NiederlassungenBeliefern o={o} s={s} produkt={produkt} name={name} />
      )}
    </article>
  );
}

const AUSSTATTUNG = { einfach: "Simple", standard: "Standard", premium: "Premium" } as const;

/**
 * Trim level of an end product made here (MN2): simple saves inputs and labor at lower
 * quality for poorer customers, premium the reverse.
 */
export function Ausstattung({
  s,
  produkt,
  name,
  stufe,
}: {
  s: StandortDetail;
  produkt: string;
  name: string;
  stufe: keyof typeof AUSSTATTUNG;
}) {
  const { los, antwort } = useAktion(`ausstattung/${s.index}/${produkt}`);
  const id = useId();
  return (
    <div className="ausstattung">
      <label htmlFor={`${id}-stufe`}>{t("werk.ausstattung")}</label>{" "}
      <select
        id={`${id}-stufe`}
        value={stufe}
        onChange={(ev) => {
          const neu = ev.target.value as keyof typeof AUSSTATTUNG;
          void los(
            [{ SetTrim: { site: s.index, product: produkt, trim: AUSSTATTUNG[neu] } }],
            t("werk.ausstattung_gesetzt", {
              produkt: name,
              stufe: t(`ausstattung.${neu}`),
            }),
          );
        }}
      >
        {(Object.keys(AUSSTATTUNG) as (keyof typeof AUSSTATTUNG)[]).map((k) => (
          <option key={k} value={k}>
            {t(`ausstattung.${k}`)}
          </option>
        ))}
      </select>
      <small className="feld-hilfe">{t("werk.ausstattung_hilfe")}</small>
      <Rueckmeldung meldung={antwort} />
    </div>
  );
}

/**
 * Supplying the company's own sales offices with a product by need, at the value of the
 * stock (production cost) instead of a market price: one tick for all of them.
 */
function NiederlassungenBeliefern({
  o,
  s,
  produkt,
  name,
}: {
  o: Angebot;
  s: StandortDetail;
  produkt: string;
  name: string;
}) {
  const { los, antwort } = useAktion(`niederlassungen/${s.index}/${produkt}`);
  const id = useId();
  const outlets = o.outlets ?? [];
  const alle = outlets.every((x) => x.supplied_here);
  const tage = s.supply_days_default ?? 20;
  const umschalten = (an: boolean) =>
    void los(
      outlets
        .filter((x) => x.supplied_here !== an)
        .map((x) => ({
          SetInternalSupply: {
            site: x.site,
            product: produkt,
            source: an ? s.index : null,
            mode: { Need: { days: tage } },
          },
        })),
      t(an ? "werk.niederlassungen_an" : "werk.niederlassungen_aus", { produkt: name }),
    );
  return (
    <div className="niederlassungen">
      <label htmlFor={`${id}-haken`}>
        <input
          id={`${id}-haken`}
          type="checkbox"
          checked={alle}
          onChange={(ev) => umschalten(ev.target.checked)}
        />{" "}
        {t("werk.niederlassungen_haken", { anzahl: outlets.length })}
      </label>
      <small className="feld-hilfe">
        {t("werk.niederlassungen_hilfe", {
          liste: outlets
            .map((x) => standortName("standorttyp.niederlassung", x.country, x.name))
            .join(", "),
        })}
      </small>
      <Rueckmeldung meldung={antwort} />
    </div>
  );
}

/**
 * Own price against market price, reference price and unit cost on one scale: is the
 * price above the cost, and where does it stand against the market?
 */
function Preisvergleich({
  einheit: e,
  eigen,
  markt,
  richt,
  stueck,
  teile,
  marge,
}: {
  einheit: string;
  eigen: number | null;
  markt: number;
  richt: number;
  stueck: number | null;
  /** The parts of the unit cost (M27), if the site makes the product. */
  teile: Stueckkosten | null;
  marge: number | null;
}) {
  const zeilen: { key: string; wert: number; klasse: string }[] = [
    ...(eigen !== null ? [{ key: "werk.eigener_preis", wert: eigen, klasse: "eigen" }] : []),
    { key: "werk.marktpreis", wert: markt, klasse: "markt" },
    { key: "werk.richtpreis", wert: richt, klasse: "richt" },
    ...(stueck !== null ? [{ key: "werk.stueckkosten", wert: stueck, klasse: "kosten" }] : []),
  ].filter((z) => z.wert > 0);
  const hoechst = Math.max(...zeilen.map((z) => z.wert), 1e-9);
  return (
    <div className="preisvergleich">
      <dl>
        {zeilen.map((z) => (
          <div key={z.key} className={`balkenzeile ${z.klasse}`}>
            <dt>
              <span>{t(z.key)}</span>
              {z.key === "werk.stueckkosten" && teile && (
                <Erklaerung wert={t("werk.stueckkosten")}>
                  <KostenTeile u={teile} einheit={e} />
                </Erklaerung>
              )}
            </dt>
            <dd>
              <span className="balken" style={{ width: `${(z.wert / hoechst) * 100}%` }} />
              <span className="balkenwert">{formatPreis(z.wert, e)}</span>
            </dd>
          </div>
        ))}
      </dl>
      {marge !== null && (
        <p className={marge < 0 ? "fehlertext" : "erfolgstext"}>
          {t(marge < 0 ? "werk.marge_negativ" : "werk.marge", { marge: formatProzent(marge) })}
        </p>
      )}
    </div>
  );
}

// --- Staff ---

function Personal({ s }: { s: StandortDetail }) {
  const { los, antwort } = useAktion(`personal/${s.index}`);
  const [aufschlag, setAufschlag] = useState(zahlFeld(s.wage_premium * 100, 1));
  const [fehler, setFehler] = useState<string | null>(null);
  const fehlt = s.staff.filter((l) => l.employed + 0.05 < l.needed);
  const max = s.wage_premium_max * 100;
  return (
    <div className="karten" data-tour="werk-personal">
      {s.staff.length === 0 ? (
        <p className="gedaempft">{t("werk.kein_personal")}</p>
      ) : (
        <div className="tabelle">
          <table className="mobil-karten" aria-label={t("werk.arbeitskraefte")}>
            <thead>
              <tr>
                <th>{t("werk.gruppe")}</th>
                <th className="zahl">{t("werk.benoetigt")}</th>
                <th className="zahl">{t("werk.beschaeftigt")}</th>
                <th className="zahl">{t("werk.frei_im_land")}</th>
                <th className="zahl">{t("werk.lohn_stunde")}</th>
              </tr>
            </thead>
            <tbody>
              {s.staff.map((l) => (
                <tr key={l.group}>
                  <td>{gruppenName(l.group)}</td>
                  <td className="zahl" data-spalte={t("werk.benoetigt")}>
                    {formatKoepfe(l.needed)}
                  </td>
                  <td
                    className={`zahl ${l.employed + 0.05 < l.needed ? "zustand-fehlt" : ""}`}
                    data-spalte={t("werk.beschaeftigt")}
                  >
                    {formatKoepfe(l.employed)}
                  </td>
                  <td className="zahl" data-spalte={t("werk.frei_im_land")}>
                    {formatKoepfe(l.free_in_country)}
                  </td>
                  <td className="zahl" data-spalte={t("werk.lohn_stunde")}>
                    <span>
                      {formatGeld(l.wage_usd)}
                      {s.wage_premium > 0 && (
                        <small className="gedaempft"> ({formatGeld(l.country_wage_usd)})</small>
                      )}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      {fehlt.map((l) => (
        <p key={l.group} className="warntext">
          {t(l.free_in_country > 0.05 ? "werk.fehlt_frei" : "werk.fehlt_keine_frei", {
            anzahl: formatKoepfe(l.needed - l.employed),
            gruppe: gruppenName(l.group),
            aufschlag: formatProzent(s.rival_premium_max),
          })}
        </p>
      ))}
      <form
        className="karte"
        aria-label={t("werk.lohnaufschlag")}
        onSubmit={(e) => {
          e.preventDefault();
          const p = zahlLesen(aufschlag);
          if (p === null || p < 0 || p > max + 1e-9) {
            setFehler(t("werk.aufschlag_bereich", { max: formatZahl(max) }));
            return;
          }
          setFehler(null);
          void los(
            [{ SetWagePremium: { site: s.index, premium: p / 100 } }],
            t("werk.aufschlag_gesetzt", { prozent: formatZahl(p, 1) }),
          );
        }}
      >
        <h3>{t("werk.lohnaufschlag")}</h3>
        <p>{t("werk.lohnaufschlag_erklaerung")}</p>
        <dl className="werte">
          <dt>{t("werk.aufschlag_jetzt")}</dt>
          <dd>{formatProzent(s.wage_premium)}</dd>
          <dt>{t("werk.aufschlag_konkurrenz")}</dt>
          <dd>{formatProzent(s.rival_premium_max)}</dd>
          <dt>{t("werk.lohnkosten_tag")}</dt>
          <dd>{formatGeld(s.wage_cost_per_day_usd)}</dd>
        </dl>
        <div className="formular-zeile">
          <ZahlFeld
            name={t("werk.lohnaufschlag")}
            einheit="%"
            wert={aufschlag}
            onWert={setAufschlag}
            hilfe={t("werk.aufschlag_hilfe", { max: formatZahl(max) })}
            gruppieren={false}
          />
          <button type="submit">{t("werk.uebernehmen")}</button>
        </div>
        {fehler && <p className="fehlertext">{fehler}</p>}
        <Rueckmeldung meldung={antwort} />
      </form>
      <Schulung key={`${s.training_target}/${s.training_source}`} s={s} />
      {s.maintenance && <Instandhaltung key={s.maintenance.level} s={s} />}
      {s.maintenance && <AutoUeberholung key={`ue-${s.maintenance.overhaul_below ?? "x"}`} s={s} />}
    </div>
  );
}

/** The site's training (W1): level, target and where it comes from, cost and effect. */
/** Maintenance level of the site (BP2): more costs more and wears less. */
function Instandhaltung({ s }: { s: StandortDetail }) {
  const m = s.maintenance!;
  const { los, antwort } = useAktion(`instandhaltung/${s.index}`);
  const [stufe, setStufe] = useState(zahlFeld(m.level * 100, 0));
  const [fehler, setFehler] = useState<string | null>(null);
  return (
    <form
      className="karte"
      aria-label={t("werk.instandhaltung")}
      onSubmit={(e) => {
        e.preventDefault();
        const p = zahlLesen(stufe);
        if (p === null || p < m.min * 100 - 1e-6 || p > m.max * 100 + 1e-6) {
          setFehler(
            t("werk.instandhaltung_bereich", {
              min: formatZahl(m.min * 100),
              max: formatZahl(m.max * 100),
            }),
          );
          return;
        }
        setFehler(null);
        void los(
          [{ SetMaintenance: { site: s.index, level: p / 100 } }],
          t("werk.instandhaltung_gesetzt", { prozent: formatZahl(p) }),
        );
      }}
    >
      <h3>{t("werk.instandhaltung")}</h3>
      <p>{t("werk.instandhaltung_erklaerung")}</p>
      <dl className="werte">
        <dt>{t("werk.instandhaltung_stufe")}</dt>
        <dd>{formatProzent(m.level)}</dd>
        <dt>{t("werk.ausfaelle_dieses_jahr")}</dt>
        <dd>{formatZahl(m.failures)}</dd>
        <dt title={t("werk.unfaelle_hilfe")}>{t("werk.unfaelle")}</dt>
        <dd>
          {t("werk.unfaelle_wert", {
            anzahl: formatZahl(m.accidents ?? 0),
            rate: formatZahl(m.accident_rate ?? 0, 1),
          })}
        </dd>
      </dl>
      <div className="formular-zeile">
        <ZahlFeld
          name={t("werk.instandhaltung_stufe")}
          einheit="%"
          wert={stufe}
          onWert={setStufe}
          gruppieren={false}
        />
        <button type="submit">{t("werk.uebernehmen")}</button>
      </div>
      {fehler && <p className="fehlertext">{fehler}</p>}
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

/** Automatic overhaul once a facility's condition falls below a share (BP2). */
function AutoUeberholung({ s }: { s: StandortDetail }) {
  const m = s.maintenance!;
  const { los, antwort } = useAktion(`ueberholung/${s.index}`);
  const [wert, setWert] = useState(
    m.overhaul_below == null ? "" : zahlFeld(m.overhaul_below * 100, 0),
  );
  const [fehler, setFehler] = useState<string | null>(null);
  const setzen = (schwelle: number | null) =>
    void los(
      [{ SetOverhaulThreshold: { site: s.index, threshold: schwelle } }],
      schwelle === null
        ? t("werk.ueberholung_strategie")
        : t("werk.ueberholung_gesetzt", { prozent: formatZahl(schwelle * 100) }),
    );
  return (
    <form
      className="karte"
      aria-label={t("werk.ueberholung_auto")}
      onSubmit={(e) => {
        e.preventDefault();
        if (wert.trim() === "") return setzen(null);
        const p = zahlLesen(wert);
        if (p === null || p < 0 || p > 100) {
          setFehler(t("werk.ueberholung_bereich"));
          return;
        }
        setFehler(null);
        setzen(p / 100);
      }}
    >
      <h3>{t("werk.ueberholung_auto")}</h3>
      <p className="feld-hilfe">
        {m.overhaul_effective
          ? t("werk.ueberholung_gilt", { prozent: formatProzent(m.overhaul_effective) })
          : t("werk.ueberholung_aus")}
      </p>
      <div className="formular-zeile">
        <ZahlFeld
          name={t("werk.ueberholung_schwelle")}
          einheit="%"
          wert={wert}
          onWert={setWert}
          gruppieren={false}
          hilfe={t("werk.ueberholung_hilfe")}
        />
        <button type="submit">{t("werk.uebernehmen")}</button>
      </div>
      {fehler && <p className="fehlertext">{fehler}</p>}
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

function Schulung({ s }: { s: StandortDetail }) {
  const { los, antwort } = useAktion(`schulung/${s.index}`);
  const [ziel, setZiel] = useState(zahlFeld((s.training_target ?? 0) * 100, 0));
  const [fehler, setFehler] = useState<string | null>(null);
  const quelle = s.training_source ?? "keine";
  return (
    <form
      className="karte"
      aria-label={t("werk.schulung")}
      onSubmit={(e) => {
        e.preventDefault();
        const p = zahlLesen(ziel);
        if (p === null || p < 0 || p > 100) {
          setFehler(t("werk.schulung_bereich"));
          return;
        }
        setFehler(null);
        void los(
          [{ SetTraining: { site: s.index, target: p / 100 } }],
          t("werk.schulung_gesetzt", { prozent: formatZahl(p) }),
        );
      }}
    >
      <h3>{t("werk.schulung")}</h3>
      <p>{t("werk.schulung_erklaerung")}</p>
      <dl className="werte">
        <dt>{t("werk.schulung_niveau")}</dt>
        <dd>{formatProzent(s.training ?? 0)}</dd>
        <dt>{t("werk.schulung_ziel")}</dt>
        <dd>
          {formatProzent(s.training_target ?? 0)}{" "}
          <small className="gedaempft">({t(`werk.schulung_quelle.${quelle}`)})</small>
        </dd>
        <dt>{t("werk.schulung_kosten_tag")}</dt>
        <dd>{formatGeld(s.training_cost_per_day_usd ?? 0)}</dd>
        <dt>{t("werk.schulung_wirkung")}</dt>
        <dd>
          {t("werk.schulung_wirkung_wert", {
            arbeit: formatProzent(s.training_labor_saving ?? 0),
            punkte: formatZahl(s.training_quality ?? 0, 1),
          })}
        </dd>
      </dl>
      <div className="formular-zeile">
        <ZahlFeld
          name={t("werk.schulung_ziel")}
          einheit="%"
          wert={ziel}
          onWert={setZiel}
          gruppieren={false}
        />
        <button type="submit">{t("werk.uebernehmen")}</button>
        {quelle === "standort" && (
          <button
            type="button"
            className="schlicht"
            onClick={() =>
              void los(
                [{ SetTraining: { site: s.index, target: null } }],
                t("werk.schulung_vorgabe_gesetzt"),
              )
            }
          >
            {t("werk.schulung_vorgabe")}
          </button>
        )}
      </div>
      {fehler && <p className="fehlertext">{fehler}</p>}
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

// --- Costs ---

/** The parts of a unit cost (M27): what each unit of the product costs and why. */
function KostenTeile({ u, einheit }: { u: Stueckkosten; einheit: string }) {
  const zeilen = KOSTENARTEN.filter((k) => Math.abs(u[k.key] as number) >= 0.005);
  return (
    <>
      <dl className="rechnung">
        {zeilen.map((k) => (
          <Fragment key={k.key}>
            <dt>{t(k.text)}</dt>
            <dd>{formatPreis(u[k.key] as number, einheit)}</dd>
          </Fragment>
        ))}
        <dt className="summe">{t("werk.stueckkosten")}</dt>
        <dd className="summe">= {formatPreis(u.total_usd, einheit)}</dd>
      </dl>
      <p className="gedaempft">
        {t("erklaerung.stueckkosten", {
          variabel: formatPreis(u.variable_usd, einheit),
          menge: `${formatZahl(u.output_per_day, u.output_per_day < 10 ? 1 : 0)} ${einheit}`,
        })}
      </p>
    </>
  );
}

const KOSTENARTEN: { key: keyof Stueckkosten; text: string }[] = [
  { key: "material_usd", text: "kostenart.material" },
  { key: "labor_usd", text: "kostenart.personal" },
  { key: "energy_usd", text: "kostenart.energie" },
  { key: "overhead_usd", text: "kostenart.gemeinkosten" },
  { key: "rent_usd", text: "kostenart.pacht" },
  { key: "facility_usd", text: "werk.anlage_kosten" },
];

function Kosten({ s, produktion }: { s: StandortDetail; produktion: Produktion }) {
  const m = s.last_month;
  return (
    <div className="karten">
      <p className="erklaerung">{t("werk.kosten_hinweis")}</p>
      {s.unit_costs.length === 0 && <p className="gedaempft">{t("werk.keine_erzeugung")}</p>}
      {s.unit_costs.map((u) => (
        <StueckkostenKarte
          key={u.product}
          u={u}
          angebot={s.offers.find((o) => o.product === u.product) ?? null}
          produktion={produktion}
        />
      ))}
      <section className="karte" aria-label={t("werk.ergebnis_vormonat")}>
        <h3>
          {t("werk.ergebnis_vormonat")}
          {m && <small>{formatMonat(m.month)}</small>}
        </h3>
        {!m ? (
          <p className="gedaempft">{t("uebersicht.noch_kein_monat")}</p>
        ) : (
          <>
            <div className="tabelle">
              <table className="ergebnis" aria-label={t("werk.ergebnis_vormonat")}>
                <tbody>
                  <tr>
                    <th scope="row">{t("kostenart.umsatz")}</th>
                    <td className="zahl">{formatGeld(m.revenue_usd)}</td>
                  </tr>
                  {m.lines.map((l) => (
                    <tr key={l.key}>
                      <th scope="row">{t(l.key)}</th>
                      <td className={`zahl ${l.usd < 0 ? "negativ" : ""}`}>{formatGeld(l.usd)}</td>
                    </tr>
                  ))}
                  <tr className="summe">
                    <th scope="row">{t("werk.ergebnis")}</th>
                    <td className={`zahl ${m.result_usd < 0 ? "negativ" : ""}`}>
                      {formatGeld(m.result_usd)}
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
            {m.products.length > 0 && (
              <div className="tabelle">
                <table aria-label={t("werk.je_produkt")}>
                  <thead>
                    <tr>
                      <th>{t("uebersicht.produkt")}</th>
                      <th className="zahl">{t("kostenart.umsatz")}</th>
                      <th className="zahl">{t("werk.rohertrag")}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {m.products.map((p) => (
                      <tr key={p.product}>
                        <td>{produktName(p.product)}</td>
                        <td className="zahl">{formatGeld(p.revenue_usd)}</td>
                        <td className={`zahl ${p.margin_usd < 0 ? "negativ" : ""}`}>
                          {formatGeld(p.margin_usd)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                <p className="feld-hilfe">{t("werk.rohertrag_hilfe")}</p>
              </div>
            )}
          </>
        )}
      </section>
    </div>
  );
}

function formatMonat(iso: string): string {
  const [jahr, monat] = iso.split("-");
  return `${t(`monat.${Number(monat)}`)} ${jahr}`;
}

function StueckkostenKarte({
  u,
  angebot,
  produktion,
}: {
  u: Stueckkosten;
  angebot: Angebot | null;
  produktion: Produktion;
}) {
  const e = einheit(produktion, u.product);
  const teile = KOSTENARTEN.map((k) => ({ ...k, wert: u[k.key] as number })).filter(
    (k) => k.wert > 0,
  );
  const skala = Math.max(u.total_usd, angebot?.price_usd ?? 0, 1e-9);
  return (
    <article
      className="karte"
      aria-label={t("werk.stueckkosten_von", { produkt: produktName(u.product) })}
    >
      <h3>
        {produktName(u.product)}
        <small>
          {t("werk.erzeugung_tag", {
            menge: formatMenge(u.output_per_day),
            einheit: e,
          })}
        </small>
      </h3>
      <div className="stapel" aria-hidden="true">
        {teile.map((k, i) => (
          <span
            key={k.key}
            className={`stapel-teil teil-${i}`}
            style={{ width: `${(k.wert / skala) * 100}%` }}
            title={`${t(k.text)}: ${formatPreis(k.wert, e)}`}
          />
        ))}
      </div>
      {angebot && (
        <div className="stapel preislinie" aria-hidden="true">
          <span
            className="stapel-preis"
            style={{ width: `${(angebot.price_usd / skala) * 100}%` }}
          />
        </div>
      )}
      <dl className="werte legende-werte">
        {teile.map((k, i) => (
          <LegendenZeile key={k.key} farbe={i} name={t(k.text)}>
            {formatPreis(k.wert, e)}
          </LegendenZeile>
        ))}
        <dt className="summe">{t("werk.stueckkosten")}</dt>
        <dd className="summe">{formatPreis(u.total_usd, e)}</dd>
        <dt>{t("werk.variable_kosten")}</dt>
        <dd>{formatPreis(u.variable_usd, e)}</dd>
        {(u.experience ?? 1) < 0.9995 && (
          <>
            <dt title={t("werk.erfahrung_hinweis")}>{t("werk.erfahrung")}</dt>
            <dd>
              {t("werk.erfahrung_wert", {
                faktor: formatProzent(u.experience ?? 1),
                betrag: formatPreis(u.learning_saving_usd ?? 0, e),
              })}
            </dd>
          </>
        )}
        {angebot && (
          <>
            <dt>{t("werk.eigener_preis")}</dt>
            <dd>{formatPreis(angebot.price_usd, e)}</dd>
          </>
        )}
      </dl>
    </article>
  );
}

function LegendenZeile({
  farbe,
  name,
  children,
}: {
  farbe: number;
  name: string;
  children: ReactNode;
}) {
  return (
    <>
      <dt>
        <span className={`farbpunkt teil-${farbe}`} aria-hidden="true" />
        {name}
      </dt>
      <dd>{children}</dd>
    </>
  );
}

/** Last part of a list: pick another product, then the card for it. */
function WeiteresProdukt({
  titel,
  produkte,
  children,
  tour,
}: {
  titel: string;
  produkte: string[];
  children: (produkt: string) => ReactNode;
  /** Mark for the introduction (`data-tour`). */
  tour?: string;
}) {
  const [produkt, setProdukt] = useState("");
  const id = useId();
  const sortiert = [...produkte].sort((a, b) => produktName(a).localeCompare(produktName(b), "de"));
  return (
    <div className="weiteres" data-tour={tour}>
      <div className="feld">
        <label htmlFor={id}>{titel}</label>
        <SuchAuswahl
          id={id}
          name={titel}
          wert={produkt}
          leer="–"
          optionen={sortiert.map((p) => ({ wert: p, text: produktName(p) }))}
          onWahl={setProdukt}
        />
      </div>
      {produkt && children(produkt)}
    </div>
  );
}
