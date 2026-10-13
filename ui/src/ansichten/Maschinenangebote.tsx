/** Machine tenders (MAA): asking for offers, comparing them, taking or turning them down. */

import { useId, type ReactNode } from "react";
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

function Kennzahl({
  name,
  wert,
  klasse,
}: {
  name: string;
  wert: ReactNode;
  klasse?: string;
}): ReactNode {
  return (
    <div className="kennzahl">
      <dt>{name}</dt>
      <dd className={klasse}>{wert}</dd>
    </div>
  );
}

function qm(wert: number): string {
  return `${formatZahl(wert, wert < 100 ? 1 : 0)} m²`;
}

function Angebot({
  a,
  ausschreibung,
  produktion,
}: {
  a: MaschinenAngebot;
  ausschreibung: MaschinenAusschreibung;
  produktion: Produktion;
}) {
  const { los, antwort } = useAktion(`angebot/${ausschreibung.id}/${a.id}`);
  const einheit = t(`einheit.${produktion.units[ausschreibung.product] ?? "t"}`);
  const preis = a.catalog_price_usd > 0 ? a.price_usd / a.catalog_price_usd : 1;
  const kosten = a.catalog_unit_cost_usd > 0 ? a.unit_cost_usd / a.catalog_unit_cost_usd : 1;
  const passt = a.free_after_m2 === null || a.free_after_m2 >= 0;
  return (
    <li className="angebot" aria-label={t(`maschinenbauer.${a.builder}`)}>
      <h4>
        {t(`maschinenbauer.${a.builder}`)}
        <small className="gedaempft"> · {t(`land.${a.builder_country}`)}</small>
      </h4>
      <dl className="kennzahlen">
        <Kennzahl
          name={t("angebot.preis")}
          wert={
            <>
              {formatGeld(a.price_usd)} <small>({abweichung(preis)})</small>
            </>
          }
          klasse={farbe(preis, false)}
        />
        <Kennzahl
          name={t("angebot.bauzeit")}
          wert={t("angebot.bauzeit_wert", {
            tage: a.build_days,
            datum: formatDatum(a.ready),
          })}
          klasse={farbe(a.catalog_build_days > 0 ? a.build_days / a.catalog_build_days : 1, false)}
        />
        <Kennzahl
          name={t("angebot.ausstoss")}
          wert={abweichung(a.factors.output)}
          klasse={farbe(a.factors.output, true)}
        />
        <Kennzahl
          name={t("angebot.arbeit")}
          wert={abweichung(a.factors.labor)}
          klasse={farbe(a.factors.labor, false)}
        />
        <Kennzahl
          name={t("angebot.vorprodukte")}
          wert={abweichung(a.factors.inputs)}
          klasse={farbe(a.factors.inputs, false)}
        />
        <Kennzahl
          name={t("angebot.ausschuss")}
          wert={abweichung(a.factors.scrap)}
          klasse={farbe(a.factors.scrap, false)}
        />
        <Kennzahl
          name={t("angebot.flaeche")}
          wert={
            <>
              {t("angebot.flaeche_wert", { je: qm(a.area_unit_m2), gesamt: qm(a.area_total_m2) })}{" "}
              <small>({abweichung(a.factors.area)})</small>
            </>
          }
          klasse={farbe(a.factors.area, false)}
        />
        {a.free_before_m2 !== null && a.free_after_m2 !== null && (
          <Kennzahl
            name={t("angebot.frei")}
            wert={t("angebot.frei_wert", {
              vorher: qm(Math.max(a.free_before_m2, 0)),
              nachher: passt ? qm(a.free_after_m2) : t("angebot.zu_klein"),
            })}
            klasse={passt ? "" : "negativ"}
          />
        )}
        <Kennzahl
          name={t("angebot.stueckkosten")}
          wert={
            <>
              {formatPreis(a.unit_cost_usd, einheit)} <small>({abweichung(kosten)})</small>
            </>
          }
          klasse={farbe(kosten, false)}
        />
      </dl>
      <details className="material">
        <summary>{t("angebot.material", { betrag: formatPreis(a.material_usd, einheit) })}</summary>
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
      <p className="gedaempft">{t("angebot.gueltig_bis", { datum: formatDatum(a.valid_until) })}</p>
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
    </li>
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
      <ul className="angebote">
        {a.offers.map((o) => (
          <Angebot key={o.id} a={o} ausschreibung={a} produktion={produktion} />
        ))}
      </ul>
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
