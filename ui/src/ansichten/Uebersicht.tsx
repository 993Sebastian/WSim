// The overview (docs/BEDIENUNG.md): first what needs doing, then how the company
// stands and develops, then the sites with a way into each plant.
import {
  formatDatum,
  formatGeld,
  formatProzent,
  formatZahl,
  landName,
  meldungText,
} from "../format";
import type { Etappe, Hinweis, Rang, Uebersicht } from "../kern";
import { t } from "../texte";
import { Erklaerung } from "./gemeinsam";
import { formatMonatKurz, Verlauf } from "./Grafik";

function Kennzahl({
  titel,
  wert,
  negativ,
  zusatz,
}: {
  titel: string;
  wert: string;
  negativ?: boolean;
  zusatz?: string;
}) {
  return (
    <div className="kennzahl">
      <dt>{titel}</dt>
      <dd className={negativ ? "negativ" : ""}>
        {wert}
        {zusatz && <small className="kennzahl-zusatz">{zusatz}</small>}
      </dd>
    </div>
  );
}

/** A place compared with the same month a year before (M29). */
function rangVergleich(
  jetzt: number | null,
  vorher: number | null | undefined,
): string | undefined {
  if (jetzt === null || vorher === null || vorher === undefined) return undefined;
  if (jetzt < vorher) return t("uebersicht.rang_besser", { plaetze: vorher - jetzt, vorher });
  if (jetzt > vorher) return t("uebersicht.rang_schlechter", { plaetze: jetzt - vorher, vorher });
  return t("uebersicht.rang_gleich");
}

/** The player's places among all companies (M29). */
function RangAnzeige({ rang }: { rang: Rang }) {
  const vorher = rang.year_before ?? undefined;
  return (
    <div className="rang" data-tour="rang">
      <div className="rang-kopf">
        <h3>{t("uebersicht.rang", { firmen: formatZahl(rang.now.companies) })}</h3>
        <Erklaerung wert={t("uebersicht.rang_wert")}>
          <p>{t("uebersicht.rang_hinweis")}</p>
        </Erklaerung>
      </div>
      <dl className="kennzahlen">
        <Kennzahl
          titel={t("uebersicht.rang_eigenkapital")}
          wert={t("uebersicht.platz", { platz: formatZahl(rang.now.equity) })}
          zusatz={rangVergleich(rang.now.equity, vorher?.equity)}
        />
        <Kennzahl
          titel={t("uebersicht.rang_umsatz")}
          wert={
            rang.now.revenue === null
              ? t("uebersicht.kein_umsatz")
              : t("uebersicht.platz", { platz: formatZahl(rang.now.revenue) })
          }
          zusatz={rangVergleich(rang.now.revenue, vorher?.revenue)}
        />
      </dl>
    </div>
  );
}

/** A measured way to a goal, in its unit. */
function fortschrittText(p: NonNullable<Etappe["progress"]>, wert: number): string {
  if (p.unit === "anteil") return formatProzent(wert);
  if (p.unit === "geld") return formatGeld(wert);
  return formatZahl(wert);
}

function etappenHinweis(e: Etappe): string {
  // The market leader's hint names the share it takes.
  return t(`etappe.${e.key}.hinweis`, {
    anteil: e.progress ? formatProzent(e.progress.target) : "",
  });
}

/** Goals after the introduction (M23): the next one with its way there, all on demand. */
function Etappen({ etappen, onAusblenden }: { etappen: Etappe[]; onAusblenden: () => void }) {
  const erreicht = etappen.filter((e) => e.reached).length;
  const naechste = etappen.find((e) => !e.reached);
  return (
    <section aria-labelledby="etappen-titel" className="etappen" data-tour="etappen">
      <div className="abschnitt-kopf">
        <h2 id="etappen-titel">
          {t("etappen.titel")}{" "}
          <small className="gedaempft">
            {t("etappen.stand", { erreicht, anzahl: etappen.length })}
          </small>
        </h2>
        <button type="button" className="schlicht" onClick={onAusblenden}>
          {t("etappen.ausblenden")}
        </button>
      </div>
      {naechste ? (
        <div className="naechste-etappe">
          <p>
            <span className="gedaempft">{t("etappen.naechste")}:</span>{" "}
            <strong>{t(`etappe.${naechste.key}`)}</strong>
          </p>
          {naechste.progress && (
            <p className="etappen-fortschritt">
              <progress
                max={naechste.progress.target}
                value={Math.max(0, Math.min(naechste.progress.current, naechste.progress.target))}
                aria-label={t(`etappe.${naechste.key}`)}
              />
              <span>
                {t("etappen.fortschritt", {
                  aktuell: fortschrittText(naechste.progress, naechste.progress.current),
                  ziel: fortschrittText(naechste.progress, naechste.progress.target),
                })}
              </span>
            </p>
          )}
          <p>
            <span className="gedaempft">{t("etappen.so_gehts")}</span> {etappenHinweis(naechste)}
          </p>
        </div>
      ) : (
        <p className="erfolgstext">{t("etappen.alle", { anzahl: etappen.length })}</p>
      )}
      <details className="alle-etappen">
        <summary>{t("etappen.alle_zeigen")}</summary>
        <ol>
          {etappen.map((e) => (
            <li key={e.key} className={e.reached ? "erreicht" : ""}>
              <span aria-hidden="true">{e.reached ? "✓" : "○"}</span>{" "}
              <span>{t(`etappe.${e.key}`)}</span>{" "}
              <small className="gedaempft">
                {e.reached
                  ? t("etappen.erreicht_am", { datum: formatDatum(e.reached) })
                  : e.progress
                    ? t("etappen.fortschritt", {
                        aktuell: fortschrittText(e.progress, e.progress.current),
                        ziel: fortschrittText(e.progress, e.progress.target),
                      })
                    : ""}
              </small>
            </li>
          ))}
        </ol>
      </details>
    </section>
  );
}

export function UebersichtAnsicht({
  uebersicht,
  geldHinweis = null,
  etappenZeigen = true,
  onEtappen,
  onHinweis,
  onWerk,
  onGruenden,
}: {
  uebersicht: Uebersicht;
  /** In which currency and at which prices the amounts are shown. */
  geldHinweis?: string | null;
  /** The goals are shown unless the player hid them (M23). */
  etappenZeigen?: boolean;
  onEtappen?: (zeigen: boolean) => void;
  onHinweis: (h: Hinweis) => void;
  onWerk: (site: number) => void;
  /** Opens the founding dialog (PE3). */
  onGruenden?: () => void;
}) {
  const f = uebersicht.company;
  const person = uebersicht.person ?? null;
  const verlauf = uebersicht.history;
  const monat = (v: number | null) =>
    v === null ? t("uebersicht.noch_kein_monat") : formatGeld(v);
  const vormonat = verlauf.at(-1);
  const davor = verlauf.at(-2);
  // Change of cash since the last month's end, or (at a month's start) in that month.
  const kassenAenderung = (() => {
    if (!vormonat || !f) return undefined;
    const seit = f.cash_usd - vormonat.cash_usd;
    const vorzeichen = (d: number) => `${d >= 0 ? "+" : ""}${formatGeld(d)}`;
    if (Math.abs(seit) >= 0.5) return t("uebersicht.seit_monatsende", { betrag: vorzeichen(seit) });
    if (davor)
      return t("uebersicht.im_vormonat", {
        betrag: vorzeichen(vormonat.cash_usd - davor.cash_usd),
      });
    return undefined;
  })();
  return (
    <main className="uebersicht" id="uebersicht">
      <h1 className="unsichtbar">{t("uebersicht.titel")}</h1>

      <section aria-labelledby="erledigen-titel" data-tour="zu-erledigen">
        <h2 id="erledigen-titel">{t("uebersicht.zu_erledigen")}</h2>
        {uebersicht.hints.length === 0 ? (
          <p className="erfolgstext">{t("uebersicht.nichts_zu_tun")}</p>
        ) : (
          <ul className="hinweise">
            {uebersicht.hints.map((h, i) => (
              <li key={i} className={`hinweis-zeile meldung-${h.message.kind}`}>
                <span className="meldungsart">{t(`meldungsart.${h.message.kind}`)}</span>
                <span>{meldungText(h.message)}</span>
                {(h.site !== null || h.message.target) && (
                  <button type="button" className="schlicht" onClick={() => onHinweis(h)}>
                    {t("uebersicht.hingehen")}
                  </button>
                )}
              </li>
            ))}
          </ul>
        )}
      </section>

      {!f && (
        <section aria-labelledby="privat-titel">
          <h2 id="privat-titel">{t("uebersicht.privat")}</h2>
          <dl className="kennzahlen">
            <Kennzahl
              titel={t("person.privatkonto")}
              wert={formatGeld(person?.cash_usd ?? 0)}
              negativ={person?.short ?? false}
            />
            <Kennzahl
              titel={t("person.vermoegen_gesamt")}
              wert={formatGeld(person?.wealth_usd ?? 0)}
            />
            {person?.place !== undefined && (person.companies ?? 0) > 0 && (
              <Kennzahl
                titel={t("uebersicht.rang_vermoegen")}
                wert={t("uebersicht.rang_vermoegen_wert", {
                  platz: formatZahl(person.place),
                  firmen: formatZahl(person.companies ?? 0),
                })}
              />
            )}
          </dl>
          <p>{t("uebersicht.ohne_firma")}</p>
          {onGruenden && (
            <div className="knopfreihe links">
              <button type="button" className="haupt" onClick={onGruenden}>
                {t("gruendung.oeffnen")}
              </button>
            </div>
          )}
        </section>
      )}

      {f && etappenZeigen && (uebersicht.milestones?.length ?? 0) > 0 && (
        <Etappen etappen={uebersicht.milestones!} onAusblenden={() => onEtappen?.(false)} />
      )}

      {f && (
        <>
          <section aria-labelledby="finanzen-titel">
            <h2 id="finanzen-titel">{t("uebersicht.finanzen")}</h2>
            <dl className="kennzahlen">
              <Kennzahl
                titel={t("spiel.kasse")}
                wert={formatGeld(f.cash_usd)}
                negativ={f.cash_usd < 0}
                zusatz={kassenAenderung}
              />
              <Kennzahl
                titel={t("uebersicht.ergebnis_monat")}
                wert={monat(f.result_last_month_usd)}
                negativ={(f.result_last_month_usd ?? 0) < 0}
              />
              <Kennzahl
                titel={t("uebersicht.umsatz_monat")}
                wert={monat(f.revenue_last_month_usd)}
              />
              <Kennzahl
                titel={t("uebersicht.ergebnis_jahr")}
                wert={formatGeld(f.result_year_usd)}
                negativ={f.result_year_usd < 0}
              />
              <Kennzahl
                titel={t("uebersicht.eigenkapital")}
                wert={formatGeld(f.equity_usd)}
                negativ={f.equity_usd < 0}
              />
              <Kennzahl titel={t("uebersicht.kredite")} wert={formatGeld(f.loans_usd)} />
            </dl>
            {geldHinweis && (
              <p className="gedaempft geld-hinweis">
                {geldHinweis} {t("geld.umstellen")}
              </p>
            )}
            {verlauf.length >= 2 && (
              <div className="verlaeufe">
                {(
                  [
                    ["uebersicht.kasse_verlauf", verlauf.map((m) => m.cash_usd), "linie"],
                    ["uebersicht.umsatz", verlauf.map((m) => m.revenue_usd), "linie"],
                    ["uebersicht.ergebnis", verlauf.map((m) => m.result_usd), "balken"],
                  ] as const
                ).map(([schluessel, werte, art]) => (
                  <figure key={schluessel} className="verlaufskarte">
                    <figcaption>
                      {t(schluessel)}{" "}
                      <small className="gedaempft">
                        {formatMonatKurz(verlauf[0]!.month)} –{" "}
                        {formatMonatKurz(verlauf.at(-1)!.month)}
                      </small>
                    </figcaption>
                    <Verlauf
                      name={t(schluessel)}
                      monate={verlauf.map((m) => m.month)}
                      werte={[...werte]}
                      art={art}
                    />
                  </figure>
                ))}
              </div>
            )}
          </section>

          <section aria-labelledby="standorte-titel">
            <h2 id="standorte-titel">{t("uebersicht.standorte")}</h2>
            {f.sites.length === 0 && <p>{t("uebersicht.keine_standorte")}</p>}
            <div className="karten-raster">
              {f.sites.map((s) => {
                const titel = `${t(s.kind)} · ${landName(s.country)}`;
                const produkte = [
                  ...new Set(s.facilities.map((a) => a.product).filter((p): p is string => !!p)),
                ];
                const imBau = s.facilities.filter((a) => a.ready > uebersicht.date).length;
                const still = s.facilities
                  .filter((a) => a.mothballed)
                  .reduce((n, a) => n + a.count, 0);
                const probleme = uebersicht.hints.filter((h) => h.site === s.index).length;
                return (
                  <article key={s.index} className="karte standortkarte" aria-label={titel}>
                    <h3>
                      {titel}
                      {s.deposit && <small>{t(`lagerstaette.${s.deposit}`)}</small>}
                    </h3>
                    <p className="gedaempft">
                      {produkte.length > 0
                        ? produkte.map((p) => t(`produkt.${p}`)).join(", ")
                        : t("uebersicht.keine_erzeugung")}
                    </p>
                    <dl className="werte">
                      <dt>{t("uebersicht.anlagen")}</dt>
                      <dd>
                        {formatZahl(s.facilities.reduce((n, a) => n + a.count, 0))}
                        {imBau > 0 && ` (${t("uebersicht.davon_im_bau", { anzahl: imBau })})`}
                        {still > 0 && ` (${t("uebersicht.davon_stillgelegt", { anzahl: still })})`}
                      </dd>
                      <dt>{t("werk.beschaeftigte")}</dt>
                      <dd>{formatZahl(s.workers, s.workers >= 10 ? 0 : 1)}</dd>
                    </dl>
                    {probleme > 0 ? (
                      <p className="warntext">
                        {t("uebersicht.hinweise_standort", { anzahl: probleme })}
                      </p>
                    ) : (
                      <p className="erfolgstext">{t("werk.alles_laeuft")}</p>
                    )}
                    <div className="knopfreihe links">
                      <button
                        type="button"
                        aria-label={t("werk.oeffnen_von", { standort: titel })}
                        onClick={() => onWerk(s.index)}
                      >
                        {t("werk.oeffnen")}
                      </button>
                    </div>
                  </article>
                );
              })}
            </div>
          </section>
        </>
      )}

      <section aria-labelledby="wettbewerb-titel">
        <h2 id="wettbewerb-titel">{t("uebersicht.wettbewerb")}</h2>
        {uebersicht.rank && <RangAnzeige rang={uebersicht.rank} />}
        {uebersicht.competitors_active + uebersicht.competitors_bankrupt === 0 ? (
          <p>{t("uebersicht.keine_wettbewerber")}</p>
        ) : (
          <details className="aufklapper">
            <summary>
              {t("uebersicht.wettbewerb_zahlen", {
                aktiv: formatZahl(uebersicht.competitors_active),
                pleite: formatZahl(uebersicht.competitors_bankrupt),
              })}
            </summary>
            <p className="gedaempft">{t("uebersicht.wettbewerb_hinweis")}</p>
            <div className="tabelle">
              <table className="mobil-karten" aria-label={t("uebersicht.groesste")}>
                <thead>
                  <tr>
                    <th>{t("uebersicht.firma")}</th>
                    <th>{t("uebersicht.sitz")}</th>
                    <th className="zahl">{t("uebersicht.eigenkapital")}</th>
                  </tr>
                </thead>
                <tbody>
                  {uebersicht.competitors.map((c) => (
                    <tr key={c.name}>
                      <td>
                        {c.name} {c.real && <span className="marke">{t("uebersicht.real")}</span>}
                      </td>
                      <td data-spalte={t("uebersicht.sitz")}>{landName(c.headquarters)}</td>
                      <td className="zahl" data-spalte={t("uebersicht.eigenkapital")}>
                        {formatGeld(c.equity_usd)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </details>
        )}
      </section>
    </main>
  );
}
