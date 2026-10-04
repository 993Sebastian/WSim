import { useState } from "react";
import { formatDatum, formatGeld, formatProzent } from "../format";
import { geld, type Abrechnung, type Befehl, type Kern, type Uebersicht } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { Rueckmeldung, useBefehl, useSicht } from "./gemeinsam";

export function FinanzenAnsicht({
  kern,
  uebersicht,
  onGeaendert,
}: {
  kern: Kern;
  uebersicht: Uebersicht;
  onGeaendert: (u: Uebersicht) => void;
}) {
  const { daten, fehler, neu } = useSicht(() => kern.finanzen(), uebersicht.date);
  const { ausfuehren, meldung } = useBefehl(kern, onGeaendert, neu);
  const [betrag, setBetrag] = useState(0);
  const [jahre, setJahre] = useState(10);
  if (!daten) return <FehlerText fehler={fehler} />;
  const abrechnungen: [string, Abrechnung | null][] = [
    ["finanzen.vormonat", daten.last_month],
    ["finanzen.laufendes_jahr", daten.year],
    ["finanzen.vorjahr", daten.last_year],
  ];
  // The core lists every cost type in the same order; types without any amount are left out.
  const arten = daten.year.lines
    .map(([art]) => art)
    .filter((art) => abrechnungen.some(([, a]) => (new Map(a?.lines).get(art) ?? 0) !== 0));
  return (
    <main className="ansicht" id="finanzen">
      <h1 className="unsichtbar">{t("ansicht.finanzen")}</h1>
      <Rueckmeldung meldung={meldung} />
      <div className="raster">
        <section aria-labelledby="bilanz">
          <h2 id="bilanz">{t("finanzen.bilanz")}</h2>
          <div className="bilanz">
            <Spalte titel={t("finanzen.aktiva")} zeilen={daten.assets} summe={daten.total_usd} />
            <Spalte titel={t("finanzen.passiva")} zeilen={daten.claims} summe={daten.total_usd} />
          </div>
          {daten.loss_carryforward_usd > 0 && (
            <p className="gedaempft">
              {t("finanzen.verlustvortrag", { betrag: formatGeld(daten.loss_carryforward_usd) })}
            </p>
          )}
        </section>

        <section aria-labelledby="erfolg">
          <h2 id="erfolg">{t("finanzen.erfolgsrechnung")}</h2>
          <div className="tabelle">
            <table>
              <thead>
                <tr>
                  <th />
                  {abrechnungen.map(([k]) => (
                    <th key={k} className="zahl">
                      {t(k)}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {arten.map((art) => (
                  <tr key={art}>
                    <td>{t(art)}</td>
                    {abrechnungen.map(([k, a]) => (
                      <td key={k} className="zahl">
                        {a ? <Betrag usd={new Map(a.lines).get(art) ?? 0} /> : "–"}
                      </td>
                    ))}
                  </tr>
                ))}
                <tr className="summe">
                  <td>{t("finanzen.ergebnis")}</td>
                  {abrechnungen.map(([k, a]) => (
                    <td key={k} className="zahl">
                      {a ? <Betrag usd={a.result_usd} /> : "–"}
                    </td>
                  ))}
                </tr>
                {(
                  [
                    "finanzen.cf_betrieb",
                    "finanzen.cf_investition",
                    "finanzen.cf_finanzierung",
                  ] as const
                ).map((z, i) => (
                  <tr key={z} className={i === 0 ? "trenner" : undefined}>
                    <td>{t(z)}</td>
                    {abrechnungen.map(([k, a]) => (
                      <td key={k} className="zahl">
                        {a ? <Betrag usd={a.cash_flow_usd[i] ?? 0} /> : "–"}
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>
      </div>

      <section aria-labelledby="kredite">
        <h2 id="kredite">{t("finanzen.kredite")}</h2>
        <p>
          {t("finanzen.rahmen", {
            rahmen: formatGeld(daten.credit_limit_usd),
            dispo: formatGeld(daten.overdraft_limit_usd),
            zins: formatProzent(daten.loan_rate),
          })}
        </p>
        {daten.loans.length > 0 && (
          <div className="tabelle">
            <table>
              <thead>
                <tr>
                  <th>{t("finanzen.beginn")}</th>
                  <th className="zahl">{t("finanzen.betrag")}</th>
                  <th className="zahl">{t("finanzen.rest")}</th>
                  <th className="zahl">{t("finanzen.zins")}</th>
                  <th className="zahl">{t("finanzen.rate")}</th>
                  <th>{t("finanzen.sondertilgung")}</th>
                </tr>
              </thead>
              <tbody>
                {daten.loans.map((k) => (
                  <tr key={k.index}>
                    <td>
                      {formatDatum(k.start)} · {t("finanzen.monate", { anzahl: k.months })}
                    </td>
                    <td className="zahl">{formatGeld(k.principal_usd)}</td>
                    <td className="zahl">{formatGeld(k.balance_usd)}</td>
                    <td className="zahl">{formatProzent(k.rate)}</td>
                    <td className="zahl">{formatGeld(k.instalment_usd)}</td>
                    <td>
                      <Tilgung
                        key={`${k.index}/${k.balance_usd}`}
                        kredit={k.index}
                        rest={k.balance_usd}
                        ausfuehren={ausfuehren}
                      />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
        <form
          className="zeilenformular"
          aria-label={t("finanzen.aufnehmen")}
          onSubmit={(e) => {
            e.preventDefault();
            void ausfuehren({ TakeLoan: { amount: geld(betrag), years: jahre } }).then(
              (ok) => ok && setBetrag(0),
            );
          }}
        >
          <strong>{t("finanzen.aufnehmen")}</strong>
          <label>
            {t("finanzen.betrag")}
            <input
              id="kredit_betrag"
              type="number"
              min={0}
              step="any"
              value={betrag || ""}
              onChange={(e) => setBetrag(Number(e.target.value))}
            />
          </label>
          <label>
            {t("finanzen.laufzeit")}
            <input
              id="kredit_jahre"
              type="number"
              min={1}
              max={daten.max_term_years}
              value={jahre}
              onChange={(e) => setJahre(Math.floor(Number(e.target.value) || 1))}
            />
          </label>
          <button type="submit" disabled={betrag <= 0}>
            {t("finanzen.aufnehmen_knopf")}
          </button>
        </form>
      </section>
    </main>
  );
}

function Betrag({ usd }: { usd: number }) {
  return <span className={usd < 0 ? "negativ" : undefined}>{formatGeld(usd)}</span>;
}

function Spalte({
  titel,
  zeilen,
  summe,
}: {
  titel: string;
  zeilen: [string, number][];
  summe: number;
}) {
  return (
    <table>
      <thead>
        <tr>
          <th colSpan={2}>{titel}</th>
        </tr>
      </thead>
      <tbody>
        {zeilen.map(([k, v]) => (
          <tr key={k}>
            <td>{t(k)}</td>
            <td className="zahl">
              <Betrag usd={v} />
            </td>
          </tr>
        ))}
        <tr className="summe">
          <td>{t("finanzen.summe")}</td>
          <td className="zahl">{formatGeld(summe)}</td>
        </tr>
      </tbody>
    </table>
  );
}

function Tilgung({
  kredit,
  rest,
  ausfuehren,
}: {
  kredit: number;
  rest: number;
  ausfuehren: (...b: Befehl[]) => Promise<boolean>;
}) {
  const [betrag, setBetrag] = useState(Math.ceil(rest));
  return (
    <form
      className="inline"
      aria-label={t("finanzen.sondertilgung")}
      onSubmit={(e) => {
        e.preventDefault();
        void ausfuehren({ RepayLoan: { loan: kredit, amount: geld(betrag) } });
      }}
    >
      <input
        className="schmal"
        type="number"
        min={0}
        step="any"
        aria-label={t("finanzen.tilgungsbetrag")}
        value={betrag}
        onChange={(e) => setBetrag(Number(e.target.value))}
      />
      <button type="submit" className="schlicht" disabled={!(betrag > 0)}>
        {t("finanzen.tilgen")}
      </button>
    </form>
  );
}
