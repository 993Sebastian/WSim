// Plots (M35): the free plots of a country to found a site on, bought or leased. The
// core decides which plots exist and what they cost; this only shows them for a choice.
import { useEffect, useId, useState } from "react";
import { formatGeld, formatZahl } from "../format";
import type { Gewerbeflaeche, Grundstueck, Kern } from "../kern";
import { t } from "../texte";

/** Rows of the list when one location is shown, and per location when all are. */
const ZEILEN = 12;
const ZEILEN_JE_LAGE = 4;

/**
 * The commercial land of a country, reloaded when the country or the game date changes:
 * undefined while loading, null without plots (or when the country has no data).
 */
export function useGewerbeflaeche(
  kern: Kern,
  land: string,
  stand: string,
): Gewerbeflaeche | null | undefined {
  const [geladen, setGeladen] = useState<{ land: string; flaeche: Gewerbeflaeche | null }>();
  useEffect(() => {
    let aktiv = true;
    kern.land(land).then(
      (l) => aktiv && setGeladen({ land, flaeche: l.land }),
      () => aktiv && setGeladen({ land, flaeche: null }),
    );
    return () => {
      aktiv = false;
    };
  }, [kern, land, stand]);
  // A new date keeps showing the old plots until the new ones are there.
  return geladen?.land === land ? geladen.flaeche : undefined;
}

/** The chosen plot (null: none yet) and whether it is leased instead of bought. */
export interface GrundstueckWahlWert {
  plot: number | null;
  lease: boolean;
}

export function grundstueckName(g: Grundstueck): string {
  return t("grundstueck.name", {
    lage: t(`lage.${g.location}`),
    flaeche: formatZahl(g.area_ha, 2),
  });
}

/** Choice of a free plot: filters by location and size, a list, buy or lease. */
export function GrundstueckWahl({
  flaeche,
  wert,
  onWert,
}: {
  flaeche: Gewerbeflaeche;
  wert: GrundstueckWahlWert;
  onWert: (w: GrundstueckWahlWert) => void;
}) {
  const id = useId();
  const [lage, setLage] = useState("alle");
  const [klasse, setKlasse] = useState("alle");
  const lagen = flaeche.price_per_ha_usd.map(([l]) => l);
  // Cheapest first: most look for a plot that is just large enough.
  const passend = flaeche.free
    .filter(
      (g) => (lage === "alle" || g.location === lage) && (klasse === "alle" || g.class === klasse),
    )
    .sort((a, b) => a.value_usd - b.value_usd || a.id - b.id);
  const sichtbar =
    lage === "alle"
      ? lagen.flatMap((l) => passend.filter((g) => g.location === l).slice(0, ZEILEN_JE_LAGE))
      : passend.slice(0, ZEILEN);
  const gewaehlt = flaeche.free.find((g) => g.id === wert.plot) ?? null;
  const frei = flaeche.area_ha - flaeche.occupied_ha;
  return (
    <fieldset className="grundstuecke" data-tour="grundstueck">
      <legend>{t("grundstueck.waehlen")}</legend>
      <p className="erklaerung">
        {t("grundstueck.hinweis", {
          frei: formatZahl(frei, 0),
          gesamt: formatZahl(flaeche.area_ha, 0),
          preise: flaeche.price_per_ha_usd
            .map(([l, preis]) => `${t(`lage.${l}`)} ${formatGeld(preis)}`)
            .join(", "),
        })}
      </p>
      <details className="aufklapper">
        <summary>{t("grundstueck.lagen_titel")}</summary>
        <ul className="schlicht-liste">
          {lagen.map((l) => (
            <li key={l}>
              <strong>{t(`lage.${l}`)}:</strong> {t(`grundstueck.lage_${l}`)}
            </li>
          ))}
        </ul>
        <p>{t("grundstueck.kauf_pacht")}</p>
      </details>
      {flaeche.free.length === 0 ? (
        <p className="fehlertext">{t("grundstueck.keins_frei")}</p>
      ) : (
        <>
          <div className="formular-zeile">
            <div className="feld">
              <label htmlFor={`${id}-lage`}>{t("grundstueck.lage")}</label>
              <select id={`${id}-lage`} value={lage} onChange={(e) => setLage(e.target.value)}>
                <option value="alle">{t("grundstueck.alle_lagen")}</option>
                {lagen.map((l) => (
                  <option key={l} value={l}>
                    {t(`lage.${l}`)}
                  </option>
                ))}
              </select>
            </div>
            <div className="feld">
              <label htmlFor={`${id}-klasse`}>{t("grundstueck.groesse")}</label>
              <select
                id={`${id}-klasse`}
                value={klasse}
                onChange={(e) => setKlasse(e.target.value)}
              >
                <option value="alle">{t("grundstueck.alle_groessen")}</option>
                {flaeche.classes.map((k) => (
                  <option key={k} value={k}>
                    {t(`grundstuecksklasse.${k}`)}
                  </option>
                ))}
              </select>
            </div>
          </div>
          {sichtbar.length === 0 ? (
            <p className="gedaempft">{t("grundstueck.keins_passend")}</p>
          ) : (
            <div className="tabelle">
              <table aria-label={t("grundstueck.freie")}>
                <thead>
                  <tr>
                    <th>
                      <span className="unsichtbar">{t("grundstueck.wahl")}</span>
                    </th>
                    <th>{t("grundstueck.lage")}</th>
                    <th>{t("grundstueck.groesse")}</th>
                    <th className="zahl">{t("grundstueck.flaeche")}</th>
                    <th className="zahl">{t("grundstueck.kaufpreis")}</th>
                    <th className="zahl">{t("grundstueck.pacht")}</th>
                  </tr>
                </thead>
                <tbody>
                  {sichtbar.map((g) => (
                    <tr
                      key={g.id}
                      className={g.id === wert.plot ? "gewaehlt" : undefined}
                      onClick={() => onWert({ ...wert, plot: g.id })}
                    >
                      <td>
                        <input
                          type="radio"
                          name={`${id}-grundstueck`}
                          checked={g.id === wert.plot}
                          onChange={() => onWert({ ...wert, plot: g.id })}
                          aria-label={grundstueckName(g)}
                        />
                      </td>
                      <td>{t(`lage.${g.location}`)}</td>
                      <td>{t(`grundstuecksklasse.${g.class}`)}</td>
                      <td className="zahl">{formatZahl(g.area_ha, 2)} ha</td>
                      <td className="zahl">{formatGeld(g.value_usd)}</td>
                      <td className="zahl">{formatGeld(g.rent_usd_year)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
          {passend.length > sichtbar.length && (
            <p className="gedaempft">
              {t("grundstueck.weitere", { anzahl: passend.length - sichtbar.length })}
            </p>
          )}
          <fieldset className="auswahlgruppe waagrecht">
            <legend>{t("grundstueck.besitz")}</legend>
            <label>
              <input
                type="radio"
                name={`${id}-besitz`}
                checked={!wert.lease}
                onChange={() => onWert({ ...wert, lease: false })}
              />
              {t("grundstueck.kaufen")}
            </label>
            <label>
              <input
                type="radio"
                name={`${id}-besitz`}
                checked={wert.lease}
                onChange={() => onWert({ ...wert, lease: true })}
              />
              {t("grundstueck.pachten")}
            </label>
          </fieldset>
          <p className={gewaehlt ? "" : "gedaempft"} aria-live="polite">
            {gewaehlt
              ? t(wert.lease ? "grundstueck.gewaehlt_pacht" : "grundstueck.gewaehlt_kauf", {
                  grundstueck: grundstueckName(gewaehlt),
                  preis: formatGeld(gewaehlt.value_usd),
                  pacht: formatGeld(gewaehlt.rent_usd_year),
                })
              : t("grundstueck.noch_keins")}
          </p>
        </>
      )}
    </fieldset>
  );
}
