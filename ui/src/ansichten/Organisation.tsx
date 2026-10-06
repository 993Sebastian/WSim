// Organisation (MA1, docs/BEDIENUNG.md): the positions of the player's sites and who holds
// them; empty positions are filled from the market of managers. "Who runs what, and what
// is still left to me?"
import { Fragment, useId, useState } from "react";
import { formatDatum, formatGeld, formatZahl, landName } from "../format";
import type {
  Kandidat,
  Kern,
  Manager,
  StandortOrganisation,
  Stelle,
  Stellenangabe,
  Uebersicht,
} from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { Befehle, Erklaerung, Rueckmeldung, useAktion, useBefehl, useSicht } from "./gemeinsam";

/** Where the answers of this view appear: above the chart, also after hiring. */
const ORT = "organisation";

/** "Werksleitung" for the head of a works, else the function ("Produktion"). */
export function stellenName(rolle: string, artText: string): string {
  if (rolle !== "leitung") return t(`bereich.${rolle}`);
  return t(`leitung.${artText.replace("standorttyp.", "")}`);
}

/** The position as the core reads it. */
export function stellenangabe(site: number, rolle: string): Stellenangabe {
  return { site, role: rolle === "leitung" ? "Head" : { Specialist: rolle } };
}

/** "Fachkompetenz Produktion", "Erkennen" … */
export function faehigkeitName(key: string): string {
  return key.startsWith("fach.")
    ? t("faehigkeit.fach", { bereich: t(`bereich.${key.slice(5)}`) })
    : t(`faehigkeit.${key}`);
}

/** The level of a skill in words; risk and talkativeness have their own scale. */
export function stufeText(key: string, stufe: number): string {
  if (key === "risiko" || key === "fragefreude") return t(`${key}.${stufe}`);
  return t(`stufe.${stufe}`);
}

function stufe(m: Manager, key: string): string {
  const f = m.skills.find((s) => s.key === key);
  return f ? stufeText(key, f.level) : t("organisation.nichts");
}

/** The expertise that counts on a position: its function's, for a head the focus. */
function fachDerStelle(m: Manager, rolle: string): string {
  return stufe(m, `fach.${rolle === "leitung" ? m.focus : rolle}`);
}

/** All skills of a manager behind an ⓘ. */
function AlleFaehigkeiten({ m }: { m: Manager }) {
  return (
    <Erklaerung wert={t("organisation.faehigkeiten")}>
      <p>{t("organisation.einschaetzung")}</p>
      <dl className="werte">
        {m.skills.map((s) => (
          <Fragment key={s.key}>
            <dt>{faehigkeitName(s.key)}</dt>
            <dd>{stufeText(s.key, s.level)}</dd>
          </Fragment>
        ))}
      </dl>
    </Erklaerung>
  );
}

function Themen({ themen }: { themen: string[] }) {
  // Personnel and logistics get their topics with later stages.
  if (themen.length === 0) return <em>{t("organisation.keine_aufgaben")}</em>;
  return <>{themen.map((k) => t(`thema.${k}`)).join(", ")}</>;
}

/** A filled position: the manager, and a dismissal with its cost after a question. */
function Inhaber({ s, stelle }: { s: StandortOrganisation; stelle: Stelle }) {
  const h = stelle.holder!;
  const { los } = useAktion(ORT);
  const [frage, setFrage] = useState(false);
  const id = useId();
  const name = h.manager.name;
  return (
    <>
      <td data-spalte={t("organisation.inhaber")}>
        <strong>{name}</strong> <AlleFaehigkeiten m={h.manager} />
        <br />
        <small>
          {t("organisation.schwerpunkt", { bereich: t(`bereich.${h.manager.focus}`) })} ·{" "}
          {faehigkeitName(`fach.${stelle.role === "leitung" ? h.manager.focus : stelle.role}`)}:{" "}
          {fachDerStelle(h.manager, stelle.role)} · {t("faehigkeit.erkennen")}:{" "}
          {stufe(h.manager, "erkennen")} · {t("organisation.seit", { datum: formatDatum(h.since) })}
        </small>
      </td>
      <td data-spalte={t("organisation.erledigt")}>
        <Themen themen={stelle.topics} />
      </td>
      <td className="zahl" data-spalte={t("organisation.gehalt")}>
        {formatGeld(h.salary_usd)}
      </td>
      <td>
        {frage ? (
          <div className="bestaetigung" role="group" aria-labelledby={`${id}-frage`}>
            <p id={`${id}-frage`}>
              {t("organisation.entlassen_frage", {
                name,
                abfindung: formatGeld(h.severance_usd),
              })}
            </p>
            <div className="knopfreihe links">
              <button
                type="button"
                className="gefahr"
                onClick={() => {
                  setFrage(false);
                  void los(
                    [{ DismissManager: { manager: h.manager.id } }],
                    t("organisation.entlassen_erfolg", { name }),
                  );
                }}
              >
                {t("organisation.entlassen_ja")}
              </button>
              <button type="button" className="schlicht" onClick={() => setFrage(false)}>
                {t("werk.abbrechen")}
              </button>
            </div>
          </div>
        ) : (
          <button
            type="button"
            className="schlicht"
            aria-label={`${t("organisation.entlassen")}: ${name}, ${stellenName(stelle.role, s.kind_text)}`}
            onClick={() => setFrage(true)}
          >
            {t("organisation.entlassen")}
          </button>
        )}
      </td>
    </>
  );
}

function StandortKarte({
  s,
  onBesetzen,
}: {
  s: StandortOrganisation;
  onBesetzen: (site: number, rolle: string) => void;
}) {
  const titel = `${t(s.kind_text)} · ${landName(s.country)}`;
  return (
    <article className="karte" aria-label={titel}>
      <h4>{titel}</h4>
      {s.next_check && (
        <p className="feld-hilfe">
          {t("organisation.naechste_pruefung", { datum: formatDatum(s.next_check) })}
        </p>
      )}
      <div className="tabelle">
        <table className="mobil-karten" aria-label={t("organisation.stelle")}>
          <thead>
            <tr>
              <th>{t("organisation.stelle")}</th>
              <th>{t("organisation.inhaber")}</th>
              <th>{t("organisation.erledigt")}</th>
              <th className="zahl">{t("organisation.gehalt")}</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {s.positions.map((p) => (
              <tr key={p.role}>
                <td>{stellenName(p.role, s.kind_text)}</td>
                {p.holder ? (
                  <Inhaber s={s} stelle={p} />
                ) : (
                  <>
                    <td data-spalte={t("organisation.inhaber")}>
                      <em>{t("organisation.frei")}</em>
                    </td>
                    <td data-spalte={t("organisation.erledigt")}>
                      <Themen themen={p.topics} />
                    </td>
                    <td className="zahl" data-spalte={t("organisation.gehalt")}>
                      {t("organisation.nichts")}
                    </td>
                    <td>
                      {/* Without topics yet (personnel, logistics …) a manager would
                          only cost his salary. */}
                      {p.topics.length > 0 && (
                        <button
                          type="button"
                          aria-label={`${t("organisation.besetzen")} ${stellenName(p.role, s.kind_text)} (${titel})`}
                          onClick={() => onBesetzen(s.site, p.role)}
                        >
                          {t("organisation.besetzen")}
                        </button>
                      )}
                    </td>
                  </>
                )}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {s.own_topics.length > 0 && (
        <p className="feld-hilfe">
          {t("organisation.selbst", {
            themen: s.own_topics.map((k) => t(`thema.${k}`)).join(", "),
          })}
        </p>
      )}
    </article>
  );
}

function Organigramm({
  kern,
  stand,
  onBesetzen,
}: {
  kern: Kern;
  stand: string;
  onBesetzen: (site: number, rolle: string) => void;
}) {
  const { daten, fehler } = useSicht(() => kern.organisation(), stand);
  const { antwort } = useAktion(ORT);
  if (!daten) return <FehlerText fehler={fehler} />;
  if (!daten.enabled) return <p>{t("organisation.aus")}</p>;
  return (
    <>
      <Rueckmeldung meldung={antwort} />
      <p>
        {t("organisation.kopf", {
          anzahl: formatZahl(daten.managers),
          gehalt: formatGeld(daten.salaries_usd),
          bewerber: formatZahl(daten.candidates),
        })}{" "}
        <Erklaerung wert={t("ansicht.organisation")}>
          <p>{t("organisation.hilfe", { tage: formatZahl(daten.check_days) })}</p>
          <p>{t("organisation.spaeter")}</p>
        </Erklaerung>
      </p>
      {daten.continents.length === 0 && <p>{t("organisation.keine_standorte")}</p>}
      {daten.continents.map((k) => (
        <section key={k.continent} aria-label={t(`kontinent.${k.continent}`)}>
          <h2>{t(`kontinent.${k.continent}`)}</h2>
          {k.countries.map((l) => (
            <section key={l.country} aria-label={landName(l.country)}>
              <h3>{landName(l.country)}</h3>
              {l.sites.map((s) => (
                <StandortKarte key={s.site} s={s} onBesetzen={onBesetzen} />
              ))}
            </section>
          ))}
        </section>
      ))}
    </>
  );
}

/** One row of the market: a candidate or an own manager, with the button to take him. */
function KandidatZeile({
  k,
  rolle,
  aktion,
  onWahl,
}: {
  k: Kandidat;
  rolle: string;
  aktion: string;
  onWahl: () => void;
}) {
  const m = k.manager;
  return (
    <tr>
      <td>
        <strong>{m.name}</strong> <AlleFaehigkeiten m={m} />
      </td>
      <td data-spalte={t("organisation.heimat")}>{landName(m.home)}</td>
      <td data-spalte={t("organisation.schwerpunkt_kurz")}>{t(`bereich.${m.focus}`)}</td>
      <td data-spalte={t("organisation.fach_der_stelle")}>{fachDerStelle(m, rolle)}</td>
      <td data-spalte={t("faehigkeit.erkennen")}>{stufe(m, "erkennen")}</td>
      <td data-spalte={t("faehigkeit.urteil")}>{stufe(m, "urteil")}</td>
      {rolle === "leitung" && (
        <td data-spalte={t("faehigkeit.fuehrung")}>{stufe(m, "fuehrung")}</td>
      )}
      <td className="zahl" data-spalte={t("organisation.forderung")}>
        {formatGeld(k.demand_usd)}
      </td>
      <td>
        <button type="button" aria-label={`${aktion}: ${m.name}`} onClick={onWahl}>
          {aktion}
        </button>
      </td>
    </tr>
  );
}

function Managermarkt({
  kern,
  stand,
  site,
  rolle,
  onZurueck,
}: {
  kern: Kern;
  stand: string;
  site: number;
  rolle: string;
  onZurueck: () => void;
}) {
  const { daten, fehler } = useSicht(() => kern.managermarkt(site, rolle), stand);
  const { los } = useAktion(ORT);
  const [kontinent, setKontinent] = useState<string | null>(null);
  const auswahlId = useId();
  if (!daten) return <FehlerText fehler={fehler} />;
  const stelle = stellenName(rolle, daten.kind_text);
  const standort = `${t(daten.kind_text)} · ${landName(daten.country)}`;
  const gewaehlt = kontinent ?? daten.continent;
  const kontinente = [...new Set(daten.candidates.map((k) => k.manager.continent))];
  const liste = daten.candidates.filter(
    (k) => gewaehlt === "alle" || k.manager.continent === gewaehlt,
  );
  const nehmen = async (k: Kandidat) => {
    const position = stellenangabe(site, rolle);
    const manager = k.manager.id;
    const ok = await los(
      [k.current ? { MoveManager: { manager, position } } : { HireManager: { manager, position } }],
      t(k.current ? "organisation.versetzt" : "organisation.eingestellt", {
        name: k.manager.name,
        stelle,
        standort,
      }),
    );
    if (ok) onZurueck();
  };
  const kopf = (
    <tr>
      <th>{t("organisation.name")}</th>
      <th>{t("organisation.heimat")}</th>
      <th>{t("organisation.schwerpunkt_kurz")}</th>
      <th>{t("organisation.fach_der_stelle")}</th>
      <th>{t("faehigkeit.erkennen")}</th>
      <th>{t("faehigkeit.urteil")}</th>
      {rolle === "leitung" && <th>{t("faehigkeit.fuehrung")}</th>}
      <th className="zahl">{t("organisation.forderung")}</th>
      <th />
    </tr>
  );
  return (
    <section aria-label={t("organisation.markt_titel", { stelle, standort })}>
      <div className="werk-kopf">
        <button type="button" className="schlicht" onClick={onZurueck}>
          {t("organisation.zurueck")}
        </button>
        <h2>{t("organisation.markt_titel", { stelle, standort })}</h2>
      </div>
      <p className="feld-hilfe">{t("organisation.markt_hilfe")}</p>
      <p className="feld-hilfe">{t("organisation.einschaetzung")}</p>
      <label htmlFor={auswahlId}>{t("organisation.kontinent_wahl")} </label>
      <select id={auswahlId} value={gewaehlt} onChange={(e) => setKontinent(e.target.value)}>
        {[daten.continent, ...kontinente.filter((k) => k !== daten.continent)].map((k) => (
          <option key={k} value={k}>
            {t(`kontinent.${k}`)}
          </option>
        ))}
        <option value="alle">{t("organisation.alle_kontinente")}</option>
      </select>
      <h3>{t("organisation.bewerber")}</h3>
      {liste.length === 0 ? (
        <p>{t("organisation.keine_bewerber")}</p>
      ) : (
        <div className="tabelle">
          <table className="mobil-karten" aria-label={t("organisation.bewerber")}>
            <thead>{kopf}</thead>
            <tbody>
              {liste.map((k) => (
                <KandidatZeile
                  key={k.manager.id}
                  k={k}
                  rolle={rolle}
                  aktion={t("organisation.einstellen")}
                  onWahl={() => void nehmen(k)}
                />
              ))}
            </tbody>
          </table>
        </div>
      )}
      {daten.own.length > 0 && (
        <>
          <h3>{t("organisation.eigene")}</h3>
          <div className="tabelle">
            <table className="mobil-karten" aria-label={t("organisation.eigene")}>
              <thead>{kopf}</thead>
              <tbody>
                {daten.own.map((k) => (
                  <KandidatZeile
                    key={k.manager.id}
                    k={k}
                    rolle={rolle}
                    aktion={t("organisation.versetzen")}
                    onWahl={() => void nehmen(k)}
                  />
                ))}
              </tbody>
            </table>
          </div>
        </>
      )}
    </section>
  );
}

export function OrganisationAnsicht({
  kern,
  uebersicht,
  onGeaendert,
}: {
  kern: Kern;
  uebersicht: Uebersicht;
  onGeaendert: (u: Uebersicht) => void;
}) {
  // Every command reloads the chart; the overview keeps cash and counters.
  const [zaehler, setZaehler] = useState(0);
  const { senden, meldung } = useBefehl(kern, onGeaendert, () => setZaehler((z) => z + 1));
  const [wahl, setWahl] = useState<{ site: number; rolle: string } | null>(null);
  const stand = `${uebersicht.date}/${zaehler}`;
  return (
    <main className="ansicht" id="organisation">
      <h1 className="unsichtbar">{t("ansicht.organisation")}</h1>
      <Befehle senden={senden} meldung={meldung}>
        {wahl === null ? (
          <Organigramm
            kern={kern}
            stand={stand}
            onBesetzen={(site, rolle) => setWahl({ site, rolle })}
          />
        ) : (
          <Managermarkt
            kern={kern}
            stand={stand}
            site={wahl.site}
            rolle={wahl.rolle}
            onZurueck={() => setWahl(null)}
          />
        )}
      </Befehle>
    </main>
  );
}
