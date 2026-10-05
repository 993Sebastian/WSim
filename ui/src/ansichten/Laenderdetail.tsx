import { useEffect, useState } from "react";
import { formatGeld, formatProzent, formatZahl, landName } from "../format";
import type { Kern, Landdetail } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { fehlerText } from "./fehler";

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
            <dt>{t("landdetail.bevoelkerung")}</dt>
            <dd>{formatZahl(land.population)}</dd>
            <dt>{t("landdetail.bip_je_kopf")}</dt>
            <dd>{formatGeld(land.gdp_per_capita_usd)}</dd>
            <dt>{t("landdetail.preisniveau")}</dt>
            <dd>{formatZahl(land.price_level, 2)}</dd>
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
                      <td className="zahl">{formatZahl(a.wage_usd, 2)} USD</td>
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
