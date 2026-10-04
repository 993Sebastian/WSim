import { useState } from "react";
import { formatGeld, formatZahl, landName } from "../format";
import type { Kern, Markt } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { useSicht } from "./gemeinsam";
import { LAENDER } from "./laender";

type Zeile = Markt["lines"][number];
type Spalte = { key: string; wert: (z: Zeile) => number | string; text?: string };

const SPALTEN: Spalte[] = [
  { key: "produkt", wert: (z) => t(`produkt.${z.product}`), text: "uebersicht.produkt" },
  { key: "preis", wert: (z) => z.price_usd },
  { key: "referenz", wert: (z) => z.reference_usd },
  { key: "staat", wert: (z) => z.state_price_usd ?? -1 },
  { key: "nachfrage", wert: (z) => z.demand_last_month },
  { key: "verkauft", wert: (z) => z.sold_last_month },
  { key: "einfuhr", wert: (z) => z.imported_last_month },
  { key: "ausfuhr", wert: (z) => z.exported_last_month },
  { key: "anbieter", wert: (z) => z.sellers },
  { key: "eigener_preis", wert: (z) => z.own_price_usd ?? -1 },
];

export function MarktAnsicht({
  kern,
  datum,
  heimat,
}: {
  kern: Kern;
  datum: string;
  heimat: string;
}) {
  const [land, setLand] = useState(heimat);
  const [nurEigene, setNurEigene] = useState(false);
  const { daten, fehler } = useSicht(() => kern.markt(land), `${datum}/${land}`);
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
                  <td className="zahl">{formatZahl(z.imported_last_month, 1)}</td>
                  <td className="zahl">{formatZahl(z.exported_last_month, 1)}</td>
                  <td className="zahl">{z.sellers}</td>
                  <td className="zahl">
                    {z.own_price_usd === null
                      ? "–"
                      : `${formatGeld(z.own_price_usd)} (${formatZahl(z.own_sold_month, 1)})`}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      <p className="gedaempft">{t("markt.hinweis")}</p>
    </main>
  );
}
