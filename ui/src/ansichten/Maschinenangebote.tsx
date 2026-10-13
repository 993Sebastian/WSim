/** Machine tenders (MAA): asking for offers, comparing them, taking or turning them down. */

import { useId } from "react";
import { formatDatum, formatGeld, formatPreis, formatZahl } from "../format";
import type { MaschinenAngebot, MaschinenAusschreibung, Produktion } from "../kern";
import { t } from "../texte";
import { useAktion, Rueckmeldung } from "./gemeinsam";

const produktName = (p: string) => t(`produkt.${p}`);

/** Signed percent against 1.0: "+19 %", "−7 %", "±0 %". */
export function abweichung(faktor: number): string {
  const p = Math.round((faktor - 1) * 100);
  if (p === 0) return "±0 %";
  return `${p > 0 ? "+" : "−"}${Math.abs(p)} %`;
}

/** Colour class of a deviation: `gut` when it helps the buyer. */
function farbe(faktor: number, hoeherIstGut: boolean): string {
  const p = Math.round((faktor - 1) * 100);
  if (p === 0) return "";
  return p > 0 === hoeherIstGut ? "positiv" : "negativ";
}

function qm(wert: number): string {
  return `${formatZahl(wert, wert < 100 ? 1 : 0)} m²`;
}

function Angebot({
  a,
  ausschreibung,
  produktion,
  hatFlaeche,
}: {
  a: MaschinenAngebot;
  ausschreibung: MaschinenAusschreibung;
  produktion: Produktion;
  hatFlaeche: boolean;
}) {
  const { los, antwort } = useAktion(`angebot/${ausschreibung.id}/${a.id}`);
  const einheit = t(`einheit.${produktion.units[ausschreibung.product] ?? "t"}`);
  const preis = a.catalog_price_usd > 0 ? a.price_usd / a.catalog_price_usd : 1;
  const kosten = a.catalog_unit_cost_usd > 0 ? a.unit_cost_usd / a.catalog_unit_cost_usd : 1;
  const passt = a.free_after_m2 === null || a.free_after_m2 >= 0;
  const zelle = (name: string, klasse?: string) => ({
    "data-spalte": name,
    className: klasse ? `zahl ${klasse}` : "zahl",
  });
  return (
    <tr aria-label={t(`maschinenbauer.${a.builder}`)}>
      <td>
        {t(`maschinenbauer.${a.builder}`)}
        <small className="block gedaempft">{t(`land.${a.builder_country}`)}</small>
      </td>
      <td {...zelle(t("angebot.preis"), farbe(preis, false))}>
        {formatGeld(a.price_usd)} <small>({abweichung(preis)})</small>
      </td>
      <td
        {...zelle(
          t("angebot.bauzeit"),
          farbe(a.catalog_build_days > 0 ? a.build_days / a.catalog_build_days : 1, false),
        )}
      >
        {t("angebot.bauzeit_wert", { tage: a.build_days })}
        <small className="block gedaempft">{formatDatum(a.ready)}</small>
      </td>
      <td {...zelle(t("angebot.ausstoss"), farbe(a.factors.output, true))}>
        {abweichung(a.factors.output)}
      </td>
      <td {...zelle(t("angebot.arbeit"), farbe(a.factors.labor, false))}>
        {abweichung(a.factors.labor)}
      </td>
      <td {...zelle(t("angebot.vorprodukte"), farbe(a.factors.inputs, false))}>
        {abweichung(a.factors.inputs)}
      </td>
      <td {...zelle(t("angebot.ausschuss"), farbe(a.factors.scrap, false))}>
        {abweichung(a.factors.scrap)}
      </td>
      <td {...zelle(t("angebot.flaeche"), farbe(a.factors.area, false))}>
        {qm(a.area_unit_m2)} <small>({abweichung(a.factors.area)})</small>
        {ausschreibung.count > 1 && (
          <small className="block gedaempft">
            {t("angebot.flaeche_gesamt", { gesamt: qm(a.area_total_m2) })}
          </small>
        )}
      </td>
      {hatFlaeche && (
        <td {...zelle(t("angebot.frei"), passt ? "" : "negativ")}>
          {a.free_before_m2 !== null && a.free_after_m2 !== null
            ? t("angebot.frei_wert", {
                vorher: qm(Math.max(a.free_before_m2, 0)),
                nachher: passt ? qm(a.free_after_m2) : t("angebot.zu_klein"),
              })
            : "–"}
        </td>
      )}
      <td {...zelle(t("angebot.stueckkosten"), farbe(kosten, false))}>
        {formatPreis(a.unit_cost_usd, einheit)} <small>({abweichung(kosten)})</small>
        <details className="material">
          <summary>
            {t("angebot.material", { betrag: formatPreis(a.material_usd, einheit) })}
          </summary>
          <ul>
            {a.materials.map((m) => (
              <li key={m.product}>
                {t("angebot.material_zeile", {
                  produkt: produktName(m.product),
                  menge: formatZahl(m.quantity, 3),
                  preis: formatGeld(m.price_usd),
                })}
              </li>
            ))}
          </ul>
          <p className="erklaerung">{t("angebot.material_hinweis")}</p>
        </details>
      </td>
      <td data-spalte={t("angebot.gueltig")}>{formatDatum(a.valid_until)}</td>
      <td data-spalte={t("angebot.aktion")}>
        <div className="knopfreihe links">
          <button
            type="button"
            className="haupt"
            disabled={!passt}
            onClick={() =>
              void los(
                [{ AcceptMachineOffer: { tender: ausschreibung.id, offer: a.id } }],
                t("angebot.angenommen", { bauzeit: a.build_days }),
              )
            }
          >
            {t("angebot.annehmen")}
          </button>
          <button
            type="button"
            onClick={() =>
              void los(
                [{ DeclineMachineOffer: { tender: ausschreibung.id, offer: a.id } }],
                t("angebot.abgelehnt"),
              )
            }
          >
            {t("angebot.ablehnen")}
          </button>
        </div>
        <Rueckmeldung meldung={antwort} />
      </td>
    </tr>
  );
}

function Ausschreibung({ a, produktion }: { a: MaschinenAusschreibung; produktion: Produktion }) {
  const { los, antwort } = useAktion(`ausschreibung/${a.id}`);
  const id = useId();
  const titel =
    a.kind === "bauen"
      ? t("ausschreibung.bauen", {
          produkt: produktName(a.product),
          anzahl: a.count,
          groesse: t(`anlagengroesse.${a.size}`),
        })
      : t("ausschreibung.umbauen", { produkt: produktName(a.product) });
  const laeuft = a.day <= a.days;
  const hatFlaeche = a.offers.some((o) => o.free_before_m2 !== null && o.free_after_m2 !== null);
  return (
    <section className="karte ausschreibung" aria-labelledby={id}>
      <h3 id={id}>{titel}</h3>
      <p className="gedaempft">
        {laeuft
          ? t("ausschreibung.laeuft", { tag: a.day, tage: a.days, ende: formatDatum(a.ends) })
          : t("ausschreibung.beendet")}
        {" · "}
        {t("ausschreibung.angebote", { anzahl: a.offers.length })}
      </p>
      {a.offers.length === 0 && <p className="gedaempft">{t("ausschreibung.noch_keine")}</p>}
      {a.offers.length > 0 && (
        <div className="tabelle">
          <table className="mobil-karten angebote" aria-label={titel}>
            <thead>
              <tr>
                <th>{t("angebot.hersteller")}</th>
                <th className="zahl">{t("angebot.k_preis")}</th>
                <th className="zahl">{t("angebot.k_bauzeit")}</th>
                <th className="zahl">{t("angebot.k_ausstoss")}</th>
                <th className="zahl">{t("angebot.k_arbeit")}</th>
                <th className="zahl">{t("angebot.k_material")}</th>
                <th className="zahl">{t("angebot.k_ausschuss")}</th>
                <th className="zahl">{t("angebot.k_flaeche")}</th>
                {hatFlaeche && <th className="zahl">{t("angebot.k_frei")}</th>}
                <th className="zahl">{t("angebot.k_stueckkosten")}</th>
                <th>{t("angebot.k_gueltig")}</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {a.offers.map((o) => (
                <Angebot
                  key={o.id}
                  a={o}
                  ausschreibung={a}
                  produktion={produktion}
                  hatFlaeche={hatFlaeche}
                />
              ))}
            </tbody>
          </table>
        </div>
      )}
      <div className="knopfreihe links">
        <button
          type="button"
          onClick={() =>
            void los([{ CancelMachineTender: { tender: a.id } }], t("ausschreibung.abgebrochen"))
          }
        >
          {t("ausschreibung.abbrechen")}
        </button>
      </div>
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

/** The tenders of a site with their offers. */
export function MaschinenAusschreibungen({
  site,
  produktion,
}: {
  site: number;
  produktion: Produktion;
}) {
  const liste = (produktion.tenders ?? []).filter((x) => x.site === site);
  if (liste.length === 0) return null;
  return (
    <div className="ausschreibungen">
      {liste.map((a) => (
        <Ausschreibung key={a.id} a={a} produktion={produktion} />
      ))}
    </div>
  );
}
