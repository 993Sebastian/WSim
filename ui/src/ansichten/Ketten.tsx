// Production chains (M25, docs/BEDIENUNG.md): how each end product comes about, from
// the raw materials up, with the unit costs at home, what the player covers already
// and where its plants get stuck. "What pays next, and what do I need for it?"
import { useId, useState } from "react";
import { formatPreis, formatProzent, formatZahl, landName } from "../format";
import type { Kern, Ketten, KettenProdukt } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { SuchAuswahl, useSicht } from "./gemeinsam";

/** Whether the player has anything to do with a product. */
const beteiligt = (p: KettenProdukt | undefined) => !!p && (p.makes || p.buys || p.sells);

/** The chains that touch something the player makes, buys or sells. */
function eigeneKetten(daten: Ketten, index: Map<string, KettenProdukt>): string[] {
  const enthaelt = (produkt: string, pfad: string[]): boolean => {
    const p = index.get(produkt);
    if (beteiligt(p)) return true;
    if (!p?.recipe || pfad.includes(produkt)) return false;
    return p.recipe.inputs.some(([i]) => enthaelt(i, [...pfad, produkt]));
  };
  return daten.roots.filter((e) => enthaelt(e, []));
}

export function KettenAnsicht({
  kern,
  stand,
  onProdukt,
}: {
  kern: Kern;
  /** Date of the game: the view reloads after a round. */
  stand: string;
  /** Opens the market of a product in the home country. */
  onProdukt: (produkt: string) => void;
}) {
  const { daten, fehler } = useSicht(() => kern.ketten(), stand);
  const [wahl, setWahl] = useState<string | null>(null);
  const id = useId();
  if (!daten) return <FehlerText fehler={fehler} />;
  const index = new Map(daten.products.map((p) => [p.product, p]));
  const eigene = eigeneKetten(daten, index);
  const spitze = wahl ?? eigene[0] ?? daten.roots[0];
  return (
    <section className="ketten" aria-label={t("markt.ketten")}>
      <p className="feld-hilfe">{t("ketten.hilfe", { land: landName(daten.country) })}</p>
      <div className="formular-zeile">
        <div className="feld">
          <label htmlFor={`${id}-spitze`}>{t("ketten.endprodukt")}</label>
          <SuchAuswahl
            id={`${id}-spitze`}
            name={t("ketten.endprodukt")}
            wert={spitze ?? ""}
            optionen={daten.roots.map((e) => ({
              wert: e,
              text: t(`produkt.${e}`) + (eigene.includes(e) ? ` – ${t("ketten.deine_kette")}` : ""),
            }))}
            onWahl={setWahl}
          />
        </div>
      </div>
      {spitze && (
        <ul className="kette">
          <Knoten produkt={spitze} index={index} pfad={[]} onProdukt={onProdukt} />
        </ul>
      )}
    </section>
  );
}

function Knoten({
  produkt,
  index,
  pfad,
  menge,
  einheitOben,
  onProdukt,
}: {
  produkt: string;
  index: Map<string, KettenProdukt>;
  /** Products above, to stop at a loop. */
  pfad: string[];
  /** Quantity per unit of the product above. */
  menge?: number;
  einheitOben?: string;
  onProdukt: (produkt: string) => void;
}) {
  const p = index.get(produkt);
  if (!p) return null;
  const e = t(`einheit.${p.unit}`);
  const r = p.recipe;
  const zeile = (
    <span className="kettenzeile">
      {menge !== undefined && einheitOben !== undefined && (
        <span className="gedaempft kettenmenge">
          {t("ketten.je", {
            menge: `${formatZahl(menge, menge < 10 ? 3 : 0)} ${e}`,
            einheit: einheitOben,
          })}
        </span>
      )}
      <button
        type="button"
        className="schlicht"
        aria-label={t("markt.oeffnen", { produkt: t(`produkt.${produkt}`) })}
        onClick={(ev) => {
          // Inside a summary the click would also fold the branch.
          ev.preventDefault();
          onProdukt(produkt);
        }}
      >
        {t(`produkt.${produkt}`)}
      </button>
      {p.makes && <span className="marke">{t("ketten.stellst_her")}</span>}
      {p.buys && <span className="marke">{t("ketten.kaufst_ein")}</span>}
      {p.sells && <span className="marke">{t("ketten.verkaufst")}</span>}
      {p.stuck.map(([ursache, detail]) => (
        <span key={`${ursache}-${detail}`} className="warntext">
          {t(`ketten.hakt.${ursache}`, { produkt: detail ? t(`produkt.${detail}`) : "" })}
        </span>
      ))}
      <span className="kettenwerte">
        {r ? (
          <>
            {t(r.extraction ? "ketten.abbau" : "ketten.verfahren", {
              anlage: t(`anlage.${r.facility}`),
            })}
            {" · "}
            {t("ketten.kosten", {
              kosten: formatPreis(r.unit_cost_usd, e),
              preis: formatPreis(p.price_usd, e),
            })}
            {r.margin !== null && (
              <span className={r.margin < 0 ? "negativ" : "positiv"}>
                {" · "}
                {t("ketten.marge", { marge: formatProzent(r.margin) })}
              </span>
            )}
            {r.missing.length > 0 && (
              <span className="warntext">
                {" · "}
                {t("ketten.fehlt", {
                  technologien: r.missing.map((k) => t(`technologie.${k}`)).join(", "),
                })}
              </span>
            )}
          </>
        ) : (
          t(
            p.state_market
              ? "ketten.staatsmarkt"
              : p.kind === "energie"
                ? "ketten.netz"
                : "ketten.nicht_erfunden",
            { preis: formatPreis(p.price_usd, e) },
          )
        )}
      </span>
    </span>
  );
  const kinder = r && !pfad.includes(produkt) ? r.inputs : [];
  if (kinder.length === 0) return <li className="kettenblatt">{zeile}</li>;
  return (
    <li>
      {/* The first two levels are open; deeper ones on demand. */}
      <details open={pfad.length < 2}>
        <summary>{zeile}</summary>
        <ul>
          {kinder.map(([eingang, q]) => (
            <Knoten
              key={eingang}
              produkt={eingang}
              index={index}
              pfad={[...pfad, produkt]}
              menge={q}
              einheitOben={e}
              onProdukt={onProdukt}
            />
          ))}
        </ul>
      </details>
    </li>
  );
}
