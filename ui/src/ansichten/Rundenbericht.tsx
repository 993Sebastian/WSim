import { useEffect, useState } from "react";
import {
  formatDatum,
  formatGeld,
  formatMenge,
  formatProzent,
  landName,
  meldungText,
} from "../format";
import type {
  AnlagenZeile,
  BerichtDetail,
  Hinweis,
  Marktanteile,
  Meldung,
  MeldungsGruppe,
  Mengenknoten,
  Periode,
  ProduktZeile,
  Rundenbericht,
  WerkZeile,
} from "../kern";
import { t } from "../texte";
import { Dialog } from "./Dialog";
import { standortName } from "./stellen";

function knotenName(k: Mengenknoten): string {
  if (k.level === "kontinent") return t(`kontinent.${k.key}`);
  if (k.level === "land") return landName(k.key);
  if (k.level === "standort") return standortName(k.kind_text ?? "", k.country ?? "", k.site_name);
  return t("bericht.mengen_firma");
}

/** Made and sold units of one level, its children folded below (top down). */
function Mengen({ k, offen = false }: { k: Mengenknoten; offen?: boolean }) {
  const name = knotenName(k);
  return (
    <details className={`mengen mengen-${k.level}`} open={offen}>
      <summary>{name}</summary>
      <div className="tabelle">
        <table className="mobil-karten" aria-label={t("bericht.mengen_von", { ort: name })}>
          <thead>
            <tr>
              <th>{t("uebersicht.produkt")}</th>
              <th className="zahl">{t("bericht.hergestellt")}</th>
              <th className="zahl">{t("bericht.verkauft")}</th>
            </tr>
          </thead>
          <tbody>
            {k.products.map((p) => {
              const e = t(`einheit.${p.unit}`);
              return (
                <tr key={p.product}>
                  <td>{t(`produkt.${p.product}`)}</td>
                  <td className="zahl" data-spalte={t("bericht.hergestellt")}>
                    {p.made > 0 ? `${formatMenge(p.made)} ${e}` : "–"}
                  </td>
                  <td className="zahl" data-spalte={t("bericht.verkauft")}>
                    {p.sold > 0 ? `${formatMenge(p.sold)} ${e}` : "–"}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      {k.children.map((c) => (
        <Mengen key={`${c.level}/${c.key}`} k={c} />
      ))}
    </details>
  );
}

const GRUPPEN: MeldungsGruppe[] = [
  "erfolg",
  "welt",
  "warnung",
  "wettbewerb",
  "forschung",
  "allgemein",
];

function Veraenderung({ jetzt, vorher }: { jetzt: number; vorher: number | undefined }) {
  if (vorher === undefined) return <td className="zahl gedaempft">–</td>;
  const d = jetzt - vorher;
  return (
    <td className={`zahl ${d < 0 ? "negativ" : "positiv"}`}>
      {d > 0 ? "+" : ""}
      {formatGeld(d)}
    </td>
  );
}

function Ergebnis({ periode, vorher }: { periode: Periode; vorher: Periode | null }) {
  // Without the change of the stock the lines would not add up to the result.
  const lager =
    Math.abs(periode.inventory_change_usd) >= 0.5 ||
    Math.abs(vorher?.inventory_change_usd ?? 0) >= 0.5;
  const zeilen: [string, number, number | undefined][] = [
    [t("bericht.umsatz"), periode.revenue_usd, vorher?.revenue_usd],
    [t("bericht.kosten"), -periode.costs_usd, vorher ? -vorher.costs_usd : undefined],
    ...(lager
      ? [
          [t("bericht.lager"), periode.inventory_change_usd, vorher?.inventory_change_usd] as [
            string,
            number,
            number | undefined,
          ],
        ]
      : []),
    [t("bericht.ergebnis"), periode.result_usd, vorher?.result_usd],
  ];
  return (
    <div className="tabelle">
      <table className="ergebnis">
        <thead>
          <tr>
            <th />
            <th className="zahl">{t("bericht.diese_runde")}</th>
            <th className="zahl">{t("bericht.vorrunde")}</th>
            <th className="zahl">{t("bericht.veraenderung")}</th>
          </tr>
        </thead>
        <tbody>
          {zeilen.map(([name, jetzt, alt]) => (
            <tr key={name}>
              <th scope="row">{name}</th>
              <td className={`zahl ${jetzt < 0 ? "negativ" : ""}`}>{formatGeld(jetzt)}</td>
              <td className="zahl">{alt === undefined ? "–" : formatGeld(alt)}</td>
              <Veraenderung jetzt={jetzt} vorher={alt} />
            </tr>
          ))}
        </tbody>
      </table>
      {lager && <p className="feld-hilfe">{t("bericht.lager_hinweis")}</p>}
      {periode.lines.length > 0 && (
        <details className="aufschluesselung">
          <summary>{t("bericht.nach_kostenart")}</summary>
          <table>
            <tbody>
              {periode.lines.map(([schluessel, betrag]) => (
                <tr key={schluessel}>
                  <td>{t(schluessel)}</td>
                  <td className={`zahl ${betrag < 0 ? "negativ" : ""}`}>{formatGeld(betrag)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </details>
      )}
    </div>
  );
}

function MeldungZeile({ m, onZiel }: { m: Meldung; onZiel: (ziel: string) => void }) {
  return (
    <li className={`meldung meldung-${m.kind}`}>
      <span className="meldungsart">{t(`meldungsart.${m.kind}`)}</span>
      <span className="meldungstext">{meldungText(m)}</span>
      {m.target && (
        <button type="button" className="schlicht" onClick={() => onZiel(m.target!)}>
          {t("bericht.ansehen")}
        </button>
      )}
    </li>
  );
}

type Grenzen = NonNullable<BerichtDetail["limits"]>;

/** Fallback for reports without limits (old saves); the data file has the real values. */
const GRENZEN: Grenzen = {
  visible_products: 5,
  attention_max: 5,
  utilization_red: 0.6,
  utilization_green: 0.85,
  share_drop: 0.3,
};

/** A share as a percentage: whole from 10 %, one decimal below; none without sales. */
function anteil(x: number | null): string {
  if (x === null) return "–";
  const prozent = x * 100;
  return `${prozent >= 10 ? Math.round(prozent) : prozent.toFixed(1).replace(".", ",")} %`;
}

/** An arrow against the round before; the value before is the tooltip. */
function Trend({
  jetzt,
  vorher,
  text,
}: {
  jetzt: number;
  vorher: number | undefined;
  text: string;
}) {
  if (vorher === undefined || Math.abs(jetzt - vorher) < 1e-9) return null;
  const hoch = jetzt > vorher;
  return (
    <span
      className={hoch ? "trend positiv" : "trend negativ"}
      title={t("bericht.vorrunde_wert", { wert: text })}
      aria-label={hoch ? t("bericht.steigt") : t("bericht.faellt")}
    >
      {hoch ? " ▲" : " ▼"}
    </span>
  );
}

function AnteilsZelle({
  land,
  wert,
  vorher,
}: {
  land?: string | null;
  wert: number | null;
  vorher: number | null | undefined;
}) {
  return (
    <>
      {land ? <small className="gedaempft">{land} </small> : null}
      {anteil(wert)}
      {wert !== null && vorher !== null && vorher !== undefined && (
        <Trend jetzt={wert} vorher={vorher} text={anteil(vorher)} />
      )}
    </>
  );
}

function ProduktZeileTr({ z, onProdukt }: { z: ProduktZeile; onProdukt: () => void }) {
  const e = t(`einheit.${z.unit}`);
  const alt = z.previous ?? undefined;
  const a: Marktanteile = z.shares;
  return (
    <tr>
      <th scope="row" data-spalte={t("uebersicht.produkt")}>
        <button type="button" className="schlicht" onClick={onProdukt}>
          {t(`produkt.${z.product}`)}
        </button>
      </th>
      <td className="zahl" data-spalte={t("bericht.spalte.prod")}>
        {z.made > 0 ? `${formatMenge(z.made)} ${e}` : "–"}
      </td>
      <td className="zahl" data-spalte={t("bericht.spalte.verk")}>
        {z.sold > 0 ? `${formatMenge(z.sold)} ${e}` : "–"}
      </td>
      <td className="zahl" data-spalte={t("bericht.spalte.umsatz")}>
        {formatGeld(z.revenue_usd)}
      </td>
      <td
        className={`zahl ${z.contribution_usd < 0 ? "negativ" : ""}`}
        data-spalte={t("bericht.spalte.deckungsbeitrag")}
      >
        {formatGeld(z.contribution_usd)}
        <Trend
          jetzt={z.contribution_usd}
          vorher={alt?.contribution_usd}
          text={formatGeld(alt?.contribution_usd ?? 0)}
        />
      </td>
      <td className="zahl" data-spalte={t("bericht.spalte.land")}>
        <AnteilsZelle land={z.country} wert={a.country} vorher={alt?.shares.country} />
      </td>
      <td className="zahl" data-spalte={t("bericht.spalte.kontinent")}>
        <AnteilsZelle wert={a.continent} vorher={alt?.shares.continent} />
      </td>
      <td className="zahl" data-spalte={t("bericht.spalte.welt")}>
        <AnteilsZelle wert={a.world} vorher={alt?.shares.world} />
      </td>
    </tr>
  );
}

/** The products of the round, the largest by revenue first, with the sums below. */
function Produkte({ d, onProdukt }: { d: BerichtDetail; onProdukt: () => void }) {
  const [alle, setAlle] = useState(false);
  const sichtbar = d.products.filter(
    (p, i) => alle || i < (d.limits ?? GRENZEN).visible_products || p.contribution_usd < 0,
  );
  const versteckt = d.products.length - sichtbar.length;
  const r = d.result;
  const summe = d.products.reduce((a, p) => a + p.revenue_usd, 0);
  return (
    <section data-tour="was-lief" aria-label={t("bericht.produkte")}>
      <h3>{t("bericht.produkte")}</h3>
      {d.products.length === 0 ? (
        <p className="gedaempft">{t("bericht.nichts_verkauft")}</p>
      ) : (
        <div className="tabelle">
          <table className="mobil-karten" aria-label={t("bericht.was_lief")}>
            <thead>
              <tr>
                <th scope="col">{t("uebersicht.produkt")}</th>
                <th scope="col" className="zahl">
                  {t("bericht.spalte.prod")}
                </th>
                <th scope="col" className="zahl">
                  {t("bericht.spalte.verk")}
                </th>
                <th scope="col" className="zahl">
                  {t("bericht.spalte.umsatz")}
                </th>
                <th scope="col" className="zahl">
                  {t("bericht.spalte.deckungsbeitrag")}
                </th>
                <th scope="col" className="zahl">
                  {t("bericht.spalte.land")}
                </th>
                <th scope="col" className="zahl">
                  {t("bericht.spalte.kontinent")}
                </th>
                <th scope="col" className="zahl">
                  {t("bericht.spalte.welt")}
                </th>
              </tr>
            </thead>
            <tbody>
              {sichtbar.map((z) => (
                <ProduktZeileTr key={z.product} z={z} onProdukt={onProdukt} />
              ))}
              {versteckt > 0 && (
                <tr>
                  <td colSpan={8}>
                    <button type="button" className="schlicht" onClick={() => setAlle(true)}>
                      {t("bericht.weitere_produkte", { anzahl: versteckt })}
                    </button>
                  </td>
                </tr>
              )}
              {alle && d.products.length > (d.limits ?? GRENZEN).visible_products && (
                <tr>
                  <td colSpan={8}>
                    <button type="button" className="schlicht" onClick={() => setAlle(false)}>
                      {t("bericht.weniger_produkte")}
                    </button>
                  </td>
                </tr>
              )}
            </tbody>
            <tfoot>
              <tr>
                <th scope="row">{t("bericht.summe")}</th>
                <td />
                <td />
                <td className="zahl">{formatGeld(summe)}</td>
                <td className={`zahl ${r.contribution_usd < 0 ? "negativ" : ""}`}>
                  {formatGeld(r.contribution_usd)}
                </td>
                <td />
                <td />
                <td />
              </tr>
              <tr>
                <th scope="row">{t("bericht.fixkosten")}</th>
                <td colSpan={3} />
                <td className="zahl negativ">{formatGeld(r.fixed_usd)}</td>
                <td colSpan={3} />
              </tr>
              {Math.abs(r.other_usd) >= 0.5 && (
                <tr>
                  <th scope="row">{t("bericht.sonstiges")}</th>
                  <td colSpan={3} />
                  <td className={`zahl ${r.other_usd < 0 ? "negativ" : ""}`}>
                    {formatGeld(r.other_usd)}
                  </td>
                  <td colSpan={3} />
                </tr>
              )}
              <tr>
                <th scope="row">{t("bericht.ergebnis")}</th>
                <td colSpan={3} />
                <td className={`zahl ${r.result_usd < 0 ? "negativ" : ""}`}>
                  {formatGeld(r.result_usd)}
                </td>
                <td colSpan={3} />
              </tr>
            </tfoot>
          </table>
        </div>
      )}
    </section>
  );
}

/** Green from the upper limit, yellow from the lower, else red. */
function auslastungsFarbe(u: number, g: Grenzen): string {
  return u >= g.utilization_green ? "gruen" : u >= g.utilization_red ? "gelb" : "rot";
}

function Balken({ wert, g }: { wert: number | null; g: Grenzen }) {
  if (wert === null) return <span className="gedaempft">–</span>;
  return (
    <span className={`auslastung auslastung-${auslastungsFarbe(wert, g)}`}>
      <progress max={1} value={wert} aria-label={t("bericht.auslastung")} />{" "}
      <span className="zahl">{Math.round(wert * 100)} %</span>
    </span>
  );
}

function AnlageZeile({ a, g }: { a: AnlagenZeile; g: Grenzen }) {
  const status =
    a.state === "laeuft" ? null : t(`bericht.anlage_status.${a.state}`, { tage: a.days_left ?? 0 });
  return (
    <li className="anlage-zeile">
      <span>
        {t(`anlage.${a.facility}`)}
        {a.count > 1 ? ` ×${a.count}` : ""}
        {a.product ? ` · ${t(`produkt.${a.product}`)}` : ""}
      </span>
      {status ? (
        <span className="gedaempft">{status}</span>
      ) : (
        <>
          <Balken wert={a.utilization} g={g} />
          {a.loss && a.utilization !== null && a.utilization < g.utilization_green && (
            <small className="grund">{t(`bericht.grund.${a.loss}`)}</small>
          )}
        </>
      )}
    </li>
  );
}

function Werke({
  werke,
  g,
  onWerk,
}: {
  werke: WerkZeile[];
  g: Grenzen;
  onWerk: (site: number) => void;
}) {
  if (werke.length === 0) return null;
  return (
    <section aria-label={t("bericht.auslastung")}>
      <h3>{t("bericht.auslastung")}</h3>
      <ul className="werke">
        {werke.map((w) => {
          const schwach = w.utilization !== null && w.utilization < g.utilization_red;
          return (
            <li key={w.site}>
              <details open={schwach}>
                <summary>
                  <button
                    type="button"
                    className="schlicht"
                    onClick={(e) => {
                      e.preventDefault();
                      onWerk(w.site);
                    }}
                  >
                    {standortName(w.kind_text, w.country, w.name)}
                  </button>{" "}
                  <Balken wert={w.utilization} g={g} />
                  {w.loss && w.utilization !== null && w.utilization < g.utilization_green && (
                    <small className="grund"> {t(`bericht.grund.${w.loss}`)}</small>
                  )}
                </summary>
                <ul className="anlagen">
                  {w.slots.map((a, i) => (
                    <AnlageZeile key={i} a={a} g={g} />
                  ))}
                </ul>
              </details>
            </li>
          );
        })}
      </ul>
    </section>
  );
}

export function RundenberichtDialog({
  bericht,
  onZiel,
  onHinweis,
  onEreignis,
  onWerk,
  onSchliessen,
}: {
  bericht: Rundenbericht;
  onZiel: (ziel: string) => void;
  onHinweis: (h: Hinweis) => void;
  onEreignis: (m: Meldung) => void;
  onWerk: (site: number) => void;
  onSchliessen: () => void;
}) {
  const kasseAenderung = bericht.cash_after_usd - bericht.cash_before_usd;
  const gruppen = GRUPPEN.map((g) => [g, bericht.messages.filter((m) => m.group === g)] as const);
  const detail = bericht.detail ?? null;
  const ergebnis = detail?.result.result_usd ?? bericht.period.result_usd;
  const vorErgebnis = bericht.previous?.result_usd;
  const prozent =
    vorErgebnis !== undefined && Math.abs(vorErgebnis) > 1e-9
      ? (ergebnis - vorErgebnis) / Math.abs(vorErgebnis)
      : null;
  // Enter ends the report like the button, unless a control has the focus.
  useEffect(() => {
    const taste = (e: KeyboardEvent) => {
      if (e.key !== "Enter" || e.ctrlKey || e.metaKey || e.altKey) return;
      const ziel = e.target as HTMLElement | null;
      if (ziel && /^(INPUT|SELECT|TEXTAREA|BUTTON|A|SUMMARY)$/.test(ziel.tagName)) return;
      onSchliessen();
    };
    window.addEventListener("keydown", taste);
    return () => window.removeEventListener("keydown", taste);
  }, [onSchliessen]);
  return (
    <Dialog titel={t("bericht.titel")} onSchliessen={onSchliessen} breit="sehr" tour="bericht">
      <p className="gedaempft">
        {t("bericht.zeitraum", {
          von: formatDatum(bericht.from),
          bis: formatDatum(bericht.to),
          tage: bericht.days,
        })}
        {(bericht.rounds ?? 1) > 1 && ` · ${t("bericht.runden", { anzahl: bericht.rounds! })}`}
        {" · "}
        <span className="weiter-hinweis">{t("bericht.enter_weiter")}</span>
      </p>
      {bericht.stop && <p className="halt-grund">{t(`bericht.halt.${bericht.stop}`)}</p>}
      <dl className="kennzahlen">
        <div>
          <dt>{t("bericht.ergebnis")}</dt>
          <dd className={ergebnis < 0 ? "negativ" : "positiv"}>
            {ergebnis > 0 ? "+" : ""}
            {formatGeld(ergebnis)}
            {vorErgebnis !== undefined && (
              <Trend jetzt={ergebnis} vorher={vorErgebnis} text={formatGeld(vorErgebnis)} />
            )}
            {prozent !== null && (
              <small className="gedaempft">
                {" "}
                ({prozent > 0 ? "+" : ""}
                {Math.round(prozent * 100)} %)
              </small>
            )}
          </dd>
        </div>
        <div>
          <dt>{t("bericht.kasse")}</dt>
          <dd>
            {formatGeld(bericht.cash_after_usd)}
            <span className={kasseAenderung < 0 ? "negativ" : "positiv"}>
              {" "}
              ({kasseAenderung > 0 ? "+" : ""}
              {formatGeld(kasseAenderung)})
            </span>
          </dd>
        </div>
        {detail && (
          <div>
            <dt>{t("bericht.privat")}</dt>
            <dd className={detail.person_change_usd < 0 ? "negativ" : "positiv"}>
              {detail.person_change_usd > 0 ? "+" : ""}
              {formatGeld(detail.person_change_usd)}
            </dd>
          </div>
        )}
      </dl>
      {detail && <Produkte d={detail} onProdukt={() => onZiel("markt")} />}
      {detail && <Werke werke={detail.sites} g={detail.limits ?? GRENZEN} onWerk={onWerk} />}

      {bericht.hints.length > 0 && (
        <section>
          <h3>{t("bericht.jetzt_tun")}</h3>
          <ul className="meldungen">
            {bericht.hints.map((h, i) => (
              <li key={i} className={`meldung meldung-${h.message.kind}`}>
                <span className="meldungsart">{t(`meldungsart.${h.message.kind}`)}</span>
                <span className="meldungstext">{meldungText(h.message)}</span>
                <button type="button" className="schlicht" onClick={() => onHinweis(h)}>
                  {t("uebersicht.hingehen")}
                </button>
              </li>
            ))}
          </ul>
        </section>
      )}

      <details className="bericht-weiteres">
        <summary>{t("bericht.weitere_auswertungen")}</summary>
        <div className="bericht-raster">
          <section>
            <h3>{t("bericht.finanzergebnis")}</h3>
            <Ergebnis periode={bericht.period} vorher={bericht.previous} />
          </section>

          {(bericht.companies?.length ?? 0) > 1 && (
            <section>
              <h3>{t("bericht.firmen")}</h3>
              <div className="tabelle">
                <table className="mobil-karten" aria-label={t("bericht.firmen")}>
                  <thead>
                    <tr>
                      <th scope="col">{t("bericht.firma")}</th>
                      <th scope="col">{t("bericht.umsatz")}</th>
                      <th scope="col">{t("bericht.ergebnis")}</th>
                      <th scope="col">{t("bericht.kasse")}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {bericht.companies?.map((f) => (
                      <tr key={f.company}>
                        <th scope="row" data-spalte={t("bericht.firma")}>
                          {f.name}
                          {f.main && (
                            <small className="gedaempft"> ({t("bericht.hauptfirma")})</small>
                          )}
                        </th>
                        <td className="zahl" data-spalte={t("bericht.umsatz")}>
                          {formatGeld(f.revenue_usd)}
                        </td>
                        <td
                          className={f.result_usd < 0 ? "zahl negativ" : "zahl"}
                          data-spalte={t("bericht.ergebnis")}
                        >
                          {formatGeld(f.result_usd)}
                        </td>
                        <td className="zahl" data-spalte={t("bericht.kasse")}>
                          {formatGeld(f.cash_after_usd)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </section>
          )}

          {bericht.quantities && (
            <section data-tour="mengen" className="ganz">
              <h3>{t("bericht.mengen")}</h3>
              <p className="feld-hilfe">{t("bericht.mengen_hilfe")}</p>
              <Mengen k={bericht.quantities} offen />
            </section>
          )}

          {gruppen.map(([gruppe, liste]) =>
            gruppe === "forschung"
              ? (bericht.research.length > 0 || liste.length > 0) && (
                  <section key={gruppe}>
                    <h3>{t("bericht.gruppe.forschung")}</h3>
                    {bericht.research.map((p) => (
                      <p key={p.technology} className="forschungszeile">
                        <span>{t(`technologie.${p.technology}`)}</span>
                        <progress max={p.needed} value={Math.min(p.points, p.needed)} />
                        <span className="zahl">
                          {formatProzent(p.needed > 0 ? p.points / p.needed : 0)}
                        </span>
                      </p>
                    ))}
                    <ul className="meldungen">
                      {liste.map((m, i) => (
                        <MeldungZeile key={i} m={m} onZiel={onZiel} />
                      ))}
                    </ul>
                  </section>
                )
              : liste.length > 0 && (
                  <section key={gruppe}>
                    <h3>{t(`bericht.gruppe.${gruppe}`)}</h3>
                    <ul className="meldungen">
                      {liste.map((m, i) =>
                        m.kind === "world_event" ? (
                          <li key={i} className="meldung meldung-world_event">
                            <span className="meldungsart">{t("meldungsart.world_event")}</span>
                            <span className="meldungstext">{meldungText(m)}</span>
                            <button
                              type="button"
                              className="schlicht"
                              onClick={() => onEreignis(m)}
                            >
                              {t("bericht.ansehen")}
                            </button>
                          </li>
                        ) : (
                          <MeldungZeile key={i} m={m} onZiel={onZiel} />
                        ),
                      )}
                    </ul>
                  </section>
                ),
          )}
        </div>
      </details>
      {bericht.messages.length === 0 && bericht.research.length === 0 && (
        <p>{t("bericht.keine_meldungen")}</p>
      )}
      {bericht.game_over && <p className="fehlertext">{t("spiel.ende")}</p>}
      <div className="knopfreihe">
        <button type="button" className="haupt" onClick={onSchliessen}>
          {t("bericht.weiter")}
        </button>
      </div>
    </Dialog>
  );
}
