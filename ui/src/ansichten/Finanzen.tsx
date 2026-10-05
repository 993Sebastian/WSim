import { useState } from "react";
import {
  ausAnzeige,
  formatDatum,
  formatGeld,
  formatProzent,
  geldEinheit,
  geldSchluessel,
  inAnzeige,
  landName,
  zahlFeld,
  zahlLesen,
} from "../format";
import {
  geld,
  type Abrechnung,
  type Finanzen,
  type Kern,
  type Uebersicht,
  type Zentren,
} from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import {
  Befehle,
  Rueckmeldung,
  Unterreiter,
  useAktion,
  useBefehl,
  useSicht,
  ZahlFeld,
} from "./gemeinsam";
import { formatMonatKurz, Verlauf } from "./Grafik";

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
  const { senden, meldung } = useBefehl(kern, onGeaendert, neu);
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
      <Befehle senden={senden} meldung={meldung}>
        <FinanzVerlauf daten={daten} />
        <Woher daten={daten} />
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
                          key={`${k.index}/${k.balance_usd}/${geldSchluessel()}`}
                          kredit={k.index}
                          rest={k.balance_usd}
                        />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
          <Kreditaufnahme jahreMax={daten.max_term_years} />
        </section>
      </Befehle>
    </main>
  );
}

function Kreditaufnahme({ jahreMax }: { jahreMax: number }) {
  const [betrag, setBetrag] = useState("");
  const [jahre, setJahre] = useState("10");
  const [fehler, setFehler] = useState<string | null>(null);
  const { los, antwort } = useAktion("kredit");
  return (
    <form
      className="karte"
      aria-label={t("finanzen.aufnehmen")}
      onSubmit={(e) => {
        e.preventDefault();
        const b = zahlLesen(betrag);
        const j = zahlLesen(jahre);
        if (b === null || b <= 0 || j === null || j < 1 || j > jahreMax) {
          setFehler(t("finanzen.kredit_werte", { max: jahreMax }));
          return;
        }
        setFehler(null);
        void los(
          [{ TakeLoan: { amount: geld(ausAnzeige(b)), years: Math.floor(j) } }],
          t("finanzen.kredit_aufgenommen", { betrag: formatGeld(ausAnzeige(b)) }),
        ).then((ok) => ok && setBetrag(""));
      }}
    >
      <h3>{t("finanzen.aufnehmen")}</h3>
      <div className="formular-zeile">
        <ZahlFeld
          name={t("finanzen.betrag")}
          einheit={geldEinheit()}
          wert={betrag}
          onWert={setBetrag}
        />
        <ZahlFeld
          name={t("finanzen.laufzeit")}
          einheit={t("finanzen.jahre")}
          wert={jahre}
          onWert={setJahre}
          hilfe={t("finanzen.laufzeit_hilfe", { max: jahreMax })}
        />
        <button type="submit">{t("finanzen.aufnehmen_knopf")}</button>
      </div>
      {fehler && <p className="fehlertext">{fehler}</p>}
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

/** Cash, revenue and result of the closed months. */
function FinanzVerlauf({ daten }: { daten: Finanzen }) {
  const v = daten.history;
  return (
    <section aria-labelledby="verlauf">
      <h2 id="verlauf">{t("finanzen.verlauf")}</h2>
      {v.length < 2 ? (
        <p className="gedaempft">{t("grafik.zu_wenig")}</p>
      ) : (
        <>
          <div className="verlaeufe">
            {(
              [
                ["uebersicht.kasse_verlauf", v.map((m) => m.cash_usd), "linie"],
                ["uebersicht.umsatz", v.map((m) => m.revenue_usd), "linie"],
                ["uebersicht.ergebnis", v.map((m) => m.result_usd), "balken"],
              ] as const
            ).map(([k, werte, art]) => (
              <figure key={k} className="verlaufskarte">
                <figcaption>{t(k)}</figcaption>
                <Verlauf name={t(k)} monate={v.map((m) => m.month)} werte={[...werte]} art={art} />
              </figure>
            ))}
          </div>
          <details className="aufklapper">
            <summary>{t("finanzen.verlauf_tabelle")}</summary>
            <div className="tabelle">
              <table className="mobil-karten" aria-label={t("finanzen.verlauf_tabelle")}>
                <thead>
                  <tr>
                    <th>{t("finanzen.monat")}</th>
                    <th className="zahl">{t("uebersicht.umsatz_monat")}</th>
                    <th className="zahl">{t("finanzen.ergebnis")}</th>
                    <th className="zahl">{t("uebersicht.kasse_verlauf")}</th>
                  </tr>
                </thead>
                <tbody>
                  {[...v].reverse().map((m) => (
                    <tr key={m.month}>
                      <td>{formatMonatKurz(m.month)}</td>
                      <td className="zahl" data-spalte={t("uebersicht.umsatz_monat")}>
                        {formatGeld(m.revenue_usd)}
                      </td>
                      <td className="zahl" data-spalte={t("finanzen.ergebnis")}>
                        <Betrag usd={m.result_usd} />
                      </td>
                      <td className="zahl" data-spalte={t("uebersicht.kasse_verlauf")}>
                        <Betrag usd={m.cash_usd} />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </details>
        </>
      )}
    </section>
  );
}

/** Where the money is made: results by site and by product. */
function Woher({ daten }: { daten: Finanzen }) {
  const [zeitraum, setZeitraum] = useState<"monat" | "jahr">(
    daten.centers_last_month ? "monat" : "jahr",
  );
  const z: Zentren | null = zeitraum === "monat" ? daten.centers_last_month : daten.centers_year;
  return (
    <section aria-labelledby="woher">
      <h2 id="woher">{t("finanzen.woher")}</h2>
      <Unterreiter
        name={t("finanzen.zeitraum")}
        bereiche={[
          { key: "monat", text: t("finanzen.vormonat") },
          { key: "jahr", text: t("finanzen.laufendes_jahr") },
        ]}
        aktiv={zeitraum}
        onWahl={setZeitraum}
      />
      {!z ? (
        <p className="gedaempft">{t("uebersicht.noch_kein_monat")}</p>
      ) : (
        <div className="raster">
          <div className="tabelle">
            <table className="mobil-karten" aria-label={t("finanzen.je_standort")}>
              <thead>
                <tr>
                  <th>{t("finanzen.je_standort")}</th>
                  <th className="zahl">{t("kostenart.umsatz")}</th>
                  <th className="zahl">{t("finanzen.ergebnis")}</th>
                </tr>
              </thead>
              <tbody>
                {z.sites.map((s) => (
                  <tr key={s.site}>
                    <td>
                      {t(s.kind_text)} · {landName(s.country)}
                    </td>
                    <td className="zahl" data-spalte={t("kostenart.umsatz")}>
                      {formatGeld(s.revenue_usd)}
                    </td>
                    <td className="zahl" data-spalte={t("finanzen.ergebnis")}>
                      <Betrag usd={s.result_usd} />
                    </td>
                  </tr>
                ))}
                {z.company_usd !== 0 && (
                  <tr>
                    <td>{t("finanzen.firma_gesamt")}</td>
                    <td className="zahl" data-spalte={t("kostenart.umsatz")}>
                      –
                    </td>
                    <td className="zahl" data-spalte={t("finanzen.ergebnis")}>
                      <Betrag usd={z.company_usd} />
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
          <div className="tabelle">
            <table className="mobil-karten" aria-label={t("finanzen.je_produkt")}>
              <thead>
                <tr>
                  <th>{t("finanzen.je_produkt")}</th>
                  <th className="zahl">{t("kostenart.umsatz")}</th>
                  <th className="zahl">{t("werk.rohertrag")}</th>
                  <th className="zahl">{t("finanzen.rohertrag_anteil")}</th>
                </tr>
              </thead>
              <tbody>
                {z.products.length === 0 && (
                  <tr>
                    <td colSpan={4} className="gedaempft">
                      {t("finanzen.kein_umsatz")}
                    </td>
                  </tr>
                )}
                {z.products.map((p) => (
                  <tr key={p.product}>
                    <td>{t(`produkt.${p.product}`)}</td>
                    <td className="zahl" data-spalte={t("kostenart.umsatz")}>
                      {formatGeld(p.revenue_usd)}
                    </td>
                    <td className="zahl" data-spalte={t("werk.rohertrag")}>
                      <Betrag usd={p.margin_usd} />
                    </td>
                    <td className="zahl" data-spalte={t("finanzen.rohertrag_anteil")}>
                      {p.revenue_usd > 0 ? formatProzent(p.margin_usd / p.revenue_usd) : "–"}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            <p className="feld-hilfe">{t("werk.rohertrag_hilfe")}</p>
          </div>
        </div>
      )}
    </section>
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

function Tilgung({ kredit, rest }: { kredit: number; rest: number }) {
  // Rounded up: the core repays at most what is left.
  const [betrag, setBetrag] = useState(zahlFeld(Math.ceil(inAnzeige(rest)), 0));
  const { los, antwort } = useAktion(`tilgung/${kredit}`);
  return (
    <form
      className="formular-zeile"
      aria-label={t("finanzen.sondertilgung")}
      onSubmit={(e) => {
        e.preventDefault();
        const b = zahlLesen(betrag);
        if (b === null || b <= 0) return;
        void los(
          [{ RepayLoan: { loan: kredit, amount: geld(ausAnzeige(b)) } }],
          t("finanzen.getilgt", { betrag: formatGeld(Math.min(ausAnzeige(b), rest)) }),
        );
      }}
    >
      <ZahlFeld
        name={t("finanzen.tilgungsbetrag")}
        einheit={geldEinheit()}
        wert={betrag}
        onWert={setBetrag}
      />
      <button type="submit" className="schlicht">
        {t("finanzen.tilgen")}
      </button>
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}
