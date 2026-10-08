import { useState } from "react";
import { formatGeld, formatProzent, landName } from "../format";
import type { Controlling, ControllingKnoten, Kern } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { Unterreiter, useSicht } from "./gemeinsam";
import { Verlauf } from "./Grafik";

type Zeitraum = Controlling["period"];

/** Name of a level: companies by name, the rest by text key; sites with their country. */
function name(n: ControllingKnoten): string {
  if (n.level === "firma") return n.name;
  const text = t(n.name);
  return n.country ? `${text} · ${landName(n.country)}` : text;
}

function Abweichung({ n }: { n: ControllingKnoten }) {
  if (n.previous_result_usd === null) return <>–</>;
  const diff = n.result_usd - n.previous_result_usd;
  const basis = Math.abs(n.previous_result_usd);
  return (
    <>
      {formatGeld(n.previous_result_usd)}
      <br />
      <small className={diff < 0 ? "fehlertext" : "gedaempft"}>
        {diff >= 0 ? "+" : "−"}
        {formatGeld(Math.abs(diff))}
        {basis > 0 && ` (${diff >= 0 ? "+" : "−"}${formatProzent(Math.abs(diff) / basis)})`}
      </small>
    </>
  );
}

/** One row and, when open, its children. */
function Zeile({
  n,
  tiefe,
  offen,
  umschalten,
  gewaehlt,
  waehlen,
}: {
  n: ControllingKnoten;
  tiefe: number;
  offen: Set<string>;
  umschalten: (key: string) => void;
  gewaehlt: string;
  waehlen: (key: string) => void;
}) {
  const auf = offen.has(n.key);
  return (
    <>
      <tr className={gewaehlt === n.key ? "gewaehlt" : undefined}>
        <td style={{ paddingLeft: `${0.25 + tiefe * 1.2}rem` }}>
          {n.children.length > 0 ? (
            <button
              type="button"
              className="aufklapper"
              aria-expanded={auf}
              aria-label={t(auf ? "controlling.zuklappen" : "controlling.aufklappen", {
                name: name(n),
              })}
              onClick={() => umschalten(n.key)}
            >
              {auf ? "▾" : "▸"}
            </button>
          ) : (
            <span className="einrueckung" />
          )}{" "}
          <button type="button" className="verweis" onClick={() => waehlen(n.key)}>
            {name(n)}
          </button>
        </td>
        <td className="zahl">{formatGeld(n.revenue_usd)}</td>
        <td className="zahl">{formatGeld(n.margin1_usd)}</td>
        <td className="zahl">{formatGeld(n.margin2_usd)}</td>
        <td className="zahl">
          <strong>{formatGeld(n.result_usd)}</strong>
        </td>
        <td className="zahl">
          <Abweichung n={n} />
        </td>
      </tr>
      {auf &&
        n.children.map((c) => (
          <Zeile
            key={c.key}
            n={c}
            tiefe={tiefe + 1}
            offen={offen}
            umschalten={umschalten}
            gewaehlt={gewaehlt}
            waehlen={waehlen}
          />
        ))}
    </>
  );
}

function suche(n: ControllingKnoten, key: string): ControllingKnoten | null {
  if (n.key === key) return n;
  for (const c of n.children) {
    const x = suche(c, key);
    if (x) return x;
  }
  return null;
}

/** The selected level: margins, cost types and the last months. */
function Einzelheiten({ n, monate }: { n: ControllingKnoten; monate: string[] }) {
  const stufen: [string, number][] = [
    ["controlling.umsatz", n.revenue_usd],
    ["controlling.variabel", n.variable_usd],
    ["controlling.db1", n.margin1_usd],
    ["controlling.fix", n.fixed_usd],
    ["controlling.db2", n.margin2_usd],
    ["controlling.uebrige", n.other_usd],
    ["controlling.ergebnis", n.result_usd],
  ];
  const reihe = monate.slice(monate.length - n.series_usd.length);
  return (
    <section aria-label={t("controlling.einzelheiten", { name: name(n) })}>
      <h3>{name(n)}</h3>
      <div className="raster">
        <div className="tabelle">
          <table className="rechnung-tabelle" aria-label={t("controlling.stufen")}>
            <tbody>
              {stufen.map(([k, v]) => (
                <tr
                  key={k}
                  className={
                    k === "controlling.db1" ||
                    k === "controlling.db2" ||
                    k === "controlling.ergebnis"
                      ? "summe"
                      : undefined
                  }
                >
                  <th scope="row">{t(k)}</th>
                  <td className="zahl">{formatGeld(v)}</td>
                  <td className="zahl gedaempft">
                    {n.revenue_usd > 0 ? formatProzent(v / n.revenue_usd) : ""}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <div className="tabelle">
          <table className="rechnung-tabelle" aria-label={t("controlling.kostenarten")}>
            <tbody>
              {n.costs.map((c) => (
                <tr key={c.key}>
                  <th scope="row">{t(c.key)}</th>
                  <td className="zahl">{formatGeld(c.usd)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
      {n.series_usd.length > 0 && (
        <Verlauf
          name={t("controlling.verlauf", { name: name(n) })}
          monate={reihe}
          werte={n.series_usd}
          art="balken"
        />
      )}
    </section>
  );
}

function Baum({ daten }: { daten: Controlling }) {
  const root = daten.root;
  const [offen, setOffen] = useState<Set<string>>(() => {
    const s = new Set<string>();
    if (root) {
      s.add(root.key);
      if (root.level === "konzern") root.children.forEach((c) => s.add(c.key));
    }
    return s;
  });
  const [gewaehlt, setGewaehlt] = useState(root?.key ?? "");
  if (!root) return <p className="gedaempft">{t("uebersicht.noch_kein_monat")}</p>;
  const umschalten = (key: string) =>
    setOffen((s) => {
      const neu = new Set(s);
      if (neu.has(key)) neu.delete(key);
      else neu.add(key);
      return neu;
    });
  const auswahl = suche(root, gewaehlt) ?? root;
  return (
    <>
      <div className="tabelle">
        <table className="controlling-baum" aria-label={t("controlling.ebenen")}>
          <thead>
            <tr>
              <th>{t("controlling.ebene")}</th>
              <th className="zahl">{t("controlling.umsatz")}</th>
              <th className="zahl">{t("controlling.db1")}</th>
              <th className="zahl">{t("controlling.db2")}</th>
              <th className="zahl">{t("controlling.ergebnis")}</th>
              <th className="zahl">{t("controlling.vorperiode")}</th>
            </tr>
          </thead>
          <tbody>
            <Zeile
              n={root}
              tiefe={0}
              offen={offen}
              umschalten={umschalten}
              gewaehlt={auswahl.key}
              waehlen={setGewaehlt}
            />
          </tbody>
        </table>
      </div>
      <Einzelheiten n={auswahl} monate={daten.months} />
    </>
  );
}

/** Finances → controlling (W7): where, with what and by what money is made or lost. */
export function ControllingAnsicht({ kern, stand }: { kern: Kern; stand: string }) {
  const [zeitraum, setZeitraum] = useState<Zeitraum>("jahr");
  const { daten, fehler } = useSicht(() => kern.controlling(zeitraum), `${stand}/${zeitraum}`);
  if (!daten) return <FehlerText fehler={fehler} />;
  return (
    <section aria-labelledby="controlling">
      <h2 id="controlling">{t("controlling.titel")}</h2>
      <p className="feld-hilfe">{t("controlling.hilfe")}</p>
      <Unterreiter
        name={t("finanzen.zeitraum")}
        bereiche={daten.periods.map((p) => ({ key: p, text: t(`controlling.zeitraum.${p}`) }))}
        aktiv={zeitraum}
        onWahl={setZeitraum}
      />
      <Baum key={`${daten.period}/${daten.root?.key ?? ""}`} daten={daten} />
    </section>
  );
}
