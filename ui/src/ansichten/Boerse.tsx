import { useId, useState } from "react";
import { formatGeld, formatProzent, formatZahl, landName, zahlFeld, zahlLesen } from "../format";
import type { AktienKurs, Boerse, BoersenFirma, EigeneNotierung, Kern } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { Rueckmeldung, useAktion, useSicht, ZahlFeld } from "./gemeinsam";
import { Verlauf } from "./Grafik";

/** A share with one decimal ("12,5 %"). */
function anteil(x: number): string {
  return `${formatZahl(x * 100, x > 0 && x < 0.1 ? 1 : 0)} %`;
}

function Veraenderung({ wert }: { wert: number | null }) {
  if (wert === null) return <>–</>;
  return (
    <span className={wert < 0 ? "fehlertext" : undefined}>
      {wert >= 0 ? "+" : "−"}
      {formatZahl(Math.abs(wert) * 100, 1)} %
    </span>
  );
}

/** Going public or new shares: the steps with what they bring. */
function Ausgabe({ own, onNeu }: { own: EigeneNotierung; onNeu: () => void }) {
  const { los, antwort } = useAktion("boerse-ausgabe");
  const id = useId();
  const [wahl, setWahl] = useState(0);
  const stufe = own.issue[Math.min(wahl, own.issue.length - 1)];
  const zuKlein = !own.listed && own.equity_usd < own.equity_min_usd;
  return (
    <section aria-labelledby={`${id}-titel`}>
      <h3 id={`${id}-titel`}>{t(own.listed ? "boerse.kapitalerhoehung" : "boerse.boersengang")}</h3>
      <p className="feld-hilfe">
        {t(own.listed ? "boerse.kapitalerhoehung_hilfe" : "boerse.boersengang_hilfe", {
          wert: formatGeld(own.issue_value_usd),
          abschlag: formatProzent(own.discount),
          max: formatProzent(own.share_max),
        })}
      </p>
      {zuKlein ? (
        <p>
          {t("boerse.eigenkapital_fehlt", {
            min: formatGeld(own.equity_min_usd),
            eigen: formatGeld(own.equity_usd),
          })}
        </p>
      ) : !stufe ? (
        <p>{t("boerse.keine_ausgabe")}</p>
      ) : (
        <>
          <div className="formular-zeile">
            <div className="feld">
              <label htmlFor={`${id}-anteil`}>{t("boerse.neue_aktien")}</label>
              <select
                id={`${id}-anteil`}
                value={wahl}
                onChange={(e) => setWahl(Number(e.target.value))}
              >
                {own.issue.map((s, i) => (
                  <option key={s.share} value={i}>
                    {anteil(s.share)}
                  </option>
                ))}
              </select>
            </div>
            <button
              type="button"
              onClick={() =>
                void los([
                  own.listed
                    ? { IssueShares: { share: stufe.share } }
                    : { GoPublic: { share: stufe.share } },
                ]).then((ok) => ok && onNeu())
              }
            >
              {t(own.listed ? "boerse.ausgeben" : "boerse.an_die_boerse")}
            </button>
          </div>
          <p>
            {t("boerse.ausgabe_ergebnis", {
              erloes: formatGeld(stufe.proceeds_usd),
              kosten: formatGeld(stufe.cost_usd),
              danach: anteil(stufe.stake_after),
            })}
          </p>
        </>
      )}
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

/** The share of last year's profit the company pays out. */
function Dividende({ own, onNeu }: { own: EigeneNotierung; onNeu: () => void }) {
  const { los, antwort } = useAktion("boerse-dividende");
  const id = useId();
  const [quote, setQuote] = useState(zahlFeld(own.payout * 100, 0));
  const n = zahlLesen(quote);
  return (
    <section aria-labelledby={`${id}-titel`}>
      <h3 id={`${id}-titel`}>{t("boerse.dividende")}</h3>
      <p className="feld-hilfe">
        {t("boerse.dividende_hilfe", {
          monat: t(`monat.${own.dividend_month}`),
          gewinn: formatGeld(own.profit_last_year_usd),
          betrag: formatGeld(own.dividend_estimate_usd),
        })}
      </p>
      <div className="formular-zeile">
        <ZahlFeld
          name={t("boerse.ausschuettungsquote")}
          einheit="%"
          wert={quote}
          onWert={(text) => setQuote(text)}
        />
        <button
          type="button"
          disabled={n === null || n < 0 || n > 100}
          onClick={() =>
            void los([{ SetDividend: { payout: (n ?? 0) / 100 } }]).then((ok) => ok && onNeu())
          }
        >
          {t("boerse.festlegen")}
        </button>
      </div>
      <p className="gedaempft">
        {t("boerse.dividende_bisher", {
          letzte: formatGeld(own.last_dividend_usd),
          privat: formatGeld(own.player_dividends_usd),
        })}
      </p>
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

function EigeneFirma({ daten, onNeu }: { daten: Boerse; onNeu: () => void }) {
  const own = daten.own;
  const firma = daten.companies.find((c) => c.relation === "eigen");
  return (
    <section aria-labelledby="boerse-eigen">
      <h2 id="boerse-eigen">{t("boerse.eigene_firma")}</h2>
      {own.subsidiary ? (
        <p>{t("boerse.tochter")}</p>
      ) : (
        <>
          {firma && (
            <p>
              {t("boerse.eigene_werte", {
                wert: formatGeld(firma.value_usd),
                kurs: formatGeld(firma.price_usd),
                anteil: anteil(own.player_stake),
                streubesitz: anteil(own.free_float),
              })}
            </p>
          )}
          <div className="raster">
            <Ausgabe key={`${own.listed}`} own={own} onNeu={onNeu} />
            {own.listed && <Dividende own={own} onNeu={onNeu} />}
          </div>
        </>
      )}
    </section>
  );
}

/** Buying or selling shares of one company at the offered steps. */
function Handel({
  firma,
  art,
  kurse,
  onNeu,
}: {
  firma: BoersenFirma;
  art: "kaufen" | "verkaufen";
  kurse: AktienKurs[];
  onNeu: () => void;
}) {
  const { los, antwort } = useAktion(`boerse-${art}-${firma.company}`);
  const [wahl, setWahl] = useState(0);
  const kurs = kurse[Math.min(wahl, kurse.length - 1)];
  if (!kurs) return null;
  const name = t(`boerse.${art}_anteil`, { firma: firma.name });
  return (
    <div className="formular-zeile">
      <select aria-label={name} value={wahl} onChange={(e) => setWahl(Number(e.target.value))}>
        {kurse.map((k, i) => (
          <option key={k.share} value={i}>
            {anteil(k.share)} · {formatGeld(k.usd)}
          </option>
        ))}
      </select>
      <button
        type="button"
        aria-label={`${t(`boerse.${art}`)}: ${firma.name}`}
        onClick={() =>
          void los([
            art === "kaufen"
              ? { BuyShares: { company: firma.company, share: kurs.share } }
              : { SellShares: { company: firma.company, share: kurs.share } },
          ]).then((ok) => ok && onNeu())
        }
      >
        {t(`boerse.${art}`)}
      </button>
      <Rueckmeldung meldung={antwort} />
    </div>
  );
}

function Einzelheiten({ firma, monate }: { firma: BoersenFirma; monate: string[] }) {
  const reihe = monate.slice(Math.max(0, monate.length - firma.series_usd.length));
  return (
    <section aria-label={t("boerse.einzelheiten", { firma: firma.name })}>
      <h3>{firma.name}</h3>
      <p>
        {t("boerse.firma_werte", {
          seit: firma.since.slice(0, 4),
          eigenkapital: formatGeld(firma.equity_usd),
          gewinn: formatGeld(firma.earnings_usd),
          dividende: formatGeld(firma.dividend_usd),
        })}
      </p>
      {firma.held > 0 && (
        <p>
          {t("boerse.dein_bestand", {
            anteil: anteil(firma.held),
            wert: formatGeld(firma.held_value_usd),
            kosten: formatGeld(firma.held_cost_usd),
          })}
        </p>
      )}
      <Verlauf
        name={t("boerse.verlauf_firma", { firma: firma.name })}
        monate={reihe}
        werte={firma.series_usd}
      />
    </section>
  );
}

/** Companies shown before "show all": the largest, besides the own ones and holdings. */
const ERSTE = 20;

function Firmen({ daten, onNeu }: { daten: Boerse; onNeu: () => void }) {
  const [gewaehlt, setGewaehlt] = useState<number | null>(null);
  const [alle, setAlle] = useState(false);
  const auswahl = daten.companies.find((c) => c.company === gewaehlt) ?? null;
  // The core sorts by market value; the own company and holdings always stay visible.
  let fremde = 0;
  const gezeigt = daten.companies.filter(
    (c) => alle || c.relation !== "fremd" || c.held > 0 || fremde++ < ERSTE,
  );
  return (
    <section aria-labelledby="boerse-firmen">
      <h2 id="boerse-firmen">{t("boerse.firmen")}</h2>
      <p className="feld-hilfe">
        {t("boerse.firmen_hilfe", { max: formatProzent(daten.trade_share_max) })}
      </p>
      {daten.companies.length === 0 ? (
        <p>{t("boerse.keine_firmen")}</p>
      ) : (
        <div className="tabelle">
          <table aria-label={t("boerse.firmen")}>
            <thead>
              <tr>
                <th>{t("boerse.firma")}</th>
                <th className="zahl">{t("boerse.boersenwert")}</th>
                <th className="zahl">{t("boerse.monat")}</th>
                <th className="zahl">{t("boerse.jahr")}</th>
                <th className="zahl">{t("boerse.kgv")}</th>
                <th className="zahl">{t("boerse.rendite")}</th>
                <th className="zahl">{t("boerse.streubesitz")}</th>
                <th className="zahl">{t("boerse.dein_anteil")}</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {gezeigt.map((c) => (
                <tr key={c.company} className={gewaehlt === c.company ? "gewaehlt" : undefined}>
                  <td>
                    <button
                      type="button"
                      className="verweis"
                      onClick={() => setGewaehlt(c.company)}
                    >
                      {c.name}
                    </button>
                    <br />
                    <small className="gedaempft">
                      {landName(c.country)}
                      {c.relation !== "fremd" && ` · ${t(`boerse.beziehung.${c.relation}`)}`}
                    </small>
                  </td>
                  <td className="zahl">
                    {formatGeld(c.value_usd)}
                    <br />
                    <small className="gedaempft">
                      {t("boerse.je_aktie", { kurs: formatGeld(c.price_usd) })}
                    </small>
                  </td>
                  <td className="zahl">
                    <Veraenderung wert={c.change_month} />
                  </td>
                  <td className="zahl">
                    <Veraenderung wert={c.change_year} />
                  </td>
                  <td className="zahl">{c.pe === null ? "–" : formatZahl(c.pe, 1)}</td>
                  <td className="zahl">
                    {c.dividend_yield === null ? "–" : `${formatZahl(c.dividend_yield * 100, 1)} %`}
                  </td>
                  <td className="zahl">{anteil(c.free_float)}</td>
                  <td className="zahl">
                    {c.held > 0 ? (
                      <>
                        {anteil(c.held)}
                        <br />
                        <small className="gedaempft">{formatGeld(c.held_value_usd)}</small>
                      </>
                    ) : (
                      "–"
                    )}
                  </td>
                  <td>
                    <Handel firma={c} art="kaufen" kurse={c.buy} onNeu={onNeu} />
                    <Handel firma={c} art="verkaufen" kurse={c.sell} onNeu={onNeu} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      {gezeigt.length < daten.companies.length && (
        <button type="button" onClick={() => setAlle(true)}>
          {t("boerse.alle_zeigen", { anzahl: daten.companies.length })}
        </button>
      )}
      {auswahl && <Einzelheiten firma={auswahl} monate={daten.months} />}
    </section>
  );
}

/** Finances → stock market (K1): the index, the own listing, shares of others. */
export function BoerseAnsicht({ kern, stand }: { kern: Kern; stand: string }) {
  const { daten, fehler, neu } = useSicht(() => kern.boerse(), stand);
  if (!daten) return <FehlerText fehler={fehler} />;
  if (!daten.enabled) return <p>{t("boerse.keine")}</p>;
  return (
    <div id="boerse">
      <section aria-labelledby="boerse-index">
        <h2 id="boerse-index">{t("boerse.titel")}</h2>
        <p className="feld-hilfe">{t("boerse.hilfe")}</p>
        <p>
          {t("boerse.index", {
            index: formatZahl(daten.index, 1),
            stimmung: `${daten.mood >= 0 ? "+" : "−"}${formatZahl(Math.abs(daten.mood) * 100, 0)} %`,
          })}
        </p>
        <Verlauf
          name={t("boerse.verlauf_index")}
          monate={daten.months}
          werte={daten.index_series}
          format={(x) => formatZahl(x, 1)}
        />
        {daten.portfolio_cost_usd > 0 && (
          <p>
            {t("boerse.depot", {
              wert: formatGeld(daten.portfolio_value_usd),
              kosten: formatGeld(daten.portfolio_cost_usd),
            })}
          </p>
        )}
      </section>
      <EigeneFirma daten={daten} onNeu={neu} />
      <Firmen daten={daten} onNeu={neu} />
    </div>
  );
}
