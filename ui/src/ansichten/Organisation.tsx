// Organisation (MA1, MA2, docs/BEDIENUNG.md): the positions of the player's sites and who
// holds them, what each may spend, and what they ask. Empty positions are filled from the
// market of managers. "Who runs what, and what is still left to me?"
import { Fragment, useId, useState } from "react";
import {
  ausAnzeige,
  formatDatum,
  formatGeld,
  formatProzent,
  formatZahl,
  geldEinheit,
  geldFeld,
  geldSchluessel,
  landName,
  stadtName,
  zahlFeld,
  zahlLesen,
} from "../format";
import { geld } from "../kern/befehle";
import type {
  Abteilung,
  Budgetvorgabe,
  EhemaligerManager,
  EinheitOrganisation,
  Kandidat,
  Kern,
  Manager,
  Organisation,
  Stadt,
  Stelle,
  Stellentyp,
  Uebersicht,
  Zentrale,
} from "../kern";
import { t } from "../texte";
import { TochterfirmenAnsicht } from "./Tochterfirmen";
import { AnliegenListe } from "./Anliegen";
import { ChronikAnsicht } from "./Chronik";
import { FehlerText } from "./Dialog";
import {
  Befehle,
  Erklaerung,
  Rueckmeldung,
  Unterreiter,
  useAktion,
  useBefehl,
  useSicht,
  ZahlEingabe,
  ZahlFeld,
} from "./gemeinsam";
import { einheitName, stellenangabe, stellenName } from "./stellen";
import { RuecksprachenAnsicht } from "./Ruecksprache";
import { StrategieAnsicht } from "./Strategie";

/** Where the answers of this view appear: above the chart, also after hiring. */
const ORT = "organisation";

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

/** The best industry of a manager, in words (MG5); nothing without one. */
function Branchenmarke({ m }: { m: Manager }) {
  const b = (m.branches ?? [])[0];
  if (!b) return null;
  return (
    <small className="feld-hilfe">
      {" "}
      {t("organisation.branche_kurz", {
        branche: t(`branche.${b.key}`),
        stufe: stufeText("branche", b.level),
      })}
    </small>
  );
}

/** Level of a manager in an industry; 0 without. */
function brancheStufe(m: Manager, branche: string | null | undefined): number {
  return (m.branches ?? []).find((b) => b.key === branche)?.level ?? 0;
}

/** All skills of a manager behind an ⓘ. */
export function AlleFaehigkeiten({ m }: { m: Manager }) {
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
        {(m.branches ?? []).map((b) => (
          <Fragment key={`branche.${b.key}`}>
            <dt>{t("faehigkeit.branche", { branche: t(`branche.${b.key}`) })}</dt>
            <dd>{stufeText("branche", b.level)}</dd>
          </Fragment>
        ))}
        <dt>{t("organisation.trefferquote")}</dt>
        <dd>{trefferquote(m.hit_rate, m.judged)}</dd>
        {m.style && (
          <>
            <dt>{t("stil.titel")}</dt>
            <dd title={t(`stil.${m.style}_hilfe`)}>{t(`stil.${m.style}`)}</dd>
          </>
        )}
        {(m.reasons?.length ?? 0) > 0 && (
          <>
            <dt>{t("stil.lieblingsgruende")}</dt>
            <dd>{m.reasons?.map((r) => t(`begruendung.${r}`)).join(", ")}</dd>
          </>
        )}
      </dl>
    </Erklaerung>
  );
}

/** A child of the person (PE2). */
function Familienmarke({ m }: { m: Manager }) {
  if (!m.family) return null;
  return (
    <span className="marke" title={t("organisation.familie_hilfe")}>
      {t("organisation.familie")}
    </span>
  );
}

/** Age and planned retirement of a manager (PE1); nothing while managers do not age. */
function Lebenslauf({ m }: { m: Manager }) {
  if (m.age == null) return null;
  return (
    <>
      <br />
      <small>
        {t("organisation.alter", { jahre: formatZahl(m.age) })}
        {m.retirement != null &&
          m.retires_at != null &&
          ` · ${t("organisation.ruhestand", {
            datum: formatDatum(m.retirement),
            alter: formatZahl(m.retires_at),
          })}`}
      </small>
    </>
  );
}

/** Holders who retire within two years (PE1). */
function Ruhestaende({ liste }: { liste: NonNullable<Organisation["retiring"]> }) {
  if (liste.length === 0) return null;
  return (
    <details className="aufklapper">
      <summary>{t("organisation.ruhestaende", { anzahl: formatZahl(liste.length) })}</summary>
      <table className="tabelle" aria-label={t("organisation.ruhestaende_titel")}>
        <thead>
          <tr>
            <th>{t("organisation.name")}</th>
            <th>{t("organisation.stelle")}</th>
            <th className="zahl">{t("organisation.alter_kurz")}</th>
            <th>{t("organisation.ruhestand_am")}</th>
            <th>{t("organisation.nachfolger_kurz")}</th>
          </tr>
        </thead>
        <tbody>
          {liste.map((r) => (
            <tr key={`${r.name}-${r.position.unit}-${r.position.role}`}>
              <td>{r.name}</td>
              <td data-spalte={t("organisation.stelle")}>
                {stellenName(r.position.role, r.position.kind_text)} · {einheitName(r.position)}
              </td>
              <td className="zahl" data-spalte={t("organisation.alter_kurz")}>
                {r.age == null ? "–" : formatZahl(r.age)}
              </td>
              <td data-spalte={t("organisation.ruhestand_am")}>{formatDatum(r.retirement)}</td>
              <td data-spalte={t("organisation.nachfolger_kurz")}>
                {r.successor ?? t("organisation.nachfolger_offen")}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </details>
  );
}

/** Managers who retired or died in the company's service (PE1). */
function Ehemalige({ liste }: { liste: EhemaligerManager[] }) {
  if (liste.length === 0) return null;
  return (
    <section aria-label={t("organisation.ehemalige")}>
      <h2>{t("organisation.ehemalige")}</h2>
      <table className="tabelle">
        <thead>
          <tr>
            <th>{t("organisation.name")}</th>
            <th>{t("organisation.stelle")}</th>
            <th>{t("organisation.zeit")}</th>
            <th className="zahl">{t("organisation.alter_kurz")}</th>
            <th>{t("organisation.abgang_titel")}</th>
          </tr>
        </thead>
        <tbody>
          {liste.map((e, i) => (
            <tr key={`${e.name}-${e.until}-${i}`}>
              <td>{e.name}</td>
              <td data-spalte={t("organisation.stelle")}>
                {stellenName(e.position.role, e.position.kind_text)} · {einheitName(e.position)}
              </td>
              <td data-spalte={t("organisation.zeit")}>
                {formatDatum(e.since)} – {formatDatum(e.until)}
              </td>
              <td className="zahl" data-spalte={t("organisation.alter_kurz")}>
                {e.age == null ? "–" : formatZahl(e.age)}
              </td>
              <td data-spalte={t("organisation.abgang_titel")}>
                {t(`organisation.abgang.${e.reason}`)}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}

/** "62 % (8 bewertet)", or that nothing was judged yet (ZA3). */
export function trefferquote(rate: number | null, judged: number): string {
  return rate === null
    ? t("organisation.trefferquote_keine")
    : t("organisation.trefferquote_wert", {
        anteil: formatProzent(rate),
        anzahl: formatZahl(judged),
      });
}

function Themen({ themen }: { themen: string[] }) {
  // Personnel and logistics get their topics with later stages.
  if (themen.length === 0) return <em>{t("organisation.keine_aufgaben")}</em>;
  return <>{themen.map((k) => t(`thema.${k}`)).join(", ")}</>;
}

/** A filled position: the manager, and a dismissal with its cost after a question. */
function Inhaber({ s, stelle }: { s: EinheitOrganisation; stelle: Stelle }) {
  const h = stelle.holder!;
  const { los } = useAktion(ORT);
  const [frage, setFrage] = useState(false);
  const id = useId();
  const name = h.manager.name;
  return (
    <>
      <td data-spalte={t("organisation.inhaber")}>
        <strong>{name}</strong> <Familienmarke m={h.manager} />
        <AlleFaehigkeiten m={h.manager} />
        <br />
        <small>
          {t("organisation.schwerpunkt", { bereich: t(`bereich.${h.manager.focus}`) })} ·{" "}
          {faehigkeitName(`fach.${stelle.role === "leitung" ? h.manager.focus : stelle.role}`)}:{" "}
          {fachDerStelle(h.manager, stelle.role)} · {t("faehigkeit.erkennen")}:{" "}
          {stufe(h.manager, "erkennen")} · {t("organisation.seit", { datum: formatDatum(h.since) })}
        </small>
        <Lebenslauf m={h.manager} />
        {h.successor && (
          <>
            <br />
            <small>
              {t("organisation.nachfolger", {
                name: h.successor.manager.name,
                alter: formatZahl(h.successor.manager.age ?? 0),
                datum: formatDatum(h.successor.since),
                gehalt: formatGeld(h.successor.salary_usd),
              })}
            </small>
          </>
        )}
      </td>
      <td data-spalte={t("organisation.erledigt")}>
        <Themen themen={stelle.topics} />
      </td>
      <td className="zahl" data-spalte={t("organisation.gehalt")}>
        {formatGeld(h.salary_usd)}
        <br />
        <small className={`zufriedenheit stufe-${h.satisfaction}`}>
          {t(`organisation.zufriedenheit.${h.satisfaction}`)}
        </small>
        {h.offer && (
          <>
            <br />
            <small className="warnung-text">
              {t("organisation.angebot", {
                firma: h.offer.company,
                gehalt: formatGeld(h.offer.salary_usd),
                datum: formatDatum(h.offer.until),
              })}
            </small>
          </>
        )}
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

/**
 * Pay and satisfaction of a manager (MA6): his market value, a raise, and for heads
 * whether they fill the free positions of their unit themselves.
 */
function GehaltUndZufriedenheit({ s, stelle }: { s: EinheitOrganisation; stelle: Stelle }) {
  const h = stelle.holder!;
  const name = h.manager.name;
  const { los } = useAktion(ORT);
  const [betrag, setBetrag] = useState(geldFeld(Math.max(h.market_usd, h.salary_usd)));
  const [fehler, setFehler] = useState<string | null>(null);
  const position = stellenangabe(s.key, stelle.role);
  return (
    <>
      <h5>{t("organisation.gehalt_titel")}</h5>
      <dl className="werte">
        <dt>{t("organisation.gehalt")}</dt>
        <dd>{formatGeld(h.salary_usd)}</dd>
        <dt>{t("organisation.marktwert")}</dt>
        <dd>{formatGeld(h.market_usd)}</dd>
        <dt>{t("organisation.zufriedenheit_titel")}</dt>
        <dd>{t(`organisation.zufriedenheit.${h.satisfaction}`)}</dd>
      </dl>
      <p className="feld-hilfe">{t("organisation.zufriedenheit_hilfe")}</p>
      <form
        key={geldSchluessel()}
        aria-label={t("organisation.gehalt_anpassen_titel", { name })}
        onSubmit={(e) => {
          e.preventDefault();
          const b = zahlLesen(betrag);
          if (b === null || ausAnzeige(b) <= h.salary_usd) {
            setFehler(t("organisation.gehalt_hoeher", { gehalt: formatGeld(h.salary_usd) }));
            return;
          }
          setFehler(null);
          void los(
            [{ RaiseSalary: { manager: h.manager.id, salary: geld(ausAnzeige(b)) } }],
            t("organisation.gehalt_angepasst", { name, gehalt: formatGeld(ausAnzeige(b)) }),
          );
        }}
      >
        <div className="formular-zeile">
          <ZahlFeld
            name={t("organisation.neues_gehalt")}
            einheit={geldEinheit()}
            wert={betrag}
            onWert={setBetrag}
          />
          <button type="submit">{t("organisation.gehalt_anpassen")}</button>
        </div>
        {fehler && <p className="fehlertext">{fehler}</p>}
      </form>
      {stelle.hires !== undefined && stelle.hires !== null && (
        <label className="schalter">
          <input
            type="checkbox"
            checked={stelle.hires}
            onChange={(e) =>
              void los(
                [{ SetHiringByHead: { position, enabled: e.target.checked } }],
                t(e.target.checked ? "organisation.stellt_ein_an" : "organisation.stellt_ein_aus", {
                  stelle: stellenName(stelle.role, s.kind_text),
                }),
              )
            }
          />{" "}
          {t("organisation.stellt_ein")}
          <span className="feld-hilfe"> {t("organisation.stellt_ein_hilfe")}</span>
        </label>
      )}
    </>
  );
}

/**
 * The headquarters (ZA1): where it is, what that means for taxes and salaries, and a
 * move to another country.
 */
/** Place of a seat: „Stadt, Land“, or only the country without cities (W2). */
function ortName(land: string, stadt: string | null): string {
  return stadt
    ? t("zentrale.ort", { stadt: stadtName(land, stadt), land: landName(land) })
    : landName(land);
}

function Hauptsitz({ z }: { z: Zentrale }) {
  const { los } = useAktion(ORT);
  const [ziel, setZiel] = useState("");
  const [zielStadt, setZielStadt] = useState("");
  const laender = [...z.countries]
    .filter((l) => l.country !== z.country)
    .map((l) => ({ ...l, name: landName(l.country) }))
    .sort((a, b) => a.name.localeCompare(b.name, "de"));
  const gewaehlt = laender.find((l) => l.country === ziel);
  const stadt = gewaehlt ? zielStadt || gewaehlt.default_city || "" : "";
  return (
    <section aria-label={t("zentrale.titel")}>
      <h2>{t("zentrale.titel")}</h2>
      <article className="karte" aria-label={t("zentrale.hauptsitz")}>
        <h4>
          {t("zentrale.hauptsitz")}: {ortName(z.country, z.city)}{" "}
          <Erklaerung wert={t("zentrale.hauptsitz")}>
            <p>{t("zentrale.hilfe")}</p>
          </Erklaerung>
        </h4>
        <dl className="werte">
          <dt>{t("zentrale.steuer")}</dt>
          <dd>{formatProzent(z.tax)}</dd>
          <dt>{t("zentrale.lohn")}</dt>
          <dd>{formatGeld(z.wage_usd)}</dd>
        </dl>
        {z.relocation ? (
          <p className="warnung-text">
            {t("zentrale.umzug", {
              land: ortName(z.relocation.country, z.relocation.city),
              datum: formatDatum(z.relocation.until),
            })}
          </p>
        ) : (
          <form
            aria-label={t("zentrale.verlegen")}
            onSubmit={(e) => {
              e.preventDefault();
              if (!gewaehlt) return;
              void los(
                [
                  {
                    SetHeadquarters: stadt
                      ? { country: gewaehlt.country, city: stadt }
                      : { country: gewaehlt.country },
                  },
                ],
                t("zentrale.verlegt", {
                  land: ortName(gewaehlt.country, stadt || null),
                  monate: z.move_months,
                }),
              ).then((ok) => {
                if (ok) {
                  setZiel("");
                  setZielStadt("");
                }
              });
            }}
          >
            <div className="formular-zeile">
              <label>
                {t("zentrale.ziel")}{" "}
                <select
                  value={ziel}
                  onChange={(e) => {
                    setZiel(e.target.value);
                    setZielStadt("");
                  }}
                >
                  <option value="">{t("zentrale.ziel_waehlen")}</option>
                  {laender.map((l) => (
                    <option key={l.country} value={l.country}>
                      {t("zentrale.ziel_option", {
                        land: l.name,
                        steuer: formatProzent(l.tax),
                        lohn: formatGeld(l.wage_usd),
                      })}
                    </option>
                  ))}
                </select>
              </label>
              {gewaehlt && gewaehlt.cities.length > 0 && (
                <label>
                  {t("zentrale.stadt")}{" "}
                  <select value={stadt} onChange={(e) => setZielStadt(e.target.value)}>
                    {gewaehlt.cities.map((s) => (
                      <option key={s} value={s}>
                        {stadtName(gewaehlt.country, s)}
                      </option>
                    ))}
                  </select>
                </label>
              )}
              <button type="submit" disabled={!gewaehlt}>
                {t("zentrale.verlegen")}
              </button>
            </div>
            <p className="feld-hilfe">
              {t("zentrale.kosten", {
                betrag: formatGeld(z.move_cost_usd),
                monate: z.move_months,
                anteil: formatProzent(z.moving_share),
              })}
            </p>
          </form>
        )}
        {z.cities.length > 0 && <Staedte z={z} />}
      </article>
      <Abteilungen z={z} />
    </section>
  );
}

/**
 * The cities of the country of the headquarters (W2): how many academics the central
 * departments find there, how many all companies want, what offices cost; a move there.
 */
function Staedte({ z }: { z: Zentrale }) {
  const { los } = useAktion(ORT);
  const name = (s: Stadt) => stadtName(z.country, s.key);
  return (
    <>
      <h4>
        {t("zentrale.staedte", { land: landName(z.country) })}{" "}
        <Erklaerung wert={t("zentrale.staedte", { land: landName(z.country) })}>
          <p>{t("zentrale.staedte_hilfe")}</p>
        </Erklaerung>
      </h4>
      <table
        className="mobil-karten"
        aria-label={t("zentrale.staedte", { land: landName(z.country) })}
      >
        <thead>
          <tr>
            <th>{t("zentrale.stadt_name")}</th>
            <th>{t("zentrale.einwohner")}</th>
            <th>{t("zentrale.akademiker")}</th>
            <th>{t("zentrale.gewuenscht")}</th>
            <th>{t("zentrale.buero")}</th>
            <th />
          </tr>
        </thead>
        <tbody>
          {z.cities.map((s) => (
            <tr key={s.key}>
              <td>
                <strong>{name(s)}</strong>
                {s.capital && <small className="feld-hilfe"> · {t("zentrale.hauptstadt")}</small>}
              </td>
              <td data-spalte={t("zentrale.einwohner")}>{formatZahl(s.population)}</td>
              <td data-spalte={t("zentrale.akademiker")}>{formatZahl(Math.floor(s.academics))}</td>
              <td data-spalte={t("zentrale.gewuenscht")}>
                <span className={s.wanted > s.academics ? "warnung-text" : undefined}>
                  {formatZahl(s.wanted)}
                </span>
              </td>
              <td data-spalte={t("zentrale.buero")}>{formatGeld(s.office_per_employee_usd)}</td>
              <td>
                {s.here ? (
                  <strong>{t("zentrale.hier")}</strong>
                ) : (
                  <button
                    type="button"
                    disabled={z.relocation !== null}
                    onClick={() =>
                      void los(
                        [{ SetHeadquarters: { country: z.country, city: s.key } }],
                        t("zentrale.stadt_verlegt", {
                          stadt: name(s),
                          monate: z.city_move_months,
                        }),
                      )
                    }
                  >
                    {t("zentrale.hierher", { stadt: name(s) })}
                  </button>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <p className="feld-hilfe">
        {t("zentrale.stadt_kosten", {
          betrag: formatGeld(z.city_move_cost_usd),
          monate: z.city_move_months,
        })}
      </p>
    </>
  );
}

/** What a department achieves, in words (ZA2). */
function wirkungText(a: Abteilung): string {
  if (!a.working) {
    return a.staff === 0
      ? t("zentrale.keine_angestellten")
      : t("zentrale.ohne_leitung", { bereich: t(`bereich.${a.function}`) });
  }
  return t(`abteilung.wirkung.${a.key}`, {
    anzahl: formatZahl(a.reach),
    anteil: formatProzent(a.effect * a.coverage),
  });
}

/** The central departments: employees, head, coverage, effect and costs (ZA2). */
function Abteilungen({ z }: { z: Zentrale }) {
  if (z.departments.length === 0) return null;
  return (
    <article className="karte" aria-label={t("zentrale.abteilungen")}>
      <h4>
        {t("zentrale.abteilungen")}{" "}
        <Erklaerung wert={t("zentrale.abteilungen")}>
          <p>{t("zentrale.abteilungen_hilfe")}</p>
          <p>{t("zentrale.abteilungen_genauigkeit")}</p>
        </Erklaerung>
      </h4>
      <p className="feld-hilfe">
        {z.employees === 0
          ? t("zentrale.abteilungen_leer")
          : t("zentrale.abteilungen_kosten", {
              anzahl: formatZahl(z.employees),
              betrag: formatGeld(z.monthly_cost_usd),
            })}
      </p>
      <div className="tabelle">
        <table className="mobil-karten" aria-label={t("zentrale.abteilungen")}>
          <thead>
            <tr>
              <th>{t("zentrale.abteilung")}</th>
              <th>{t("zentrale.leitung")}</th>
              <th>{t("zentrale.angestellte")}</th>
              <th>{t("zentrale.abdeckung")}</th>
              <th>{t("zentrale.wirkung")}</th>
              <th>{t("zentrale.kosten_monat")}</th>
            </tr>
          </thead>
          <tbody>
            {z.departments.map((a) => (
              <AbteilungZeile key={a.kind} a={a} />
            ))}
          </tbody>
        </table>
      </div>
    </article>
  );
}

function AbteilungZeile({ a }: { a: Abteilung }) {
  const { los } = useAktion(ORT);
  const [anzahl, setAnzahl] = useState(zahlFeld(a.staff, 0));
  const n = zahlLesen(anzahl);
  const gueltig = n !== null && n >= 0 && Number.isInteger(n);
  const name = t(`abteilung.${a.key}`);
  return (
    <tr>
      <td>
        <strong>{name}</strong>
      </td>
      <td data-spalte={t("zentrale.leitung")}>
        {a.head ? (
          <>
            {a.head}
            {a.head_level !== null && (
              <small className="feld-hilfe">
                {" "}
                · {faehigkeitName(`fach.${a.function}`)}: {stufeText("fach", a.head_level)}
              </small>
            )}
            {a.head_hit_rate !== null && (
              <small className="feld-hilfe">
                {" "}
                · {t("organisation.trefferquote")}: {trefferquote(a.head_hit_rate, a.head_judged)}
              </small>
            )}
          </>
        ) : (
          <span className="warnung-text">
            {t("zentrale.leitung_frei", { bereich: t(`bereich.${a.function}`) })}
          </span>
        )}
        {a.release_limit_usd !== null && (
          <small className="feld-hilfe">
            {" "}
            · {t("zentrale.freigabe_bis", { betrag: formatGeld(a.release_limit_usd) })}
          </small>
        )}
      </td>
      <td data-spalte={t("zentrale.angestellte")}>
        <form
          className="formular-zeile"
          aria-label={t("zentrale.angestellte_von", { abteilung: name })}
          onSubmit={(e) => {
            e.preventDefault();
            if (!gueltig || n === null) return;
            void los(
              [{ StaffDepartment: { department: a.kind, staff: n } }],
              t("zentrale.besetzt", { abteilung: name, anzahl: formatZahl(n) }),
            );
          }}
        >
          <ZahlEingabe
            className="schmal"
            aria-label={t("zentrale.angestellte_von", { abteilung: name })}
            wert={anzahl}
            onWert={setAnzahl}
            ganzzahlig
          />
          <button type="submit" disabled={!gueltig || n === a.staff}>
            {t("zentrale.festlegen")}
          </button>
        </form>
        {a.staffed < a.staff && (
          <small className="warnung-text">
            {t("zentrale.nur_besetzt", { besetzt: formatZahl(a.staffed) })}
          </small>
        )}
      </td>
      <td data-spalte={t("zentrale.abdeckung")}>
        {a.staffed === 0 ? "–" : formatProzent(a.coverage)}
        <small className="feld-hilfe">
          {" "}
          {t("zentrale.faelle", {
            faelle: formatZahl(a.capacity),
            last: formatZahl(a.workload),
          })}
        </small>
      </td>
      <td data-spalte={t("zentrale.wirkung")}>{wirkungText(a)}</td>
      <td data-spalte={t("zentrale.kosten_monat")}>
        {formatGeld(a.monthly_cost_usd)}
        <small className="feld-hilfe">
          {" "}
          {t("zentrale.je_angestelltem", { betrag: formatGeld(a.cost_per_employee_usd) })}
        </small>
      </td>
    </tr>
  );
}

/** "+1.200 $" */
function mitVorzeichen(usd: number): string {
  return usd > 0 ? `+${formatGeld(usd)}` : formatGeld(usd);
}

/** What a filled position may spend, what it decided itself and what it does not ask (MA2). */
function StellenDetails({
  s,
  stelle,
  sockel,
  onAnliegen,
}: {
  s: EinheitOrganisation;
  stelle: Stelle;
  sockel: [number, number];
  onAnliegen: () => void;
}) {
  const b = stelle.budget!;
  const name = stellenName(stelle.role, s.kind_text);
  const position = stellenangabe(s.key, stelle.role);
  const { los } = useAktion(ORT);
  const [je, setJe] = useState(zahlFeld(b.shares[0] * 100, 1));
  const [jahr, setJahr] = useState(zahlFeld(b.shares[1] * 100, 1));
  const [fehler, setFehler] = useState<string | null>(null);
  const offen = stelle.open_concerns ?? 0;
  const log = stelle.log ?? [];
  const still = stelle.quiet ?? [];
  const kurz =
    b.per_decision_usd === 0
      ? t("organisation.budget_immer")
      : t("organisation.budget_kurz", {
          entscheidung: formatGeld(b.per_decision_usd),
          jahr: formatGeld(b.per_year_usd),
        });
  return (
    <details className="stelle-details">
      <summary>
        <strong>{name}</strong> · {kurz}
        {offen > 0 && (
          <span className="zaehler" title={t("organisation.offene_anliegen", { anzahl: offen })}>
            {offen}
          </span>
        )}
      </summary>
      <dl className="werte">
        <dt>{t("organisation.budget_entscheidung")}</dt>
        <dd>
          {formatGeld(b.per_decision_usd)} ({formatProzent(b.shares[0])})
        </dd>
        <dt>{t("organisation.budget_jahr")}</dt>
        <dd>
          {formatGeld(b.per_year_usd)} ({formatProzent(b.shares[1])})
        </dd>
        <dt>{t("organisation.budget_verbraucht")}</dt>
        <dd>{formatGeld(b.spent_usd)}</dd>
        <dt>{t("organisation.budget_basis")}</dt>
        <dd>{formatGeld(b.base_usd)}</dd>
      </dl>
      <p className="feld-hilfe">
        {t("organisation.budget_hilfe", {
          entscheidung: formatZahl(sockel[0], 1),
          jahr: formatZahl(sockel[1], 1),
        })}
      </p>
      <form
        aria-label={t("organisation.budget_titel", { stelle: name })}
        onSubmit={(e) => {
          e.preventDefault();
          const a = zahlLesen(je);
          const j = zahlLesen(jahr);
          if (a === null || j === null || a < 0 || j > 100 || a > j) {
            setFehler(t("organisation.budget_bereich"));
            return;
          }
          setFehler(null);
          void los(
            [{ SetBudget: { position, shares: [a / 100, j / 100] } }],
            t("organisation.budget_gesetzt", { stelle: name }),
          );
        }}
      >
        <div className="formular-zeile">
          <ZahlFeld
            name={t("organisation.budget_je_entscheidung")}
            einheit="%"
            wert={je}
            onWert={setJe}
            gruppieren={false}
          />
          <ZahlFeld
            name={t("organisation.budget_je_jahr")}
            einheit="%"
            wert={jahr}
            onWert={setJahr}
            gruppieren={false}
          />
          <button type="submit">{t("organisation.budget_uebernehmen")}</button>
          {b.custom && (
            <button
              type="button"
              className="schlicht"
              onClick={() => {
                setJe(zahlFeld(b.defaults[0] * 100, 1));
                setJahr(zahlFeld(b.defaults[1] * 100, 1));
                void los(
                  [{ SetBudget: { position, shares: null } }],
                  t("organisation.budget_gesetzt", { stelle: name }),
                );
              }}
            >
              {t("organisation.budget_standard", {
                entscheidung: formatProzent(b.defaults[0]),
                jahr: formatProzent(b.defaults[1]),
              })}
            </button>
          )}
        </div>
        {fehler && <p className="fehlertext">{fehler}</p>}
      </form>
      <GehaltUndZufriedenheit s={s} stelle={stelle} />
      <h5>{t("organisation.entscheidungen")}</h5>
      {log.length === 0 ? (
        <p className="gedaempft">{t("organisation.keine_entscheidungen")}</p>
      ) : (
        <ul className="stellen-protokoll">
          {log.map((l, i) => (
            <li key={i}>
              {formatDatum(l.date)} · {t(`thema.${l.topic}`)}
              {l.product && ` · ${t(`produkt.${l.product}`)}`}: {t(`option.${l.kind}`)}
              {l.amount_usd > 0 && ` · ${formatGeld(l.amount_usd)}`}
              {l.effect_usd !== null &&
                ` · ${t("organisation.erwartet", { wirkung: mitVorzeichen(l.effect_usd) })}`}
            </li>
          ))}
        </ul>
      )}
      {still.length > 0 && (
        <>
          <h5>{t("organisation.stille_themen")}</h5>
          <ul className="stellen-protokoll">
            {still.map((q) => (
              <li key={q.id}>
                {q.until
                  ? t("organisation.still_bis", {
                      thema: t(`thema.${q.topic}`),
                      datum: formatDatum(q.until),
                    })
                  : t("organisation.still_immer", { thema: t(`thema.${q.topic}`) })}{" "}
                <button
                  type="button"
                  className="schlicht"
                  aria-label={`${t("organisation.wieder_fragen")}: ${t(`thema.${q.topic}`)} (${name})`}
                  onClick={() =>
                    void los(
                      [{ AskAgain: { position, topic: q.id } }],
                      t("organisation.wieder_fragen_erfolg", {
                        stelle: name,
                        thema: t(`thema.${q.topic}`),
                      }),
                    )
                  }
                >
                  {t("organisation.wieder_fragen")}
                </button>
              </li>
            ))}
          </ul>
        </>
      )}
      {offen > 0 && (
        <p>
          <button type="button" className="schlicht" onClick={onAnliegen}>
            {t("organisation.zu_den_anliegen", { anzahl: offen })}
          </button>
        </p>
      )}
    </details>
  );
}

function EinheitKarte({
  s,
  sockel,
  onBesetzen,
  onAnliegen,
}: {
  s: EinheitOrganisation;
  sockel: [number, number];
  onBesetzen: (einheit: string, rolle: string, reiter?: MarktReiter) => void;
  onAnliegen: () => void;
}) {
  const titel = einheitName(s);
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
                      {p.effect && <span className="feld-hilfe">{t(p.effect)}</span>}
                    </td>
                    <td className="zahl" data-spalte={t("organisation.gehalt")}>
                      {t("organisation.nichts")}
                    </td>
                    <td>
                      {/* Without topics yet (personnel, logistics …) a manager would
                          only cost his salary, unless the position works otherwise. */}
                      {(p.topics.length > 0 || p.effect) && (
                        <button
                          type="button"
                          aria-label={`${t("organisation.besetzen")} ${stellenName(p.role, s.kind_text)} (${titel})`}
                          onClick={() => onBesetzen(s.key, p.role)}
                        >
                          {t("organisation.besetzen")}
                        </button>
                      )}
                      {(p.topics.length > 0 || p.effect) && (
                        <button
                          type="button"
                          aria-label={`${t("organisation.abwerben_stelle")} ${stellenName(p.role, s.kind_text)} (${titel})`}
                          onClick={() => onBesetzen(s.key, p.role, "abwerben")}
                        >
                          {t("organisation.abwerben_stelle")}
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
      {s.positions
        .filter((p) => p.holder && p.budget)
        .map((p) => (
          <StellenDetails key={p.role} s={s} stelle={p} sockel={sockel} onAnliegen={onAnliegen} />
        ))}
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

/** "Werk: Werksleitung", "Land: Produktion" … */
function typName(k: Stellentyp): string {
  return `${t(k.kind_text)}: ${stellenName(k.role, k.kind_text)}`;
}

function geltungName(v: Pick<Budgetvorgabe, "scope" | "scope_key">): string {
  if (v.scope === "firma") return t("organisation.vorgabe_firma");
  if (v.scope === "kontinent") return t(`kontinent.${v.scope_key ?? ""}`);
  return landName(v.scope_key ?? "");
}

/** The kind of a position as the command reads it. */
function typBefehl(k: Stellentyp) {
  const level =
    k.level === "standort"
      ? { Site: k.site_type ?? "" }
      : k.level === "land"
        ? ("Country" as const)
        : ("Continent" as const);
  return { level, role: k.role === "leitung" ? ("Head" as const) : { Specialist: k.role } };
}

function geltungBefehl(v: Pick<Budgetvorgabe, "scope" | "scope_key">) {
  if (v.scope === "kontinent") return { Continent: v.scope_key ?? "" };
  if (v.scope === "land") return { Country: v.scope_key ?? "" };
  return "Company" as const;
}

/**
 * Budgets for all positions of a type (MA3): "alle Werksleitungen in Europa 3 % / 8 %".
 * A position's own budget comes first, then the rule of its country, its continent, the
 * company.
 */
function Budgetvorgaben({ daten }: { daten: Organisation }) {
  const { los } = useAktion(ORT);
  const typen = daten.kinds ?? [];
  const vorgaben = daten.rules ?? [];
  const [typ, setTyp] = useState(0);
  const [geltung, setGeltung] = useState("firma:");
  const [je, setJe] = useState("3");
  const [jahr, setJahr] = useState("8");
  const [fehler, setFehler] = useState<string | null>(null);
  const id = useId();
  if (typen.length === 0) return null;
  const geltungen = [
    { scope: "firma" as const, scope_key: null },
    ...daten.continents.map((k) => ({ scope: "kontinent" as const, scope_key: k.continent })),
    ...daten.continents.flatMap((k) =>
      k.countries.map((l) => ({ scope: "land" as const, scope_key: l.country })),
    ),
  ];
  const gewaehlt =
    geltungen.find((g) => `${g.scope}:${g.scope_key ?? ""}` === geltung) ?? geltungen[0]!;
  return (
    <details className="budgetvorgaben">
      <summary>
        {t("organisation.vorgaben", { anzahl: vorgaben.length })}{" "}
        <Erklaerung wert={t("organisation.vorgaben_titel")}>
          <p>{t("organisation.vorgaben_hilfe")}</p>
        </Erklaerung>
      </summary>
      {vorgaben.length > 0 && (
        <ul className="stellen-protokoll">
          {vorgaben.map((v, i) => (
            <li key={i}>
              {typName(v.kind)} · {geltungName(v)}: {formatProzent(v.shares[0])} /{" "}
              {formatProzent(v.shares[1])}{" "}
              <button
                type="button"
                className="schlicht"
                aria-label={`${t("organisation.vorgabe_entfernen")}: ${typName(v.kind)}, ${geltungName(v)}`}
                onClick={() =>
                  void los(
                    [
                      {
                        SetBudgetRule: {
                          kind: typBefehl(v.kind),
                          scope: geltungBefehl(v),
                          shares: null,
                        },
                      },
                    ],
                    t("organisation.vorgabe_entfernt"),
                  )
                }
              >
                {t("organisation.vorgabe_entfernen")}
              </button>
            </li>
          ))}
        </ul>
      )}
      <form
        aria-label={t("organisation.vorgaben_titel")}
        onSubmit={(e) => {
          e.preventDefault();
          const a = zahlLesen(je);
          const j = zahlLesen(jahr);
          const k = typen[typ];
          if (!k || a === null || j === null || a < 0 || j > 100 || a > j) {
            setFehler(t("organisation.budget_bereich"));
            return;
          }
          setFehler(null);
          void los(
            [
              {
                SetBudgetRule: {
                  kind: typBefehl(k),
                  scope: geltungBefehl(gewaehlt),
                  shares: [a / 100, j / 100],
                },
              },
            ],
            t("organisation.vorgabe_gesetzt", { typ: typName(k), wo: geltungName(gewaehlt) }),
          );
        }}
      >
        <div className="formular-zeile">
          <div className="feld">
            <label htmlFor={`${id}-typ`}>{t("organisation.vorgabe_typ")}</label>
            <select id={`${id}-typ`} value={typ} onChange={(e) => setTyp(Number(e.target.value))}>
              {typen.map((k, i) => (
                <option key={i} value={i}>
                  {typName(k)}
                </option>
              ))}
            </select>
          </div>
          <div className="feld">
            <label htmlFor={`${id}-wo`}>{t("organisation.vorgabe_wo")}</label>
            <select id={`${id}-wo`} value={geltung} onChange={(e) => setGeltung(e.target.value)}>
              {geltungen.map((g) => (
                <option
                  key={`${g.scope}:${g.scope_key ?? ""}`}
                  value={`${g.scope}:${g.scope_key ?? ""}`}
                >
                  {geltungName(g)}
                </option>
              ))}
            </select>
          </div>
          <ZahlFeld
            name={t("organisation.budget_je_entscheidung")}
            einheit="%"
            wert={je}
            onWert={setJe}
            gruppieren={false}
          />
          <ZahlFeld
            name={t("organisation.budget_je_jahr")}
            einheit="%"
            wert={jahr}
            onWert={setJahr}
            gruppieren={false}
          />
          <button type="submit">{t("organisation.vorgabe_setzen")}</button>
        </div>
        {fehler && <p className="fehlertext">{fehler}</p>}
      </form>
    </details>
  );
}

/** The market is always in reach: it opens at the first free position (MG1). */
function MarktEinstieg({
  daten,
  onBesetzen,
}: {
  daten: Organisation;
  onBesetzen: (einheit: string, rolle: string, reiter?: MarktReiter) => void;
}) {
  const stelle = ersteFreieStelle(daten);
  return (
    <div className="markt-einstieg">
      <button
        type="button"
        disabled={!stelle}
        onClick={() => stelle && onBesetzen(stelle.einheit, stelle.rolle, "abwerben")}
      >
        {t("organisation.managermarkt")}
      </button>
      {!stelle && <small className="feld-hilfe"> {t("organisation.markt_ohne_stelle")}</small>}
    </div>
  );
}

function Organigramm({
  kern,
  stand,
  onBesetzen,
  onAnliegen,
}: {
  kern: Kern;
  stand: string;
  onBesetzen: (einheit: string, rolle: string, reiter?: MarktReiter) => void;
  onAnliegen: () => void;
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
      <MarktEinstieg daten={daten} onBesetzen={onBesetzen} />
      {daten.continents.length === 0 && <p>{t("organisation.keine_standorte")}</p>}
      <Ruhestaende liste={daten.retiring ?? []} />
      <Budgetvorgaben daten={daten} />
      {daten.board && (
        <section aria-label={t("ebene.vorstand")}>
          <h2>{t("ebene.vorstand")}</h2>
          <p className="feld-hilfe">{t("organisation.vorstand_hilfe")}</p>
          <EinheitKarte
            s={daten.board}
            sockel={daten.budget_floor ?? [0, 0]}
            onBesetzen={onBesetzen}
            onAnliegen={onAnliegen}
          />
        </section>
      )}
      {daten.central && <Hauptsitz z={daten.central} />}
      {daten.continents.map((k) => (
        <section key={k.continent} aria-label={t(`kontinent.${k.continent}`)}>
          <h2>{t(`kontinent.${k.continent}`)}</h2>
          {k.unit && (
            <EinheitKarte
              s={k.unit}
              sockel={daten.budget_floor ?? [0, 0]}
              onBesetzen={onBesetzen}
              onAnliegen={onAnliegen}
            />
          )}
          {k.countries.map((l) => (
            <section key={l.country} aria-label={landName(l.country)}>
              <h3>{landName(l.country)}</h3>
              {l.unit && (
                <EinheitKarte
                  s={l.unit}
                  sockel={daten.budget_floor ?? [0, 0]}
                  onBesetzen={onBesetzen}
                  onAnliegen={onAnliegen}
                />
              )}
              {l.sites.map((s) => (
                <EinheitKarte
                  key={s.key}
                  s={s}
                  sockel={daten.budget_floor ?? [0, 0]}
                  onBesetzen={onBesetzen}
                  onAnliegen={onAnliegen}
                />
              ))}
            </section>
          ))}
        </section>
      ))}
      <Ehemalige liste={daten.former ?? []} />
    </>
  );
}

/** The two lists of the market: free candidates and managers of other companies (MG1). */
export type MarktReiter = "frei" | "abwerben";

/** The first free position of the organisation, for the entry "Managermarkt". */
function ersteFreieStelle(daten: Organisation): { einheit: string; rolle: string } | null {
  const einheiten: EinheitOrganisation[] = [
    ...(daten.board ? [daten.board] : []),
    ...daten.continents.flatMap((k) => [
      ...(k.unit ? [k.unit] : []),
      ...k.countries.flatMap((l) => [...(l.unit ? [l.unit] : []), ...l.sites]),
    ]),
  ];
  for (const e of einheiten) {
    const p = e.positions.find((x) => !x.holder && (x.topics.length > 0 || x.effect));
    if (p) return { einheit: e.key, rolle: p.role };
  }
  return null;
}

/** Satisfaction level of a manager of another company as text. */
function zufriedenheitText(stufe: number | null | undefined): string {
  return stufe == null ? "–" : t(`zufriedenheit.${stufe}`);
}

/** One row of managers of other companies: who, where, what he asks, the button to offer. */
function AbwerbeZeile({ k, rolle, onWahl }: { k: Kandidat; rolle: string; onWahl: () => void }) {
  const m = k.manager;
  const gesperrt = k.blocked_until != null;
  return (
    <tr>
      <td>
        <strong>{m.name}</strong> <Familienmarke m={m} />
        <Branchenmarke m={m} />
        <AlleFaehigkeiten m={m} />
      </td>
      <td data-spalte={t("organisation.firma")}>{k.company ?? "–"}</td>
      <td data-spalte={t("organisation.stelle")}>
        {k.current ? stellenName(k.current.role, k.current.unit) : "–"}
      </td>
      <td data-spalte={t("organisation.heimat")}>{landName(m.home)}</td>
      <td data-spalte={t("organisation.schwerpunkt_kurz")}>{t(`bereich.${m.focus}`)}</td>
      <td data-spalte={t("organisation.fach_der_stelle")}>{fachDerStelle(m, rolle)}</td>
      <td data-spalte={t("organisation.zufriedenheit")}>{zufriedenheitText(k.satisfaction)}</td>
      <td className="zahl" data-spalte={t("organisation.forderung")}>
        {formatGeld(k.demand_usd)}
        {k.over_cash && (
          <span className="warnung" title={t("organisation.ueber_kasse_hilfe")}>
            {" "}
            {t("organisation.ueber_kasse")}
          </span>
        )}
      </td>
      <td>
        <button
          type="button"
          disabled={gesperrt}
          aria-label={`${t("organisation.abwerben")}: ${m.name}`}
          onClick={onWahl}
        >
          {t("organisation.abwerben")}
        </button>
        {gesperrt && (
          <small className="feld-hilfe">
            {" "}
            {t("organisation.gesperrt_bis", { datum: formatDatum(k.blocked_until ?? "") })}
          </small>
        )}
      </td>
    </tr>
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
        <strong>{m.name}</strong> <Familienmarke m={m} />
        <Branchenmarke m={m} />
        {k.new && <span className="marke"> {t("organisation.neu")}</span>}
        {k.ai_days != null && k.ai_days > 0 && (
          <small className="feld-hilfe">
            {" "}
            {t("organisation.ki_in_tagen", { tage: formatZahl(k.ai_days) })}
          </small>
        )}
        <AlleFaehigkeiten m={m} />
      </td>
      <td className="zahl" data-spalte={t("organisation.alter_kurz")}>
        {m.age == null ? "–" : formatZahl(m.age)}
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
        {k.over_cash && (
          <span className="warnung" title={t("organisation.ueber_kasse_hilfe")}>
            {" "}
            {t("organisation.ueber_kasse")}
          </span>
        )}
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
  einheit,
  rolle,
  reiter: anfang,
  onZurueck,
}: {
  kern: Kern;
  stand: string;
  einheit: string;
  rolle: string;
  reiter: MarktReiter;
  onZurueck: () => void;
}) {
  const { daten, fehler } = useSicht(() => kern.managermarkt(einheit, rolle), stand);
  const { los } = useAktion(ORT);
  const [kontinent, setKontinent] = useState<string | null>(null);
  const [reiter, setReiter] = useState<MarktReiter>(anfang);
  const [land, setLand] = useState("alle");
  const [funktion, setFunktion] = useState("alle");
  const [branche, setBranche] = useState("alle");
  const [nurBereit, setNurBereit] = useState(false);
  const [mindest, setMindest] = useState(0);
  const auswahlId = useId();
  if (!daten) return <FehlerText fehler={fehler} />;
  const stelle = stellenName(rolle, daten.kind_text);
  const standort = einheitName(daten);
  const gewaehlt = kontinent ?? daten.continent;
  const kontinente = [...new Set(daten.candidates.map((k) => k.manager.continent))];
  // The industry of the works first (MG5), else as the core ordered them.
  const nachBranche = (l: Kandidat[]) =>
    daten.branch
      ? [...l].sort(
          (x, y) => brancheStufe(y.manager, daten.branch) - brancheStufe(x.manager, daten.branch),
        )
      : l;
  const hatBranche = (k: Kandidat) =>
    branche === "alle" || (k.manager.branches ?? []).some((b) => b.key === branche);
  const alleBranchen = [
    ...new Set(
      [...daten.candidates, ...(daten.others ?? []), ...daten.own].flatMap((k) =>
        (k.manager.branches ?? []).map((b) => b.key),
      ),
    ),
  ].sort((x, y) => t(`branche.${x}`).localeCompare(t(`branche.${y}`), "de"));
  const liste = nachBranche(
    daten.candidates.filter(
      (k) => (gewaehlt === "alle" || k.manager.continent === gewaehlt) && hatBranche(k),
    ),
  );
  const abwerben = async (k: Kandidat) => {
    const ok = await los(
      [{ PoachManager: { manager: k.manager.id, position: stellenangabe(einheit, rolle) } }],
      t("organisation.abwerbe_angebot", { name: k.manager.name, stelle, standort }),
    );
    if (ok) onZurueck();
  };
  const fremde = nachBranche(
    (daten.others ?? []).filter(
      (k) =>
        hatBranche(k) &&
        (gewaehlt === "alle" || k.manager.continent === gewaehlt) &&
        (land === "alle" || k.manager.home === land) &&
        (funktion === "alle" || k.manager.focus === funktion) &&
        (!nurBereit || k.satisfaction === 0) &&
        (k.manager.skills.find(
          (s) => s.key === `fach.${rolle === "leitung" ? k.manager.focus : rolle}`,
        )?.level ?? 0) >= mindest,
    ),
  );
  const nehmen = async (k: Kandidat) => {
    const position = stellenangabe(einheit, rolle);
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
      <th className="zahl">{t("organisation.alter_kurz")}</th>
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
      {daten.candidates.some((k) => k.over_cash) && (
        <p className="warnung">
          {t("organisation.ueber_kasse_warnung", { kasse: formatGeld(daten.cash_usd ?? 0) })}
        </p>
      )}
      <div className="unterreiter" role="group" aria-label={t("organisation.markt_reiter")}>
        {(["frei", "abwerben"] as const).map((r) => (
          <button
            key={r}
            type="button"
            className={reiter === r ? "haupt" : undefined}
            aria-pressed={reiter === r}
            onClick={() => setReiter(r)}
          >
            {t(`organisation.reiter_${r}`)}
            {r === "abwerben" && ` (${formatZahl((daten.others ?? []).length)})`}
          </button>
        ))}
      </div>
      {reiter === "abwerben" && <p className="feld-hilfe">{t("organisation.abwerben_hinweis")}</p>}
      <label htmlFor={auswahlId}>{t("organisation.kontinent_wahl")} </label>
      <select id={auswahlId} value={gewaehlt} onChange={(e) => setKontinent(e.target.value)}>
        {[daten.continent, ...kontinente.filter((k) => k !== daten.continent)].map((k) => (
          <option key={k} value={k}>
            {t(`kontinent.${k}`)}
          </option>
        ))}
        <option value="alle">{t("organisation.alle_kontinente")}</option>
      </select>
      <div className="feld">
        <label htmlFor={`${auswahlId}-branche`}>{t("organisation.branche_wahl")}</label>
        <select
          id={`${auswahlId}-branche`}
          value={branche}
          onChange={(e) => setBranche(e.target.value)}
        >
          <option value="alle">{t("organisation.alle_branchen")}</option>
          {alleBranchen.map((b) => (
            <option key={b} value={b}>
              {t(`branche.${b}`)}
            </option>
          ))}
        </select>
      </div>
      {reiter === "frei" && (
        <>
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
        </>
      )}
      {reiter === "abwerben" && (
        <>
          <div className="formular-zeile">
            <div className="feld">
              <label htmlFor={`${auswahlId}-land`}>{t("organisation.land_wahl")}</label>
              <select
                id={`${auswahlId}-land`}
                value={land}
                onChange={(e) => setLand(e.target.value)}
              >
                <option value="alle">{t("organisation.alle_laender")}</option>
                {[...new Set((daten.others ?? []).map((k) => k.manager.home))]
                  .sort((x, y) => landName(x).localeCompare(landName(y), "de"))
                  .map((l) => (
                    <option key={l} value={l}>
                      {landName(l)}
                    </option>
                  ))}
              </select>
            </div>
            <div className="feld">
              <label htmlFor={`${auswahlId}-funktion`}>{t("organisation.funktion_wahl")}</label>
              <select
                id={`${auswahlId}-funktion`}
                value={funktion}
                onChange={(e) => setFunktion(e.target.value)}
              >
                <option value="alle">{t("organisation.alle_funktionen")}</option>
                {[...new Set((daten.others ?? []).map((k) => k.manager.focus))].map((f) => (
                  <option key={f} value={f}>
                    {t(`bereich.${f}`)}
                  </option>
                ))}
              </select>
            </div>
            <div className="feld">
              <label htmlFor={`${auswahlId}-stufe`}>{t("organisation.mindeststufe")}</label>
              <select
                id={`${auswahlId}-stufe`}
                value={mindest}
                onChange={(e) => setMindest(Number(e.target.value))}
              >
                {[0, 1, 2, 3].map((n) => (
                  <option key={n} value={n}>
                    {n === 0 ? t("organisation.egal") : `≥ ${n}`}
                  </option>
                ))}
              </select>
            </div>
            <label>
              <input
                type="checkbox"
                checked={nurBereit}
                onChange={(e) => setNurBereit(e.target.checked)}
              />{" "}
              {t("organisation.wechselbereit")}
            </label>
          </div>
          <h3>{t("organisation.reiter_abwerben")}</h3>
          {fremde.length === 0 ? (
            <p>{t("organisation.keine_fremden")}</p>
          ) : (
            <div className="tabelle">
              <table className="mobil-karten" aria-label={t("organisation.reiter_abwerben")}>
                <thead>
                  <tr>
                    <th>{t("organisation.name")}</th>
                    <th>{t("organisation.firma")}</th>
                    <th>{t("organisation.stelle")}</th>
                    <th>{t("organisation.heimat")}</th>
                    <th>{t("organisation.schwerpunkt_kurz")}</th>
                    <th>{t("organisation.fach_der_stelle")}</th>
                    <th>{t("organisation.zufriedenheit")}</th>
                    <th className="zahl">{t("organisation.forderung")}</th>
                    <th />
                  </tr>
                </thead>
                <tbody>
                  {fremde.map((k) => (
                    <AbwerbeZeile
                      key={k.manager.id}
                      k={k}
                      rolle={rolle}
                      onWahl={() => void abwerben(k)}
                    />
                  ))}
                </tbody>
              </table>
            </div>
          )}
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
  const [wahl, setWahl] = useState<{
    einheit: string;
    rolle: string;
    reiter: MarktReiter;
  } | null>(null);
  const offen = uebersicht.concerns_open ?? 0;
  // Open concerns have a deadline: the inbox comes first while there are some.
  const [bereich, setBereich] = useState<
    "stellen" | "anliegen" | "chronik" | "strategie" | "ruecksprache" | "tochterfirmen"
  >(offen > 0 ? "anliegen" : "stellen");
  const stand = `${uebersicht.date}/${zaehler}`;
  return (
    <main className="ansicht" id="organisation">
      <h1 className="unsichtbar">{t("ansicht.organisation")}</h1>
      <Befehle senden={senden} meldung={meldung}>
        {wahl === null ? (
          <>
            <Unterreiter
              name={t("ansicht.organisation")}
              bereiche={[
                { key: "stellen", text: t("organisation.reiter_stellen") },
                { key: "anliegen", text: t("organisation.reiter_anliegen"), zaehler: offen },
                { key: "chronik", text: t("organisation.reiter_chronik") },
                { key: "strategie", text: t("organisation.reiter_strategie") },
                { key: "ruecksprache", text: t("organisation.reiter_ruecksprache") },
                { key: "tochterfirmen", text: t("organisation.reiter_tochterfirmen") },
              ]}
              aktiv={bereich}
              onWahl={setBereich}
            />
            {bereich === "stellen" && (
              <Organigramm
                kern={kern}
                stand={stand}
                onBesetzen={(einheit, rolle, reiter = "frei") =>
                  setWahl({ einheit, rolle, reiter })
                }
                onAnliegen={() => setBereich("anliegen")}
              />
            )}
            {bereich === "anliegen" && <AnliegenListe kern={kern} stand={stand} />}
            {bereich === "chronik" && <ChronikAnsicht kern={kern} stand={stand} />}
            {bereich === "strategie" && <StrategieAnsicht kern={kern} stand={stand} />}
            {bereich === "tochterfirmen" && <TochterfirmenAnsicht kern={kern} stand={stand} />}
            {bereich === "ruecksprache" && (
              <RuecksprachenAnsicht
                kern={kern}
                stand={stand}
                onAnliegen={() => setBereich("anliegen")}
              />
            )}
          </>
        ) : (
          <Managermarkt
            kern={kern}
            stand={stand}
            einheit={wahl.einheit}
            rolle={wahl.rolle}
            reiter={wahl.reiter}
            onZurueck={() => setWahl(null)}
          />
        )}
      </Befehle>
    </main>
  );
}
