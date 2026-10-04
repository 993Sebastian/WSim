import { useMemo, useState, type FormEvent, type ReactNode } from "react";
import { formatDatum, formatGeld, formatZahl, landName } from "../format";
import {
  geld,
  type AnlageDetail,
  type Befehl,
  type Kern,
  type StandortDetail,
  type Uebersicht,
  type Versorgung,
} from "../kern";
import type { Produktion } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { Rueckmeldung, useBefehl, useSicht } from "./gemeinsam";

type Ausfuehren = (...b: Befehl[]) => Promise<boolean>;

function ursacheText(a: AnlageDetail): string {
  if (!a.cause) return "";
  const d = a.cause.detail;
  let detail = "";
  if (d && a.cause.key === "ursache.arbeitskraefte") {
    const [stufe, richtung] = d.split(".");
    detail = t(`qualifikation.${stufe}`) + (richtung ? ` (${t(`fachrichtung.${richtung}`)})` : "");
  } else if (d) {
    detail = t(`produkt.${d}`);
  }
  return t(a.cause.key, { detail });
}

export function ProduktionAnsicht({
  kern,
  uebersicht,
  onGeaendert,
}: {
  kern: Kern;
  uebersicht: Uebersicht;
  onGeaendert: (u: Uebersicht) => void;
}) {
  const { daten, fehler, neu } = useSicht(() => kern.produktion(), uebersicht.date);
  const { ausfuehren, meldung } = useBefehl(kern, onGeaendert, neu);
  if (!daten) return <FehlerText fehler={fehler} />;
  return (
    <main className="ansicht" id="produktion">
      <h1 className="unsichtbar">{t("ansicht.produktion")}</h1>
      <Rueckmeldung meldung={meldung} />
      <StandortGruenden
        produktion={daten}
        heimat={uebersicht.company.headquarters}
        ausfuehren={ausfuehren}
      />
      {daten.sites.map((s) => (
        <Standort key={s.index} standort={s} produktion={daten} ausfuehren={ausfuehren} />
      ))}
    </main>
  );
}

function StandortGruenden({
  produktion,
  heimat,
  ausfuehren,
}: {
  produktion: Produktion;
  heimat: string;
  ausfuehren: Ausfuehren;
}) {
  const [land, setLand] = useState(heimat);
  const [art, setArt] = useState(produktion.site_types[1]?.kind ?? "Factory");
  const laender = useMemo(
    () =>
      Object.keys(import.meta.glob("../../../data/laender/*.yaml"))
        .map((p) => p.split("/").pop()!.replace(".yaml", ""))
        .map((k) => ({ k, name: landName(k) }))
        .sort((a, b) => a.name.localeCompare(b.name, "de")),
    [],
  );
  const gruenden = (e: FormEvent) => {
    e.preventDefault();
    void ausfuehren({ FoundSite: { country: land, kind: art } });
  };
  return (
    <form className="zeilenformular" onSubmit={gruenden} aria-label={t("produktion.gruenden")}>
      <strong>{t("produktion.gruenden")}</strong>
      <label>
        {t("produktion.land")}
        <select id="gruenden_land" value={land} onChange={(e) => setLand(e.target.value)}>
          {laender.map(({ k, name }) => (
            <option key={k} value={k}>
              {name}
            </option>
          ))}
        </select>
      </label>
      <label>
        {t("produktion.standorttyp")}
        <select id="gruenden_typ" value={art} onChange={(e) => setArt(e.target.value)}>
          {produktion.site_types.map((s) => (
            <option key={s.kind} value={s.kind}>
              {t(s.kind_text)} ({formatGeld(s.cost_usd)})
            </option>
          ))}
        </select>
      </label>
      <button type="submit">{t("produktion.gruenden_knopf")}</button>
    </form>
  );
}

function Standort({
  standort: s,
  produktion,
  ausfuehren,
}: {
  standort: StandortDetail;
  produktion: Produktion;
  ausfuehren: Ausfuehren;
}) {
  const baubar = produktion.facilities
    .filter((f) => f.site_type === s.kind)
    .sort((a, b) => a.investment_usd - b.investment_usd);
  const [anlage, setAnlage] = useState(baubar[0]?.key ?? "");
  const [anzahl, setAnzahl] = useState(1);
  // Products made here or in stock can be offered; orders outside the recipes stay visible.
  const angebotbar = new Set([
    ...s.slots.map((a) => a.product).filter((p): p is string => !!p),
    ...s.stock.map((l) => l.product),
  ]);
  const ohneAngebot = [...angebotbar].filter((p) => !s.offers.some((o) => o.product === p));
  const einkaufZeilen: Versorgung[] = [
    ...s.inputs,
    ...s.orders
      .filter((o) => !s.inputs.some((v) => v.product === o.product))
      .map((o) => ({
        product: o.product,
        need_per_day: 0,
        stock: s.stock.find((l) => l.product === o.product)?.quantity ?? 0,
        days: null,
        own: false,
        ordered: true,
      })),
  ];

  return (
    <article className="standort" aria-label={`${t(s.kind_text)} ${landName(s.country)}`}>
      <h2>
        {t(s.kind_text)} · {landName(s.country)}
        {s.deposit && ` · ${t(`lagerstaette.${s.deposit}`)}`}
        {s.deposit_ready && s.deposit_ready > produktion.date && (
          <small className="gedaempft">
            {t("produktion.erschliessung_bis", { datum: formatDatum(s.deposit_ready) })}
          </small>
        )}
        <small>{t("uebersicht.beschaeftigte", { anzahl: formatZahl(s.workers, 1) })}</small>
      </h2>

      {s.slots.length > 0 && (
        <div className="tabelle">
          <table>
            <thead>
              <tr>
                <th>{t("uebersicht.anlage")}</th>
                <th>{t("produktion.rezept")}</th>
                <th className="zahl">{t("uebersicht.auslastung")}</th>
                <th className="zahl">{t("produktion.leistung")}</th>
                <th>{t("produktion.ursache")}</th>
              </tr>
            </thead>
            <tbody>
              {s.slots.map((a) => (
                <AnlageZeile
                  // A new key after a change resets the form to the values of the core.
                  key={`${a.index}/${a.utilization}`}
                  anlage={a}
                  standort={s}
                  produktion={produktion}
                  ausfuehren={ausfuehren}
                />
              ))}
            </tbody>
          </table>
        </div>
      )}

      {baubar.length > 0 && (
        <form
          className="zeilenformular"
          aria-label={t("produktion.bauen")}
          onSubmit={(e) => {
            e.preventDefault();
            void ausfuehren({ BuildFacility: { site: s.index, facility: anlage, count: anzahl } });
          }}
        >
          <label>
            {t("produktion.bauen")}
            <select
              id={`bauen_${s.index}`}
              value={anlage}
              onChange={(e) => setAnlage(e.target.value)}
            >
              {baubar.map((f) => (
                <option key={f.key} value={f.key}>
                  {t(`anlage.${f.key}`)} – {formatGeld(f.investment_usd)},{" "}
                  {t("produktion.bauzeit", { tage: f.build_days })}
                </option>
              ))}
            </select>
          </label>
          <label>
            {t("produktion.anzahl")}
            <input
              id={`anzahl_${s.index}`}
              className="schmal"
              type="number"
              min={1}
              value={anzahl}
              onChange={(e) => setAnzahl(Math.max(1, Math.floor(Number(e.target.value) || 1)))}
            />
          </label>
          <button type="submit">{t("produktion.bauen_knopf")}</button>
        </form>
      )}

      {s.kind === "Extraction" && (
        <Erschliessung standort={s} datum={produktion.date} ausfuehren={ausfuehren} />
      )}

      {s.kind !== "ResearchCenter" && (
        <>
          <h3>{t("produktion.vorprodukte")}</h3>
          <div className="tabelle">
            <table aria-label={t("produktion.vorprodukte")}>
              <thead>
                <tr>
                  <th>{t("uebersicht.produkt")}</th>
                  <th className="zahl">{t("produktion.bedarf")}</th>
                  <th className="zahl">{t("uebersicht.lager")}</th>
                  <th className="zahl">{t("produktion.reichweite")}</th>
                  <th className="zahl">{t("produktion.gekauft_vormonat")}</th>
                  <th>{t("produktion.einkauf")}</th>
                </tr>
              </thead>
              <tbody>
                {einkaufZeilen.map((v) => {
                  const auftrag = s.orders.find((o) => o.product === v.product);
                  const tage = v.days ?? Infinity;
                  const zustand = tage < 1 ? "fehlt" : tage < 7 ? "knapp" : "gut";
                  return (
                    <tr key={v.product}>
                      <td>
                        {t(`produkt.${v.product}`)}
                        {v.own && <span className="marke">{t("produktion.eigen")}</span>}
                      </td>
                      <td className="zahl">
                        {v.need_per_day > 0 ? formatZahl(v.need_per_day, 2) : "–"}
                      </td>
                      <td className="zahl">{formatZahl(v.stock, 1)}</td>
                      <td className={`zahl zustand-${zustand}`}>
                        {v.days === null
                          ? "–"
                          : t("produktion.tage", { tage: formatZahl(v.days, 0) })}
                      </td>
                      <td className="zahl">
                        {auftrag ? formatZahl(auftrag.bought_last_month, 1) : "–"}
                      </td>
                      <td>
                        <EinkaufFormular
                          key={`${v.product}/${auftrag?.target ?? Math.ceil(v.need_per_day * 20)}`}
                          standort={s.index}
                          produkt={v.product}
                          auftrag={auftrag}
                          bedarf={v.need_per_day}
                          ausfuehren={ausfuehren}
                        />
                      </td>
                    </tr>
                  );
                })}
                <NeueZeile
                  titel={t("produktion.neuer_einkauf")}
                  spalten={5}
                  produkte={produktion.products.filter(
                    (p) => !einkaufZeilen.some((v) => v.product === p),
                  )}
                >
                  {(p) => (
                    <EinkaufFormular
                      key={p}
                      standort={s.index}
                      produkt={p}
                      auftrag={undefined}
                      bedarf={0}
                      ausfuehren={ausfuehren}
                    />
                  )}
                </NeueZeile>
              </tbody>
            </table>
          </div>

          <h3>{t("produktion.verkauf")}</h3>
          <div className="tabelle">
            <table aria-label={t("produktion.verkauf")}>
              <thead>
                <tr>
                  <th>{t("uebersicht.produkt")}</th>
                  <th className="zahl">{t("produktion.preis")}</th>
                  <th className="zahl">{t("produktion.verkauft_vormonat")}</th>
                  <th className="zahl">{t("produktion.verkauft_monat")}</th>
                  <th>{t("produktion.preisregel")}</th>
                </tr>
              </thead>
              <tbody>
                {s.offers.map((o) => (
                  <tr key={o.product}>
                    <td>{t(`produkt.${o.product}`)}</td>
                    <td className="zahl">{formatGeld(o.price_usd)}</td>
                    <td className="zahl">{formatZahl(o.sold_last_month, 1)}</td>
                    <td className="zahl">{formatZahl(o.sold_month, 1)}</td>
                    <td>
                      <VerkaufFormular
                        standort={s.index}
                        produkt={o.product}
                        angebot={o}
                        ausfuehren={ausfuehren}
                      />
                    </td>
                  </tr>
                ))}
                {ohneAngebot.map((p) => (
                  <tr key={p}>
                    <td>{t(`produkt.${p}`)}</td>
                    <td className="zahl">–</td>
                    <td className="zahl">–</td>
                    <td className="zahl">–</td>
                    <td>
                      <VerkaufFormular
                        standort={s.index}
                        produkt={p}
                        angebot={null}
                        ausfuehren={ausfuehren}
                      />
                    </td>
                  </tr>
                ))}
                <NeueZeile
                  titel={t("produktion.neues_angebot")}
                  spalten={4}
                  produkte={produktion.products.filter(
                    (p) => !s.offers.some((o) => o.product === p) && !ohneAngebot.includes(p),
                  )}
                >
                  {(p) => (
                    <VerkaufFormular
                      key={p}
                      standort={s.index}
                      produkt={p}
                      angebot={null}
                      ausfuehren={ausfuehren}
                    />
                  )}
                </NeueZeile>
              </tbody>
            </table>
          </div>
        </>
      )}

      <p className="lagerzeile">
        <strong>{t("uebersicht.lager")}:</strong>{" "}
        {s.stock.length === 0
          ? t("uebersicht.lager_leer")
          : s.stock
              .map((l) => `${t(`produkt.${l.product}`)} ${formatZahl(l.quantity, 1)}`)
              .join(" · ")}
      </p>
    </article>
  );
}

function AnlageZeile({
  anlage: a,
  standort: s,
  produktion,
  ausfuehren,
}: {
  anlage: AnlageDetail;
  standort: StandortDetail;
  produktion: Produktion;
  ausfuehren: Ausfuehren;
}) {
  const rezepte = produktion.recipes.filter((r) => r.facility === a.facility);
  const [auslastung, setAuslastung] = useState(Math.round(a.utilization * 100));
  const setze = (rezept: string | null, prozent: number) =>
    void ausfuehren({
      SetProduction: {
        site: s.index,
        slot: a.index,
        recipe: rezept,
        utilization: Math.min(100, Math.max(0, prozent)) / 100,
      },
    });
  const ursache = ursacheText(a);
  return (
    <tr>
      <td>
        {a.count > 1 ? `${a.count} × ` : ""}
        {t(`anlage.${a.facility}`)}
        {a.ready > produktion.date && (
          <small className="gedaempft">
            {" "}
            {t("uebersicht.im_bau", { datum: formatDatum(a.ready) })}
          </small>
        )}
      </td>
      <td>
        {s.kind === "ResearchCenter" ? (
          <span className="gedaempft">{t("produktion.labor")}</span>
        ) : (
          <select
            aria-label={t("produktion.rezept_von", { anlage: t(`anlage.${a.facility}`) })}
            value={a.recipe ?? ""}
            onChange={(e) => setze(e.target.value || null, auslastung)}
          >
            <option value="">{t("produktion.kein_rezept")}</option>
            {rezepte.map((r) => (
              <option key={r.key} value={r.key}>
                {t(`rezept.${r.key}`)}
              </option>
            ))}
          </select>
        )}
      </td>
      <td className="zahl">
        <input
          className="schmal"
          type="number"
          min={0}
          max={100}
          step={5}
          aria-label={t("produktion.auslastung_von", { anlage: t(`anlage.${a.facility}`) })}
          value={auslastung}
          onChange={(e) => setAuslastung(Number(e.target.value))}
          onBlur={() =>
            auslastung !== Math.round(a.utilization * 100) && setze(a.recipe, auslastung)
          }
          onKeyDown={(e) => e.key === "Enter" && e.currentTarget.blur()}
        />{" "}
        %
      </td>
      <td className="zahl">
        {formatZahl(a.made_per_day, 2)} / {formatZahl(a.planned_per_day, 2)}
      </td>
      <td
        className={a.cause ? "ursache" : ""}
        title={a.cause ? t(`${a.cause.key}.hilfe`) : undefined}
      >
        {ursache || (a.recipe ? t("produktion.laeuft") : "")}
        {a.cause && <span className="unsichtbar"> {t(`${a.cause.key}.hilfe`)}</span>}
      </td>
    </tr>
  );
}

function EinkaufFormular({
  standort,
  produkt,
  auftrag,
  bedarf,
  ausfuehren,
}: {
  standort: number;
  produkt: string;
  auftrag: { target: number; max_price_usd: number } | undefined;
  bedarf: number;
  ausfuehren: Ausfuehren;
}) {
  const [ziel, setZiel] = useState(auftrag ? auftrag.target : Math.ceil(bedarf * 20));
  const [preis, setPreis] = useState(auftrag ? Math.round(auftrag.max_price_usd) : 0);
  const name = t(`produkt.${produkt}`);
  return (
    <form
      className="inline"
      aria-label={t("produktion.einkauf_von", { produkt: name })}
      onSubmit={(e) => {
        e.preventDefault();
        void ausfuehren({
          SetPurchase: {
            site: standort,
            product: produkt,
            target: ziel,
            max_price: geld(preis),
            min_quality: 0,
          },
        });
      }}
    >
      <input
        className="schmal"
        type="number"
        min={0}
        step="any"
        aria-label={t("produktion.ziellager", { produkt: name })}
        value={ziel}
        onChange={(e) => setZiel(Number(e.target.value))}
      />
      <input
        className="schmal"
        type="number"
        min={0}
        step="any"
        aria-label={t("produktion.hoechstpreis", { produkt: name })}
        placeholder={t("produktion.hoechstpreis_kurz")}
        value={preis || ""}
        onChange={(e) => setPreis(Number(e.target.value))}
      />
      <button type="submit" className="schlicht" disabled={!auftrag && !(ziel > 0 && preis > 0)}>
        {auftrag ? t("produktion.aendern") : t("produktion.kaufen")}
      </button>
      {auftrag && (
        <button
          type="button"
          className="schlicht"
          onClick={() =>
            void ausfuehren({
              SetPurchase: {
                site: standort,
                product: produkt,
                target: 0,
                max_price: 0,
                min_quality: 0,
              },
            })
          }
        >
          {t("produktion.stoppen")}
        </button>
      )}
    </form>
  );
}

function VerkaufFormular({
  standort,
  produkt,
  angebot,
  ausfuehren,
}: {
  standort: number;
  produkt: string;
  angebot: { mode: string; floor_usd: number; price_usd: number; keep: number } | null;
  ausfuehren: Ausfuehren;
}) {
  const [art, setArt] = useState<"markt" | "fest">(angebot?.mode === "fest" ? "fest" : "markt");
  const [betrag, setBetrag] = useState(
    Math.round(angebot ? (angebot.mode === "fest" ? angebot.price_usd : angebot.floor_usd) : 0),
  );
  const name = t(`produkt.${produkt}`);
  return (
    <form
      className="inline"
      aria-label={t("produktion.verkauf_von", { produkt: name })}
      onSubmit={(e) => {
        e.preventDefault();
        void ausfuehren({
          SetSale: {
            site: standort,
            product: produkt,
            mode:
              art === "fest"
                ? { Fixed: geld(betrag) }
                : { Market: { markup: 0, floor: geld(betrag) } },
            keep: angebot?.keep ?? 0,
          },
        });
      }}
    >
      <select
        aria-label={t("produktion.preisart", { produkt: name })}
        value={art}
        onChange={(e) => setArt(e.target.value as "markt" | "fest")}
      >
        <option value="markt">{t("produktion.marktpreis")}</option>
        <option value="fest">{t("produktion.festpreis")}</option>
      </select>
      <input
        className="schmal"
        type="number"
        min={0}
        step="any"
        aria-label={
          art === "fest"
            ? t("produktion.festpreis_von", { produkt: name })
            : t("produktion.untergrenze_von", { produkt: name })
        }
        value={betrag || ""}
        placeholder={art === "fest" ? t("produktion.festpreis") : t("produktion.untergrenze")}
        onChange={(e) => setBetrag(Number(e.target.value))}
      />
      <button type="submit" className="schlicht">
        {angebot ? t("produktion.aendern") : t("produktion.anbieten")}
      </button>
      {angebot && (
        <button
          type="button"
          className="schlicht"
          onClick={() =>
            void ausfuehren({ SetSale: { site: standort, product: produkt, mode: null, keep: 0 } })
          }
        >
          {t("produktion.stoppen")}
        </button>
      )}
    </form>
  );
}

/** Last row of a table: pick another product, then the form for it. */
function NeueZeile({
  titel,
  spalten,
  produkte,
  children,
}: {
  titel: string;
  spalten: number;
  produkte: string[];
  children: (produkt: string) => ReactNode;
}) {
  const [produkt, setProdukt] = useState("");
  const sortiert = [...produkte].sort((a, b) =>
    t(`produkt.${a}`).localeCompare(t(`produkt.${b}`), "de"),
  );
  return (
    <tr className="neue-zeile">
      <td colSpan={spalten}>
        <label>
          {titel}{" "}
          <select aria-label={titel} value={produkt} onChange={(e) => setProdukt(e.target.value)}>
            <option value="">–</option>
            {sortiert.map((p) => (
              <option key={p} value={p}>
                {t(`produkt.${p}`)}
              </option>
            ))}
          </select>
        </label>
      </td>
      <td>{produkt && children(produkt)}</td>
    </tr>
  );
}

function Erschliessung({
  standort: s,
  datum,
  ausfuehren,
}: {
  standort: StandortDetail;
  datum: string;
  ausfuehren: Ausfuehren;
}) {
  const [lager, setLager] = useState(s.free_deposits[0]?.key ?? "");
  if (s.deposit) return null;
  if (s.free_deposits.length === 0)
    return (
      <p className="gedaempft">{t("produktion.keine_lagerstaette", { jahr: datum.slice(0, 4) })}</p>
    );
  return (
    <form
      className="zeilenformular"
      aria-label={t("produktion.erschliessen")}
      onSubmit={(e) => {
        e.preventDefault();
        void ausfuehren({ DevelopDeposit: { site: s.index, deposit: lager } });
      }}
    >
      <label>
        {t("produktion.erschliessen")}
        <select id={`lager_${s.index}`} value={lager} onChange={(e) => setLager(e.target.value)}>
          {s.free_deposits.map((d) => (
            <option key={d.key} value={d.key}>
              {t(`lagerstaette.${d.key}`)} ({t(`produkt.${d.resource}`)}) – {formatGeld(d.cost_usd)}
              , {t("produktion.bauzeit", { tage: d.days })},{" "}
              {t("produktion.foerderung_jahr", { menge: formatZahl(d.output_per_year) })}
            </option>
          ))}
        </select>
      </label>
      <button type="submit">{t("produktion.erschliessen_knopf")}</button>
    </form>
  );
}
