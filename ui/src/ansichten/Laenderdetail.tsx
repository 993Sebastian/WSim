import { useEffect, useState } from "react";
import {
  formatFlaeche,
  formatGeld,
  formatProzent,
  formatZahl,
  formatZahlKurz,
  landName,
} from "../format";
import type { Gewerbeflaeche, Kern, Landdetail, Zoelle } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { fehlerText } from "./fehler";

/** "1923-12" → "Dezember 1923"; a start in January is just the year. */
function zeitpunkt(iso: string): string {
  const [jahr, monat] = iso.split("-");
  return Number(monat) === 1 ? jahr! : `${t(`monat.${Number(monat)}`)} ${jahr}`;
}

/** The currency in use with its rate, and the currencies of the country over time. */
function Waehrung({ land }: { land: Landdetail }) {
  const jetzt = land.currencies.find((p) => p.current);
  if (!jetzt) return null;
  const verlauf = land.currencies
    .map((p, i) => {
      const name = t(`waehrung.${p.currency}`);
      return i === 0
        ? name
        : t("landdetail.waehrung_ab", { zeit: zeitpunkt(p.from), waehrung: name });
    })
    .join(", ");
  return (
    <>
      <dt>{t("landdetail.waehrung")}</dt>
      <dd>
        {t(`waehrung.${jetzt.currency}`)} ({jetzt.symbol})
        {land.currency_per_usd !== null &&
          ` · ${t("landdetail.kurs", {
            kurs: formatZahlKurz(land.currency_per_usd),
            zeichen: jetzt.symbol,
            jahr: land.date.slice(0, 4),
          })}`}
        {land.currencies.length > 1 && (
          <small className="feld-hilfe">{t("landdetail.waehrungen", { liste: verlauf })}</small>
        )}
      </dd>
    </>
  );
}

/** Commercial land: how much is taken, the free plots by location and their price. */
function Gewerbeflaechen({ flaeche }: { flaeche: Gewerbeflaeche }) {
  const anteil = flaeche.area_ha > 0 ? flaeche.occupied_ha / flaeche.area_ha : 0;
  return (
    <>
      <h3>{t("landdetail.gewerbeflaeche")}</h3>
      <p>
        {t("landdetail.gewerbeflaeche_wert", {
          gesamt: formatFlaeche(flaeche.area_ha),
          anteil: formatProzent(anteil),
        })}
      </p>
      <div className="tabelle">
        <table>
          <thead>
            <tr>
              <th>{t("grundstueck.lage")}</th>
              <th className="zahl">{t("landdetail.freie_grundstuecke")}</th>
              <th className="zahl">{t("landdetail.groesstes")}</th>
              <th className="zahl">{t("landdetail.bodenpreis")}</th>
            </tr>
          </thead>
          <tbody>
            {flaeche.price_per_ha_usd.map(([lage, preis]) => {
              const frei = flaeche.free.filter((g) => g.location === lage);
              return (
                <tr key={lage}>
                  <td>{t(`lage.${lage}`)}</td>
                  <td className="zahl">{formatZahl(frei.length)}</td>
                  <td className="zahl">
                    {frei.length > 0 ? formatFlaeche(Math.max(...frei.map((g) => g.area_ha))) : "–"}
                  </td>
                  <td className="zahl">{formatGeld(preis)}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </>
  );
}

/** Import tariffs: the average, the highest groups, zones and embargoes (W3). */
function Zollsaetze({ zoelle }: { zoelle: Zoelle }) {
  const liste = (keys: string[], text: (k: string) => string) =>
    keys.length > 0 ? keys.map(text).join(", ") : t("landdetail.zoll_keine");
  return (
    <>
      <h3>{t("landdetail.zoelle")}</h3>
      <dl className="werte">
        <dt>{t("landdetail.zoll_schnitt")}</dt>
        <dd>{formatProzent(zoelle.average)}</dd>
        <dt>{t("landdetail.zoll_hoechste")}</dt>
        <dd>
          {zoelle.groups
            .slice(0, 3)
            .map(([g, z]) => `${t(`warengruppe.${g}`)} ${formatProzent(z)}`)
            .join(", ")}
        </dd>
        <dt>{t("landdetail.zoll_zonen")}</dt>
        <dd>{liste(zoelle.zones, (z) => t(`zoll.zone.${z}`))}</dd>
        <dt>{t("landdetail.zoll_sperren")}</dt>
        <dd>{liste(zoelle.embargoes, landName)}</dd>
      </dl>
      <small className="feld-hilfe">{t("landdetail.zoll_hilfe")}</small>
    </>
  );
}

function gruppenName(gruppe: string): string {
  const [qualifikation, fach] = gruppe.split(".");
  const name = t(`qualifikation.${qualifikation}`);
  return fach ? `${name} (${t(`fachrichtung.${fach}`)})` : name;
}

export function Laenderdetail({
  kern,
  schluessel,
  datum,
  onSchliessen,
  onGruenden,
}: {
  kern: Kern;
  schluessel: string;
  datum: string;
  onSchliessen: () => void;
  onGruenden?: () => void;
}) {
  const [land, setLand] = useState<Landdetail | null>(null);
  const [fehler, setFehler] = useState<string | null>(null);

  useEffect(() => {
    let aktiv = true;
    kern
      .land(schluessel)
      .then((l) => {
        if (!aktiv) return;
        setFehler(null);
        setLand(l);
      })
      .catch((e: unknown) => {
        if (!aktiv) return;
        setLand(null);
        setFehler(fehlerText(e));
      });
    return () => {
      aktiv = false;
    };
  }, [kern, schluessel, datum]);

  const titel = landName(schluessel);
  return (
    <aside className="laenderdetail" aria-label={titel}>
      <header className="dialog-kopf">
        <h2>{titel}</h2>
        <button
          type="button"
          className="schlicht"
          onClick={onSchliessen}
          aria-label={t("dialog.schliessen")}
        >
          ×
        </button>
      </header>
      <FehlerText fehler={fehler} />
      {onGruenden && (
        <div className="knopfreihe links">
          <button type="button" onClick={onGruenden}>
            {t("landdetail.gruenden")}
          </button>
        </div>
      )}
      {land && land.key === schluessel && (
        <>
          <dl className="werte">
            {land.members.length > 0 && (
              <>
                <dt>{t("landdetail.umfasst")}</dt>
                <dd>{land.members.map((m) => t(`teilland.${m}`)).join(", ")}</dd>
              </>
            )}
            <dt>{t("landdetail.bevoelkerung")}</dt>
            <dd>{formatZahl(land.population)}</dd>
            <dt>{t("landdetail.bip_je_kopf")}</dt>
            <dd>{formatGeld(land.gdp_per_capita_usd)}</dd>
            <dt>{t("landdetail.preisniveau")}</dt>
            <dd>{formatZahl(land.price_level, 2)}</dd>
            <Waehrung land={land} />
            <dt>{t("landdetail.gini")}</dt>
            <dd>{formatZahl(land.gini, 3)}</dd>
            <dt>{t("landdetail.strom")}</dt>
            <dd>
              {formatGeld(land.electricity_price_usd_mwh)}/MWh ·{" "}
              {t("landdetail.netz", { anteil: formatProzent(land.grid_share) })}
            </dd>
            <dt>{t("landdetail.steuern")}</dt>
            <dd>
              {t("landdetail.steuern_wert", {
                gewinn: formatProzent(land.corporate_tax),
                dividende: formatProzent(land.dividend_tax),
              })}
            </dd>
            <dt>{t("landdetail.infrastruktur")}</dt>
            <dd>
              {t("landdetail.infrastruktur_wert", {
                schiene: formatProzent(land.infrastructure[0] ?? 0),
                strasse: formatProzent(land.infrastructure[1] ?? 0),
                hafen: formatProzent(land.infrastructure[2] ?? 0),
              })}
            </dd>
            <dt>{t("landdetail.stabilitaet")}</dt>
            <dd>{formatProzent(land.stability)}</dd>
          </dl>

          {land.land && <Gewerbeflaechen flaeche={land.land} />}
          {land.tariffs && <Zollsaetze zoelle={land.tariffs} />}

          <h3>{t("landdetail.arbeitskraefte")}</h3>
          <div className="tabelle">
            <table>
              <thead>
                <tr>
                  <th>{t("landdetail.gruppe")}</th>
                  <th className="zahl">{t("landdetail.personen")}</th>
                  <th className="zahl">{t("landdetail.verfuegbar")}</th>
                  <th className="zahl">{t("landdetail.lohn")}</th>
                </tr>
              </thead>
              <tbody>
                {land.labor
                  .filter((a) => a.persons > 0)
                  .map((a) => (
                    <tr key={a.group}>
                      <td>{gruppenName(a.group)}</td>
                      <td className="zahl">{formatZahl(a.persons)}</td>
                      <td className="zahl">{formatZahl(a.available)}</td>
                      <td className="zahl">{formatGeld(a.wage_usd)}</td>
                    </tr>
                  ))}
              </tbody>
            </table>
          </div>

          {land.deposits.length > 0 && (
            <>
              <h3>{t("landdetail.lagerstaetten")}</h3>
              <ul className="liste kompakt">
                {land.deposits.map((d) => (
                  <li key={d.key}>
                    <span>
                      {t(`lagerstaette.${d.key}`)} · {t(`produkt.${d.resource}`)}
                    </span>
                    <span className="gedaempft">
                      {d.undiscovered
                        ? t("karte.lager.unentdeckt")
                        : t("landdetail.konzessionen", {
                            frei: d.free_concessions,
                            gesamt: d.concessions,
                          })}
                    </span>
                  </li>
                ))}
              </ul>
            </>
          )}

          <h3>{t("landdetail.firmen")}</h3>
          {land.companies.length === 0 ? (
            <p className="gedaempft">{t("landdetail.keine_firmen")}</p>
          ) : (
            <ul className="liste kompakt">
              {land.companies.slice(0, 12).map((c) => (
                <li key={c.name} className={c.own ? "eigen" : ""}>
                  <span>{c.name}</span>
                  <span className="gedaempft">
                    {t("landdetail.standorte_anzahl", { anzahl: c.sites })}
                  </span>
                </li>
              ))}
            </ul>
          )}

          {land.markets.length > 0 && (
            <>
              <h3>{t("landdetail.maerkte")}</h3>
              <div className="tabelle">
                <table>
                  <thead>
                    <tr>
                      <th>{t("uebersicht.produkt")}</th>
                      <th className="zahl">{t("landdetail.preis")}</th>
                      <th className="zahl">{t("landdetail.nachfrage")}</th>
                      <th className="zahl">{t("landdetail.verkauft")}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {land.markets.slice(0, 15).map((m) => (
                      <tr key={m.product}>
                        <td>{t(`produkt.${m.product}`)}</td>
                        <td className="zahl">{formatGeld(m.price_usd)}</td>
                        <td className="zahl">{formatZahl(m.demand_last_month)}</td>
                        <td className="zahl">{formatZahl(m.sold_last_month)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </>
          )}
        </>
      )}
    </aside>
  );
}
