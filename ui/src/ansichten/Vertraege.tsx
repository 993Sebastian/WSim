import { useEffect, useId, useState } from "react";
import {
  ausAnzeige,
  formatDatum,
  formatGeld,
  formatMenge,
  formatProzent,
  geldEinheit,
  geldFeld,
  landName,
  zahlFeld,
  zahlLesen,
} from "../format";
import {
  geld,
  type Kern,
  type Vertraege,
  type Vertrag,
  type Vertragspartnerliste,
  type VertragsStandort,
} from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { fehlerText } from "./fehler";
import { Rueckmeldung, useAktion, useSicht, ZahlFeld } from "./gemeinsam";

const einheit = (u: string) => t(`einheit.${u}`);

const standortName = (s: VertragsStandort) => `${t(s.kind_text)} ${landName(s.country)}`;

/** One contract with what the player can do with it. */
function VertragZeile({ v, onNeu }: { v: Vertrag; onNeu: () => void }) {
  const { los, antwort } = useAktion(`vertrag-${v.id}`);
  const menge = (m: number) => `${formatMenge(m)} ${einheit(v.unit)}`;
  const zeit =
    v.start && v.end
      ? t("vertrag.zeitraum", { von: formatDatum(v.start), bis: formatDatum(v.end) })
      : t("vertrag.monate", { anzahl: v.months });
  return (
    <tr>
      <td>
        {t(`produkt.${v.product}`)}
        <br />
        <small className="gedaempft">{t(`vertrag.rolle.${v.role}`)}</small>
      </td>
      <td>
        {v.partner}
        <br />
        <small className="gedaempft">
          {v.role === "verkauf"
            ? `${landName(v.own_country)} → ${landName(v.partner_country)}`
            : `${landName(v.partner_country)} → ${landName(v.own_country)}`}
        </small>
      </td>
      <td className="zahl">{menge(v.per_month)}</td>
      <td className="zahl">
        {formatGeld(v.price_usd)}
        <br />
        <small className="gedaempft">
          {t("vertrag.marktpreis", { preis: formatGeld(v.market_price_usd) })}
        </small>
      </td>
      <td>{zeit}</td>
      <td className="zahl">
        {v.status === "laufend" ? (
          <>
            {menge(v.delivered_month)}
            <br />
            <small className="gedaempft">
              {t("vertrag.gesamt", { menge: menge(v.delivered_total) })}
            </small>
          </>
        ) : (
          menge(v.delivered_total)
        )}
      </td>
      <td>
        {t(`vertrag.status.${v.status}`)}
        {(v.penalties_paid_usd > 0 || v.penalties_received_usd > 0) && (
          <>
            <br />
            <small className="gedaempft">
              {t("vertrag.strafen", {
                gezahlt: formatGeld(v.penalties_paid_usd),
                erhalten: formatGeld(v.penalties_received_usd),
              })}
            </small>
          </>
        )}
      </td>
      <td>
        {v.answer && (
          <div className="knopfreihe">
            <button
              type="button"
              className="haupt"
              onClick={() =>
                void los([{ AnswerContract: { contract: v.id, accept: true } }]).then(
                  (ok) => ok && onNeu(),
                )
              }
            >
              {t("vertrag.annehmen")}
            </button>
            <button
              type="button"
              onClick={() =>
                void los([{ AnswerContract: { contract: v.id, accept: false } }]).then(
                  (ok) => ok && onNeu(),
                )
              }
            >
              {t("vertrag.ablehnen")}
            </button>
          </div>
        )}
        {v.can_cancel && !v.answer && (
          <button
            type="button"
            onClick={() =>
              void los([{ CancelContract: { contract: v.id } }]).then((ok) => ok && onNeu())
            }
          >
            {v.status === "angeboten"
              ? t("vertrag.zurueckziehen")
              : t("vertrag.kuendigen", { strafe: formatGeld(v.cancel_fee_usd) })}
          </button>
        )}
        <Rueckmeldung meldung={antwort} />
      </td>
    </tr>
  );
}

/** A new contract: own site, product, partner, then the terms. */
function NeuerVertrag({
  kern,
  daten,
  stand,
  onNeu,
}: {
  kern: Kern;
  daten: Vertraege;
  stand: string;
  onNeu: () => void;
}) {
  const id = useId();
  const { los, antwort } = useAktion("vertrag-neu");
  const [standort, setStandort] = useState<number | null>(daten.sites[0]?.site ?? null);
  const s: VertragsStandort | undefined = daten.sites.find((x) => x.site === standort);
  const produkte = s ? [...new Set([...s.sells, ...s.buys])] : [];
  const [produkt, setProdukt] = useState<string>(produkte[0] ?? "");
  const [liste, setListe] = useState<Vertragspartnerliste | null>(null);
  const [fehler, setFehler] = useState<string | null>(null);
  const [partner, setPartner] = useState<number | null>(null);
  const [menge, setMenge] = useState("");
  const [preis, setPreis] = useState("");
  const [monate, setMonate] = useState(String(daten.months_default));
  const [qualitaet, setQualitaet] = useState("0");
  const [strafe, setStrafe] = useState(zahlFeld(daten.penalty_default * 100, 1));

  useEffect(() => {
    if (standort === null || !produkt) return;
    let aktiv = true;
    kern
      .vertragspartner(standort, produkt)
      .then((l) => {
        if (!aktiv) return;
        setListe(l);
        setFehler(null);
        const erster = l.partners[0];
        setPartner(erster?.site ?? null);
        if (erster) {
          const wunsch = l.own_per_month > 0 ? 0.5 * l.own_per_month : erster.free_per_month;
          setMenge(zahlFeld(Math.min(erster.free_per_month, wunsch), 1));
          setPreis(geldFeld(erster.suggested_price_usd));
        }
      })
      .catch((e: unknown) => aktiv && setFehler(fehlerText(e)));
    return () => {
      aktiv = false;
    };
  }, [kern, standort, produkt, stand]);

  if (daten.sites.length === 0) return <p>{t("vertrag.keine_standorte")}</p>;
  const p = liste?.partners.find((x) => x.site === partner);
  const werte = {
    menge: zahlLesen(menge),
    preis: zahlLesen(preis),
    monate: zahlLesen(monate),
    qualitaet: zahlLesen(qualitaet),
    strafe: zahlLesen(strafe),
  };
  const gueltig =
    liste !== null &&
    partner !== null &&
    Object.values(werte).every((w) => w !== null) &&
    (werte.menge ?? 0) > 0;
  const anbieten = () => {
    if (!gueltig || !liste || partner === null || standort === null) return;
    const verkauf = liste.role === "verkauf";
    void los(
      [
        {
          ProposeContract: {
            seller: verkauf ? standort : partner,
            buyer: verkauf ? partner : standort,
            product: produkt,
            per_month: werte.menge ?? 0,
            price: geld(ausAnzeige(werte.preis ?? 0)),
            months: Math.round(werte.monate ?? 0),
            min_quality: werte.qualitaet ?? 0,
            penalty: (werte.strafe ?? 0) / 100,
          },
        },
      ],
      t("vertrag.abgeschlossen"),
    ).then((ok) => ok && onNeu());
  };
  const e = liste ? einheit(liste.unit) : "";
  return (
    <section aria-labelledby={`${id}-titel`}>
      <h3 id={`${id}-titel`}>{t("vertrag.neu")}</h3>
      <p className="feld-hilfe">{t("vertrag.neu_hilfe")}</p>
      <div className="formular-zeile">
        <div className="feld">
          <label htmlFor={`${id}-standort`}>{t("vertrag.standort")}</label>
          <select
            id={`${id}-standort`}
            value={standort ?? ""}
            onChange={(ev) => {
              const neu = Number(ev.target.value);
              setStandort(neu);
              const n = daten.sites.find((x) => x.site === neu);
              setProdukt(n ? ([...n.sells, ...n.buys][0] ?? "") : "");
            }}
          >
            {daten.sites.map((x) => (
              <option key={x.site} value={x.site}>
                {standortName(x)}
              </option>
            ))}
          </select>
        </div>
        <div className="feld">
          <label htmlFor={`${id}-produkt`}>{t("vertrag.produkt")}</label>
          <select
            id={`${id}-produkt`}
            value={produkt}
            onChange={(ev) => setProdukt(ev.target.value)}
          >
            {produkte.map((k) => (
              <option key={k} value={k}>
                {t(`produkt.${k}`)} ·{" "}
                {s?.sells.includes(k) ? t("vertrag.rolle.verkauf") : t("vertrag.rolle.einkauf")}
              </option>
            ))}
          </select>
        </div>
      </div>
      <FehlerText fehler={fehler} />
      {liste && liste.partners.length === 0 && <p>{t(`vertrag.keine_partner.${liste.role}`)}</p>}
      {liste && liste.partners.length > 0 && (
        <>
          <div className="feld">
            <label htmlFor={`${id}-partner`}>
              {liste.role === "verkauf" ? t("vertrag.abnehmer") : t("vertrag.lieferant")}
            </label>
            <select
              id={`${id}-partner`}
              value={partner ?? ""}
              onChange={(ev) => {
                const neu = Number(ev.target.value);
                setPartner(neu);
                const x = liste.partners.find((y) => y.site === neu);
                if (x) setPreis(geldFeld(x.suggested_price_usd));
              }}
            >
              {liste.partners.map((x) => (
                <option key={x.site} value={x.site}>
                  {t("vertrag.partner_wahl", {
                    firma: x.company,
                    land: landName(x.country),
                    menge: `${formatMenge(x.free_per_month)} ${e}`,
                    preis: formatGeld(x.suggested_price_usd),
                  })}
                </option>
              ))}
            </select>
            {p && p.delivery_cost_usd > 0 && (
              <small className="feld-hilfe">
                {t("vertrag.fracht", { kosten: `${formatGeld(p.delivery_cost_usd)}/${e}` })}
              </small>
            )}
          </div>
          <div className="formular-zeile">
            <ZahlFeld
              name={t("vertrag.menge")}
              einheit={`${e}/${t("vertrag.monat")}`}
              wert={menge}
              onWert={setMenge}
              hilfe={
                liste.own_per_month > 0
                  ? t(`vertrag.eigen.${liste.role}`, {
                      menge: `${formatMenge(liste.own_per_month)} ${e}`,
                    })
                  : undefined
              }
            />
            <ZahlFeld
              name={t("vertrag.preis")}
              einheit={`${geldEinheit()}/${e}`}
              wert={preis}
              onWert={setPreis}
              hilfe={t("vertrag.preis_hilfe")}
            />
            <ZahlFeld
              name={t("vertrag.laufzeit")}
              einheit={t("vertrag.monate_einheit")}
              ganzzahlig
              wert={monate}
              onWert={setMonate}
            />
            <ZahlFeld name={t("vertrag.qualitaet")} wert={qualitaet} onWert={setQualitaet} />
            <ZahlFeld
              name={t("vertrag.strafe")}
              einheit="%"
              wert={strafe}
              onWert={setStrafe}
              hilfe={t("vertrag.strafe_hilfe", {
                max: formatProzent(daten.penalty_max),
                monate: daten.cancel_months,
              })}
            />
          </div>
          <div className="knopfreihe">
            <button type="button" className="haupt" disabled={!gueltig} onClick={anbieten}>
              {t("vertrag.anbieten")}
            </button>
          </div>
          <Rueckmeldung meldung={antwort} />
        </>
      )}
    </section>
  );
}

/** Market → supply contracts (W4): running contracts, proposals, a new contract. */
export function VertraegeAnsicht({ kern, stand }: { kern: Kern; stand: string }) {
  const { daten, fehler, neu } = useSicht(() => kern.vertraege(), stand);
  if (!daten) return <FehlerText fehler={fehler} />;
  if (!daten.enabled) return <p>{t("vertrag.keine")}</p>;
  return (
    <div id="vertraege">
      <p className="feld-hilfe">{t("vertrag.hilfe")}</p>
      {daten.contracts.length === 0 ? (
        <p>{t("vertrag.leer")}</p>
      ) : (
        <div className="tabelle">
          <table aria-label={t("vertrag.liste")}>
            <thead>
              <tr>
                <th>{t("vertrag.produkt")}</th>
                <th>{t("vertrag.partner")}</th>
                <th className="zahl">{t("vertrag.menge")}</th>
                <th className="zahl">{t("vertrag.preis")}</th>
                <th>{t("vertrag.laufzeit")}</th>
                <th className="zahl">{t("vertrag.geliefert")}</th>
                <th>{t("vertrag.stand")}</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {daten.contracts.map((v) => (
                <VertragZeile key={v.id} v={v} onNeu={neu} />
              ))}
            </tbody>
          </table>
        </div>
      )}
      <NeuerVertrag kern={kern} daten={daten} stand={stand} onNeu={neu} />
    </div>
  );
}
