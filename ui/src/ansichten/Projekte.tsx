// Development projects PM0–PM7 (PD2, docs/PRODUKTENTWICKLUNG.md): start a project, hire the
// project leaders, set the people per role and see staffing scenarios. No game logic here:
// every figure comes from the view.
import { useId, useState } from "react";
import { formatDatum, formatGeld, formatProzent, formatZahl, landName } from "../format";
import type {
  Forschung,
  LaborRolle,
  Leitungszeile,
  Meilensteinbericht,
  Partnerfirma,
  Projekt,
  Projektkennzahlen,
  Projektleiter,
  Verbesserung,
} from "../kern";
import { t } from "../texte";
import { Rueckmeldung, useAktion } from "./gemeinsam";
import { einheitAus } from "./stellen";

/** The roles of the laboratory by their key in the views. */
export const ROLLEN: Record<string, LaborRolle> = {
  produktmanager: "ProductManager",
  entwickler: "Developer",
  fertigungsplaner: "ProductionPlanner",
  laborpersonal: "Technician",
};

const DELEGATION: Record<string, "ShowAll" | "Standard" | "LeadDecides"> = {
  alles: "ShowAll",
  standard: "Standard",
  leiter: "LeadDecides",
};

const ENTSCHEIDUNG: Record<string, "Continue" | "Conditions" | "Loop" | "Pause" | "Abort"> = {
  weiter: "Continue",
  auflagen: "Conditions",
  schleife: "Loop",
  pause: "Pause",
  abbruch: "Abort",
};

const produktName = (p: string) => t(`produkt.${p}`);

function zentrumName(z: Forschung["centers"][number]): string {
  return z.name ? `${z.name} · ${landName(z.country)}` : landName(z.country);
}

export function Projekte({
  daten,
  vorwahl,
  onZentren,
}: {
  daten: Forschung;
  /** A product chosen elsewhere to start a project for. */
  vorwahl: string | null;
  onZentren: () => void;
}) {
  const p = daten.projects;
  if (!p) return null;
  const laufend = p.projects.filter((x) => x.status === "laufend");
  const fertig = p.projects.filter((x) => x.status !== "laufend");
  return (
    <div className="projekte">
      <p className="erklaerung">{t("projekte.erklaerung")}</p>
      <NeuesProjekt daten={daten} vorwahl={vorwahl} onZentren={onZentren} />
      <Leiter leiter={p.leads} gehalt={p.salary_usd_year} />
      {p.suggestions.length > 0 && <Vorschlaege daten={daten} />}
      {(p.license_offers.length > 0 || p.knowledge.length > 0) && <Lizenzen daten={daten} />}
      {p.leadership.length > 0 && <Entwicklungsleitung daten={daten} />}
      {p.improvement_targets.length > 0 && <Weiterentwicklungen daten={daten} />}
      {(laufend.length > 0 || p.improvements.some((x) => x.status === "laufend")) && (
        <Portfolio daten={daten} />
      )}
      <section aria-label={t("projekte.laufend")}>
        <h3>{t("projekte.laufend")}</h3>
        {laufend.length === 0 && <p className="gedaempft">{t("projekte.keine")}</p>}
        {laufend.map((x) => (
          <ProjektKarte key={x.id} x={x} partner={p.partners} />
        ))}
      </section>
      {fertig.length > 0 && (
        <section aria-label={t("projekte.abgeschlossen")}>
          <h3>{t("projekte.abgeschlossen")}</h3>
          <ul className="liste">
            {fertig.map((x) => (
              <li key={x.id}>
                {t("projekte.abgeschlossen_zeile", {
                  produkt: produktName(x.product),
                  status: t(`projekte.status.${x.status}`),
                  datum: x.finished ? formatDatum(x.finished) : "",
                  kosten: formatGeld(x.spent_usd),
                })}
              </li>
            ))}
          </ul>
        </section>
      )}
    </div>
  );
}

function NeuesProjekt({
  daten,
  vorwahl,
  onZentren,
}: {
  daten: Forschung;
  vorwahl: string | null;
  onZentren: () => void;
}) {
  const p = daten.projects;
  const { los, antwort } = useAktion("projekt/neu");
  const id = useId();
  const angebote = [...(p?.offers ?? [])].sort((a, b) =>
    produktName(a.product).localeCompare(produktName(b.product), "de"),
  );
  const labore = daten.centers.filter((z) => z.ready);
  const leiter = (p?.leads ?? []).filter((l) => l.hired && !l.busy);
  const [produkt, setProdukt] = useState(vorwahl ?? "");
  const [zentrum, setZentrum] = useState(labore[0]?.site ?? -1);
  const [leiterId, setLeiterId] = useState(leiter[0]?.id ?? -1);
  const [tempo, setTempo] = useState(1);
  const [ausstattung, setAusstattung] = useState(1);
  if (!p) return null;
  const wahl = angebote.find((a) => a.product === (produkt || angebote[0]?.product));
  const bereit = wahl !== undefined && zentrum >= 0 && leiter.some((l) => l.id === leiterId);
  return (
    <section aria-label={t("projekte.neu")}>
      <h3>{t("projekte.neu")}</h3>
      {labore.length === 0 && (
        <p className="erklaerung">
          {t("projekte.erst_labor")}{" "}
          <button type="button" className="schlicht" onClick={onZentren}>
            {t("forschung.zu_den_zentren")}
          </button>
        </p>
      )}
      {leiter.length === 0 && <p className="erklaerung">{t("projekte.erst_leiter")}</p>}
      {angebote.length === 0 ? (
        <p className="gedaempft">{t("projekte.keine_angebote")}</p>
      ) : (
        <form
          className="formular-zeile"
          aria-label={t("projekte.neu")}
          onSubmit={(e) => {
            e.preventDefault();
            if (!bereit || !wahl) return;
            void los(
              [
                {
                  StartProject: {
                    site: zentrum,
                    product: wahl.product,
                    lead: leiterId,
                    tempo,
                    trim: ausstattung,
                  },
                },
              ],
              t("projekte.gestartet", { produkt: produktName(wahl.product) }),
            );
          }}
        >
          <div className="feld">
            <label htmlFor={`${id}-produkt`}>{t("projekte.produkt")}</label>
            <select
              id={`${id}-produkt`}
              value={wahl?.product ?? ""}
              onChange={(e) => setProdukt(e.target.value)}
            >
              {angebote.map((a) => (
                <option key={a.product} value={a.product}>
                  {produktName(a.product)}
                </option>
              ))}
            </select>
          </div>
          <div className="feld">
            <label htmlFor={`${id}-zentrum`}>{t("forschung.zentrum")}</label>
            <select
              id={`${id}-zentrum`}
              value={zentrum}
              onChange={(e) => setZentrum(Number(e.target.value))}
            >
              {labore.map((z) => (
                <option key={z.site} value={z.site}>
                  {zentrumName(z)}
                </option>
              ))}
            </select>
          </div>
          <div className="feld">
            <label htmlFor={`${id}-leiter`}>{t("projekte.leiter")}</label>
            <select
              id={`${id}-leiter`}
              value={leiterId}
              onChange={(e) => setLeiterId(Number(e.target.value))}
            >
              {leiter.map((l) => (
                <option key={l.id} value={l.id}>
                  {l.name}
                </option>
              ))}
            </select>
          </div>
          <div className="feld">
            <label htmlFor={`${id}-tempo`}>{t("projekte.tempo")}</label>
            <select
              id={`${id}-tempo`}
              value={tempo}
              onChange={(e) => setTempo(Number(e.target.value))}
            >
              {p.tempos.map((k, i) => (
                <option key={k} value={i}>
                  {t(`projekte.tempo.${k}`)}
                </option>
              ))}
            </select>
          </div>
          <div className="feld">
            <label htmlFor={`${id}-ausstattung`}>{t("projekte.ausstattung")}</label>
            <select
              id={`${id}-ausstattung`}
              value={ausstattung}
              onChange={(e) => setAusstattung(Number(e.target.value))}
            >
              {p.trims.map((k, i) => (
                <option key={k} value={i}>
                  {t(`projekte.ausstattung.${k}`)}
                </option>
              ))}
            </select>
          </div>
          <button type="submit" className="haupt" disabled={!bereit}>
            {t("projekte.starten")}
          </button>
        </form>
      )}
      {wahl && (
        <p className="feld-hilfe">
          {t("projekte.schaetzung", {
            kosten: formatGeld(wahl.cost_usd),
            tage: formatZahl(Math.ceil(wahl.days)),
          })}
        </p>
      )}
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

function Leiter({ leiter, gehalt }: { leiter: Projektleiter[]; gehalt: number }) {
  const { los, antwort } = useAktion("projekt/leiter");
  const eigene = leiter.filter((l) => l.hired);
  const markt = leiter.filter((l) => !l.hired);
  const zeile = (l: Projektleiter) => (
    <tr key={l.id}>
      <th scope="row">{l.name}</th>
      <td>{formatZahl(l.age, 0)}</td>
      <td>{t(`projekte.charakter.${l.character}`)}</td>
      <td>{formatProzent(l.experience)}</td>
      <td>
        {t("projekte.bilanz", { fertig: l.finished, abgebrochen: l.aborted })}
        {l.balance !== null && (
          <small className="gedaempft">
            {" "}
            {t("vorschlag.bilanz", { wert: formatProzent(l.balance), n: l.balance_count })}
          </small>
        )}
      </td>
      <td>
        {l.hired ? (
          l.busy ? (
            t("projekte.leiter_beschaeftigt")
          ) : (
            t("projekte.leiter_frei")
          )
        ) : (
          <button
            type="button"
            aria-label={`${t("projekte.einstellen")}: ${l.name}`}
            onClick={() =>
              void los([{ HireProjectLead: { lead: l.id } }], t("projekte.eingestellt"))
            }
          >
            {t("projekte.einstellen")}
          </button>
        )}{" "}
        {l.hired && (
          <button
            type="button"
            aria-label={`${t("projekte.entlassen")}: ${l.name}`}
            disabled={l.busy}
            onClick={() =>
              void los([{ DismissProjectLead: { lead: l.id } }], t("projekte.entlassen_ok"))
            }
          >
            {t("projekte.entlassen")}
          </button>
        )}
      </td>
    </tr>
  );
  const kopf = (
    <thead>
      <tr>
        <th>{t("projekte.name")}</th>
        <th>{t("projekte.alter")}</th>
        <th>{t("projekte.charakter")}</th>
        <th>{t("projekte.erfahrung")}</th>
        <th>{t("projekte.bilanz_titel")}</th>
        <th />
      </tr>
    </thead>
  );
  return (
    <section aria-label={t("projekte.leiter_titel")}>
      <h3>{t("projekte.leiter_titel")}</h3>
      <p className="feld-hilfe">{t("projekte.leiter_hilfe", { gehalt: formatGeld(gehalt) })}</p>
      {eigene.length > 0 && (
        <div className="tabelle">
          <table className="mobil-karten" aria-label={t("projekte.eigene_leiter")}>
            <caption>{t("projekte.eigene_leiter")}</caption>
            {kopf}
            <tbody>{eigene.map(zeile)}</tbody>
          </table>
        </div>
      )}
      {markt.length > 0 && (
        <div className="tabelle">
          <table className="mobil-karten" aria-label={t("projekte.markt_leiter")}>
            <caption>{t("projekte.markt_leiter")}</caption>
            {kopf}
            <tbody>{markt.map(zeile)}</tbody>
          </table>
        </div>
      )}
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

function ProjektKarte({ x, partner }: { x: Projekt; partner: Partnerfirma[] }) {
  const { los, antwort } = useAktion(`projekt/${x.id}`);
  const [partnerId, setPartnerId] = useState(partner[0]?.company ?? -1);
  const name = produktName(x.product);
  const delta = (d: number) =>
    d === 0
      ? t("projekte.szenario_bedarf")
      : d > 0
        ? t("projekte.szenario_mehr", { n: d })
        : t("projekte.szenario_weniger", { n: -d });
  return (
    <article className="karte projektkarte" aria-label={name}>
      <h4>{name}</h4>
      {x.event && <Ereignis x={x} />}
      {x.pending && <Entscheidung x={x} />}
      <progress value={x.progress} max={1} aria-label={t("projekte.fortschritt")} />
      <dl className="werte">
        <dt>{t("projekte.fortschritt")}</dt>
        <dd>
          {t("projekte.meilenstein", {
            erreicht: x.milestone,
            naechster: x.next_milestone ?? 7,
            anteil: formatProzent(x.progress),
          })}
        </dd>
        <dt>{t("projekte.leiter")}</dt>
        <dd>{x.lead}</dd>
        {(x.head_start > 0 || x.platform > 0) && (
          <>
            <dt>{t("projekte.vorsprung")}</dt>
            <dd>
              {t("projekte.vorsprung_werte", {
                wissen: formatProzent(x.head_start),
                plattform: formatProzent(x.platform),
              })}
            </dd>
          </>
        )}
        {x.subsidy && (
          <>
            <dt>{t("projekte.foerderung")}</dt>
            <dd>
              {t("projekte.foerderung_werte", {
                programm: t(`projektfoerderung.${x.subsidy.program}`),
                anteil: formatProzent(x.subsidy.share),
                erhalten: formatGeld(x.subsidy.received_usd),
              })}
            </dd>
          </>
        )}
        {x.first_mover && (
          <>
            <dt>{t("projekte.erster")}</dt>
            <dd>{t("projekte.erster_text")}</dd>
          </>
        )}
        {x.license_from && (
          <>
            <dt>{t("projekte.lizenz_von")}</dt>
            <dd>{x.license_from}</dd>
          </>
        )}
        {x.partner && (
          <>
            <dt>{t("projekte.partner")}</dt>
            <dd>{x.partner}</dd>
          </>
        )}
        {(x.fast_track || x.pilot) && (
          <>
            <dt>{t("projekte.besonderheiten")}</dt>
            <dd>
              {[
                x.fast_track ? t("projekte.fast_track_aktiv") : null,
                x.pilot ? t(`projekte.pilot.${x.pilot}`) : null,
              ]
                .filter(Boolean)
                .join(", ")}
            </dd>
          </>
        )}
        {x.promise && (
          <>
            <dt>{t("vorschlag.zusage")}</dt>
            <dd>
              {t("vorschlag.zusage_werte", {
                kosten: formatGeld(x.promise.project_cost),
                tage: formatZahl(Math.ceil(x.promise.project_days)),
                beitrag: formatGeld(x.promise.contribution_year),
              })}
            </dd>
          </>
        )}
        <dt>{t("projekte.tempo")}</dt>
        <dd>
          {t(`projekte.tempo.${x.tempo}`)} · {t(`projekte.ausstattung.${x.trim}`)}
        </dd>
        <dt>{t("projekte.restdauer")}</dt>
        <dd>{t("projekte.tage", { tage: formatZahl(Math.ceil(x.days_left)) })}</dd>
        <dt>{t("projekte.risiko")}</dt>
        <dd>
          {formatProzent(x.risk)}
          {x.penalty > 0 && (
            <small className="gedaempft">
              {" "}
              {t("projekte.abschlag", { anteil: formatProzent(x.penalty) })}
            </small>
          )}
        </dd>
        <dt>{t("projekte.kosten")}</dt>
        <dd>
          {t("projekte.kosten_stand", {
            ausgegeben: formatGeld(x.spent_usd),
            gesamt: formatGeld(x.budget_usd),
          })}
        </dd>
      </dl>
      <div className="tabelle">
        <table className="mobil-karten" aria-label={t("projekte.besetzung")}>
          <caption>{t("projekte.besetzung")}</caption>
          <thead>
            <tr>
              <th>{t("labor.rolle")}</th>
              <th>{t("projekte.bedarf")}</th>
              <th>{t("projekte.zugewiesen")}</th>
              <th>{t("projekte.frei")}</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {x.roles.map((r) => {
              const rolle = ROLLEN[r.role];
              if (rolle === undefined) return null;
              return (
                <tr key={r.role}>
                  <th scope="row">{t(`labor.rolle.${r.role}`)}</th>
                  <td>{formatZahl(r.need)}</td>
                  <td className={r.assigned < r.need ? "warnung" : undefined}>
                    {formatZahl(r.assigned)}
                  </td>
                  <td>{formatZahl(r.free)}</td>
                  <td>
                    <button
                      type="button"
                      aria-label={`${t("projekte.mehr")}: ${t(`labor.rolle.${r.role}`)}`}
                      disabled={r.free === 0}
                      onClick={() =>
                        void los([
                          {
                            AssignProjectStaff: {
                              project: x.id,
                              role: rolle,
                              count: r.assigned + 1,
                            },
                          },
                        ])
                      }
                    >
                      +
                    </button>{" "}
                    <button
                      type="button"
                      aria-label={`${t("projekte.weniger")}: ${t(`labor.rolle.${r.role}`)}`}
                      disabled={r.assigned === 0}
                      onClick={() =>
                        void los([
                          {
                            AssignProjectStaff: {
                              project: x.id,
                              role: rolle,
                              count: r.assigned - 1,
                            },
                          },
                        ])
                      }
                    >
                      −
                    </button>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      <p className="feld-hilfe">
        {x.auto ? t("projekte.automatisch") : t("projekte.von_hand")}{" "}
        {!x.auto && (
          <button
            type="button"
            className="schlicht"
            onClick={() => void los([{ AutoAssignProject: { project: x.id } }])}
          >
            {t("projekte.auf_automatik")}
          </button>
        )}
      </p>
      <div className="tabelle">
        <table className="mobil-karten" aria-label={t("projekte.szenarien")}>
          <caption>{t("projekte.szenarien")}</caption>
          <thead>
            <tr>
              <th>{t("projekte.szenario")}</th>
              <th>{t("projekte.restdauer")}</th>
              <th>{t("projekte.risiko")}</th>
              <th>{t("projekte.abschlag_titel")}</th>
            </tr>
          </thead>
          <tbody>
            {x.scenarios.map((s) => (
              <tr key={s.delta}>
                <th scope="row">{delta(s.delta)}</th>
                <td>{t("projekte.tage", { tage: formatZahl(Math.ceil(s.days)) })}</td>
                <td>{formatProzent(s.risk)}</td>
                <td>{formatProzent(s.penalty)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <Berichte x={x} />
      <p className="feld-hilfe">
        <label>
          {t("projekte.delegation")}{" "}
          <select
            value={x.delegation}
            onChange={(e) =>
              void los([
                {
                  SetProjectDelegation: {
                    project: x.id,
                    mode: DELEGATION[e.target.value] ?? "Standard",
                  },
                },
              ])
            }
          >
            {Object.keys(DELEGATION).map((k) => (
              <option key={k} value={k}>
                {t(`projekte.delegation.${k}`)}
              </option>
            ))}
          </select>
        </label>
      </p>
      {x.paused && (
        <div className="knopfreihe links">
          <button type="button" onClick={() => void los([{ ResumeProject: { project: x.id } }])}>
            {t("projekte.fortsetzen")}
          </button>
        </div>
      )}
      {x.can_partner && partner.length > 0 && (
        <p className="zeile">
          <label>
            {t("projekte.partner_vorschlagen")}{" "}
            <select value={partnerId} onChange={(e) => setPartnerId(Number(e.target.value))}>
              {partner.map((c) => (
                <option key={c.company} value={c.company}>
                  {c.name}
                </option>
              ))}
            </select>
          </label>{" "}
          <button
            type="button"
            disabled={partnerId < 0}
            onClick={() =>
              void los(
                [{ ProposePartnership: { project: x.id, partner: partnerId } }],
                t("projekte.partner_ok"),
              )
            }
          >
            {t("projekte.partner_fragen")}
          </button>
        </p>
      )}
      <div className="knopfreihe links">
        <button
          type="button"
          onClick={() =>
            void los(
              [{ AbortProject: { project: x.id } }],
              t("projekte.abgebrochen_ok", { produkt: name }),
            )
          }
        >
          {t("projekte.abbrechen")}
        </button>
      </div>
      <Rueckmeldung meldung={antwort} />
    </article>
  );
}

/** The decision the project waits for: report, choices and the five decisions. */
function Entscheidung({ x }: { x: Projekt }) {
  const { los, antwort } = useAktion(`projekt/${x.id}/entscheid`);
  const pending = x.pending;
  const [wahl, setWahl] = useState<number | null>(null);
  const [mitVorschlag, setMitVorschlag] = useState(true);
  if (!pending) return null;
  const bericht = x.reports.find((r) => r.milestone === pending.milestone);
  const gewaehlt = wahl ?? pending.choices.findIndex((c) => c.preferred);
  const entscheide = (key: string) =>
    void los([
      {
        DecideMilestone: {
          project: x.id,
          decision: ENTSCHEIDUNG[key] ?? "Continue",
          choice:
            pending.choices.length > 0
              ? Math.max(gewaehlt, 0)
              : pending.proposal
                ? Number(mitVorschlag)
                : null,
        },
      },
    ]);
  return (
    <section className="hinweis" role="group" aria-label={t("projekte.entscheidung")}>
      <h4>{t("projekte.entscheidung_titel", { meilenstein: pending.milestone })}</h4>
      <p>
        {t("projekte.entscheidung_frist", { datum: formatDatum(pending.deadline) })}{" "}
        {t("projekte.empfehlung", {
          empfehlung: t(`projekte.entscheid.${pending.recommendation}`),
        })}
      </p>
      {bericht && <Bericht b={bericht} />}
      {pending.proposal && (
        <label className="zeile">
          <input
            type="checkbox"
            checked={mitVorschlag}
            onChange={(e) => setMitVorschlag(e.target.checked)}
          />{" "}
          {t(`projekte.vorschlag.${pending.proposal}`)}
          <small className="gedaempft"> {t(`projekte.vorschlag.${pending.proposal}_hilfe`)}</small>
        </label>
      )}
      {pending.choices.length > 0 && (
        <fieldset>
          <legend>{t(`projekte.wahl.${pending.choice_kind}`)}</legend>
          {pending.choices.map((c, i) => (
            <label key={c.key} className="zeile">
              <input
                type="radio"
                name={`wahl-${x.id}`}
                checked={gewaehlt === i}
                onChange={() => setWahl(i)}
              />{" "}
              {t(`projekte.${pending.choice_kind}.${c.key}`)}
              <small className="gedaempft">
                {" "}
                {t("projekte.wahl_faktoren", {
                  kosten: formatProzent(c.cost - 1),
                  dauer: formatProzent(c.time - 1),
                  risiko: formatProzent(c.risk - 1),
                })}
              </small>
            </label>
          ))}
        </fieldset>
      )}
      <div className="knopfreihe links">
        {Object.keys(ENTSCHEIDUNG).map((k) => (
          <button
            key={k}
            type="button"
            className={k === pending.recommendation ? "primaer" : undefined}
            onClick={() => entscheide(k)}
          >
            {t(`projekte.entscheid.${k}`)}
          </button>
        ))}
      </div>
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

const KENNZAHLEN: [keyof Projektkennzahlen, "geld" | "zahl" | "jahre" | "prozent"][] = [
  ["unit_cost", "geld"],
  ["price", "geld"],
  ["volume_year", "zahl"],
  ["contribution_year", "geld"],
  ["investment", "geld"],
  ["project_cost", "geld"],
  ["project_days", "zahl"],
  ["payback_years", "jahre"],
  ["quality", "prozent"],
];

function Bericht({ b }: { b: Meilensteinbericht }) {
  const wert = (v: number, art: string) =>
    art === "geld"
      ? formatGeld(v)
      : art === "prozent"
        ? formatProzent(v)
        : art === "jahre"
          ? t("projekte.jahre", { n: formatZahl(Math.round(v * 10) / 10) })
          : formatZahl(Math.round(v));
  return (
    <div className="bericht">
      <p>
        {["termin", "budget", "qualitaet", "risiko"].map((n, i) => (
          <span key={n} className={`ampel ampel-${b.lights[i] ?? "gruen"}`}>
            {t(`projekte.ampel.${n}`)}: {t(`projekte.ampel.${b.lights[i] ?? "gruen"}`)}{" "}
          </span>
        ))}
      </p>
      <div className="tabelle">
        <table className="mobil-karten" aria-label={t("projekte.kennzahlen")}>
          <caption>{t("projekte.kennzahlen")}</caption>
          <thead>
            <tr>
              <th>{t("projekte.kennzahl")}</th>
              <th>{t("projekte.prognose")}</th>
              {b.baseline && <th>{t("projekte.plan")}</th>}
              {b.deviation && <th>{t("projekte.abweichung")}</th>}
            </tr>
          </thead>
          <tbody>
            {KENNZAHLEN.map(([k, art]) => (
              <tr key={k}>
                <th scope="row">{t(`projekte.kz.${k}`)}</th>
                <td>{wert(b.kpis[k], art)}</td>
                {b.baseline && <td>{wert(b.baseline[k], art)}</td>}
                {b.deviation && <td>{formatProzent(b.deviation[k])}</td>}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {b.council.length > 0 && (
        <div className="gremium">
          <h5>{t("projekte.gremium")}</h5>
          <ul className="liste">
            {b.council.map((v) => (
              <li key={v.role}>
                {t(`projekte.gremium.${v.role}`)}: {t(`projekte.gremium.${v.stance}`)}
                {v.dissent && (
                  <small className="gedaempft"> {t("projekte.gremium.gegenmeinung")}</small>
                )}
              </li>
            ))}
          </ul>
          {b.council_changed && <p className="gedaempft">{t("projekte.gremium.zurueckgestuft")}</p>}
        </div>
      )}
      {b.causes.length > 0 && (
        <ul className="liste">
          {b.causes.map((c) => (
            <li key={c}>{t(`projekte.ursache.${c}`)}</li>
          ))}
        </ul>
      )}
    </div>
  );
}

/** Earlier reports of the project with the decision taken. */
function Berichte({ x }: { x: Projekt }) {
  const erledigt = x.reports.filter((r) => r.decided !== null);
  if (erledigt.length === 0) return null;
  return (
    <details>
      <summary>{t("projekte.berichte", { n: erledigt.length })}</summary>
      {erledigt.map((r) => (
        <div key={r.milestone}>
          <h5>
            {t("projekte.bericht_zeile", {
              meilenstein: r.milestone,
              datum: formatDatum(r.date),
              entscheidung: t(`projekte.entscheid.${r.decided?.[0] ?? "weiter"}`),
              von: t(r.decided?.[1] ? "projekte.von_dir" : "projekte.vom_leiter"),
            })}
          </h5>
          <Bericht b={r} />
        </div>
      ))}
    </details>
  );
}

/** The event the project waits for: story and two or three options. */
function Ereignis({ x }: { x: Projekt }) {
  const { los, antwort } = useAktion(`projekt/${x.id}/ereignis`);
  const e = x.event;
  if (!e) return null;
  const folge = (v: number, was: string) =>
    v === 0
      ? null
      : t(`projekte.ereignis.${was}`, {
          anteil: formatProzent(Math.abs(v)),
          vorzeichen: v > 0 ? "+" : "−",
        });
  return (
    <section className="hinweis" role="group" aria-label={t("projekte.ereignis")}>
      <h4>{t(`projektereignis.${e.key}.titel`)}</h4>
      <p>{t(`projektereignis.${e.key}.text`)}</p>
      <p className="feld-hilfe">
        {t("projekte.entscheidung_frist", { datum: formatDatum(e.deadline) })}
      </p>
      <ul className="liste">
        {e.options.map((o, i) => (
          <li key={o.key}>
            <button
              type="button"
              className={i === e.recommended ? "primaer" : undefined}
              onClick={() => void los([{ DecideEvent: { project: x.id, option: i } }])}
            >
              {t(`projektereignis.${e.key}.${o.key}`)}
            </button>{" "}
            <small className="gedaempft">
              {[
                folge(o.cost, "kosten"),
                folge(o.delay, o.delay > 0 ? "verzug" : "gewinn"),
                folge(o.quality, "qualitaet"),
              ]
                .filter(Boolean)
                .join(" · ")}
              {i === e.recommended && ` · ${t("projekte.ereignis.empfohlen")}`}
            </small>
          </li>
        ))}
      </ul>
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

const zielName = (z: string) => t(`verbesserung.ziel.${z}`);

/** Improvement projects WM0–WM3: start a step, choose an approach, release the gain (PD5). */
function Weiterentwicklungen({ daten }: { daten: Forschung }) {
  const p = daten.projects;
  const { los, antwort } = useAktion("projekt/verbesserung");
  const id = useId();
  const labore = daten.centers.filter((z) => z.ready);
  const leiter = (p?.leads ?? []).filter((l) => l.hired);
  const angebote = [...(p?.improvement_offers ?? [])].sort(
    (a, b) =>
      produktName(a.product).localeCompare(produktName(b.product), "de") ||
      a.target.localeCompare(b.target),
  );
  const [wahl, setWahl] = useState("");
  const [zentrum, setZentrum] = useState(labore[0]?.site ?? -1);
  const [leiterId, setLeiterId] = useState(leiter[0]?.id ?? -1);
  if (!p) return null;
  const laufend = p.improvements.filter((x) => x.status === "laufend");
  const fertig = p.improvements.filter((x) => x.status !== "laufend");
  const gewaehlt = angebote.find((a) => `${a.product}/${a.target}` === wahl) ?? angebote[0];
  const bereit = gewaehlt !== undefined && zentrum >= 0 && leiter.some((l) => l.id === leiterId);
  return (
    <section aria-label={t("verbesserung.titel")}>
      <h3>{t("verbesserung.titel")}</h3>
      <p className="erklaerung">{t("verbesserung.erklaerung", { stufen: p.improvement_stages })}</p>
      {angebote.length === 0 ? (
        <p className="gedaempft">{t("verbesserung.keine_angebote")}</p>
      ) : (
        <form
          className="formular-zeile"
          aria-label={t("verbesserung.neu")}
          onSubmit={(e) => {
            e.preventDefault();
            if (!bereit || !gewaehlt) return;
            void los(
              [
                {
                  StartImprovement: {
                    site: zentrum,
                    product: gewaehlt.product,
                    target: Math.max(0, p.improvement_targets.indexOf(gewaehlt.target)),
                    lead: leiterId,
                  },
                },
              ],
              t("verbesserung.gestartet", {
                produkt: produktName(gewaehlt.product),
                ziel: zielName(gewaehlt.target),
              }),
            );
          }}
        >
          <div className="feld">
            <label htmlFor={`${id}-ziel`}>{t("verbesserung.ziel")}</label>
            <select
              id={`${id}-ziel`}
              value={gewaehlt ? `${gewaehlt.product}/${gewaehlt.target}` : ""}
              onChange={(e) => setWahl(e.target.value)}
            >
              {angebote.map((a) => (
                <option key={`${a.product}/${a.target}`} value={`${a.product}/${a.target}`}>
                  {t("verbesserung.option", {
                    produkt: produktName(a.product),
                    ziel: zielName(a.target),
                    stufen: a.stages,
                  })}
                </option>
              ))}
            </select>
          </div>
          <div className="feld">
            <label htmlFor={`${id}-zentrum`}>{t("forschung.zentrum")}</label>
            <select
              id={`${id}-zentrum`}
              value={zentrum}
              onChange={(e) => setZentrum(Number(e.target.value))}
            >
              {labore.map((z) => (
                <option key={z.site} value={z.site}>
                  {zentrumName(z)}
                </option>
              ))}
            </select>
          </div>
          <div className="feld">
            <label htmlFor={`${id}-leiter`}>{t("projekte.leiter")}</label>
            <select
              id={`${id}-leiter`}
              value={leiterId}
              onChange={(e) => setLeiterId(Number(e.target.value))}
            >
              {leiter.map((l) => (
                <option key={l.id} value={l.id}>
                  {l.name}
                </option>
              ))}
            </select>
          </div>
          <button type="submit" className="haupt" disabled={!bereit}>
            {t("verbesserung.starten")}
          </button>
        </form>
      )}
      {gewaehlt && (
        <p className="feld-hilfe">
          {t("projekte.schaetzung", {
            kosten: formatGeld(gewaehlt.cost_usd),
            tage: formatZahl(Math.ceil(gewaehlt.days)),
          })}
        </p>
      )}
      {laufend.map((x) => (
        <WeiterentwicklungKarte key={x.id} x={x} />
      ))}
      {fertig.length > 0 && (
        <ul className="liste">
          {fertig.map((x) => (
            <li key={x.id}>
              {t("verbesserung.fertig_zeile", {
                produkt: produktName(x.product),
                ziel: zielName(x.target),
                status: t(`projekte.status.${x.status}`),
                datum: x.finished ? formatDatum(x.finished) : "",
                kosten: formatGeld(x.spent_usd),
              })}
            </li>
          ))}
        </ul>
      )}
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

function WeiterentwicklungKarte({ x }: { x: Verbesserung }) {
  const { los, antwort } = useAktion(`verbesserung/${x.id}`);
  const [ansatz, setAnsatz] = useState(0);
  const name = `${produktName(x.product)} · ${zielName(x.target)}`;
  return (
    <article className="karte projektkarte" aria-label={name}>
      <h4>{name}</h4>
      <progress value={x.progress} max={1} aria-label={t("projekte.fortschritt")} />
      <p>
        {t(`verbesserung.phase.${Math.min(x.phase, 2)}`)}{" "}
        <small className="gedaempft">
          {t("projekte.kosten_stand", { ausgegeben: formatGeld(x.spent_usd), gesamt: "–" })}
        </small>
      </p>
      {x.result !== null && x.expected !== null && (
        <p>
          {t("verbesserung.ergebnis", {
            erreicht: formatProzent(x.result),
            erwartet: formatProzent(x.expected),
          })}
        </p>
      )}
      {x.pending === "ansatz" && (
        <fieldset>
          <legend>{t("verbesserung.ansaetze")}</legend>
          {x.approaches.map((a, i) => (
            <label key={i} className="zeile">
              <input
                type="radio"
                name={`ansatz-${x.id}`}
                checked={ansatz === i}
                onChange={() => setAnsatz(i)}
              />{" "}
              {t("verbesserung.ansatz", {
                nr: i + 1,
                gewinn: formatProzent(a.gain),
                kosten: formatProzent(a.cost),
              })}
            </label>
          ))}
          <div className="knopfreihe links">
            <button
              type="button"
              className="haupt"
              onClick={() => void los([{ ChooseApproach: { project: x.id, approach: ansatz } }])}
            >
              {t("verbesserung.waehlen")}
            </button>
          </div>
        </fieldset>
      )}
      {x.pending === "freigabe" && (
        <div className="knopfreihe links" role="group" aria-label={t("verbesserung.freigabe")}>
          <button
            type="button"
            className="haupt"
            onClick={() => void los([{ ReleaseImprovement: { project: x.id, immediate: false } }])}
          >
            {t("verbesserung.gestaffelt")}
          </button>
          <button
            type="button"
            onClick={() => void los([{ ReleaseImprovement: { project: x.id, immediate: true } }])}
          >
            {t("verbesserung.sofort")}
          </button>
        </div>
      )}
      {x.deadline && (
        <p className="feld-hilfe">
          {t("projekte.entscheidung_frist", { datum: formatDatum(x.deadline) })}
        </p>
      )}
      <div className="knopfreihe links">
        <button
          type="button"
          onClick={() =>
            void los(
              [{ AbortImprovement: { project: x.id } }],
              t("verbesserung.abgebrochen_ok", { produkt: name }),
            )
          }
        >
          {t("projekte.abbrechen")}
        </button>
      </div>
      <Rueckmeldung meldung={antwort} />
    </article>
  );
}

/** Product suggestions of the strategy: start as a project or decline (PD6). */
function Vorschlaege({ daten }: { daten: Forschung }) {
  const p = daten.projects;
  const { los, antwort } = useAktion("projekt/vorschlag");
  const leiter = (p?.leads ?? []).filter((l) => l.hired && !l.busy);
  const [leiterId, setLeiterId] = useState(leiter[0]?.id ?? -1);
  const id = useId();
  if (!p) return null;
  const offen = p.suggestions.filter((x) => x.status === "offen");
  const alt = p.suggestions.filter((x) => x.status !== "offen");
  const bereit = leiter.some((l) => l.id === leiterId);
  return (
    <section aria-label={t("vorschlag.titel")}>
      <h3>{t("vorschlag.titel")}</h3>
      <p className="erklaerung">{t("vorschlag.erklaerung")}</p>
      {p.suggestion_hit_rate !== null && (
        <p className="feld-hilfe">
          {t("vorschlag.trefferquote", { anteil: formatProzent(p.suggestion_hit_rate) })}
        </p>
      )}
      {offen.length === 0 ? (
        <p className="gedaempft">{t("vorschlag.keine")}</p>
      ) : (
        <>
          <div className="feld">
            <label htmlFor={`${id}-leiter`}>{t("projekte.leiter")}</label>
            <select
              id={`${id}-leiter`}
              value={leiterId}
              onChange={(e) => setLeiterId(Number(e.target.value))}
            >
              {leiter.map((l) => (
                <option key={l.id} value={l.id}>
                  {l.name}
                </option>
              ))}
            </select>
          </div>
          {offen.map((v) => (
            <article key={v.id} className="karte" aria-label={produktName(v.product)}>
              <h4>{produktName(v.product)}</h4>
              <p>{t(`vorschlag.begruendung.${v.reason}`)}</p>
              <dl className="werte">
                <dt>{t("projekte.kosten")}</dt>
                <dd>{formatGeld(v.forecast.project_cost)}</dd>
                <dt>{t("projekte.restdauer")}</dt>
                <dd>
                  {t("projekte.tage", { tage: formatZahl(Math.ceil(v.forecast.project_days)) })}
                </dd>
                <dt>{t("projekte.kz.contribution_year")}</dt>
                <dd>{formatGeld(v.forecast.contribution_year)}</dd>
                <dt>{t("projekte.kz.payback_years")}</dt>
                <dd>
                  {t("projekte.jahre", {
                    n: formatZahl(Math.round(v.forecast.payback_years * 10) / 10),
                  })}
                </dd>
                <dt>{t("vorschlag.gueltig_bis")}</dt>
                <dd>{formatDatum(v.expires)}</dd>
              </dl>
              <div className="knopfreihe links">
                <button
                  type="button"
                  className="haupt"
                  disabled={!bereit}
                  onClick={() =>
                    void los(
                      [
                        {
                          StartSuggestedProject: {
                            suggestion: v.id,
                            lead: leiterId,
                            tempo: 1,
                            trim: 1,
                          },
                        },
                      ],
                      t("projekte.gestartet", { produkt: produktName(v.product) }),
                    )
                  }
                >
                  {t("vorschlag.starten")}
                </button>
                <button
                  type="button"
                  onClick={() => void los([{ DeclineSuggestion: { suggestion: v.id } }])}
                >
                  {t("vorschlag.ablehnen")}
                </button>
              </div>
            </article>
          ))}
        </>
      )}
      {alt.length > 0 && (
        <ul className="liste">
          {alt.map((v) => (
            <li key={v.id}>
              {t("vorschlag.zeile", {
                produkt: produktName(v.product),
                status: t(`vorschlag.status.${v.status}`),
                datum: formatDatum(v.created),
              })}
              {v.review &&
                ` · ${t("vorschlag.nachkalkulation", {
                  wert: formatProzent(v.review[0]),
                  urteil: t(v.review[1] ? "vorschlag.treffer" : "vorschlag.fehlschlag"),
                })}`}
            </li>
          ))}
        </ul>
      )}
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

/** Know-how of released projects of other companies for sale, and what aborted ones left (PD8). */
function Lizenzen({ daten }: { daten: Forschung }) {
  const p = daten.projects;
  const { los, antwort } = useAktion("projekt/lizenz");
  const labore = daten.centers.filter((z) => z.ready);
  const leiter = (p?.leads ?? []).filter((l) => l.hired && !l.busy);
  const [zentrum, setZentrum] = useState(labore[0]?.site ?? -1);
  const [leiterId, setLeiterId] = useState(leiter[0]?.id ?? -1);
  const id = useId();
  if (!p) return null;
  const bereit = labore.some((z) => z.site === zentrum) && leiter.some((l) => l.id === leiterId);
  return (
    <section aria-label={t("lizenz.titel")}>
      <h3>{t("lizenz.titel")}</h3>
      <p className="erklaerung">{t("lizenz.erklaerung")}</p>
      {p.knowledge.length > 0 && (
        <ul className="liste">
          {p.knowledge.map(([produkt, anteil]) => (
            <li key={produkt}>
              {t("lizenz.teilwissen", {
                produkt: produktName(produkt),
                anteil: formatProzent(anteil),
              })}
            </li>
          ))}
        </ul>
      )}
      {p.license_offers.length > 0 && (
        <>
          <div className="feld">
            <label htmlFor={`${id}-zentrum`}>{t("forschung.zentrum")}</label>
            <select
              id={`${id}-zentrum`}
              value={zentrum}
              onChange={(e) => setZentrum(Number(e.target.value))}
            >
              {labore.map((z) => (
                <option key={z.site} value={z.site}>
                  {zentrumName(z)}
                </option>
              ))}
            </select>
          </div>
          <div className="feld">
            <label htmlFor={`${id}-leiter`}>{t("projekte.leiter")}</label>
            <select
              id={`${id}-leiter`}
              value={leiterId}
              onChange={(e) => setLeiterId(Number(e.target.value))}
            >
              {leiter.map((l) => (
                <option key={l.id} value={l.id}>
                  {l.name}
                </option>
              ))}
            </select>
          </div>
          {p.license_offers.map((o) => (
            <article
              key={o.project}
              className="karte"
              aria-label={t("lizenz.karte", { produkt: produktName(o.product) })}
            >
              <h4>{produktName(o.product)}</h4>
              <dl className="werte">
                <dt>{t("lizenz.verkaeufer")}</dt>
                <dd>{o.seller}</dd>
                <dt>{t("lizenz.preis")}</dt>
                <dd>{formatGeld(o.price_usd)}</dd>
                <dt>{t("lizenz.gebuehren")}</dt>
                <dd>
                  {t("lizenz.gebuehren_werte", {
                    summe: formatGeld(o.fee_usd),
                    jahre: o.fee_years,
                  })}
                </dd>
                <dt>{t("lizenz.start")}</dt>
                <dd>{t("lizenz.start_ab", { meilenstein: o.from_milestone })}</dd>
              </dl>
              <div className="knopfreihe links">
                <button
                  type="button"
                  className="haupt"
                  disabled={!bereit}
                  onClick={() =>
                    void los(
                      [
                        {
                          BuyProjectLicense: {
                            seller_project: o.project,
                            site: zentrum,
                            lead: leiterId,
                            tempo: 1,
                            trim: 1,
                          },
                        },
                      ],
                      t("projekte.gestartet", { produkt: produktName(o.product) }),
                    )
                  }
                >
                  {t("lizenz.kaufen")}
                </button>
              </div>
            </article>
          ))}
        </>
      )}
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

const LABORROLLEN = ["produktmanager", "entwickler", "fertigungsplaner", "laborpersonal"];

/** Who plans the staff of each laboratory, with what budget, and the frame they work in (PD11). */
function Entwicklungsleitung({ daten }: { daten: Forschung }) {
  const p = daten.projects;
  const { los, antwort } = useAktion("projekt/leitung");
  if (!p) return null;
  const setzen = (z: Leitungszeile, aenderung: { own?: boolean; dismiss?: boolean }) =>
    void los(
      [
        {
          SetDevFrame: {
            unit: einheitAus(z.unit),
            own: aenderung.own ?? z.own,
            dismiss: aenderung.dismiss ?? z.dismiss,
            reserve: null,
            branches: [],
          },
        },
      ],
      t("leitung_entw.gesetzt"),
    );
  return (
    <section aria-label={t("leitung_entw.titel")}>
      <h3>{t("leitung_entw.titel")}</h3>
      <p className="erklaerung">{t("leitung_entw.erklaerung")}</p>
      <div className="tabelle">
        <table>
          <thead>
            <tr>
              <th>{t("forschung.zentrum")}</th>
              <th>{t("leitung_entw.leitung")}</th>
              {LABORROLLEN.map((r) => (
                <th key={r}>{t(`labor.rolle.${r}`)}</th>
              ))}
              <th>{t("leitung_entw.budget")}</th>
              <th>{t("leitung_entw.verzug")}</th>
              <th>{t("leitung_entw.rahmen")}</th>
            </tr>
          </thead>
          <tbody>
            {p.leadership.map((z) => {
              const zentrum = daten.centers.find((c) => c.site === z.site);
              return (
                <tr key={z.site}>
                  <td>{zentrum ? zentrumName(zentrum) : z.site}</td>
                  <td>
                    {t(`leitung_entw.ebene.${z.level}`)} · {z.manager}
                  </td>
                  {LABORROLLEN.map((r, i) => (
                    <td key={r}>
                      {t("leitung_entw.ist_ziel", {
                        ist: z.have[i] ?? 0,
                        unterwegs: z.coming[i] ?? 0,
                        ziel: z.target[i] ?? 0,
                      })}
                    </td>
                  ))}
                  <td>
                    {t("leitung_entw.budget_werte", {
                      kosten: formatGeld(z.cost_usd),
                      budget: formatGeld(z.budget_usd),
                    })}
                  </td>
                  <td>
                    {formatZahl(z.delay)}
                    {z.severance_usd > 0 &&
                      ` · ${t("leitung_entw.abfindung", { betrag: formatGeld(z.severance_usd) })}`}
                  </td>
                  <td>
                    <label>
                      <input
                        type="checkbox"
                        checked={z.own}
                        onChange={(e) => setzen(z, { own: e.target.checked })}
                      />{" "}
                      {t("leitung_entw.selbst")}
                    </label>
                    <label>
                      <input
                        type="checkbox"
                        checked={z.dismiss}
                        onChange={(e) => setzen(z, { dismiss: e.target.checked })}
                      />{" "}
                      {t("leitung_entw.entlassen")}
                    </label>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

/** All running projects at a glance: phase, traffic light, budget, people, subsidy (PD9). */
function Portfolio({ daten }: { daten: Forschung }) {
  const p = daten.projects;
  if (!p) return null;
  const rang = { gruen: 0, gelb: 1, rot: 2 } as Record<string, number>;
  const ampel = (x: Projekt): string => {
    const letzter = x.reports[x.reports.length - 1];
    if (!letzter) return "gruen";
    return letzter.lights.reduce((a, b) => ((rang[b] ?? 0) > (rang[a] ?? 0) ? b : a), "gruen");
  };
  const neu = p.projects.filter((x) => x.status === "laufend");
  const weiter = p.improvements.filter((x) => x.status === "laufend");
  return (
    <section aria-label={t("portfolio.titel")}>
      <h3>{t("portfolio.titel")}</h3>
      <div className="tabelle">
        <table className="mobil-karten" aria-label={t("portfolio.titel")}>
          <thead>
            <tr>
              <th>{t("portfolio.produkt")}</th>
              <th>{t("portfolio.art")}</th>
              <th>{t("portfolio.phase")}</th>
              <th>{t("portfolio.ampel")}</th>
              <th>{t("portfolio.budget")}</th>
              <th>{t("portfolio.personen")}</th>
              <th>{t("portfolio.zuschuss")}</th>
            </tr>
          </thead>
          <tbody>
            {neu.map((x) => (
              <tr key={`n${x.id}`}>
                <th scope="row">{produktName(x.product)}</th>
                <td>{t("portfolio.neu")}</td>
                <td>
                  {t("portfolio.meilenstein", {
                    erreicht: x.milestone,
                    naechster: x.next_milestone ?? 7,
                  })}
                </td>
                <td>
                  <span className={`ampel ampel-${ampel(x)}`}>
                    {t(`projekte.ampel.${ampel(x)}`)}
                  </span>
                </td>
                <td>
                  {formatGeld(x.spent_usd)} / {formatGeld(x.budget_usd)}
                </td>
                <td>{x.roles.reduce((s, r) => s + r.assigned, 0)}</td>
                <td>{x.subsidy ? formatGeld(x.subsidy.received_usd) : "–"}</td>
              </tr>
            ))}
            {weiter.map((x) => (
              <tr key={`w${x.id}`}>
                <th scope="row">{produktName(x.product)}</th>
                <td>{t("portfolio.weiter")}</td>
                <td>{t("portfolio.stufe", { phase: x.phase + 1 })}</td>
                <td>–</td>
                <td>{formatGeld(x.spent_usd)}</td>
                <td>–</td>
                <td>–</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}
