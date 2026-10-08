import { useId, useState } from "react";
import { formatGeld, formatMenge, formatProzent, zahlLesen } from "../format";
import {
  type Fahrzeugangebot,
  type Flottenbestand,
  type Kern,
  type Logistik,
  type Logistikmonat,
} from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { Rueckmeldung, useAktion, useSicht, ZahlFeld } from "./gemeinsam";

type Weg = Logistik["mode"];

const BEFEHL: Record<Weg, "Market" | "State" | "Fleet"> = {
  markt: "Market",
  staat: "State",
  flotte: "Fleet",
};

const fahrzeug = (key: string) => t(`verkehrsmittel.${key}`);
const tkm = (wert: number) => `${formatMenge(wert)} tkm`;
const promille = (anteil: number) => `${formatMenge(anteil * 1000)} ‰`;

/** What a run costs against the market; vehicles dearer than the market stay at home. */
function Fahrtkosten({ anteil }: { anteil: number | null }) {
  if (anteil === null) return <>–</>;
  return (
    <>
      {formatProzent(anteil)}
      {anteil >= 1 && (
        <>
          <br />
          <small className="gedaempft">{t("logistik.teurer")}</small>
        </>
      )}
    </>
  );
}

/** The way of the own loads and whether the fleet carries for others. */
function WegWahl({ daten, onNeu }: { daten: Logistik; onNeu: () => void }) {
  const { los, antwort } = useAktion("logistik-weg");
  const [weg, setWeg] = useState<Weg>(daten.mode);
  const [fremd, setFremd] = useState(daten.carry_for_others);
  const id = useId();
  const geaendert = weg !== daten.mode || fremd !== daten.carry_for_others;
  const text: Record<Weg, string> = {
    markt: t("logistik.weg.markt_hilfe"),
    staat: t("logistik.weg.staat_hilfe", {
      aufschlag: formatProzent(daten.state_surcharge),
      faktor: formatProzent(daten.state_risk_factor),
    }),
    flotte: t("logistik.weg.flotte_hilfe", { marge: formatProzent(daten.market_margin) }),
  };
  return (
    <section aria-labelledby={`${id}-titel`}>
      <h2 id={`${id}-titel`}>{t("logistik.weg_titel")}</h2>
      <fieldset className="auswahlgruppe">
        <legend>{t("logistik.weg_frage")}</legend>
        {(["markt", "staat", "flotte"] as const).map((w) => (
          <label key={w}>
            <input
              type="radio"
              name={`${id}-weg`}
              value={w}
              checked={weg === w}
              onChange={() => setWeg(w)}
            />
            <span>
              <strong>{t(`logistik.weg.${w}`)}</strong>
              <br />
              <small className="gedaempft">{text[w]}</small>
            </span>
          </label>
        ))}
      </fieldset>
      <label>
        <input type="checkbox" checked={fremd} onChange={(e) => setFremd(e.target.checked)} />{" "}
        {t("logistik.fremdfracht", { anteil: formatProzent(daten.rental_share) })}
      </label>
      <p className="feld-hilfe">
        {t("logistik.risiko", {
          land: promille(daten.risk_land),
          see: promille(daten.risk_sea),
        })}
      </p>
      <button
        type="button"
        disabled={!geaendert}
        onClick={() =>
          void los([{ SetLogistics: { mode: BEFEHL[weg], carry_for_others: fremd } }]).then(
            (ok) => ok && onNeu(),
          )
        }
      >
        {t("logistik.uebernehmen")}
      </button>
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

/** Vehicles of one kind in the fleet, with selling. */
function BestandZeile({ b, onNeu }: { b: Flottenbestand; onNeu: () => void }) {
  const { los, antwort } = useAktion(`flotte-${b.vehicle}`);
  const [anzahl, setAnzahl] = useState("1");
  const n = zahlLesen(anzahl);
  return (
    <tr>
      <td>
        {fahrzeug(b.vehicle)}
        <br />
        <small className="gedaempft">{t(`logistik.wegart.${b.way}`)}</small>
      </td>
      <td className="zahl">{b.count}</td>
      <td className="zahl">{tkm(b.capacity_tkm)}</td>
      <td className="zahl">
        {b.capacity_tkm > 0 ? formatProzent(b.used_tkm / b.capacity_tkm) : "–"}
        <br />
        <small className="gedaempft">
          {t("logistik.vormonat_wert", { wert: tkm(b.used_last_tkm) })}
        </small>
      </td>
      <td className="zahl">
        <Fahrtkosten anteil={b.running_share} />
      </td>
      <td className="zahl">
        {formatGeld(b.book_value_usd)}
        <br />
        <small className="gedaempft">
          {t("logistik.erloes_je", { betrag: formatGeld(b.sale_usd) })}
        </small>
      </td>
      <td>
        <div className="formular-zeile">
          <ZahlFeld
            name={t("logistik.anzahl")}
            ganzzahlig
            wert={anzahl}
            onWert={(text) => setAnzahl(text)}
          />
          <button
            type="button"
            disabled={!n || n < 1}
            onClick={() =>
              void los([{ SellVehicles: { vehicle: b.vehicle, count: n ?? 0 } }]).then(
                (ok) => ok && onNeu(),
              )
            }
          >
            {t("logistik.verkaufen")}
          </button>
        </div>
        <Rueckmeldung meldung={antwort} />
      </td>
    </tr>
  );
}

/** A vehicle to buy. */
function AngebotZeile({ f, onNeu }: { f: Fahrzeugangebot; onNeu: () => void }) {
  const { los, antwort } = useAktion(`fahrzeug-${f.vehicle}`);
  const [anzahl, setAnzahl] = useState("1");
  const n = zahlLesen(anzahl);
  return (
    <tr>
      <td>
        {fahrzeug(f.vehicle)}
        <br />
        <small className="gedaempft">
          {t(`logistik.wegart.${f.way}`)} ·{" "}
          {f.classes.map((c) => t(`transportklasse.${c}`)).join(", ")}
        </small>
      </td>
      <td className="zahl">
        {formatMenge(f.payload_t)} t
        <br />
        <small className="gedaempft">
          {t("logistik.km_tag", { km: formatMenge(f.km_per_day) })}
        </small>
      </td>
      <td className="zahl">{tkm(f.capacity_tkm)}</td>
      <td className="zahl">
        {formatGeld(f.price_usd)}
        <br />
        <small className="gedaempft">
          {t("logistik.je_monat", { betrag: formatGeld(f.monthly_cost_usd) })}
        </small>
      </td>
      <td className="zahl">
        <Fahrtkosten anteil={f.running_share} />
      </td>
      <td>
        <div className="formular-zeile">
          <ZahlFeld
            name={t("logistik.anzahl")}
            ganzzahlig
            wert={anzahl}
            onWert={(text) => setAnzahl(text)}
          />
          <button
            type="button"
            disabled={!n || n < 1}
            onClick={() =>
              void los([{ BuyVehicles: { vehicle: f.vehicle, count: n ?? 0 } }]).then(
                (ok) => ok && onNeu(),
              )
            }
          >
            {t("logistik.kaufen")}
          </button>
        </div>
        <Rueckmeldung meldung={antwort} />
      </td>
    </tr>
  );
}

function Monat({ titel, m }: { titel: string; m: Logistikmonat }) {
  const zeilen: [string, string][] = [
    [t("logistik.zahl.flotte"), tkm(m.fleet_tkm)],
    [t("logistik.zahl.markt"), tkm(m.market_tkm)],
    [t("logistik.zahl.staat"), tkm(m.state_tkm)],
    [t("logistik.zahl.fremd"), formatGeld(m.rental_usd)],
    [t("logistik.zahl.unterhalt"), formatGeld(m.upkeep_usd)],
    [t("logistik.zahl.abschreibung"), formatGeld(m.depreciation_usd)],
    [
      t("logistik.zahl.verluste"),
      t("logistik.verluste_wert", { anzahl: m.losses, wert: formatGeld(m.lost_value_usd) }),
    ],
  ];
  return (
    <div className="tabelle">
      <table className="rechnung-tabelle" aria-label={titel}>
        <caption>{titel}</caption>
        <tbody>
          {zeilen.map(([k, v]) => (
            <tr key={k}>
              <th scope="row">{k}</th>
              <td className="zahl">{v}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/** Market → logistics (W5): way of the own loads, fleet, vehicles to buy, figures. */
export function LogistikAnsicht({ kern, stand }: { kern: Kern; stand: string }) {
  const { daten, fehler, neu } = useSicht(() => kern.logistik(), stand);
  if (!daten) return <FehlerText fehler={fehler} />;
  if (!daten.enabled) return <p>{t("logistik.keine")}</p>;
  return (
    <div id="logistik">
      <p className="feld-hilfe">{t("logistik.hilfe")}</p>
      {/* A new view restarts the choice from what holds now. */}
      <WegWahl key={`${daten.mode}-${daten.carry_for_others}`} daten={daten} onNeu={neu} />
      <h2>{t("logistik.flotte_titel")}</h2>
      {daten.fleet.length === 0 ? (
        <p>{t("logistik.flotte_leer")}</p>
      ) : (
        <div className="tabelle">
          <table aria-label={t("logistik.flotte_titel")}>
            <thead>
              <tr>
                <th>{t("logistik.fahrzeug")}</th>
                <th className="zahl">{t("logistik.anzahl")}</th>
                <th className="zahl">{t("logistik.kapazitaet")}</th>
                <th className="zahl">{t("logistik.genutzt")}</th>
                <th className="zahl">{t("logistik.fahrtkosten")}</th>
                <th className="zahl">{t("logistik.buchwert")}</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {daten.fleet.map((b) => (
                <BestandZeile key={b.vehicle} b={b} onNeu={neu} />
              ))}
            </tbody>
          </table>
        </div>
      )}
      <h2>{t("logistik.kauf_titel")}</h2>
      <p className="feld-hilfe">{t("logistik.kauf_hilfe")}</p>
      <div className="tabelle">
        <table aria-label={t("logistik.kauf_titel")}>
          <thead>
            <tr>
              <th>{t("logistik.fahrzeug")}</th>
              <th className="zahl">{t("logistik.nutzlast")}</th>
              <th className="zahl">{t("logistik.kapazitaet")}</th>
              <th className="zahl">{t("logistik.preis")}</th>
              <th className="zahl">{t("logistik.fahrtkosten")}</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {daten.vehicles.map((f) => (
              <AngebotZeile key={f.vehicle} f={f} onNeu={neu} />
            ))}
          </tbody>
        </table>
      </div>
      <h2>{t("logistik.zahlen_titel")}</h2>
      <div className="formular-zeile">
        <Monat titel={t("logistik.dieser_monat")} m={daten.month} />
        <Monat titel={t("logistik.vormonat")} m={daten.last_month} />
      </div>
    </div>
  );
}
