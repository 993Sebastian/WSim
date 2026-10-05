// The overview (docs/BEDIENUNG.md): first what needs doing, then how the company
// stands and develops, then the sites with a way into each plant.
import { formatGeld, formatZahl, landName, meldungText } from "../format";
import type { Hinweis, Uebersicht } from "../kern";
import { t } from "../texte";
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

export function UebersichtAnsicht({
  uebersicht,
  onHinweis,
  onWerk,
}: {
  uebersicht: Uebersicht;
  onHinweis: (h: Hinweis) => void;
  onWerk: (site: number) => void;
}) {
  const f = uebersicht.company;
  const verlauf = uebersicht.history;
  const monat = (v: number | null) =>
    v === null ? t("uebersicht.noch_kein_monat") : formatGeld(v);
  const vormonat = verlauf.at(-1);
  const davor = verlauf.at(-2);
  // Change of cash since the last month's end, or (at a month's start) in that month.
  const kassenAenderung = (() => {
    if (!vormonat) return undefined;
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

      <section aria-labelledby="erledigen-titel">
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
          <Kennzahl titel={t("uebersicht.umsatz_monat")} wert={monat(f.revenue_last_month_usd)} />
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
                    {formatMonatKurz(verlauf[0]!.month)} – {formatMonatKurz(verlauf.at(-1)!.month)}
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

      <section aria-labelledby="wettbewerb-titel">
        <h2 id="wettbewerb-titel">{t("uebersicht.wettbewerb")}</h2>
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
