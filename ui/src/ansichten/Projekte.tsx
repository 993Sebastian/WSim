// Development projects PM0–PM7 (PD2, docs/PRODUKTENTWICKLUNG.md): start a project, hire the
// project leaders, set the people per role and see staffing scenarios. No game logic here:
// every figure comes from the view.
import { useId, useState } from "react";
import { formatDatum, formatGeld, formatProzent, formatZahl, landName } from "../format";
import type { Forschung, LaborRolle, Projekt, Projektleiter } from "../kern";
import { t } from "../texte";
import { Rueckmeldung, useAktion } from "./gemeinsam";

/** The roles of the laboratory by their key in the views. */
export const ROLLEN: Record<string, LaborRolle> = {
  produktmanager: "ProductManager",
  entwickler: "Developer",
  fertigungsplaner: "ProductionPlanner",
  laborpersonal: "Technician",
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
      <section aria-label={t("projekte.laufend")}>
        <h3>{t("projekte.laufend")}</h3>
        {laufend.length === 0 && <p className="gedaempft">{t("projekte.keine")}</p>}
        {laufend.map((x) => (
          <ProjektKarte key={x.id} x={x} />
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
      <td>{t("projekte.bilanz", { fertig: l.finished, abgebrochen: l.aborted })}</td>
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

function ProjektKarte({ x }: { x: Projekt }) {
  const { los, antwort } = useAktion(`projekt/${x.id}`);
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
