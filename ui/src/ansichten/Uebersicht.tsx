import { formatDatum, formatGeld, formatProzent, formatZahl, landName } from "../format";
import type { Uebersicht } from "../kern";
import { t } from "../texte";

function Kennzahl({ titel, wert, negativ }: { titel: string; wert: string; negativ?: boolean }) {
  return (
    <div className="kennzahl">
      <dt>{titel}</dt>
      <dd className={negativ ? "negativ" : ""}>{wert}</dd>
    </div>
  );
}

export function UebersichtAnsicht({ uebersicht }: { uebersicht: Uebersicht }) {
  const f = uebersicht.company;
  const monat = (v: number | null) =>
    v === null ? t("uebersicht.noch_kein_monat") : formatGeld(v);
  return (
    <main className="uebersicht" id="uebersicht">
      <h1 className="unsichtbar">{t("uebersicht.titel")}</h1>
      <section aria-labelledby="finanzen-titel">
        <h2 id="finanzen-titel">{t("uebersicht.finanzen")}</h2>
        <dl className="kennzahlen">
          <Kennzahl
            titel={t("spiel.kasse")}
            wert={formatGeld(f.cash_usd)}
            negativ={f.cash_usd < 0}
          />
          <Kennzahl
            titel={t("uebersicht.eigenkapital")}
            wert={formatGeld(f.equity_usd)}
            negativ={f.equity_usd < 0}
          />
          <Kennzahl titel={t("uebersicht.kredite")} wert={formatGeld(f.loans_usd)} />
          <Kennzahl
            titel={t("uebersicht.ergebnis_jahr")}
            wert={formatGeld(f.result_year_usd)}
            negativ={f.result_year_usd < 0}
          />
          <Kennzahl
            titel={t("uebersicht.ergebnis_monat")}
            wert={monat(f.result_last_month_usd)}
            negativ={(f.result_last_month_usd ?? 0) < 0}
          />
          <Kennzahl titel={t("uebersicht.umsatz_monat")} wert={monat(f.revenue_last_month_usd)} />
        </dl>
      </section>

      <section aria-labelledby="standorte-titel">
        <h2 id="standorte-titel">{t("uebersicht.standorte")}</h2>
        {f.sites.length === 0 && <p>{t("uebersicht.keine_standorte")}</p>}
        {f.sites.map((s) => (
          <article key={s.index} className="standort">
            <h3>
              {t(s.kind)} · {landName(s.country)}
              {s.deposit && ` · ${t(`lagerstaette.${s.deposit}`)}`}
              <small>{t("uebersicht.beschaeftigte", { anzahl: formatZahl(s.workers, 1) })}</small>
            </h3>
            {s.facilities.length > 0 && (
              <div className="tabelle">
                <table>
                  <thead>
                    <tr>
                      <th>{t("uebersicht.anlage")}</th>
                      <th>{t("uebersicht.produkt")}</th>
                      <th className="zahl">{t("uebersicht.auslastung")}</th>
                      <th className="zahl">{t("uebersicht.tagesleistung")}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {s.facilities.map((a, i) => (
                      <tr key={i}>
                        <td>
                          {a.count > 1 ? `${a.count} × ` : ""}
                          {t(`anlage.${a.facility}`)}
                        </td>
                        <td>{a.product ? t(`produkt.${a.product}`) : "–"}</td>
                        <td className="zahl">
                          {a.ready > uebersicht.date
                            ? t("uebersicht.im_bau", { datum: formatDatum(a.ready) })
                            : formatProzent(a.utilization)}
                        </td>
                        <td className="zahl">{formatZahl(a.output_per_day, 2)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
            <p className="lagerzeile">
              <strong>{t("uebersicht.lager")}:</strong>{" "}
              {s.stock.length === 0
                ? t("uebersicht.lager_leer")
                : s.stock
                    .map((l) => `${t(`produkt.${l.product}`)} ${formatZahl(l.quantity, 1)}`)
                    .join(" · ")}
            </p>
          </article>
        ))}
      </section>

      <section aria-labelledby="wettbewerb-titel">
        <h2 id="wettbewerb-titel">{t("uebersicht.wettbewerb")}</h2>
        {uebersicht.competitors_active + uebersicht.competitors_bankrupt === 0 ? (
          <p>{t("uebersicht.keine_wettbewerber")}</p>
        ) : (
          <>
            <p className="gedaempft">
              {t("uebersicht.wettbewerb_zahlen", {
                aktiv: formatZahl(uebersicht.competitors_active),
                pleite: formatZahl(uebersicht.competitors_bankrupt),
              })}
            </p>
            <div className="tabelle">
              <table>
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
                      <td>{landName(c.headquarters)}</td>
                      <td className="zahl">{formatGeld(c.equity_usd)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </>
        )}
      </section>
    </main>
  );
}
