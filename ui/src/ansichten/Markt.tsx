import { useState } from "react";
import { formatGeld, formatProzent, formatZahl, landName } from "../format";
import { geld, type Kern, type Markt, type Uebersicht } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { Rueckmeldung, useBefehl, useSicht } from "./gemeinsam";
import { LAENDER } from "./laender";

type Zeile = Markt["lines"][number];
type Marke = Markt["brands"][number];
type Spalte = { key: string; wert: (z: Zeile) => number | string; text?: string };

const SPALTEN: Spalte[] = [
  { key: "produkt", wert: (z) => t(`produkt.${z.product}`), text: "uebersicht.produkt" },
  { key: "preis", wert: (z) => z.price_usd },
  { key: "referenz", wert: (z) => z.reference_usd },
  { key: "staat", wert: (z) => z.state_price_usd ?? -1 },
  { key: "nachfrage", wert: (z) => z.demand_last_month },
  { key: "verkauft", wert: (z) => z.sold_last_month },
  { key: "ein_ausfuhr", wert: (z) => z.imported_last_month - z.exported_last_month },
  { key: "anbieter", wert: (z) => z.sellers },
  { key: "fuehrer", wert: (z) => z.leader_share },
  { key: "eigener_preis", wert: (z) => z.own_price_usd ?? -1 },
  { key: "anteil", wert: (z) => z.own_share },
];

export function MarktAnsicht({
  kern,
  uebersicht,
  onGeaendert,
}: {
  kern: Kern;
  uebersicht: Uebersicht;
  onGeaendert: (u: Uebersicht) => void;
}) {
  const [land, setLand] = useState(uebersicht.company.headquarters);
  const [nurEigene, setNurEigene] = useState(false);
  const { daten, fehler, neu } = useSicht(() => kern.markt(land), `${uebersicht.date}/${land}`);
  const { ausfuehren, meldung } = useBefehl(kern, onGeaendert, neu);
  const [sortierung, setSortierung] = useState<{ spalte: number; ab: boolean }>({
    spalte: 0,
    ab: false,
  });
  const sp = SPALTEN[sortierung.spalte]!;
  const zeilen = (daten?.lines ?? [])
    .filter((z) => !nurEigene || z.own_price_usd !== null)
    .sort((a, b) => {
      const x = sp.wert(a);
      const y = sp.wert(b);
      const c = typeof x === "string" ? x.localeCompare(String(y), "de") : x - (y as number);
      return sortierung.ab ? -c : c;
    });
  return (
    <main className="ansicht" id="markt">
      <h1 className="unsichtbar">{t("ansicht.markt")}</h1>
      <div className="zeilenformular">
        <label>
          {t("markt.land")}
          <select id="markt_land" value={land} onChange={(e) => setLand(e.target.value)}>
            {LAENDER.map((k) => (
              <option key={k} value={k}>
                {landName(k)}
              </option>
            ))}
          </select>
        </label>
        <label className="ankreuz">
          <input
            type="checkbox"
            checked={nurEigene}
            onChange={(e) => setNurEigene(e.target.checked)}
          />
          {t("markt.nur_eigene")}
        </label>
      </div>
      {!daten ? (
        <FehlerText fehler={fehler} />
      ) : (
        <>
          <div className="tabelle">
            <table aria-label={t("markt.titel", { land: landName(daten.country) })}>
              <thead>
                <tr>
                  {SPALTEN.map((c, i) => (
                    <th
                      key={c.key}
                      className={i > 0 ? "zahl" : undefined}
                      aria-sort={
                        sortierung.spalte === i
                          ? sortierung.ab
                            ? "descending"
                            : "ascending"
                          : undefined
                      }
                    >
                      <button
                        type="button"
                        className="spaltenkopf"
                        onClick={() =>
                          setSortierung((alt) => ({
                            spalte: i,
                            ab: alt.spalte === i ? !alt.ab : i > 0,
                          }))
                        }
                      >
                        {t(c.text ?? `markt.${c.key}`)}
                        {sortierung.spalte === i ? (sortierung.ab ? " ▼" : " ▲") : ""}
                      </button>
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {zeilen.map((z) => (
                  <tr key={z.product} className={z.own_price_usd !== null ? "eigen" : undefined}>
                    <td>{t(`produkt.${z.product}`)}</td>
                    <td className="zahl">{formatGeld(z.price_usd)}</td>
                    <td className="zahl gedaempft">{formatGeld(z.reference_usd)}</td>
                    <td className="zahl">
                      {z.state_price_usd === null ? "–" : formatGeld(z.state_price_usd)}
                    </td>
                    <td className="zahl">{formatZahl(z.demand_last_month, 1)}</td>
                    <td className="zahl">{formatZahl(z.sold_last_month, 1)}</td>
                    <td className="zahl">
                      {`${formatZahl(z.imported_last_month, 1)} / ${formatZahl(z.exported_last_month, 1)}`}
                    </td>
                    <td className="zahl">{z.sellers}</td>
                    <td className="name-kurz" title={z.leader ?? undefined}>
                      {z.leader === null ? "–" : `${formatProzent(z.leader_share)} ${z.leader}`}
                    </td>
                    <td className="zahl">
                      {z.own_price_usd === null
                        ? "–"
                        : `${formatGeld(z.own_price_usd)} (${formatZahl(z.own_sold_last_month, 1)})`}
                    </td>
                    <td className="zahl">
                      {z.own_price_usd === null && z.own_share === 0
                        ? "–"
                        : formatProzent(z.own_share)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <p className="gedaempft">{t("markt.hinweis")}</p>
          <h2>{t("markt.marke_titel")}</h2>
          <p className="gedaempft">
            {daten.medium === null
              ? t("markt.kein_werbemittel")
              : t("markt.werbemittel", {
                  mittel: t(`werbemittel.${daten.medium}`),
                  betrag: formatGeld(daten.reach_usd),
                })}
          </p>
          <div className="tabelle">
            <table aria-label={t("markt.marke_tabelle", { land: landName(daten.country) })}>
              <thead>
                <tr>
                  <th>{t("markt.warengruppe")}</th>
                  <th className="zahl">{t("markt.bekanntheit")}</th>
                  <th>{t("markt.konkurrenz")}</th>
                  <th>{t("markt.werbebudget")}</th>
                </tr>
              </thead>
              <tbody>
                {daten.brands.map((m) => (
                  <tr key={m.group}>
                    <td>{t(`warengruppe.${m.group}`)}</td>
                    <td className="zahl">{formatProzent(m.own_awareness)}</td>
                    <td>
                      {m.top === null
                        ? "–"
                        : t("markt.konkurrenz_wert", {
                            firma: m.top,
                            bekanntheit: formatProzent(m.top_awareness),
                          })}
                    </td>
                    <td>
                      <Werbebudget
                        key={`${daten.country}/${m.group}/${m.budget_usd}`}
                        land={daten.country}
                        marke={m}
                        ausfuehren={ausfuehren}
                      />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <Rueckmeldung meldung={meldung} />
          <p className="gedaempft">{t("markt.marke_hinweis")}</p>
        </>
      )}
    </main>
  );
}

function Werbebudget({
  land,
  marke,
  ausfuehren,
}: {
  land: string;
  marke: Marke;
  ausfuehren: ReturnType<typeof useBefehl>["ausfuehren"];
}) {
  const [betrag, setBetrag] = useState(marke.budget_usd);
  const gruppe = t(`warengruppe.${marke.group}`);
  return (
    <form
      className="inline"
      aria-label={t("markt.werbung_fuer", { gruppe })}
      onSubmit={(e) => {
        e.preventDefault();
        void ausfuehren({
          SetAdvertising: { country: land, group: marke.group, budget: geld(betrag) },
        });
      }}
    >
      <input
        className="schmal"
        type="number"
        min={0}
        step="any"
        aria-label={t("markt.budget_fuer", { gruppe })}
        value={betrag}
        onChange={(e) => setBetrag(Number(e.target.value))}
      />
      <button type="submit" className="schlicht" disabled={!(betrag >= 0)}>
        {t("markt.budget_setzen")}
      </button>
    </form>
  );
}
