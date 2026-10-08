import { useId, useState } from "react";
import { ausAnzeige, formatGeld, geldEinheit, geldFeld, landName, zahlLesen } from "../format";
import { geld, type Kern, type Konzern, type KonzernZeile, type Tochter } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { LAENDER } from "./laender";
import { Rueckmeldung, useAktion, useSicht, ZahlFeld } from "./gemeinsam";

type Schwerpunkt = Tochter["focus"];

const BEFEHL: Record<Schwerpunkt, "Production" | "Logistics" | "Investment" | "Bank"> = {
  produktion: "Production",
  logistik: "Logistics",
  investment: "Investment",
  bank: "Bank",
};
const SCHWERPUNKTE = ["produktion", "logistik", "investment", "bank"] as const;

/** Founding a subsidiary: name, seat, capital, focus. */
function Gruendung({ daten, onNeu }: { daten: Konzern; onNeu: () => void }) {
  const { los, antwort } = useAktion("tochter-neu");
  const id = useId();
  const [name, setName] = useState("");
  const [land, setLand] = useState(LAENDER.includes("DEU") ? "DEU" : (LAENDER[0] ?? ""));
  const [kapital, setKapital] = useState(geldFeld(daten.min_capital_usd));
  const [schwerpunkt, setSchwerpunkt] = useState<Schwerpunkt>("produktion");
  const betrag = zahlLesen(kapital);
  return (
    <section aria-labelledby={`${id}-titel`}>
      <h2 id={`${id}-titel`}>{t("tochter.neu_titel")}</h2>
      <p className="feld-hilfe">
        {t("tochter.neu_hilfe", {
          min: formatGeld(daten.min_capital_usd),
          kosten: formatGeld(daten.founding_cost_usd),
        })}
      </p>
      <div className="formular-zeile">
        <div className="feld">
          <label htmlFor={`${id}-name`}>{t("tochter.name")}</label>
          <input id={`${id}-name`} value={name} onChange={(e) => setName(e.target.value)} />
        </div>
        <div className="feld">
          <label htmlFor={`${id}-land`}>{t("tochter.sitz")}</label>
          <select id={`${id}-land`} value={land} onChange={(e) => setLand(e.target.value)}>
            {LAENDER.map((k) => (
              <option key={k} value={k}>
                {landName(k)}
              </option>
            ))}
          </select>
        </div>
        <ZahlFeld
          name={t("tochter.kapital")}
          einheit={geldEinheit()}
          wert={kapital}
          onWert={(text) => setKapital(text)}
        />
      </div>
      <fieldset className="auswahlgruppe">
        <legend>{t("tochter.schwerpunkt")}</legend>
        {SCHWERPUNKTE.map((s) => (
          <label key={s}>
            <input
              type="radio"
              name={`${id}-schwerpunkt`}
              checked={schwerpunkt === s}
              onChange={() => setSchwerpunkt(s)}
            />{" "}
            <strong>{t(`tochter.schwerpunkt.${s}`)}</strong>{" "}
            <small className="gedaempft">{t(`tochter.schwerpunkt.${s}_hilfe`)}</small>
          </label>
        ))}
      </fieldset>
      <button
        type="button"
        disabled={!name.trim() || betrag === null || betrag <= 0}
        onClick={() =>
          void los([
            {
              FoundSubsidiary: {
                name: name.trim(),
                country: land,
                capital: geld(ausAnzeige(betrag ?? 0)),
                focus: BEFEHL[schwerpunkt],
              },
            },
          ]).then((ok) => {
            if (ok) {
              setName("");
              onNeu();
            }
          })
        }
      >
        {t("tochter.gruenden")}
      </button>
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

/** One subsidiary with capital moves and its focus. */
function TochterZeile({ x, onNeu }: { x: Tochter; onNeu: () => void }) {
  const { los, antwort } = useAktion(`tochter-${x.company}`);
  const [betrag, setBetrag] = useState("");
  const n = zahlLesen(betrag);
  const bewegen = (vorzeichen: 1 | -1) =>
    void los([
      { MoveCapital: { company: x.company, amount: vorzeichen * geld(ausAnzeige(n ?? 0)) } },
    ]).then((ok) => {
      if (ok) {
        setBetrag("");
        onNeu();
      }
    });
  return (
    <tr>
      <td>
        {x.name}
        <br />
        <small className="gedaempft">
          {landName(x.country)}
          {!x.direct && ` · ${t("tochter.enkel", { mutter: x.parent })}`}
        </small>
      </td>
      <td>
        {x.direct ? (
          <select
            aria-label={t("tochter.schwerpunkt")}
            value={x.focus}
            onChange={(e) =>
              void los([
                {
                  SetSubsidiaryFocus: {
                    company: x.company,
                    focus: BEFEHL[e.target.value as Schwerpunkt],
                  },
                },
              ]).then((ok) => ok && onNeu())
            }
          >
            {SCHWERPUNKTE.map((s) => (
              <option key={s} value={s}>
                {t(`tochter.schwerpunkt.${s}`)}
              </option>
            ))}
          </select>
        ) : (
          t(`tochter.schwerpunkt.${x.focus}`)
        )}
      </td>
      <td className="zahl">{formatGeld(x.cash_usd)}</td>
      <td className="zahl">
        {formatGeld(x.equity_usd)}
        <br />
        <small className="gedaempft">
          {t("tochter.eingezahlt", { betrag: formatGeld(x.paid_in_usd) })}
        </small>
      </td>
      <td className="zahl">
        {formatGeld(x.result_year_usd)}
        <br />
        <small className="gedaempft">
          {t("tochter.umsatz", { betrag: formatGeld(x.revenue_year_usd) })}
        </small>
      </td>
      <td className="zahl">
        {t("tochter.bestand", { standorte: x.sites, fahrzeuge: x.vehicles })}
      </td>
      <td>
        {x.direct && (
          <>
            <div className="formular-zeile">
              <ZahlFeld
                name={t("tochter.betrag")}
                einheit={geldEinheit()}
                wert={betrag}
                onWert={(text) => setBetrag(text)}
                hilfe={t("tochter.ausschuettung_max", {
                  betrag: formatGeld(x.payout_max_usd),
                })}
              />
              <button type="button" disabled={!n || n <= 0} onClick={() => bewegen(1)}>
                {t("tochter.einlegen")}
              </button>
              <button type="button" disabled={!n || n <= 0} onClick={() => bewegen(-1)}>
                {t("tochter.ausschuetten")}
              </button>
            </div>
            <Rueckmeldung meldung={antwort} />
          </>
        )}
      </td>
    </tr>
  );
}

/** Sites of the player and its direct subsidiaries, to move within the group. */
function Standorte({ daten, onNeu }: { daten: Konzern; onNeu: () => void }) {
  const { los, antwort } = useAktion("konzern-standort");
  const id = useId();
  const firmen = [
    { company: daten.company, name: daten.company_name },
    ...daten.subsidiaries
      .filter((x) => x.direct)
      .map((x) => ({ company: x.company, name: x.name })),
  ];
  const [ziel, setZiel] = useState<Record<number, number>>({});
  if (daten.sites.length === 0) return null;
  return (
    <section aria-labelledby={`${id}-titel`}>
      <h2 id={`${id}-titel`}>{t("tochter.standorte_titel")}</h2>
      <p className="feld-hilfe">{t("tochter.standorte_hilfe")}</p>
      <div className="tabelle">
        <table aria-label={t("tochter.standorte_titel")}>
          <thead>
            <tr>
              <th>{t("tochter.standort")}</th>
              <th>{t("tochter.gehoert")}</th>
              <th className="zahl">{t("tochter.buchwert")}</th>
              <th>{t("tochter.an")}</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {daten.sites.map((s) => {
              const andere = firmen.filter((f) => f.company !== s.company);
              const wahl = ziel[s.site] ?? andere[0]?.company;
              return (
                <tr key={s.site}>
                  <td>
                    {t(s.kind_text)} {landName(s.country)}
                  </td>
                  <td>{s.company_name}</td>
                  <td className="zahl">{formatGeld(s.book_usd)}</td>
                  <td>
                    <select
                      aria-label={t("tochter.an")}
                      value={wahl}
                      onChange={(e) => setZiel((z) => ({ ...z, [s.site]: Number(e.target.value) }))}
                    >
                      {andere.map((f) => (
                        <option key={f.company} value={f.company}>
                          {f.name}
                        </option>
                      ))}
                    </select>
                  </td>
                  <td>
                    <button
                      type="button"
                      disabled={wahl === undefined}
                      onClick={() =>
                        void los([{ TransferSite: { site: s.site, to: wahl ?? 0 } }]).then(
                          (ok) => ok && onNeu(),
                        )
                      }
                    >
                      {t("tochter.uebertragen")}
                    </button>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

function Zeilen({
  titel,
  zeilen,
  summe,
}: {
  titel: string;
  zeilen: KonzernZeile[];
  summe?: number;
}) {
  return (
    <div className="tabelle">
      <table className="rechnung-tabelle" aria-label={titel}>
        <caption>{titel}</caption>
        <tbody>
          {zeilen.map((z) => (
            <tr key={z.key}>
              <th scope="row">{t(z.key)}</th>
              <td className="zahl">{formatGeld(z.usd)}</td>
            </tr>
          ))}
          {summe !== undefined && (
            <tr className="summe">
              <th scope="row">{t("tochter.summe")}</th>
              <td className="zahl">{formatGeld(summe)}</td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}

/** Organisation → subsidiaries (W6): founding, the subsidiaries, sites, the group. */
export function TochterfirmenAnsicht({ kern, stand }: { kern: Kern; stand: string }) {
  const { daten, fehler, neu } = useSicht(() => kern.konzern(), stand);
  if (!daten) return <FehlerText fehler={fehler} />;
  if (!daten.enabled) return <p>{t("tochter.keine")}</p>;
  return (
    <div id="tochterfirmen">
      <p className="feld-hilfe">{t("tochter.hilfe")}</p>
      {daten.subsidiaries.length === 0 ? (
        <p>{t("tochter.leer")}</p>
      ) : (
        <div className="tabelle">
          <table aria-label={t("tochter.liste")}>
            <thead>
              <tr>
                <th>{t("tochter.firma")}</th>
                <th>{t("tochter.schwerpunkt")}</th>
                <th className="zahl">{t("tochter.kasse")}</th>
                <th className="zahl">{t("tochter.eigenkapital")}</th>
                <th className="zahl">{t("tochter.ergebnis")}</th>
                <th className="zahl">{t("tochter.besitz")}</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {daten.subsidiaries.map((x) => (
                <TochterZeile key={x.company} x={x} onNeu={neu} />
              ))}
            </tbody>
          </table>
        </div>
      )}
      <Gruendung daten={daten} onNeu={neu} />
      <Standorte daten={daten} onNeu={neu} />
      <h2>{t("tochter.konzern_titel")}</h2>
      <p className="feld-hilfe">
        {t("tochter.konzern_hilfe", {
          eigen: formatGeld(daten.own_equity_usd),
          konzern: formatGeld(daten.group_equity_usd),
        })}
      </p>
      <div className="formular-zeile">
        <Zeilen titel={t("tochter.aktiva")} zeilen={daten.assets} summe={daten.total_assets_usd} />
        <Zeilen
          titel={t("tochter.passiva")}
          zeilen={daten.liabilities}
          summe={daten.total_assets_usd}
        />
        <Zeilen titel={t("tochter.guv")} zeilen={daten.income} summe={daten.result_year_usd} />
      </div>
    </div>
  );
}
