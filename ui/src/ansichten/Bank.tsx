import { useId, useState } from "react";
import { formatDatum, formatGeld, formatZahl, zahlFeld, zahlLesen } from "../format";
import type { Bank, BankZeile, Kern } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { Rueckmeldung, useAktion, useSicht, ZahlFeld } from "./gemeinsam";

/** A rate with one decimal and a sign ("+1,5 %"). */
function zins(x: number, vorzeichen = false): string {
  const text = `${formatZahl(Math.abs(x) * 100, 1)} %`;
  if (!vorzeichen) return x < 0 ? `−${text}` : text;
  return `${x < 0 ? "−" : "+"}${text}`;
}

/** What a bank offers: deposit spread, loan discount, credit standard (in percent). */
function Einstellungen({ b, onNeu }: { b: BankZeile; onNeu: () => void }) {
  const { los, antwort } = useAktion(`bank-${b.company}`);
  const [aufschlag, setAufschlag] = useState(zahlFeld(b.deposit_spread * 100, 1));
  const [nachlass, setNachlass] = useState(zahlFeld(b.loan_discount * 100, 0));
  const [grenze, setGrenze] = useState(zahlFeld(b.max_debt_ratio * 100, 0));
  const werte = [zahlLesen(aufschlag), zahlLesen(nachlass), zahlLesen(grenze)];
  const gueltig = werte.every((w) => w !== null);
  return (
    <>
      <div className="formular-zeile">
        <ZahlFeld
          name={t("bank.einlagenaufschlag")}
          einheit="%"
          wert={aufschlag}
          onWert={setAufschlag}
          negativ
          gruppieren={false}
          hilfe={t("bank.einlagenzins", { zins: zins(b.deposit_rate) })}
        />
        <ZahlFeld
          name={t("bank.kreditnachlass")}
          einheit="%"
          wert={nachlass}
          onWert={setNachlass}
          gruppieren={false}
        />
        <ZahlFeld
          name={t("bank.verschuldung_max")}
          einheit="%"
          wert={grenze}
          onWert={setGrenze}
          gruppieren={false}
        />
        <button
          type="button"
          disabled={!gueltig}
          onClick={() =>
            void los([
              {
                SetBank: {
                  company: b.company,
                  settings: {
                    deposit_spread: (werte[0] ?? 0) / 100,
                    loan_discount: (werte[1] ?? 0) / 100,
                    max_debt_ratio: (werte[2] ?? 0) / 100,
                  },
                },
              },
            ]).then((ok) => ok && onNeu())
          }
        >
          {t("bank.festlegen")}
        </button>
      </div>
      <Rueckmeldung meldung={antwort} />
    </>
  );
}

function EineBank({ b, onNeu }: { b: BankZeile; onNeu: () => void }) {
  const id = useId();
  return (
    <section aria-labelledby={`${id}-titel`}>
      <h3 id={`${id}-titel`}>
        {b.name} <small className="gedaempft">{t(b.own ? "bank.eigene" : "bank.tochter")}</small>
      </h3>
      <p>
        {t("bank.einlagen", {
          einlagen: formatGeld(b.deposits_usd),
          ziel: formatGeld(b.deposit_target_usd),
          kapazitaet: formatGeld(b.capacity_usd),
        })}
      </p>
      <p>
        {t("bank.kasse", {
          kasse: formatGeld(b.cash_usd),
          reserve: formatGeld(b.reserve_usd),
          spielraum: formatGeld(b.room_usd),
        })}
      </p>
      <p>
        {t("bank.jahr", {
          zinsen: formatGeld(b.interest_year_usd),
          ausfaelle: formatGeld(b.write_offs_year_usd),
          ergebnis: formatGeld(b.result_year_usd),
        })}
      </p>
      <Einstellungen
        key={`${b.deposit_spread}/${b.loan_discount}/${b.max_debt_ratio}`}
        b={b}
        onNeu={onNeu}
      />
      <h4>{t("bank.kredite")}</h4>
      {b.loans.length === 0 ? (
        <p className="gedaempft">{t("bank.keine_kredite")}</p>
      ) : (
        <div className="tabelle">
          <table aria-label={t("bank.kredite")}>
            <thead>
              <tr>
                <th>{t("bank.kreditnehmer")}</th>
                <th className="zahl">{t("bank.rest")}</th>
                <th className="zahl">{t("bank.zins")}</th>
                <th>{t("bank.beginn")}</th>
              </tr>
            </thead>
            <tbody>
              {b.loans.map((k, i) => (
                <tr key={`${k.borrower}/${k.start}/${i}`}>
                  <td>{k.borrower}</td>
                  <td className="zahl">{formatGeld(k.balance_usd)}</td>
                  <td className="zahl">{zins(k.rate)}</td>
                  <td>{formatDatum(k.start)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}

/** Finances → bank (K4): the banks of the player's group. */
export function BankAnsicht({ kern, stand }: { kern: Kern; stand: string }) {
  const { daten, fehler, neu } = useSicht(() => kern.bank(), stand);
  if (!daten) return <FehlerText fehler={fehler} />;
  return <BankInhalt daten={daten} onNeu={neu} />;
}

function BankInhalt({ daten, onNeu }: { daten: Bank; onNeu: () => void }) {
  if (!daten.enabled) return <p>{t("bank.keine")}</p>;
  return (
    <section aria-labelledby="bank">
      <h2 id="bank">{t("bank.titel")}</h2>
      <p className="feld-hilfe">{t("bank.hilfe", { leitzins: zins(daten.base_rate) })}</p>
      {daten.banks.length === 0 ? (
        <p>{t(daten.subsidiaries ? "bank.leer" : "bank.leer_ohne_toechter")}</p>
      ) : (
        daten.banks.map((b) => <EineBank key={b.company} b={b} onNeu={onNeu} />)
      )}
    </section>
  );
}
