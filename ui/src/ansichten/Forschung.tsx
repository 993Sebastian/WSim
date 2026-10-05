// Research (docs/BEDIENUNG.md, M19): the technology tree by field and time – what is
// known, in progress, possible or locked, what each technology costs and how long it
// takes, and what it opens: processes (recipes), products and facilities. A second area
// runs the research centers.
import { type ReactNode, useId, useState } from "react";
import {
  formatDatum,
  formatGeld,
  formatMenge,
  formatProzent,
  formatZahl,
  landName,
  zahlFeld,
  zahlLesen,
} from "../format";
import type { Forschung, Kern, Technologie, Uebersicht } from "../kern";
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
import { LAENDER } from "./laender";

const technologieName = (k: string) => t(`technologie.${k}`);
const produktName = (p: string) => t(`produkt.${p}`);
const einheit = (d: Forschung, p: string) => t(`einheit.${d.units[p] ?? "t"}`);

/** Eras of the tree: columns from left (oldest) to right. */
const EPOCHEN: { bis: number; key: string }[] = [
  { bis: 1850, key: "bis_1850" },
  { bis: 1875, key: "bis_1875" },
  { bis: 1899, key: "bis_1899" },
  { bis: 1909, key: "bis_1909" },
  { bis: 1919, key: "bis_1919" },
  { bis: Infinity, key: "ab_1920" },
];

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
  const [bereich, setBereich] = useState<"baum" | "zentren">("baum");
  if (!daten) return <FehlerText fehler={fehler} />;
  return (
    <main className="ansicht" id="forschung">
      <h1 className="unsichtbar">{t("ansicht.forschung")}</h1>
      <Befehle senden={senden} meldung={meldung}>
        <Unterreiter
          name={t("forschung.bereiche")}
          bereiche={[
            { key: "baum", text: t("forschung.baum") },
            {
              key: "zentren",
              text: t("forschung.zentren"),
              zaehler: daten.centers.filter((z) => z.project === null).length,
            },
          ]}
          aktiv={bereich}
          onWahl={setBereich}
        />
        {bereich === "baum" ? (
          <Baum daten={daten} onZentren={() => setBereich("zentren")} onStandorte={onStandorte} />
        ) : (
          <Zentren daten={daten} heimat={uebersicht.company.headquarters} />
        )}
      </Befehle>
    </main>
  );
}

function Baum({
  daten,
  onZentren,
  onStandorte,
}: {
  daten: Forschung;
  onZentren: () => void;
  onStandorte: () => void;
}) {
  const schmal = typeof window !== "undefined" && window.matchMedia?.("(max-width: 700px)").matches;
  const [darstellung, setDarstellung] = useState<"baum" | "liste">(schmal ? "liste" : "baum");
  const erste =
    daten.technologies.find((x) => x.status === "in_arbeit") ??
    daten.technologies.find((x) => x.status === "erforschbar") ??
    daten.technologies[0];
  const [gewaehlt, setGewaehlt] = useState<string | null>(erste?.key ?? null);
  const id = useId();
  const detailId = `${id}-detail`;
  const tech = daten.technologies.find((x) => x.key === gewaehlt) ?? null;
  // On a phone the details sit below the list item or the tree: bring them into view.
  const waehle = (k: string) => {
    setGewaehlt(k);
    if (schmal) {
      requestAnimationFrame(() =>
        document.getElementById(detailId)?.scrollIntoView?.({ block: "start", behavior: "smooth" }),
      );
    }
  };
  const detail = tech && (
    <TechnologieDetail
      key={tech.key}
      id={detailId}
      tech={tech}
      daten={daten}
      onWahl={waehle}
      onZentren={onZentren}
      onStandorte={onStandorte}
    />
  );
  const imListeneintrag = schmal && darstellung === "liste";
  return (
    <>
      <p className="erklaerung">{t("forschung.erklaerung")}</p>
      <div className="formular-zeile">
        <fieldset className="auswahlgruppe waagrecht">
          <legend>{t("forschung.darstellung")}</legend>
          {(["baum", "liste"] as const).map((d) => (
            <label key={d}>
              <input
                type="radio"
                name={`${id}-darstellung`}
                checked={darstellung === d}
                onChange={() => setDarstellung(d)}
              />
              {t(`forschung.darstellung.${d}`)}
            </label>
          ))}
        </fieldset>
        <ul className="legende" aria-label={t("karte.legende")}>
          {(["bekannt", "in_arbeit", "erforschbar", "gesperrt"] as const).map((s) => (
            <li key={s}>
              <span className={`punkt technik-${s}`} />
              {t(`forschung.status.${s}`)}
            </li>
          ))}
        </ul>
      </div>
      <div className="technik-layout">
        {darstellung === "baum" ? (
          <BaumGrafik daten={daten} gewaehlt={gewaehlt} onWahl={waehle} />
        ) : (
          <Liste
            daten={daten}
            gewaehlt={gewaehlt}
            onWahl={waehle}
            detail={imListeneintrag ? detail : null}
          />
        )}
        {!imListeneintrag && detail}
      </div>
    </>
  );
}

// --- The tree: lanes by field, columns by era, lines to the prerequisites ---

const SPALTE = 192;
const KNOTEN_B = 176;
const KNOTEN_H = 38;
const ZEILE = 44;
const KOPF = 26;

function BaumGrafik({
  daten,
  gewaehlt,
  onWahl,
}: {
  daten: Forschung;
  gewaehlt: string | null;
  onWahl: (k: string) => void;
}) {
  const felder = [...new Set(daten.technologies.map((x) => x.field))];
  const epoche = (jahr: number) => EPOCHEN.findIndex((e) => jahr <= e.bis);
  // Nodes of one field and era stack downwards, oldest first.
  const zellen = new Map<string, Technologie[]>();
  for (const x of [...daten.technologies].sort((a, b) => a.invention_year - b.invention_year)) {
    const k = `${x.field}/${epoche(x.invention_year)}`;
    zellen.set(k, [...(zellen.get(k) ?? []), x]);
  }
  const spalten = EPOCHEN.filter((_, i) =>
    daten.technologies.some((x) => epoche(x.invention_year) === i),
  ).map((e) => EPOCHEN.indexOf(e));
  const hoehe = new Map(
    felder.map((f) => [
      f,
      Math.max(1, ...spalten.map((s) => zellen.get(`${f}/${s}`)?.length ?? 0)),
    ]),
  );
  const lage = new Map<string, { x: number; y: number }>();
  let y = KOPF;
  const bahnen: { feld: string; y: number; h: number }[] = [];
  for (const f of felder) {
    const h = hoehe.get(f)! * ZEILE + 10;
    bahnen.push({ feld: f, y, h });
    spalten.forEach((s, i) => {
      (zellen.get(`${f}/${s}`) ?? []).forEach((x, n) =>
        lage.set(x.key, { x: i * SPALTE + 8, y: y + 6 + n * ZEILE }),
      );
    });
    y += h;
  }
  const breite = spalten.length * SPALTE + 8;
  const kanten = daten.technologies.flatMap((x) =>
    x.prerequisites
      .filter((v) => lage.has(v))
      .map((v) => ({ von: lage.get(v)!, nach: lage.get(x.key)!, key: `${v}>${x.key}`, x })),
  );
  const start = gewaehlt ? lage.get(gewaehlt) : undefined;
  return (
    <div className="baum-rahmen">
      {/* The fields stay in view while the eras scroll. */}
      <div className="bahnkoepfe" style={{ height: y }}>
        {bahnen.map((b, i) => (
          <div
            key={b.feld}
            className={i % 2 ? "bahn" : "bahn ungerade"}
            style={{ top: b.y, height: b.h }}
          >
            {t(`fachrichtung.${b.feld}`)}
          </div>
        ))}
      </div>
      <div
        className="baum-scroll"
        ref={(el) => {
          // Open at the frontier: the chosen technology near the right edge, its
          // predecessors to the left.
          if (el && start && !el.dataset.gerollt) {
            el.dataset.gerollt = "1";
            el.scrollLeft = Math.max(0, start.x + KNOTEN_B + 24 - el.clientWidth);
          }
        }}
      >
        <div
          className="baum-flaeche"
          style={{ width: breite, height: y }}
          role="group"
          aria-label={t("forschung.baum")}
        >
          <svg className="technikbaum" width={breite} height={y} aria-hidden="true">
            {bahnen.map((b, i) => (
              <rect
                key={b.feld}
                className={i % 2 ? "bahn" : "bahn ungerade"}
                x={0}
                y={b.y}
                width={breite}
                height={b.h}
              />
            ))}
            {spalten.map((s, i) => (
              <text key={s} className="epoche" x={i * SPALTE + 8} y={17}>
                {t(`forschung.epoche.${EPOCHEN[s]!.key}`)}
              </text>
            ))}
            {kanten.map(({ von, nach, key, x }) => {
              const x1 = von.x + KNOTEN_B;
              const y1 = von.y + KNOTEN_H / 2;
              const y2 = nach.y + KNOTEN_H / 2;
              // Same column: around the right side; else from right edge to left edge.
              const d =
                nach.x <= von.x
                  ? `M ${x1} ${y1} C ${x1 + 14} ${y1}, ${x1 + 14} ${y2}, ${nach.x + KNOTEN_B} ${y2}`
                  : `M ${x1} ${y1} C ${x1 + 18} ${y1}, ${nach.x - 18} ${y2}, ${nach.x} ${y2}`;
              return (
                <path
                  key={key}
                  d={d}
                  className={`kante ${gewaehlt === x.key || key.startsWith(`${gewaehlt}>`) ? "aktiv" : ""}`}
                />
              );
            })}
          </svg>
          {daten.technologies.map((x) => {
            const p = lage.get(x.key);
            if (!p) return null;
            const name = technologieName(x.key);
            return (
              <button
                key={x.key}
                type="button"
                className={`knoten technik-${x.status}${gewaehlt === x.key ? " gewaehlt" : ""}`}
                style={{ left: p.x, top: p.y, width: KNOTEN_B, height: KNOTEN_H }}
                aria-label={`${name}, ${x.invention_year}, ${t(`forschung.status.${x.status}`)}`}
                aria-pressed={gewaehlt === x.key}
                title={name}
                onClick={() => onWahl(x.key)}
              >
                <span>{name}</span>
                <small>{x.invention_year}</small>
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}

function Liste({
  daten,
  gewaehlt,
  onWahl,
  detail,
}: {
  daten: Forschung;
  gewaehlt: string | null;
  onWahl: (k: string) => void;
  /** Shown below the chosen item (phones). */
  detail: ReactNode;
}) {
  const felder = [...new Set(daten.technologies.map((x) => x.field))];
  return (
    <div className="technikliste">
      {felder.map((f) => (
        <section key={f} aria-label={t(`fachrichtung.${f}`)}>
          <h3>{t(`fachrichtung.${f}`)}</h3>
          <ul>
            {daten.technologies
              .filter((x) => x.field === f)
              .sort((a, b) => a.invention_year - b.invention_year)
              .map((x) => (
                <li key={x.key}>
                  <button
                    type="button"
                    className={`technikzeile technik-${x.status}${gewaehlt === x.key ? " gewaehlt" : ""}`}
                    aria-pressed={gewaehlt === x.key}
                    onClick={() => onWahl(x.key)}
                  >
                    <span className={`punkt technik-${x.status}`} />
                    <span>{technologieName(x.key)}</span>
                    <small>{x.invention_year}</small>
                  </button>
                  {gewaehlt === x.key && detail}
                </li>
              ))}
          </ul>
        </section>
      ))}
    </div>
  );
}

function TechnologieDetail({
  id,
  tech: x,
  daten,
  onWahl,
  onZentren,
  onStandorte,
}: {
  id: string;
  tech: Technologie;
  daten: Forschung;
  onWahl: (k: string) => void;
  onZentren: () => void;
  onStandorte: () => void;
}) {
  const name = technologieName(x.key);
  const { los, antwort } = useAktion(`forschen/${x.key}`);
  const dabei = daten.centers.filter((z) => z.project === x.key);
  // Centers with a laboratory that could switch to this technology.
  const bereit = daten.centers.filter((z) => z.labs.length > 0 && z.project !== x.key);
  const [zentrum, setZentrum] = useState(bereit[0]?.site ?? -1);
  const formId = useId();
  const fortschritt = x.needed ? Math.min(x.points / x.needed, 1) : 0;
  return (
    <section id={id} className="karte technikdetail" aria-label={name}>
      <h3>{name}</h3>
      <p className="technik-kopf">
        <span className={`chip technik-${x.status}`}>{t(`forschung.status.${x.status}`)}</span>
        <span className="gedaempft">
          {t(`fachrichtung.${x.field}`)} · {t("forschung.erfunden", { jahr: x.invention_year })}
        </span>
      </p>

      {x.status !== "bekannt" && x.needed !== null && (
        <dl className="werte technikwerte">
          <dt>{t("forschung.fortschritt")}</dt>
          <dd>
            <span className="technik-fortschritt">
              <progress max={1} value={fortschritt} aria-label={t("forschung.fortschritt")} />
              <span className="zahl">{formatProzent(fortschritt)}</span>
            </span>
            <small className="gedaempft">
              {t("forschung.punkte", {
                punkte: formatZahl(x.points, 0),
                bedarf: formatZahl(x.needed, 0),
              })}
            </small>
          </dd>
          {x.factor !== null && Math.abs(x.factor - 1) > 0.01 && (
            <>
              <dt>{t("forschung.faktor")}</dt>
              <dd>
                {t(x.factor > 1 ? "forschung.vorgriff" : "forschung.nachzuegler", {
                  faktor: formatZahl(x.factor, 2),
                })}
              </dd>
            </>
          )}
          {dabei.length > 0 && (
            <>
              <dt>{t("forschung.forscht_daran")}</dt>
              <dd>
                {dabei
                  .map((z) =>
                    z.ready
                      ? landName(z.country)
                      : `${landName(z.country)} (${
                          z.building_until
                            ? t("forschung.labor_im_bau", { datum: formatDatum(z.building_until) })
                            : t("forschung.ohne_labor")
                        })`,
                  )
                  .join(", ")}
              </dd>
            </>
          )}
          {x.days !== null ? (
            <>
              <dt>{t("forschung.dauer")}</dt>
              <dd>
                {t("forschung.tage_etwa", { tage: formatZahl(Math.ceil(x.days)) })}{" "}
                <small className="gedaempft">
                  {t("forschung.mit_deinen_zentren", { punkte: formatZahl(x.points_per_day, 1) })}
                </small>
              </dd>
            </>
          ) : (
            x.one_lab && (
              <>
                <dt>{t("forschung.dauer")}</dt>
                <dd>
                  {t("forschung.tage_etwa", { tage: formatZahl(Math.ceil(x.one_lab.days)) })}{" "}
                  <small className="gedaempft">
                    {t("forschung.mit_einem_labor", { land: landName(x.one_lab.country) })}
                  </small>
                </dd>
              </>
            )
          )}
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
        </dl>
      )}

      {(x.status === "erforschbar" || x.status === "in_arbeit") &&
        (bereit.length === 0 ? (
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
            aria-label={t("forschung.forschen_an", { technologie: name })}
            onSubmit={(e) => {
              e.preventDefault();
              void los(
                [{ SetResearch: { site: zentrum, technology: x.key } }],
                t("forschung.gestartet", { technologie: name }),
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
                    {landName(z.country)}
                    {z.project
                      ? ` (${t("forschung.statt", { technologie: technologieName(z.project) })})`
                      : ""}
                  </option>
                ))}
              </select>
            </div>
            <button type="submit" className="haupt">
              {t("forschung.starten")}
            </button>
          </form>
        ))}
      {/* Outside the form: it goes once the center works on this technology. */}
      <Rueckmeldung meldung={antwort} />
      {x.status === "gesperrt" && (
        <p className="warntext">
          {t("forschung.erst_voraussetzungen", {
            liste: x.prerequisites
              .filter((v) => daten.technologies.find((y) => y.key === v)?.status !== "bekannt")
              .map(technologieName)
              .join(", "),
          })}
        </p>
      )}

      <dl className="werte">
        <dt>{t("forschung.voraussetzungen")}</dt>
        <dd>
          {x.prerequisites.length === 0
            ? t("forschung.keine")
            : x.prerequisites.map((v) => (
                <button key={v} type="button" className="verweis knapp" onClick={() => onWahl(v)}>
                  {technologieName(v)}
                </button>
              ))}
        </dd>
        <dt>{t("forschung.fuehrt_zu")}</dt>
        <dd>
          {x.leads_to.length === 0
            ? t("forschung.keine")
            : x.leads_to.map((v) => (
                <button key={v} type="button" className="verweis knapp" onClick={() => onWahl(v)}>
                  {technologieName(v)}
                </button>
              ))}
        </dd>
      </dl>

      <h4>{t("forschung.schaltet_frei")}</h4>
      {x.facilities.length === 0 && x.recipes.length === 0 && (
        <p className="gedaempft">{t("forschung.nichts_frei")}</p>
      )}
      {x.facilities.length > 0 && (
        <>
          <h5>{t("forschung.anlagen")}</h5>
          <ul className="freischaltungen">
            {x.facilities.map((f) => (
              <li key={f.key}>
                <strong>{t(`anlage.${f.key}`)}</strong>{" "}
                <span className="gedaempft">
                  {t("forschung.anlage_info", {
                    art: t(f.kind_text),
                    betrag: formatGeld(f.investment_usd),
                    tage: f.build_days,
                  })}
                </span>
              </li>
            ))}
          </ul>
        </>
      )}
      {x.recipes.length > 0 && (
        <>
          <h5>{t("forschung.verfahren")}</h5>
          <ul className="freischaltungen">
            {x.recipes.map((r) => (
              <li key={r.key}>
                <strong>{t(`rezept.${r.key}`)}</strong>
                <span>
                  {r.inputs_per_day.length > 0 &&
                    `${r.inputs_per_day.map(([p, q]) => `${formatMenge(q)} ${einheit(daten, p)} ${produktName(p)}`).join(" + ")} → `}
                  {formatMenge(r.output_per_day)} {einheit(daten, r.product)}{" "}
                  {produktName(r.product)} {t("forschung.je_anlage_tag")}
                </span>
                <span className="gedaempft">
                  {t("forschung.braucht_anlage", { anlage: t(`anlage.${r.facility}`) })}
                  {r.facility_missing &&
                    ` ${t("forschung.anlage_fehlt", { technologie: technologieName(r.facility_missing) })}`}
                </span>
              </li>
            ))}
          </ul>
        </>
      )}
      {x.products.length > 0 && (
        <>
          <h5>{t("forschung.produkte")}</h5>
          <p>{x.products.map(produktName).join(", ")}</p>
        </>
      )}
      {x.status === "bekannt" && x.facilities.length > 0 && (
        <div className="knopfreihe links">
          <button type="button" onClick={onStandorte}>
            {t("forschung.anlage_bauen")}
          </button>
        </div>
      )}
    </section>
  );
}

// --- Research centers ---

function Zentren({ daten, heimat }: { daten: Forschung; heimat: string }) {
  const erforschbar = daten.technologies.filter(
    (x) => x.status === "erforschbar" || x.status === "in_arbeit",
  );
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
        <Zentrum key={z.site} z={z} daten={daten} erforschbar={erforschbar} />
      ))}
      <ZentrumGruenden heimat={heimat} />
    </div>
  );
}

function Zentrum({
  z,
  daten,
  erforschbar,
}: {
  z: Forschung["centers"][number];
  daten: Forschung;
  erforschbar: Technologie[];
}) {
  const { los, antwort } = useAktion(`zentrum/${z.site}`);
  const [projekt, setProjekt] = useState(z.project ?? "");
  const id = useId();
  const titel = t("forschung.zentrum_in", { land: landName(z.country) });
  const fertig = z.labs.filter((l) => l.ready <= daten.date);
  const plaetze = fertig.reduce((n, l) => n + l.count * daten.laboratory_posts * l.utilization, 0);
  const tech = daten.technologies.find((x) => x.key === z.project);
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
        {tech && tech.needed !== null && (
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
      </dl>
      {z.labs.length === 0 && <p className="gedaempft">{t("forschung.erst_labor")}</p>}
      {z.labs.length > 0 && (
        <form
          className="formular-zeile"
          aria-label={t("forschung.projekt_von", { land: landName(z.country) })}
          onSubmit={(e) => {
            e.preventDefault();
            void los(
              [{ SetResearch: { site: z.site, technology: projekt || null } }],
              projekt
                ? t("forschung.gestartet", { technologie: technologieName(projekt) })
                : t("forschung.angehalten"),
            );
          }}
        >
          <div className="feld">
            <label htmlFor={`${id}-projekt`}>{t("forschung.projekt")}</label>
            <select
              id={`${id}-projekt`}
              value={projekt}
              onChange={(e) => setProjekt(e.target.value)}
            >
              <option value="">{t("forschung.kein_projekt")}</option>
              {erforschbar.map((x) => (
                <option key={x.key} value={x.key}>
                  {technologieName(x.key)}
                </option>
              ))}
            </select>
          </div>
          <button type="submit">{t("werk.uebernehmen")}</button>
        </form>
      )}
      {z.labs.map((l, i) => (
        <LaborAuslastung
          key={`${l.slot}/${l.utilization}`}
          site={z.site}
          labor={l}
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
  name,
}: {
  site: number;
  labor: Forschung["centers"][number]["labs"][number];
  name: string;
}) {
  const [prozent, setProzent] = useState(zahlFeld(labor.utilization * 100, 0));
  const [fehler, setFehler] = useState<string | null>(null);
  const { los, antwort } = useAktion(`labor/${site}/${labor.slot}`);
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
      />
      <button type="submit">{t("werk.uebernehmen")}</button>
      {fehler && <p className="fehlertext">{fehler}</p>}
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

function ZentrumGruenden({ heimat }: { heimat: string }) {
  const [land, setLand] = useState(heimat);
  const { los, antwort } = useAktion("zentrum-gruenden");
  const id = useId();
  return (
    <form
      className="karte"
      aria-label={t("forschung.gruenden")}
      onSubmit={(e) => {
        e.preventDefault();
        void los(
          [{ FoundSite: { country: land, kind: "ResearchCenter" } }],
          t("forschung.gegruendet", { land: landName(land) }),
        );
      }}
    >
      <h3>{t("forschung.gruenden")}</h3>
      <div className="formular-zeile">
        <div className="feld">
          <label htmlFor={`${id}-land`}>{t("produktion.land")}</label>
          <select id={`${id}-land`} value={land} onChange={(e) => setLand(e.target.value)}>
            {LAENDER.map((k) => (
              <option key={k} value={k}>
                {landName(k)}
              </option>
            ))}
          </select>
        </div>
        <button type="submit">{t("produktion.gruenden_knopf")}</button>
      </div>
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}
