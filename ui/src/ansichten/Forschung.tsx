// Research (docs/BEDIENUNG.md, Q6): products to research – known, in progress,
// researchable or locked, what each costs and how long it takes –, the development and
// manufacturing research of known products, the research centers and patents.
import { Fragment, type FormEvent, useId, useState } from "react";
import {
  formatDatum,
  formatGeld,
  formatProzent,
  formatZahl,
  landName,
  zahlFeld,
  zahlLesen,
} from "../format";
import type {
  Entwicklungswirkung,
  Forschung,
  Kern,
  Patent,
  Produktforschung,
  Technologie,
  Uebersicht,
  Weiterentwicklung,
} from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import {
  Befehle,
  Rueckmeldung,
  Unterreiter,
  useAktion,
  useBefehl,
  useSicht,
  ZahlFeld,
} from "./gemeinsam";
import { GrundstueckWahl, useGewerbeflaeche, type GrundstueckWahlWert } from "./Grundstuecke";
import { LAENDER } from "./laender";

const technologieName = (k: string) => t(`technologie.${k}`);
const produktName = (p: string) => t(`produkt.${p}`);

export function ForschungAnsicht({
  kern,
  uebersicht,
  onGeaendert,
  onStandorte,
}: {
  kern: Kern;
  uebersicht: Uebersicht;
  onGeaendert: (u: Uebersicht) => void;
  /** To the sites, to build an opened facility. */
  onStandorte: () => void;
}) {
  const { daten, fehler, neu } = useSicht(() => kern.forschung(), uebersicht.date);
  const { senden, meldung } = useBefehl(kern, onGeaendert, neu);
  const [bereich, setBereich] = useState<"produkte" | "entwicklung" | "zentren" | "patente">(
    "produkte",
  );
  const patente = daten?.technologies.filter((x) => x.patent) ?? [];
  if (!daten) return <FehlerText fehler={fehler} />;
  return (
    <main className="ansicht" id="forschung">
      <h1 className="unsichtbar">{t("ansicht.forschung")}</h1>
      <Befehle senden={senden} meldung={meldung}>
        <Unterreiter
          name={t("forschung.bereiche")}
          bereiche={[
            { key: "produkte", text: t("forschung.produkte_reiter") },
            { key: "entwicklung", text: t("forschung.entwicklung") },
            {
              key: "zentren",
              text: t("forschung.zentren"),
              zaehler: daten.centers.filter(untaetig).length,
            },
            ...(daten.patent_countries?.length
              ? [
                  {
                    key: "patente" as const,
                    text: t("forschung.patente"),
                    zaehler: patente.filter((x) => x.patent?.deadline).length,
                  },
                ]
              : []),
          ]}
          aktiv={bereich}
          onWahl={setBereich}
        />
        {bereich === "produkte" && (
          <Produkte
            daten={daten}
            onZentren={() => setBereich("zentren")}
            onStandorte={onStandorte}
          />
        )}
        {bereich === "entwicklung" && (
          <Entwicklung daten={daten} onZentren={() => setBereich("zentren")} />
        )}
        {bereich === "patente" && <Patente daten={daten} technologien={patente} />}
        {bereich === "zentren" && (
          <Zentren kern={kern} daten={daten} heimat={uebersicht.company?.headquarters ?? ""} />
        )}
      </Befehle>
    </main>
  );
}

/** Whether a product is an end product or an input; null when the view does not say. */
export function warenart(daten: Forschung, produkt: string): "endprodukt" | "vormaterial" | null {
  const k = daten.product_kinds?.[produkt];
  if (!k) return null;
  return k === "endprodukt" ? "endprodukt" : "vormaterial";
}

/** A small mark "Vormaterial" or "Endprodukt" after a product. */
function ArtMarke({ daten, produkt }: { daten: Forschung; produkt: string }) {
  const a = warenart(daten, produkt);
  return a ? <small className={`artmarke ${a}`}>{t(`forschung.art.${a}`)}</small> : null;
}

// --- Products to research (Q6) ---

type StatusFilter = "offen" | "bekannt" | "gesperrt" | "alle";

/** Order of the list: in progress, researchable, locked, known; then by name. */
const RANG: Record<Produktforschung["status"], number> = {
  in_arbeit: 0,
  erforschbar: 1,
  gesperrt: 2,
  bekannt: 3,
};

/**
 * Research by product (Q6): which products the player knows, which can be researched
 * (all inputs exist in the world), how much it takes, and which center works on it.
 */
function Produkte({
  daten,
  onZentren,
  onStandorte,
}: {
  daten: Forschung;
  onZentren: () => void;
  onStandorte: () => void;
}) {
  const alle = daten.products ?? [];
  const [branche, setBranche] = useState("");
  const [status, setStatus] = useState<StatusFilter>("offen");
  const [art, setArt] = useState<"alle" | "vormaterial" | "endprodukt">("alle");
  const [suche, setSuche] = useState("");
  const erste = alle.find((x) => x.status === "in_arbeit") ?? null;
  const [gewaehlt, setGewaehlt] = useState<string | null>(erste?.product ?? null);
  const id = useId();
  const branchen = [...new Set(alle.map((x) => x.branch))].sort((a, b) =>
    t(`branche.${a}`).localeCompare(t(`branche.${b}`), "de"),
  );
  const gesucht = suche.trim().toLocaleLowerCase("de");
  const liste = alle
    .filter((x) => branche === "" || x.branch === branche)
    .filter((x) =>
      status === "alle"
        ? true
        : status === "offen"
          ? x.status === "erforschbar" || x.status === "in_arbeit"
          : x.status === status,
    )
    .filter((x) => art === "alle" || warenart(daten, x.product) === art)
    .filter(
      (x) => gesucht === "" || produktName(x.product).toLocaleLowerCase("de").includes(gesucht),
    )
    .sort(
      (a, b) =>
        RANG[a.status] - RANG[b.status] ||
        produktName(a.product).localeCompare(produktName(b.product), "de"),
    );
  const auswahl = alle.find((x) => x.product === gewaehlt) ?? null;
  return (
    <div className="produktforschung">
      <p className="erklaerung">{t("forschung.produkte_erklaerung")}</p>
      <div className="formular-zeile">
        <label>
          {t("forschung.produkt_branche")}{" "}
          <select value={branche} onChange={(e) => setBranche(e.target.value)}>
            <option value="">{t("forschung.alle_branchen")}</option>
            {branchen.map((b) => (
              <option key={b} value={b}>
                {t(`branche.${b}`)}
              </option>
            ))}
          </select>
        </label>
        <label>
          {t("forschung.status_filter")}{" "}
          <select value={status} onChange={(e) => setStatus(e.target.value as StatusFilter)}>
            {(["offen", "bekannt", "gesperrt", "alle"] as const).map((s) => (
              <option key={s} value={s}>
                {t(`forschung.status_filter.${s}`)}
              </option>
            ))}
          </select>
        </label>
        <label>
          {t("forschung.art.filter")}{" "}
          <select
            value={art}
            onChange={(e) => setArt(e.target.value as "alle" | "vormaterial" | "endprodukt")}
          >
            {(["alle", "vormaterial", "endprodukt"] as const).map((a) => (
              <option key={a} value={a}>
                {t(`forschung.art.${a}_viele`)}
              </option>
            ))}
          </select>
        </label>
        <div className="feld">
          <label htmlFor={`${id}-suche`}>{t("markt.suchen")}</label>
          <input
            id={`${id}-suche`}
            type="search"
            value={suche}
            placeholder={t("markt.suchen_platzhalter")}
            onChange={(e) => setSuche(e.target.value)}
          />
        </div>
      </div>
      {liste.length === 0 ? (
        <p className="gedaempft">{t("forschung.produkte_leer")}</p>
      ) : (
        <table className="tabelle" aria-label={t("forschung.produkte")}>
          <thead>
            <tr>
              <th>{t("forschung.produkt")}</th>
              <th>{t("forschung.stand_spalte")}</th>
              <th className="zahl">{t("forschung.jahr")}</th>
              <th className="zahl">{t("forschung.aufwand")}</th>
            </tr>
          </thead>
          <tbody>
            {liste.map((x) => (
              <Fragment key={x.product}>
                <tr className={gewaehlt === x.product ? "gewaehlt" : undefined}>
                  <td>
                    <button
                      type="button"
                      className="schlicht"
                      aria-expanded={gewaehlt === x.product}
                      onClick={() => setGewaehlt(gewaehlt === x.product ? null : x.product)}
                    >
                      {produktName(x.product)}
                    </button>{" "}
                    <ArtMarke daten={daten} produkt={x.product} />
                  </td>
                  <td>
                    <span className={`punkt technik-${x.status}`} />{" "}
                    {t(`forschung.produktstatus.${x.status}`)}
                  </td>
                  <td className="zahl">{x.historical_year ?? "–"}</td>
                  <td className="zahl">
                    {x.status === "bekannt"
                      ? "–"
                      : formatZahl(Math.max(x.needed - x.collected, 0), 0)}
                  </td>
                </tr>
                {gewaehlt === x.product && auswahl && (
                  <tr>
                    <td colSpan={4}>
                      <ProduktDetail
                        x={auswahl}
                        daten={daten}
                        onZentren={onZentren}
                        onStandorte={onStandorte}
                      />
                    </td>
                  </tr>
                )}
              </Fragment>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function ProduktDetail({
  x,
  daten,
  onZentren,
  onStandorte,
}: {
  x: Produktforschung;
  daten: Forschung;
  onZentren: () => void;
  onStandorte: () => void;
}) {
  const name = produktName(x.product);
  const { los, antwort } = useAktion(`produktforschung/${x.product}`);
  const dabei = daten.centers.filter((z) => x.sites.includes(z.site));
  const bereit = daten.centers.filter((z) => z.labs.length > 0 && !x.sites.includes(z.site));
  const [zentrum, setZentrum] = useState(bereit[0]?.site ?? -1);
  const formId = useId();
  const fortschritt = x.needed > 0 ? Math.min(x.collected / x.needed, 1) : 0;
  const offen = Math.max(x.needed - x.collected, 0);
  return (
    <section className="produktdetail" aria-label={name}>
      <dl className="werte">
        <dt>{t("forschung.stand_spalte")}</dt>
        <dd>
          {t(`forschung.produktstatus.${x.status}`)}
          {x.status !== "bekannt" && (
            <small className="gedaempft">
              {" "}
              {x.in_world ? t("forschung.in_der_welt") : t("forschung.nicht_in_der_welt")}
            </small>
          )}
        </dd>
        <dt>{t("forschung.vorprodukte")}</dt>
        <dd>
          {x.inputs.length === 0
            ? t("forschung.keine")
            : x.inputs.map((p, i) => (
                <Fragment key={p}>
                  {i > 0 && ", "}
                  {x.missing_inputs.includes(p) ? (
                    <strong className="warnung">{produktName(p)}</strong>
                  ) : (
                    produktName(p)
                  )}
                </Fragment>
              ))}
        </dd>
        {x.missing_inputs.length > 0 && (
          <>
            <dt>{t("forschung.gesperrt_weil")}</dt>
            <dd>
              {t("forschung.vorprodukt_fehlt", {
                liste: x.missing_inputs.map(produktName).join(", "),
              })}
            </dd>
          </>
        )}
        {x.status !== "bekannt" && (
          <>
            <dt>{t("forschung.fortschritt")}</dt>
            <dd>
              <span className="technik-fortschritt">
                <progress max={1} value={fortschritt} aria-label={t("forschung.fortschritt")} />
                <span className="zahl">{formatProzent(fortschritt)}</span>
              </span>{" "}
              <small className="gedaempft">
                {t("forschung.punkte", {
                  punkte: formatZahl(x.collected, 0),
                  bedarf: formatZahl(x.needed, 0),
                })}
              </small>
            </dd>
            {x.historical_year !== null && (
              <>
                <dt>{t("forschung.historisch")}</dt>
                <dd>
                  {x.historical_year > Number(daten.date.slice(0, 4))
                    ? t("forschung.vor_der_zeit", { jahr: x.historical_year })
                    : t("forschung.erfunden", { jahr: x.historical_year })}
                </dd>
              </>
            )}
            <dt>{t("forschung.dauer")}</dt>
            <dd>
              {x.days !== null ? (
                <>
                  {t("forschung.tage_etwa", { tage: formatZahl(Math.ceil(x.days)) })}{" "}
                  <small className="gedaempft">
                    {t("forschung.mit_deinen_zentren", {
                      punkte: formatZahl(x.points_per_day, 1),
                    })}
                  </small>
                </>
              ) : x.one_lab ? (
                <>
                  {t("forschung.tage_etwa", { tage: formatZahl(Math.ceil(x.one_lab.days)) })}{" "}
                  <small className="gedaempft">
                    {t("forschung.mit_einem_labor", { land: landName(x.one_lab.country) })}
                  </small>
                </>
              ) : offen <= 0 ? (
                t("forschung.gleich_fertig")
              ) : (
                "–"
              )}
            </dd>
            {x.one_lab && (
              <>
                <dt>{t("forschung.kosten")}</dt>
                <dd>
                  {t("forschung.kosten_etwa", { kosten: formatGeld(x.one_lab.cost_usd) })}{" "}
                  <small className="gedaempft">
                    {t("forschung.kosten_hinweis", {
                      forscher: formatZahl(x.one_lab.researchers),
                    })}
                  </small>
                </dd>
              </>
            )}
          </>
        )}
        {dabei.length > 0 && (
          <>
            <dt>{t("forschung.forscht_daran")}</dt>
            <dd>{dabei.map(zentrumOrt).join(", ")}</dd>
          </>
        )}
      </dl>
      {x.status === "bekannt" ? (
        <div className="knopfreihe links">
          <button type="button" onClick={onStandorte}>
            {t("forschung.anlage_bauen")}
          </button>
        </div>
      ) : x.status === "gesperrt" ? null : bereit.length === 0 ? (
        dabei.length === 0 && (
          <p className="erklaerung">
            {t("forschung.erst_zentrum", { betrag: formatGeld(daten.laboratory_usd) })}{" "}
            <button type="button" className="schlicht" onClick={onZentren}>
              {t("forschung.zu_den_zentren")}
            </button>
          </p>
        )
      ) : (
        <form
          className="formular-zeile"
          aria-label={t("forschung.erforschen_an", { produkt: name })}
          onSubmit={(e) => {
            e.preventDefault();
            if (zentrum < 0) return;
            void los(
              [{ SetProductResearch: { site: zentrum, product: x.product } }],
              t("forschung.produkt_gestartet", { produkt: name }),
            );
          }}
        >
          <div className="feld">
            <label htmlFor={`${formId}-zentrum`}>{t("forschung.zentrum")}</label>
            <select
              id={`${formId}-zentrum`}
              value={zentrum}
              onChange={(e) => setZentrum(Number(e.target.value))}
            >
              {bereit.map((z) => (
                <option key={z.site} value={z.site}>
                  {zentrumOrt(z)}
                  {z.research_product
                    ? ` (${t("forschung.statt", { technologie: produktName(z.research_product) })})`
                    : ""}
                </option>
              ))}
            </select>
          </div>
          <button type="submit" className="haupt">
            {t("forschung.erforschen")}
          </button>
        </form>
      )}
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

// --- Research centers ---

/** Own products first, then by name. */
function sortiert(liste: Weiterentwicklung[]): Weiterentwicklung[] {
  return [...liste].sort(
    (a, b) =>
      Number(b.own) - Number(a.own) ||
      produktName(a.product).localeCompare(produktName(b.product), "de"),
  );
}

function wirkungText(w: Entwicklungswirkung): string {
  const teile = [t("forschung.wirkung.qualitaet", { wert: formatZahl(w.quality, 0) })];
  if (w.labor_saved > 0)
    teile.push(t("forschung.wirkung.arbeit", { wert: formatProzent(w.labor_saved) }));
  if (w.inputs_saved > 0)
    teile.push(t("forschung.wirkung.vorprodukte", { wert: formatProzent(w.inputs_saved) }));
  if ((w.output_gained ?? 0) > 0)
    teile.push(t("forschung.wirkung.ausstoss", { wert: formatProzent(w.output_gained!) }));
  return t("forschung.wirkung_text", { liste: teile.join(", ") });
}

/**
 * Development of products (M37): what each level brings, where the player and the
 * competitors stand, and what the next level costs; a research center works on it.
 */
function Entwicklung({ daten, onZentren }: { daten: Forschung; onZentren: () => void }) {
  const eigene = daten.developments.filter((d) => d.own);
  const [nurEigene, setNurEigene] = useState(eigene.length > 0);
  const [suche, setSuche] = useState("");
  const [art, setArt] = useState<"alle" | "vormaterial" | "endprodukt">("alle");
  const gesucht = suche.trim().toLocaleLowerCase("de");
  const liste = sortiert(nurEigene ? eigene : daten.developments)
    .filter((d) => art === "alle" || warenart(daten, d.product) === art)
    .filter(
      (d) => gesucht === "" || produktName(d.product).toLocaleLowerCase("de").includes(gesucht),
    );
  const id = useId();
  return (
    <div className="entwicklung" data-tour="weiterentwicklung">
      <p className="erklaerung">
        {t("forschung.entwicklung_erklaerung", {
          wirkung: wirkungText(daten.development_max),
        })}
      </p>
      <label className="schalter" htmlFor={`${id}-eigene`}>
        <input
          id={`${id}-eigene`}
          type="checkbox"
          checked={nurEigene}
          onChange={(e) => setNurEigene(e.target.checked)}
        />{" "}
        {t("forschung.nur_eigene", { anzahl: eigene.length })}
      </label>
      <label>
        {t("forschung.art.filter")}{" "}
        <select
          value={art}
          onChange={(e) => setArt(e.target.value as "alle" | "vormaterial" | "endprodukt")}
        >
          {(["alle", "vormaterial", "endprodukt"] as const).map((a) => (
            <option key={a} value={a}>
              {t(`forschung.art.${a}_viele`)}
            </option>
          ))}
        </select>
      </label>
      {!nurEigene && (
        <div className="feld">
          <label htmlFor={`${id}-suche`}>{t("markt.suchen")}</label>
          <input
            id={`${id}-suche`}
            type="search"
            value={suche}
            placeholder={t("markt.suchen_platzhalter")}
            onChange={(e) => setSuche(e.target.value)}
          />
        </div>
      )}
      {liste.length === 0 ? (
        <p className="gedaempft">{t("forschung.entwicklung_leer")}</p>
      ) : (
        <div className="karten">
          {liste.map((d) => (
            <EntwicklungKarte key={d.product} d={d} daten={daten} onZentren={onZentren} />
          ))}
        </div>
      )}
    </div>
  );
}

/** A center without a technology, a product or shares of products to work on. */
function untaetig(z: Forschung["centers"][number]): boolean {
  return z.project === null && anteile(z).length === 0 && (z.process_shares?.length ?? 0) === 0;
}

/** The products a center develops with their shares of its researchers. */
function anteile(z: Forschung["centers"][number]): [string, number][] {
  return z.shares ?? (z.development ? [[z.development, 1]] : []);
}

function EntwicklungKarte({
  d,
  daten,
  onZentren,
}: {
  d: Weiterentwicklung;
  daten: Forschung;
  onZentren: () => void;
}) {
  const name = produktName(d.product);
  const { los, antwort } = useAktion(`entwicklung/${d.product}`);
  const fertigung = (z: Forschung["centers"][number]) => z.process_shares ?? [];
  const dabei = daten.centers.filter(
    (z) => anteile(z).some(([p]) => p === d.product) || fertigung(z).some(([p]) => p === d.product),
  );
  const bereit = daten.centers.filter((z) => z.labs.length > 0);
  const [zentrum, setZentrum] = useState(bereit[0]?.site ?? -1);
  const gewaehlt = bereit.find((z) => z.site === zentrum);
  const bisher = gewaehlt ? (anteile(gewaehlt).find(([p]) => p === d.product)?.[1] ?? 0) : 0;
  const bisherF = gewaehlt ? (fertigung(gewaehlt).find(([p]) => p === d.product)?.[1] ?? 0) : 0;
  const neuAnfang = bisher <= 0 && bisherF <= 0;
  const [anteil, setAnteil] = useState(zahlFeld(neuAnfang ? 50 : bisher * 100, 0));
  const [anteilF, setAnteilF] = useState(zahlFeld(neuAnfang ? 50 : bisherF * 100, 0));
  const formId = useId();
  const fortschritt = d.needed ? Math.min(d.points / d.needed, 1) : 0;
  const absenden = (e: FormEvent) => {
    e.preventDefault();
    const z = bereit.find((x) => x.site === zentrum);
    const a = zahlLesen(anteil);
    const f = zahlLesen(anteilF);
    if (!z || a === null || f === null || a < 0 || f < 0 || a + f > 100) return;
    const setze = (liste: [string, number][], wert: number): [string, number][] => {
      const andere = liste.filter(([p]) => p !== d.product);
      return wert > 0 ? [...andere, [d.product, wert / 100]] : andere;
    };
    void los(
      [
        {
          SetResearchShares: {
            site: z.site,
            shares: setze(anteile(z), a),
            process: setze(fertigung(z), f),
          },
        },
      ],
      a + f > 0
        ? t("forschung.anteile_gesetzt", {
            produkt: name,
            anteil: formatZahl(a, 0),
            fertigung: formatZahl(f, 0),
          })
        : t("forschung.anteil_beendet", { produkt: name }),
    );
  };
  return (
    <article className="karte entwicklungkarte" aria-label={name}>
      <h3>
        {name} <ArtMarke daten={daten} produkt={d.product} />
      </h3>
      <dl className="werte">
        <dt>{t("forschung.niveau")}</dt>
        <dd>
          <span className="technik-fortschritt">
            <progress max={1} value={d.share} aria-label={t("forschung.niveau")} />
            <span className="zahl">{formatProzent(d.share)}</span>
          </span>{" "}
          <small className="gedaempft">
            {t("forschung.punkte_fuehrend", {
              punkte: formatZahl(d.total_points, 0),
              fuehrend: formatZahl(d.leader_points, 0),
            })}
          </small>
        </dd>
        <dt>{t("forschung.wettbewerber")}</dt>
        <dd>
          {d.leader_points <= 0
            ? t("forschung.noch_niemand")
            : t("forschung.wettbewerber_niveau", { niveau: formatProzent(d.best_rival_share) })}
        </dd>
        <dt title={t("forschung.reife_hilfe")}>{t("forschung.reife")}</dt>
        <dd>{formatProzent(d.maturity)}</dd>
        <dt>{t("forschung.wirkung")}</dt>
        <dd>
          {d.share > 0 ? wirkungText(d.effect) : t("forschung.wirkung_keine")}
          <br />
          <small className="gedaempft">
            {t("forschung.beim_fuehrenden", { wirkung: wirkungText(d.lead_effect) })}
          </small>
        </dd>
        <dt title={t("forschung.fertigung_hilfe")}>{t("forschung.fertigung")}</dt>
        <dd>
          {(d.process_points ?? 0) > 0 ? (
            <>
              <span className="technik-fortschritt">
                <progress
                  max={1}
                  value={d.process_share ?? 0}
                  aria-label={t("forschung.fertigung")}
                />
                <span className="zahl">{formatProzent(d.process_share ?? 0)}</span>
              </span>{" "}
              <small className="gedaempft">
                {t("forschung.fertigung_stand", {
                  punkte: formatZahl(d.process_points ?? 0, 0),
                  reife: formatProzent(d.process_maturity ?? 0),
                })}
              </small>
            </>
          ) : (
            t("forschung.wirkung_keine")
          )}
        </dd>
        {d.next !== null && d.needed !== null && (
          <>
            <dt>{t("forschung.naechste_generation", { stufe: d.next })}</dt>
            <dd>
              <span className="technik-fortschritt">
                <progress max={1} value={fortschritt} aria-label={t("forschung.fortschritt")} />
                <span className="zahl">{formatProzent(fortschritt)}</span>
              </span>{" "}
              <small className="gedaempft">
                {t("forschung.punkte", {
                  punkte: formatZahl(d.points, 0),
                  bedarf: formatZahl(d.needed, 0),
                })}
              </small>
            </dd>
            <dt>{t("forschung.dauer")}</dt>
            <dd>
              {d.days !== null ? (
                <>
                  {t("forschung.tage_etwa", { tage: formatZahl(Math.ceil(d.days)) })}{" "}
                  <small className="gedaempft">
                    {t("forschung.mit_deinen_zentren", {
                      punkte: formatZahl(d.points_per_day, 1),
                    })}
                  </small>
                </>
              ) : d.one_lab ? (
                <>
                  {t("forschung.tage_etwa", { tage: formatZahl(Math.ceil(d.one_lab.days)) })}{" "}
                  <small className="gedaempft">
                    {t("forschung.mit_einem_labor", { land: landName(d.one_lab.country) })}
                  </small>
                </>
              ) : (
                "–"
              )}
            </dd>
            {d.one_lab && (
              <>
                <dt>{t("forschung.kosten")}</dt>
                <dd>
                  {t("forschung.kosten_etwa", { kosten: formatGeld(d.one_lab.cost_usd) })}{" "}
                  <small className="gedaempft">
                    {t("forschung.kosten_hinweis", {
                      forscher: formatZahl(d.one_lab.researchers),
                    })}
                  </small>
                </dd>
              </>
            )}
          </>
        )}
        {dabei.length > 0 && (
          <>
            <dt>{t("forschung.forscht_daran")}</dt>
            <dd>
              {dabei
                .map((z) =>
                  t("forschung.zentrum_anteile", {
                    zentrum: zentrumOrt(z),
                    anteil: formatProzent(anteile(z).find(([p]) => p === d.product)?.[1] ?? 0),
                    fertigung: formatProzent(fertigung(z).find(([p]) => p === d.product)?.[1] ?? 0),
                  }),
                )
                .join(", ")}
            </dd>
          </>
        )}
      </dl>
      {bereit.length === 0 ? (
        dabei.length === 0 && (
          <p className="erklaerung">
            {t("forschung.erst_zentrum", { betrag: formatGeld(daten.laboratory_usd) })}{" "}
            <button type="button" className="schlicht" onClick={onZentren}>
              {t("forschung.zu_den_zentren")}
            </button>
          </p>
        )
      ) : (
        <form
          className="formular-zeile"
          aria-label={t("forschung.entwickeln_an", { produkt: name })}
          onSubmit={absenden}
        >
          <div className="feld">
            <label htmlFor={`${formId}-zentrum`}>{t("forschung.zentrum")}</label>
            <select
              id={`${formId}-zentrum`}
              value={zentrum}
              onChange={(e) => setZentrum(Number(e.target.value))}
            >
              {bereit.map((z) => (
                <option key={z.site} value={z.site}>
                  {zentrumOrt(z)}
                </option>
              ))}
            </select>
          </div>
          <ZahlFeld
            name={t("forschung.anteil_forscher")}
            einheit="%"
            wert={anteil}
            onWert={setAnteil}
            hilfe={t("forschung.anteil_hilfe")}
          />
          <ZahlFeld
            name={t("forschung.anteil_fertigung")}
            einheit="%"
            wert={anteilF}
            onWert={setAnteilF}
            hilfe={t("forschung.fertigung_hilfe")}
          />
          <button type="submit" className="haupt">
            {t("forschung.weiterentwickeln")}
          </button>
        </form>
      )}
      <Rueckmeldung meldung={antwort} />
    </article>
  );
}

function Zentren({ kern, daten, heimat }: { kern: Kern; daten: Forschung; heimat: string }) {
  return (
    <div className="karten">
      <p className="erklaerung">
        {t("forschung.zentren_erklaerung", {
          plaetze: formatZahl(daten.laboratory_posts),
          betrag: formatGeld(daten.laboratory_usd),
        })}
      </p>
      {daten.centers.length === 0 && <p className="gedaempft">{t("forschung.keine_zentren")}</p>}
      {daten.centers.map((z) => (
        <Zentrum key={z.site} z={z} daten={daten} />
      ))}
      <ZentrumGruenden kern={kern} heimat={heimat} datum={daten.date} />
    </div>
  );
}

/**
 * The research center's project, shares and laboratories in the site view: the same
 * card as under Forschung → Zentren, with its own view of the research.
 */
export function ForschungImWerk({
  kern,
  site,
  stand,
  onGeaendert,
}: {
  kern: Kern;
  site: number;
  /** Current date: the view reloads with every round. */
  stand: string;
  onGeaendert?: (u: Uebersicht) => void;
}) {
  const { daten, fehler, neu } = useSicht(() => kern.forschung(), `${stand}/${site}`);
  const { senden, meldung } = useBefehl(kern, onGeaendert ?? (() => {}), neu);
  if (!daten) return <FehlerText fehler={fehler} />;
  const z = daten.centers.find((c) => c.site === site);
  if (!z) return null;
  return (
    <Befehle senden={senden} meldung={meldung}>
      <div className="karten">
        <Zentrum z={z} daten={daten} />
      </div>
    </Befehle>
  );
}

function Zentrum({ z, daten }: { z: Forschung["centers"][number]; daten: Forschung }) {
  const { los, antwort } = useAktion(`zentrum/${z.site}`);
  // "r:<product>" to research (Q6), "p:<product>" to develop (M37), "t:<technology>" for
  // a technology set before Q6 (kept while it runs); empty: no project.
  const [projekt, setProjekt] = useState(
    z.research_product
      ? `r:${z.research_product}`
      : z.project
        ? `t:${z.project}`
        : z.development
          ? `p:${z.development}`
          : "",
  );
  const erforschbar = (daten.products ?? [])
    .filter((x) => x.status === "erforschbar" || x.status === "in_arbeit")
    .sort((a, b) => produktName(a.product).localeCompare(produktName(b.product), "de"));
  const ziel = daten.products?.find((x) => x.product === z.research_product);
  const id = useId();
  const titel = z.name
    ? `${z.name} · ${landName(z.country)}`
    : t("forschung.zentrum_in", { land: landName(z.country) });
  const fertig = z.labs.filter((l) => l.ready <= daten.date);
  const plaetze = fertig.reduce((n, l) => n + l.count * daten.laboratory_posts * l.utilization, 0);
  const tech = daten.technologies.find((x) => x.key === z.project);
  const entwicklungen = anteile(z)
    .map(([p]) => daten.developments.find((d) => d.product === p))
    .filter((d) => d !== undefined);
  const aufgeteilt = (z.shares?.length ?? 0) > 0;
  const entwickelbar = sortiert(daten.developments.filter((d) => d.next !== null));
  return (
    <article className="karte zentrumkarte" aria-label={titel}>
      <h3>{titel}</h3>
      <dl className="werte">
        <dt>{t("forschung.stand_zentrum")}</dt>
        <dd>
          {z.ready
            ? t("forschung.forscher_von", {
                anzahl: formatZahl(z.researchers, 1),
                plaetze: formatZahl(plaetze, 0),
              })
            : z.building_until
              ? t("forschung.labor_im_bau", { datum: formatDatum(z.building_until) })
              : t("forschung.ohne_labor")}
        </dd>
        {ziel && (
          <>
            <dt>{t("forschung.fortschritt")}</dt>
            <dd>
              {t("forschung.projekt_stand", {
                technologie: produktName(ziel.product),
                anteil: formatProzent(
                  ziel.needed > 0 ? Math.min(ziel.collected / ziel.needed, 1) : 0,
                ),
              })}
              {ziel.days !== null && (
                <small className="gedaempft">
                  {" "}
                  {t("forschung.noch_tage", { tage: formatZahl(Math.ceil(ziel.days)) })}
                </small>
              )}
            </dd>
          </>
        )}
        {!ziel && tech && tech.needed !== null && (
          <>
            <dt>{t("forschung.fortschritt")}</dt>
            <dd>
              {t("forschung.projekt_stand", {
                technologie: technologieName(tech.key),
                anteil: formatProzent(Math.min(tech.points / tech.needed, 1)),
              })}
              {tech.days !== null && (
                <small className="gedaempft">
                  {" "}
                  {t("forschung.noch_tage", { tage: formatZahl(Math.ceil(tech.days)) })}
                </small>
              )}
            </dd>
          </>
        )}
        {entwicklungen.map(
          (entwicklung) =>
            entwicklung.needed !== null &&
            entwicklung.next !== null && (
              <Fragment key={entwicklung.product}>
                <dt>{t("forschung.fortschritt")}</dt>
                <dd>
                  {t("forschung.entwicklung_stand", {
                    produkt: produktName(entwicklung.product),
                    stufe: entwicklung.next,
                    anteil: formatProzent(Math.min(entwicklung.points / entwicklung.needed, 1)),
                  })}
                  {entwicklung.days !== null && (
                    <small className="gedaempft">
                      {" "}
                      {t("forschung.noch_tage", { tage: formatZahl(Math.ceil(entwicklung.days)) })}
                    </small>
                  )}
                </dd>
              </Fragment>
            ),
        )}
      </dl>
      {z.labs.length === 0 && <p className="gedaempft">{t("forschung.erst_labor")}</p>}
      {z.labs.length > 0 && (
        <form
          className="formular-zeile"
          aria-label={t("forschung.projekt_von", { land: landName(z.country) })}
          onSubmit={(e) => {
            e.preventDefault();
            const [art, key] = [projekt.slice(0, 1), projekt.slice(2)];
            if (art === "r") {
              void los(
                [{ SetProductResearch: { site: z.site, product: key } }],
                t("forschung.produkt_gestartet", { produkt: produktName(key) }),
              );
            } else if (art === "t") {
              void los(
                [{ SetResearch: { site: z.site, technology: key } }],
                t("forschung.gestartet", { technologie: technologieName(key) }),
              );
            } else if (art === "p") {
              void los(
                [{ SetDevelopment: { site: z.site, product: key } }],
                t("forschung.entwicklung_gestartet", { produkt: produktName(key) }),
              );
            } else {
              void los(
                [
                  { SetResearch: { site: z.site, technology: null } },
                  { SetDevelopment: { site: z.site, product: null } },
                ],
                t("forschung.angehalten"),
              );
            }
          }}
        >
          <div className="feld">
            <label htmlFor={`${id}-projekt`}>{t("forschung.projekt")}</label>
            <select
              id={`${id}-projekt`}
              value={projekt}
              onChange={(e) => setProjekt(e.target.value)}
            >
              <option value="">
                {aufgeteilt
                  ? t("forschung.aufgeteilt", { anzahl: z.shares?.length ?? 0 })
                  : t("forschung.kein_projekt")}
              </option>
              {z.project && !z.research_product && (
                <option value={`t:${z.project}`}>{technologieName(z.project)}</option>
              )}
              <optgroup label={t("forschung.produkte_reiter")}>
                {erforschbar.map((x) => (
                  <option key={x.product} value={`r:${x.product}`}>
                    {t("forschung.erforschen_option", { produkt: produktName(x.product) })}
                  </option>
                ))}
              </optgroup>
              {entwickelbar.length > 0 && (
                <optgroup label={t("forschung.entwicklung")}>
                  {entwickelbar.map((d) => (
                    <option key={d.product} value={`p:${d.product}`}>
                      {t("forschung.entwicklung_option", {
                        produkt: produktName(d.product),
                        stufe: d.next ?? 0,
                      })}
                    </option>
                  ))}
                </optgroup>
              )}
            </select>
          </div>
          <button type="submit">{t("werk.uebernehmen")}</button>
        </form>
      )}
      {z.labs.map((l, i) => (
        <LaborAuslastung
          key={`${l.slot}/${l.utilization}/${l.ready}`}
          site={z.site}
          labor={l}
          heute={daten.date}
          titel={t("forschung.labor_nr", { nummer: i + 1 })}
          name={
            z.labs.length === 1
              ? t("forschung.labor_auslastung")
              : t("forschung.labor_auslastung_nr", { nummer: i + 1 })
          }
        />
      ))}
      {daten.laboratory && (
        <div className="knopfreihe links">
          <button
            type="button"
            className={z.labs.length === 0 ? "haupt" : undefined}
            onClick={() =>
              void los(
                [{ BuildFacility: { site: z.site, facility: daten.laboratory!, count: 1 } }],
                t("forschung.labor_begonnen"),
              )
            }
          >
            {z.labs.length === 0
              ? t("forschung.labor_bauen", { betrag: formatGeld(daten.laboratory_usd) })
              : t("forschung.labor_erweitern", { betrag: formatGeld(daten.laboratory_usd) })}
          </button>
        </div>
      )}
      <Rueckmeldung meldung={antwort} />
    </article>
  );
}

function LaborAuslastung({
  site,
  labor,
  heute,
  titel,
  name,
}: {
  site: number;
  labor: Forschung["centers"][number]["labs"][number];
  heute: string;
  titel: string;
  name: string;
}) {
  const [prozent, setProzent] = useState(zahlFeld(labor.utilization * 100, 0));
  const [fehler, setFehler] = useState<string | null>(null);
  const { los, antwort } = useAktion(`labor/${site}/${labor.slot}`);
  if (labor.ready > heute) {
    const tage = Math.max(1, Math.round((Date.parse(labor.ready) - Date.parse(heute)) / 864e5));
    return (
      <p className="gedaempft" role="group" aria-label={name}>
        <strong>{titel}</strong>:{" "}
        {t("forschung.labor_im_bau_rest", { datum: formatDatum(labor.ready), tage })}
      </p>
    );
  }
  return (
    <form
      className="formular-zeile"
      aria-label={name}
      onSubmit={(e) => {
        e.preventDefault();
        const p = zahlLesen(prozent);
        if (p === null || p < 0 || p > 100) {
          setFehler(t("werk.auslastung_bereich"));
          return;
        }
        setFehler(null);
        void los(
          [{ SetProduction: { site, slot: labor.slot, recipe: null, utilization: p / 100 } }],
          t("werk.anlage_geaendert"),
        );
      }}
    >
      <ZahlFeld
        name={name}
        einheit="%"
        wert={prozent}
        onWert={setProzent}
        hilfe={t("forschung.labor_hilfe")}
        gruppieren={false}
      />
      <button type="submit">{t("werk.uebernehmen")}</button>
      {fehler && <p className="fehlertext">{fehler}</p>}
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

function ZentrumGruenden({ kern, heimat, datum }: { kern: Kern; heimat: string; datum: string }) {
  const [land, setLand] = useState(heimat);
  const { los, antwort } = useAktion("zentrum-gruenden");
  const id = useId();
  const flaeche = useGewerbeflaeche(kern, land, datum);
  const [wahl, setWahl] = useState<GrundstueckWahlWert>({ plot: null, lease: false });
  const grundstueck = flaeche?.free.find((g) => g.id === wahl.plot) ?? null;
  // Without plots (null) the core founds the center without one.
  const bereit = flaeche === null || grundstueck !== null;
  return (
    <form
      className="karte"
      aria-label={t("forschung.gruenden")}
      onSubmit={(e) => {
        e.preventDefault();
        if (!bereit) return;
        void los(
          [
            grundstueck
              ? {
                  FoundSiteOnPlot: {
                    plot: grundstueck.id,
                    kind: "ResearchCenter",
                    lease: wahl.lease,
                  },
                }
              : { FoundSite: { country: land, kind: "ResearchCenter" } },
          ],
          t("forschung.gegruendet", { land: landName(land) }),
        );
      }}
    >
      <h3>{t("forschung.gruenden")}</h3>
      <div className="formular-zeile">
        <div className="feld">
          <label htmlFor={`${id}-land`}>{t("produktion.land")}</label>
          <select
            id={`${id}-land`}
            value={land}
            onChange={(e) => {
              setLand(e.target.value);
              setWahl({ ...wahl, plot: null });
            }}
          >
            {LAENDER.map((k) => (
              <option key={k} value={k}>
                {landName(k)}
              </option>
            ))}
          </select>
        </div>
      </div>
      {flaeche && <GrundstueckWahl flaeche={flaeche} wert={wahl} onWert={setWahl} />}
      <div className="knopfreihe links">
        <button type="submit" disabled={!bereit}>
          {t("produktion.gruenden_knopf")}
        </button>
      </div>
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

/** One line on a technology's patent or the player's open claim (P7). */
function PatentZeile({ patent: p }: { patent: Patent }) {
  if (p.own && p.deadline && p.countries.length === 0) {
    return <p className="hinweis">{t("patent.anspruch", { bis: formatDatum(p.deadline) })}</p>;
  }
  const text = t(p.own ? "patent.eigenes" : "patent.fremdes", {
    firma: p.holder,
    bis: p.until ? formatDatum(p.until) : "–",
    laender: p.countries.length,
  });
  return (
    <p className={p.blocks_player_in.length > 0 ? "warnung" : "gedaempft"}>
      {text}
      {p.blocks_player_in.length > 0 &&
        ` ${t("patent.sperrt_dich", { laender: p.blocks_player_in.map(landName).join(", ") })}`}
      {!p.own && p.free_for_player && ` ${t("patent.lizenz_vorhanden")}`}
    </p>
  );
}

/** A research center in a choice: its name and country, else the country. */
function zentrumOrt(z: Forschung["centers"][number]): string {
  return z.name ? `${z.name} · ${landName(z.country)}` : landName(z.country);
}

/** Patents (P7): file own claims country by country, own patents, those of others. */
function Patente({ daten, technologien }: { daten: Forschung; technologien: Technologie[] }) {
  const offen = technologien.filter((x) => x.patent?.own && x.patent.deadline);
  const eigene = technologien.filter((x) => x.patent?.own && x.patent.countries.length > 0);
  const fremde = technologien.filter((x) => x.patent && !x.patent.own);
  return (
    <section className="patente" aria-label={t("forschung.patente")}>
      <p className="erklaerung">{t("patent.erklaerung")}</p>
      <h2>{t("patent.anmelden")}</h2>
      {offen.length === 0 && <p className="gedaempft">{t("patent.keine_ansprueche")}</p>}
      {offen.map((x) => (
        <Anmeldung key={x.key} tech={x} laender={daten.patent_countries ?? []} />
      ))}
      <h2>{t("patent.deine")}</h2>
      {eigene.length === 0 ? (
        <p className="gedaempft">{t("patent.keine_eigenen")}</p>
      ) : (
        <ul className="liste">
          {eigene.map((x) => (
            <li key={x.key}>
              <strong>{technologieName(x.key)}</strong> <PatentZeile patent={x.patent!} />
            </li>
          ))}
        </ul>
      )}
      <h2>{t("patent.fremde")}</h2>
      {fremde.length === 0 ? (
        <p className="gedaempft">{t("patent.keine_fremden")}</p>
      ) : (
        <ul className="liste">
          {fremde.map((x) => (
            <li key={x.key}>
              <strong>{technologieName(x.key)}</strong> <PatentZeile patent={x.patent!} />
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

/** Filing an own claim: countries to choose (largest markets first), cost, deadline. */
function Anmeldung({
  tech: x,
  laender,
}: {
  tech: Technologie;
  laender: NonNullable<Forschung["patent_countries"]>;
}) {
  const p = x.patent!;
  const frei = laender.filter((l) => !p.countries.includes(l.country));
  const [wahl, setWahl] = useState<string[]>(() =>
    frei.filter((l) => l.suggested).map((l) => l.country),
  );
  const [alle, setAlle] = useState(false);
  const { los, antwort } = useAktion(`patent/${x.key}`);
  const kosten = frei
    .filter((l) => wahl.includes(l.country))
    .reduce((summe, l) => summe + l.cost_usd, 0);
  const gezeigt = alle ? frei : frei.slice(0, 15);
  const name = technologieName(x.key);
  return (
    <form
      className="karte"
      aria-label={t("patent.anmeldung", { technologie: name })}
      onSubmit={(e) => {
        e.preventDefault();
        if (wahl.length === 0) return;
        void los(
          [{ FilePatent: { technology: x.key, countries: wahl } }],
          t("patent.angemeldet", { technologie: name, laender: wahl.length }),
        );
      }}
    >
      <h3>{name}</h3>
      <p className="gedaempft">
        {t("patent.frist", { bis: formatDatum(p.deadline!) })}
        {p.countries.length > 0 &&
          ` ${t("patent.schon", { laender: p.countries.map(landName).join(", ") })}`}
      </p>
      <fieldset className="auswahlgruppe">
        <legend>{t("patent.laender")}</legend>
        <p>
          <button
            type="button"
            className="verweis"
            onClick={() => {
              if (wahl.length === frei.length) {
                setWahl([]);
              } else {
                setWahl(frei.map((l) => l.country));
                setAlle(true);
              }
            }}
          >
            {wahl.length === frei.length
              ? t("patent.keine_auswaehlen")
              : t("patent.alle_auswaehlen", { anzahl: frei.length })}
          </button>
        </p>
        {gezeigt.map((l) => (
          <label key={l.country}>
            <input
              type="checkbox"
              checked={wahl.includes(l.country)}
              onChange={(e) =>
                setWahl((w) =>
                  e.target.checked ? [...w, l.country] : w.filter((c) => c !== l.country),
                )
              }
            />
            {landName(l.country)} – {formatGeld(l.cost_usd)}
          </label>
        ))}
      </fieldset>
      {frei.length > gezeigt.length && (
        <button type="button" className="verweis" onClick={() => setAlle(true)}>
          {t("patent.alle_laender", { anzahl: frei.length })}
        </button>
      )}
      <p>
        <button type="submit" disabled={wahl.length === 0}>
          {t("patent.anmelden_knopf", { anzahl: wahl.length, kosten: formatGeld(kosten) })}
        </button>
      </p>
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}
