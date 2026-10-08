import { useId, useState } from "react";
import { formatDatum, formatGeld, formatProzent, formatZahl, zahlLesen } from "../format";
import { geld, type Anleihen } from "../kern";
import { t } from "../texte";
import { Rueckmeldung, useAktion, ZahlFeld } from "./gemeinsam";

function Stufe({ grade }: { grade: string | null }) {
  return <>{t(grade ? `bonitaet.${grade}` : "anleihen.keine_stufe")}</>;
}

/** Why no bond sells, or nothing. */
function Hinderung({ a }: { a: Anleihen }) {
  if (a.equity_usd < a.equity_min_usd)
    return (
      <p>
        {t("anleihen.firma_zu_klein", {
          min: formatGeld(a.equity_min_usd),
          eigen: formatGeld(a.equity_usd),
        })}
      </p>
    );
  if (!a.has_figures) return <p>{t("anleihen.keine_zahlen")}</p>;
  if (a.quotes.length === 0) return <p>{t("anleihen.keine_anleger")}</p>;
  return null;
}

/** A new bond: one of the offered amounts and a term. */
function Ausgabe({ a }: { a: Anleihen }) {
  const { los, antwort } = useAktion("anleihe");
  const id = useId();
  const [wahl, setWahl] = useState(0);
  const [jahre, setJahre] = useState(String(Math.min(10, a.term_max_years)));
  const angebot = a.quotes[Math.min(wahl, a.quotes.length - 1)];
  const j = zahlLesen(jahre);
  const gueltig = j !== null && j >= a.term_min_years && j <= a.term_max_years;
  if (!angebot) return null;
  return (
    <div className="formular-zeile">
      <div className="feld">
        <label htmlFor={`${id}-betrag`}>{t("anleihen.betrag")}</label>
        <select id={`${id}-betrag`} value={wahl} onChange={(e) => setWahl(Number(e.target.value))}>
          {a.quotes.map((q, i) => (
            <option key={q.amount_usd} value={i}>
              {t("anleihen.angebot", {
                betrag: formatGeld(q.amount_usd),
                stufe: t(`bonitaet.${q.grade}`),
                kupon: formatZahl(q.coupon * 100, 1),
              })}
            </option>
          ))}
        </select>
      </div>
      <ZahlFeld
        name={t("finanzen.laufzeit")}
        einheit={t("finanzen.jahre")}
        ganzzahlig
        wert={jahre}
        onWert={setJahre}
        hilfe={t("anleihen.laufzeit_hilfe", { min: a.term_min_years, max: a.term_max_years })}
      />
      <button
        type="button"
        disabled={!gueltig}
        onClick={() =>
          void los([
            {
              IssueBond: {
                amount: geld(angebot.amount_usd),
                years: Math.floor(j ?? 0),
              },
            },
          ])
        }
      >
        {t("anleihen.ausgeben")}
      </button>
      <Rueckmeldung meldung={antwort} />
    </div>
  );
}

function Rueckkauf({ index, preis }: { index: number; preis: number }) {
  const { los, antwort } = useAktion(`anleihe/${index}`);
  return (
    <>
      <button type="button" onClick={() => void los([{ RedeemBond: { bond: index } }])}>
        {t("anleihen.zurueckkaufen", { betrag: formatGeld(preis) })}
      </button>
      <Rueckmeldung meldung={antwort} />
    </>
  );
}

/** Finances → bonds (K2): the grade, new bonds and the bonds outstanding. */
export function AnleihenTeil({ a }: { a: Anleihen }) {
  return (
    <section aria-labelledby="anleihen">
      <h2 id="anleihen">{t("anleihen.titel")}</h2>
      <p className="feld-hilfe">
        {t("anleihen.hilfe", {
          kosten: formatProzent(a.cost_share),
          aufschlag: formatProzent(a.redeem_premium),
        })}
      </p>
      <p>
        {t("anleihen.bonitaet")}: <strong>{<Stufe grade={a.grade} />}</strong> ·{" "}
        {t("anleihen.kennzahlen", {
          verschuldung: formatProzent(a.debt_ratio),
          deckung: a.coverage === null ? t("anleihen.ohne_zinsen") : formatZahl(a.coverage, 1),
        })}
      </p>
      <Hinderung a={a} />
      <Ausgabe a={a} />
      {a.bonds.length > 0 && (
        <div className="tabelle">
          <table aria-label={t("anleihen.liste")}>
            <thead>
              <tr>
                <th>{t("anleihen.ausgegeben")}</th>
                <th className="zahl">{t("finanzen.betrag")}</th>
                <th className="zahl">{t("anleihen.kupon")}</th>
                <th>{t("anleihen.bonitaet")}</th>
                <th>{t("anleihen.faellig")}</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {a.bonds.map((b) => (
                <tr key={b.index}>
                  <td>{formatDatum(b.issued)}</td>
                  <td className="zahl">{formatGeld(b.principal_usd)}</td>
                  <td className="zahl">{formatZahl(b.coupon * 100, 1)} %</td>
                  <td>{t(`bonitaet.${b.grade}`)}</td>
                  <td>{formatDatum(b.maturity)}</td>
                  <td>
                    <Rueckkauf index={b.index} preis={b.redeem_usd} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}
