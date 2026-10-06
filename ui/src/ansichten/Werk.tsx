// The plant view (docs/BEDIENUNG.md): one site with its areas – facilities, purchasing,
// sales, staff and costs. Every decision shows the numbers it rests on next to it, and
// the answer of the core next to its button.
import { Fragment, useId, useState, type FormEvent, type ReactNode } from "react";
import {
  ausAnzeige,
  formatDatum,
  formatGeld,
  formatMenge,
  formatPreis,
  formatProzent,
  formatZahl,
  geldEinheit,
  geldFeld,
  geldRunden,
  geldSchluessel,
  inAnzeige,
  landName,
  zahlFeld,
  zahlLesen,
} from "../format";
import type {
  AnlageDetail,
  Angebot,
  Befehl,
  Produktion,
  StandortDetail,
  Stueckkosten,
  Versorgung,
} from "../kern";
import { geld } from "../kern";
import { t } from "../texte";
import { Erklaerung, Rueckmeldung, Unterreiter, useAktion, ZahlFeld } from "./gemeinsam";

type Bereich = "anlagen" | "einkauf" | "verkauf" | "personal" | "kosten";

/** Unit of a product as text, e.g. "t" or "Stück". */
export function einheit(produktion: Produktion, produkt: string): string {
  return t(`einheit.${produktion.units[produkt] ?? "t"}`);
}

const produktName = (p: string) => t(`produkt.${p}`);

/** Labor group key "fachkraft.metall" as text. */
export function gruppenName(gruppe: string): string {
  const [stufe, richtung] = gruppe.split(".");
  return t(`qualifikation.${stufe}`) + (richtung ? ` (${t(`fachrichtung.${richtung}`)})` : "");
}

export function ursacheText(a: AnlageDetail): string {
  if (!a.cause) return "";
  const d = a.cause.detail;
  let detail = "";
  if (d && a.cause.key === "ursache.arbeitskraefte") detail = gruppenName(d);
  else if (d) detail = produktName(d);
  return t(a.cause.key, { detail });
}

/** Workers: whole persons from ten on, else one decimal (markets are scaled down). */
export function formatKoepfe(anzahl: number): string {
  return formatZahl(anzahl, anzahl >= 10 ? 0 : 1);
}

export function standortTitel(s: StandortDetail): string {
  return `${t(s.kind_text)} · ${landName(s.country)}`;
}

const BEREICHE: Bereich[] = ["anlagen", "einkauf", "verkauf", "personal", "kosten"];

export function Werk({
  standort: s,
  produktion,
  bereich: start = null,
  onZurueck,
}: {
  standort: StandortDetail;
  produktion: Produktion;
  /** Area to show first. */
  bereich?: string | null;
  onZurueck: () => void;
}) {
  const [bereich, setBereich] = useState<Bereich>(
    BEREICHE.includes(start as Bereich) ? (start as Bereich) : "anlagen",
  );
  const forschung = s.kind === "ResearchCenter";
  const engpaesse = s.slots.filter((a) => a.cause && a.cause.key !== "ursache.im_bau").length;
  const knapp = s.inputs.filter((v) => !v.own && v.days !== null && v.days < 7).length;
  const fehlend = s.staff.filter((l) => l.employed + 0.05 < l.needed).length;
  const bereiche: { key: Bereich; text: string; zaehler?: number }[] = forschung
    ? [
        { key: "anlagen", text: t("werk.anlagen"), zaehler: engpaesse },
        { key: "personal", text: t("werk.personal"), zaehler: fehlend },
      ]
    : [
        { key: "anlagen", text: t("werk.anlagen"), zaehler: engpaesse },
        { key: "einkauf", text: t("werk.einkauf"), zaehler: knapp },
        { key: "verkauf", text: t("werk.verkauf") },
        { key: "personal", text: t("werk.personal"), zaehler: fehlend },
        { key: "kosten", text: t("werk.kosten") },
      ];
  const titel = standortTitel(s);
  return (
    <section className="werk" aria-label={titel} data-tour="werk">
      <div className="werk-kopf">
        <button type="button" className="schlicht" onClick={onZurueck}>
          ← {t("werk.alle_standorte")}
        </button>
        <h2>
          {titel}
          {s.deposit && <small>{t(`lagerstaette.${s.deposit}`)}</small>}
        </h2>
      </div>
      <WerkKennzahlen s={s} />
      <Unterreiter
        name={t("werk.bereiche")}
        bereiche={bereiche}
        aktiv={bereich}
        onWahl={setBereich}
      />
      {bereich === "anlagen" && <Anlagen s={s} produktion={produktion} />}
      {bereich === "einkauf" && <Einkauf s={s} produktion={produktion} />}
      {bereich === "verkauf" && <Verkauf s={s} produktion={produktion} />}
      {bereich === "personal" && <Personal key={s.wage_premium} s={s} />}
      {bereich === "kosten" && <Kosten s={s} produktion={produktion} />}
    </section>
  );
}

function WerkKennzahlen({ s }: { s: StandortDetail }) {
  const benoetigt = s.staff.reduce((summe, l) => summe + l.needed, 0);
  const m = s.last_month;
  return (
    <dl className="kennzahlen">
      <div>
        <dt>{t("uebersicht.umsatz_monat")}</dt>
        <dd>{m ? formatGeld(m.revenue_usd) : "–"}</dd>
      </div>
      <div>
        <dt>{t("uebersicht.ergebnis_monat")}</dt>
        <dd className={m && m.result_usd < 0 ? "negativ" : ""}>
          {m ? formatGeld(m.result_usd) : t("uebersicht.noch_kein_monat")}
        </dd>
      </div>
      <div>
        <dt>{t("werk.beschaeftigte")}</dt>
        <dd>
          {formatKoepfe(s.workers)}
          {benoetigt > s.workers + 0.05 && (
            <small className="warntext">
              {" "}
              {t("werk.von_benoetigt", { anzahl: formatKoepfe(benoetigt) })}
            </small>
          )}
        </dd>
      </div>
      <div>
        <dt>{t("werk.lohnkosten_tag")}</dt>
        <dd>{formatGeld(s.wage_cost_per_day_usd)}</dd>
      </div>
    </dl>
  );
}

// --- Facilities ---

function Anlagen({ s, produktion }: { s: StandortDetail; produktion: Produktion }) {
  return (
    <div className="karten" data-tour="werk-anlagen">
      {s.slots.length === 0 && <p className="gedaempft">{t("werk.keine_anlagen")}</p>}
      {s.slots.map((a) => (
        // New values from the core start the form afresh.
        <AnlageKarte
          key={`${a.index}/${a.recipe}/${a.utilization}`}
          a={a}
          s={s}
          produktion={produktion}
        />
      ))}
      {s.kind === "Extraction" && <Erschliessung s={s} datum={produktion.date} />}
      <AnlageBauen s={s} produktion={produktion} />
    </div>
  );
}

function mengen(produktion: Produktion, liste: [string, number][]): string {
  return liste
    .map(([p, q]) => `${formatMenge(q)} ${einheit(produktion, p)} ${produktName(p)}`)
    .join(" + ");
}

function AnlageKarte({
  a,
  s,
  produktion,
}: {
  a: AnlageDetail;
  s: StandortDetail;
  produktion: Produktion;
}) {
  const name = t(`anlage.${a.facility}`);
  const { los, antwort } = useAktion(`anlage/${s.index}/${a.index}`);
  const [rezept, setRezept] = useState(a.recipe ?? "");
  const [auslastung, setAuslastung] = useState(zahlFeld(a.utilization * 100, 0));
  const [fehler, setFehler] = useState<string | null>(null);
  const rezepte = produktion.recipes.filter((r) => r.facility === a.facility);
  const labor = s.kind === "ResearchCenter";
  const id = useId();
  const imBau = a.ready > produktion.date;
  const ursache = ursacheText(a);

  const absenden = (e: FormEvent) => {
    e.preventDefault();
    const prozent = zahlLesen(auslastung);
    if (prozent === null || prozent < 0 || prozent > 100) {
      setFehler(t("werk.auslastung_bereich"));
      return;
    }
    setFehler(null);
    void los(
      [
        {
          SetProduction: {
            site: s.index,
            slot: a.index,
            recipe: labor ? a.recipe : rezept || null,
            utilization: prozent / 100,
          },
        },
      ],
      t("werk.anlage_geaendert"),
    );
  };

  return (
    <article className="karte" aria-label={name} data-tour="anlage">
      <h3>
        {a.count > 1 ? `${a.count} × ` : ""}
        {name}
        {imBau && <small>{t("uebersicht.im_bau", { datum: formatDatum(a.ready) })}</small>}
        {a.operation !== "laeuft" && a.operation_date && (
          <small>
            {t(a.operation === "stillgelegt" ? "werk.stillgelegt_seit" : "werk.wiederanlauf_bis", {
              datum: formatDatum(a.operation_date),
            })}
          </small>
        )}
      </h3>
      {a.product && (
        <p className="rezeptzeile">
          {a.inputs_per_day.length > 0 && `${mengen(produktion, a.inputs_per_day)} → `}
          <strong>
            {formatMenge(a.planned_per_day)} {einheit(produktion, a.product)}{" "}
            {produktName(a.product)}
          </strong>{" "}
          {t("werk.je_tag_geplant")}
        </p>
      )}
      <dl className="werte">
        {a.product && !imBau && (
          <>
            <dt>{t("werk.gestern")}</dt>
            <dd>
              {t("werk.ist_von_plan", {
                ist: formatMenge(a.made_per_day),
                plan: formatMenge(a.planned_per_day),
                einheit: einheit(produktion, a.product),
              })}
            </dd>
          </>
        )}
        <dt>{a.cause ? t("produktion.ursache") : t("werk.betrieb")}</dt>
        <dd className={a.cause ? "warntext" : "erfolgstext"}>
          {ursache || (a.recipe || labor ? t("produktion.laeuft") : "")}
          {a.cause && <small className="feld-hilfe">{t(`${a.cause.key}.hilfe`)}</small>}
        </dd>
        <dt>{t("werk.zustand")}</dt>
        <dd>{formatProzent(a.condition)}</dd>
      </dl>
      <form
        className="formular-zeile"
        aria-label={t("werk.anlage_steuern", { anlage: name })}
        onSubmit={absenden}
      >
        {!labor && (
          <div className="feld">
            <label htmlFor={`${id}-rezept`}>{t("werk.verfahren")}</label>
            <select id={`${id}-rezept`} value={rezept} onChange={(e) => setRezept(e.target.value)}>
              <option value="">{t("produktion.kein_rezept")}</option>
              {rezepte.map((r) => (
                <option key={r.key} value={r.key}>
                  {t(`rezept.${r.key}`)}
                </option>
              ))}
            </select>
          </div>
        )}
        <ZahlFeld
          name={t("werk.auslastung")}
          einheit="%"
          wert={auslastung}
          onWert={setAuslastung}
          gruppieren={false}
        />
        <button type="submit">{t("werk.uebernehmen")}</button>
        {fehler && <p className="fehlertext">{fehler}</p>}
        <Rueckmeldung meldung={antwort} />
      </form>
      {!imBau && <AnlageAbbau a={a} s={s} name={name} />}
    </article>
  );
}

/**
 * Shutting down, starting up again and selling (M22): with what each costs or brings,
 * so that the player sees whether a facility is worth keeping.
 */
function AnlageAbbau({ a, s, name }: { a: AnlageDetail; s: StandortDetail; name: string }) {
  const { los, antwort } = useAktion(`abbau/${s.index}/${a.index}`);
  const [einheiten, setEinheiten] = useState(zahlFeld(a.count, 0));
  const [frage, setFrage] = useState(false);
  const id = useId();
  const k = Math.floor(zahlLesen(einheiten) ?? 0);
  const gueltig = k >= 1 && k <= a.count;
  const anteil = gueltig ? k / a.count : 0;
  const erloes = a.sale_value_usd * anteil;
  const buchwert = a.book_value_usd * anteil;
  const befehl = (art: "MothballFacility" | "SellFacility") =>
    art === "MothballFacility"
      ? { MothballFacility: { site: s.index, slot: a.index, count: k } }
      : { SellFacility: { site: s.index, slot: a.index, count: k } };
  return (
    <details className="anlage-abbau">
      <summary>{t("werk.stilllegen_verkaufen")}</summary>
      <dl className="werte">
        <dt>{t("werk.wartung_monat")}</dt>
        <dd>
          {formatGeld(a.maintenance_month_usd)}
          <small className="feld-hilfe">
            {t("werk.wartung_stillgelegt", {
              betrag: formatGeld(a.maintenance_mothballed_month_usd),
            })}
          </small>
        </dd>
        <dt>{t("werk.restbuchwert")}</dt>
        <dd>{formatGeld(a.book_value_usd)}</dd>
        <dt>{t("werk.verkaufserloes")}</dt>
        <dd>{formatGeld(a.sale_value_usd)}</dd>
      </dl>
      {a.count > 1 && (
        <ZahlFeld
          name={t("werk.einheiten")}
          einheit={t("werk.von_einheiten", { anzahl: a.count })}
          ganzzahlig
          wert={einheiten}
          onWert={(w) => {
            setEinheiten(w);
            setFrage(false);
          }}
        />
      )}
      {!gueltig && <p className="fehlertext">{t("werk.einheiten_bereich", { anzahl: a.count })}</p>}
      {frage ? (
        <div className="bestaetigung" role="group" aria-labelledby={`${id}-frage`}>
          <p id={`${id}-frage`}>
            {t(buchwert > erloes ? "werk.verkaufen_frage_verlust" : "werk.verkaufen_frage", {
              anzahl: k,
              anlage: name,
              erloes: formatGeld(erloes),
              verlust: formatGeld(Math.abs(buchwert - erloes)),
            })}
          </p>
          <div className="knopfreihe links">
            <button
              type="button"
              className="gefahr"
              onClick={() => {
                setFrage(false);
                void los(
                  [befehl("SellFacility")],
                  t("werk.verkauft_meldung", {
                    anzahl: k,
                    anlage: name,
                    erloes: formatGeld(erloes),
                  }),
                );
              }}
            >
              {t("werk.verkaufen_ja")}
            </button>
            <button type="button" className="schlicht" onClick={() => setFrage(false)}>
              {t("werk.abbrechen")}
            </button>
          </div>
        </div>
      ) : (
        <div className="knopfreihe links">
          {a.operation === "laeuft" && (
            <button
              type="button"
              disabled={!gueltig}
              onClick={() =>
                void los(
                  [befehl("MothballFacility")],
                  t("werk.stillgelegt_meldung", { anzahl: k, anlage: name }),
                )
              }
            >
              {t("werk.stilllegen")}
            </button>
          )}
          {a.operation === "stillgelegt" && (
            <button
              type="button"
              onClick={() =>
                void los(
                  [{ RestartFacility: { site: s.index, slot: a.index } }],
                  t("werk.angefahren_meldung", { anlage: name, tage: a.restart_days }),
                )
              }
            >
              {t("werk.wieder_anfahren", { kosten: formatGeld(a.restart_cost_usd) })}
            </button>
          )}
          <button
            type="button"
            className="schlicht"
            disabled={!gueltig}
            onClick={() => setFrage(true)}
          >
            {t("werk.verkaufen")}
          </button>
        </div>
      )}
      <p className="erklaerung">
        {t("werk.abbau_erklaerung", {
          tage: a.restart_days,
          kosten: formatGeld(a.restart_cost_usd),
        })}
      </p>
      <Rueckmeldung meldung={antwort} />
    </details>
  );
}

function AnlageBauen({ s, produktion }: { s: StandortDetail; produktion: Produktion }) {
  const baubar = produktion.facilities
    .filter((f) => f.site_type === s.kind)
    .sort((a, b) => a.investment_usd - b.investment_usd);
  const [anlage, setAnlage] = useState(baubar[0]?.key ?? "");
  const [anzahl, setAnzahl] = useState("1");
  const { los, antwort } = useAktion(`bauen/${s.index}`);
  const id = useId();
  if (baubar.length === 0) return null;
  const f = baubar.find((x) => x.key === anlage) ?? baubar[0]!;
  const rezepte = produktion.recipes.filter((r) => f.recipes.includes(r.key));
  const zahl = Math.floor(zahlLesen(anzahl) ?? 0);
  return (
    <form
      className="karte bauen"
      aria-label={t("produktion.bauen")}
      onSubmit={(e) => {
        e.preventDefault();
        if (zahl < 1) return;
        void los(
          [{ BuildFacility: { site: s.index, facility: f.key, count: zahl } }],
          t("werk.bau_begonnen", { anlage: t(`anlage.${f.key}`), tage: f.build_days }),
        );
      }}
    >
      <h3>{t("produktion.bauen")}</h3>
      <div className="formular-zeile">
        <div className="feld">
          <label htmlFor={`${id}-anlage`}>{t("werk.anlage")}</label>
          <select id={`${id}-anlage`} value={f.key} onChange={(e) => setAnlage(e.target.value)}>
            {baubar.map((x) => (
              <option key={x.key} value={x.key}>
                {t(`anlage.${x.key}`)} – {formatGeld(x.investment_usd)}
              </option>
            ))}
          </select>
        </div>
        <ZahlFeld name={t("produktion.anzahl")} ganzzahlig wert={anzahl} onWert={setAnzahl} />
      </div>
      <dl className="werte">
        <dt>{t("werk.investition")}</dt>
        <dd>
          {formatGeld(f.investment_usd * Math.max(zahl, 1))}
          {zahl > 1 && (
            <small className="gedaempft">
              {" "}
              ({t("werk.je_anlage", { betrag: formatGeld(f.investment_usd) })})
            </small>
          )}
        </dd>
        <dt>{t("werk.bauzeit")}</dt>
        <dd>{t("produktion.tage", { tage: f.build_days })}</dd>
        <dt>{t("werk.leistung")}</dt>
        <dd>
          {rezepte.length === 0 && <span className="gedaempft">{t("werk.kein_verfahren")}</span>}
          <ul className="schlicht-liste">
            {rezepte.map((r) => (
              <li key={r.key}>
                {r.inputs_per_day.length > 0 && `${mengen(produktion, r.inputs_per_day)} → `}
                {formatMenge(r.output_per_day)} {einheit(produktion, r.product)}{" "}
                {produktName(r.product)} {t("werk.je_tag")}
              </li>
            ))}
          </ul>
        </dd>
      </dl>
      <div className="knopfreihe links">
        <button type="submit" className="haupt" disabled={zahl < 1}>
          {t("produktion.bauen_knopf")}
        </button>
      </div>
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

function Erschliessung({ s, datum }: { s: StandortDetail; datum: string }) {
  const [lager, setLager] = useState(s.free_deposits[0]?.key ?? "");
  const { los, antwort } = useAktion(`erschliessen/${s.index}`);
  const id = useId();
  if (s.deposit) {
    return s.deposit_ready && s.deposit_ready > datum ? (
      <p className="erklaerung">
        {t("produktion.erschliessung_bis", { datum: formatDatum(s.deposit_ready) })}
      </p>
    ) : null;
  }
  if (s.free_deposits.length === 0)
    return (
      <p className="gedaempft">{t("produktion.keine_lagerstaette", { jahr: datum.slice(0, 4) })}</p>
    );
  return (
    <form
      className="karte"
      aria-label={t("produktion.erschliessen")}
      onSubmit={(e) => {
        e.preventDefault();
        void los([{ DevelopDeposit: { site: s.index, deposit: lager } }]);
      }}
    >
      <h3>{t("produktion.erschliessen")}</h3>
      <div className="feld">
        <label htmlFor={`${id}-lager`}>{t("werk.lagerstaette")}</label>
        <select id={`${id}-lager`} value={lager} onChange={(e) => setLager(e.target.value)}>
          {s.free_deposits.map((d) => (
            <option key={d.key} value={d.key}>
              {t(`lagerstaette.${d.key}`)} ({produktName(d.resource)}) – {formatGeld(d.cost_usd)},{" "}
              {t("produktion.bauzeit", { tage: d.days })},{" "}
              {t("produktion.foerderung_jahr", { menge: formatZahl(d.output_per_year) })}
            </option>
          ))}
        </select>
      </div>
      <div className="knopfreihe links">
        <button type="submit">{t("produktion.erschliessen_knopf")}</button>
      </div>
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

// --- Purchasing ---

function Einkauf({ s, produktion }: { s: StandortDetail; produktion: Produktion }) {
  const zeilen: Versorgung[] = [
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
        market_price_usd: s.prices[o.product]?.market_usd ?? 0,
      })),
  ];
  return (
    <div className="karten" data-tour="werk-einkauf">
      <p className="erklaerung">{t("werk.einkauf_hinweis")}</p>
      {zeilen.length === 0 && (
        <p className="gedaempft">
          {t(s.slots.length === 0 ? "werk.kein_einkauf" : "werk.kein_bedarf")}
        </p>
      )}
      {zeilen.map((v) => {
        const auftrag = s.orders.find((o) => o.product === v.product);
        return (
          <EinkaufKarte
            key={`${v.product}/${auftrag?.target}/${auftrag?.max_price_usd}/${geldSchluessel()}`}
            v={v}
            s={s}
            produktion={produktion}
          />
        );
      })}
      <WeiteresProdukt
        tour="einkauf-neu"
        titel={t("produktion.neuer_einkauf")}
        produkte={produktion.products.filter((p) => !zeilen.some((v) => v.product === p))}
      >
        {(p) => (
          <EinkaufKarte
            key={p}
            v={{
              product: p,
              need_per_day: 0,
              stock: s.stock.find((l) => l.product === p)?.quantity ?? 0,
              days: null,
              own: false,
              ordered: false,
              market_price_usd: s.prices[p]?.market_usd ?? 0,
            }}
            s={s}
            produktion={produktion}
          />
        )}
      </WeiteresProdukt>
    </div>
  );
}

function reichweiteZustand(tage: number | null): "fehlt" | "knapp" | "gut" {
  const d = tage ?? Infinity;
  return d < 1 ? "fehlt" : d < 7 ? "knapp" : "gut";
}

function EinkaufKarte({
  v,
  s,
  produktion,
}: {
  v: Versorgung;
  s: StandortDetail;
  produktion: Produktion;
}) {
  const auftrag = s.orders.find((o) => o.product === v.product);
  const name = produktName(v.product);
  const e = einheit(produktion, v.product);
  const { los, antwort } = useAktion(`einkauf/${s.index}/${v.product}`);
  const vorschlagPreis = v.market_price_usd * 1.1;
  // Without a need here (trade) the player chooses the stock: the field starts empty.
  const [ziel, setZiel] = useState(
    auftrag
      ? zahlFeld(auftrag.target, 1)
      : v.need_per_day > 0
        ? zahlFeld(Math.ceil(v.need_per_day * 20), 1)
        : "",
  );
  const [preis, setPreis] = useState(geldFeld(auftrag ? auftrag.max_price_usd : vorschlagPreis));
  const [fehler, setFehler] = useState<string | null>(null);
  const zustand = reichweiteZustand(v.days);

  const absenden = (ev: FormEvent) => {
    ev.preventDefault();
    const z = zahlLesen(ziel);
    const p = zahlLesen(preis);
    if (z === null || z <= 0 || p === null || p <= 0) {
      setFehler(t("werk.einkauf_werte"));
      return;
    }
    setFehler(null);
    void los(
      [
        {
          SetPurchase: {
            site: s.index,
            product: v.product,
            target: z,
            max_price: geld(ausAnzeige(p)),
            min_quality: 0,
          },
        },
      ],
      t("werk.einkauf_gesetzt", { produkt: name }),
    );
  };

  return (
    <article
      className="karte"
      aria-label={t("produktion.einkauf_von", { produkt: name })}
      data-tour={auftrag ? "einkauf-auftrag" : "einkauf-karte"}
    >
      <h3>
        {name}
        {v.own && <span className="marke">{t("produktion.eigen")}</span>}
      </h3>
      <dl className="werte">
        <dt>{t("produktion.bedarf")}</dt>
        <dd>{v.need_per_day > 0 ? `${formatMenge(v.need_per_day)} ${e}` : "–"}</dd>
        <dt>{t("uebersicht.lager")}</dt>
        <dd>
          {formatMenge(v.stock)} {e}
        </dd>
        <dt>{t("produktion.reichweite")}</dt>
        <dd className={`zustand-${zustand}`}>
          {v.days === null ? "–" : t("produktion.tage", { tage: formatZahl(v.days, 0) })}
        </dd>
        <dt>{t("werk.marktpreis")}</dt>
        <dd>{v.market_price_usd > 0 ? formatPreis(v.market_price_usd, e) : "–"}</dd>
        {auftrag && (
          <>
            <dt>{t("produktion.gekauft_vormonat")}</dt>
            <dd>
              {formatZahl(auftrag.bought_last_month, 1)} {e}
            </dd>
          </>
        )}
      </dl>
      {v.own && !auftrag && <p className="erklaerung">{t("werk.eigenes_vorprodukt")}</p>}
      <form
        className="formular-zeile"
        aria-label={t("werk.einkauf_festlegen", { produkt: name })}
        onSubmit={absenden}
      >
        <ZahlFeld
          name={t("werk.ziellager")}
          einheit={e}
          wert={ziel}
          onWert={setZiel}
          hilfe={t("werk.ziellager_hilfe")}
        />
        <ZahlFeld
          name={t("werk.hoechstpreis")}
          einheit={`${geldEinheit()}/${e}`}
          wert={preis}
          onWert={setPreis}
          hilfe={t("werk.hoechstpreis_hilfe")}
        />
        <div className="knopfreihe links">
          <button type="submit">
            {auftrag ? t("werk.einkauf_aendern") : t("werk.einkauf_starten")}
          </button>
          {auftrag && (
            <button
              type="button"
              className="schlicht"
              onClick={() =>
                void los(
                  [
                    {
                      SetPurchase: {
                        site: s.index,
                        product: v.product,
                        target: 0,
                        max_price: 0,
                        min_quality: 0,
                      },
                    },
                  ],
                  t("werk.einkauf_gestoppt", { produkt: name }),
                )
              }
            >
              {t("produktion.stoppen")}
            </button>
          )}
        </div>
        {fehler && <p className="fehlertext">{fehler}</p>}
        <Rueckmeldung meldung={antwort} />
      </form>
    </article>
  );
}

// --- Sales ---

function Verkauf({ s, produktion }: { s: StandortDetail; produktion: Produktion }) {
  const angebotbar = new Set([
    ...s.slots.map((a) => a.product).filter((p): p is string => !!p),
    ...s.stock.map((l) => l.product),
  ]);
  const ohneAngebot = [...angebotbar].filter((p) => !s.offers.some((o) => o.product === p));
  return (
    <div className="karten" data-tour="werk-verkauf">
      <p className="erklaerung">{t("werk.verkauf_hinweis")}</p>
      {s.offers.map((o) => (
        <AngebotKarte
          key={`${o.product}/${o.mode}/${o.price_usd}/${o.floor_usd}/${o.keep}/${geldSchluessel()}`}
          produkt={o.product}
          o={o}
          s={s}
          produktion={produktion}
        />
      ))}
      {ohneAngebot.map((p) => (
        <AngebotKarte
          key={`${p}/${geldSchluessel()}`}
          produkt={p}
          o={null}
          s={s}
          produktion={produktion}
        />
      ))}
      <WeiteresProdukt
        tour="angebot-neu"
        titel={t("produktion.neues_angebot")}
        produkte={produktion.products.filter(
          (p) => !s.offers.some((o) => o.product === p) && !ohneAngebot.includes(p),
        )}
      >
        {(p) => (
          <AngebotKarte
            key={`${p}/${geldSchluessel()}`}
            produkt={p}
            o={null}
            s={s}
            produktion={produktion}
          />
        )}
      </WeiteresProdukt>
    </div>
  );
}

function AngebotKarte({
  produkt,
  o,
  s,
  produktion,
}: {
  produkt: string;
  o: Angebot | null;
  s: StandortDetail;
  produktion: Produktion;
}) {
  const name = produktName(produkt);
  const e = einheit(produktion, produkt);
  const { los, antwort } = useAktion(`verkauf/${s.index}/${produkt}`);
  const preise = s.prices[produkt];
  const markt = o?.market_price_usd ?? preise?.market_usd ?? 0;
  const richt = o?.reference_usd ?? preise?.reference_usd ?? 0;
  const stueck = o?.unit_cost_usd ?? s.unit_costs.find((u) => u.product === produkt)?.total_usd;
  const [art, setArt] = useState<"markt" | "fest">(o?.mode ?? "markt");
  const preisFeld = geldFeld(o ? o.price_usd : markt);
  const [preis, setPreis] = useState(preisFeld);
  const [mindest, setMindest] = useState(geldFeld(o ? o.floor_usd : 0));
  const [behalten, setBehalten] = useState(zahlFeld(o ? o.keep : 0, 1));
  const [fehler, setFehler] = useState<string | null>(null);
  const id = useId();
  const lager = o?.stock ?? s.stock.find((l) => l.product === produkt)?.quantity ?? 0;
  const eigenbedarf = o?.used_here ?? s.inputs.some((v) => v.product === produkt);

  const absenden = (ev: FormEvent) => {
    ev.preventDefault();
    const p = zahlLesen(preis);
    const m = art === "markt" ? zahlLesen(mindest || "0") : 0;
    const k = zahlLesen(behalten || "0");
    if (p === null || p <= 0 || m === null || m < 0 || k === null || k < 0) {
      setFehler(t("werk.verkauf_werte"));
      return;
    }
    setFehler(null);
    const befehle: Befehl[] = [
      {
        SetSale: {
          site: s.index,
          product: produkt,
          mode:
            art === "fest"
              ? { Fixed: geld(ausAnzeige(p)) }
              : {
                  Market: {
                    markup: o?.mode === "markt" ? o.markup : 0,
                    floor: geld(ausAnzeige(m)),
                  },
                },
          keep: k,
        },
      },
    ];
    // An automatic price goes on from the typed price (if the player changed it).
    if (art === "markt" && (!o || preis.trim() !== preisFeld))
      befehle.push({ SetPrice: { site: s.index, product: produkt, price: geld(ausAnzeige(p)) } });
    void los(befehle, t(o ? "werk.angebot_geaendert" : "werk.angebot_neu", { produkt: name }));
  };

  const schritt = (faktor: number) => {
    if (!o) return;
    // Rounded in the shown currency, as the field would show it.
    const neu = ausAnzeige(geldRunden(inAnzeige(o.price_usd) * faktor));
    void los(
      [{ SetPrice: { site: s.index, product: produkt, price: geld(neu) } }],
      t("werk.preis_gesetzt", { preis: formatPreis(neu, e) }),
    );
  };

  return (
    <article
      className="karte"
      aria-label={t("produktion.verkauf_von", { produkt: name })}
      data-tour={o ? "angebot-aktiv" : "angebot-karte"}
    >
      <h3>
        {name}
        {!o && <small>{t("werk.nicht_angeboten")}</small>}
      </h3>
      <dl className="werte">
        <dt>{t("uebersicht.lager")}</dt>
        <dd>
          {formatMenge(lager)} {e}
        </dd>
        {o && (
          <>
            <dt>{t("produktion.verkauft_vormonat")}</dt>
            <dd>
              {formatZahl(o.sold_last_month, 1)} {e}
            </dd>
            <dt>{t("produktion.verkauft_monat")}</dt>
            <dd>
              {formatZahl(o.sold_month, 1)} {e}
            </dd>
          </>
        )}
      </dl>
      <Preisvergleich
        einheit={e}
        eigen={o?.price_usd ?? null}
        markt={markt}
        richt={richt}
        stueck={stueck ?? null}
        teile={s.unit_costs.find((u) => u.product === produkt) ?? null}
        marge={o?.margin ?? null}
      />
      <form
        className="formular-spalten"
        aria-label={t("werk.preis_festlegen", { produkt: name })}
        onSubmit={absenden}
      >
        <fieldset className="auswahlgruppe">
          <legend>{t("werk.preisart")}</legend>
          <label>
            <input
              type="radio"
              name={`${id}-art`}
              checked={art === "markt"}
              onChange={() => setArt("markt")}
            />
            {t("werk.automatisch")}
          </label>
          <label>
            <input
              type="radio"
              name={`${id}-art`}
              checked={art === "fest"}
              onChange={() => setArt("fest")}
            />
            {t("werk.fest")}
          </label>
          <small className="feld-hilfe">
            {art === "markt" ? t("werk.automatisch_hilfe") : t("werk.fest_hilfe")}
          </small>
        </fieldset>
        <div className="formular-zeile" data-tour={o ? "preis" : undefined}>
          <ZahlFeld
            name={art === "markt" ? t("werk.preis_jetzt") : t("werk.preis")}
            einheit={`${geldEinheit()}/${e}`}
            wert={preis}
            onWert={setPreis}
          />
          {o && (
            <div className="schnellknoepfe" role="group" aria-label={t("werk.preis_schnell")}>
              <button
                type="button"
                aria-label={t("werk.preis_minus", { produkt: name })}
                onClick={() => schritt(0.95)}
              >
                −5 %
              </button>
              <button
                type="button"
                aria-label={t("werk.preis_plus", { produkt: name })}
                onClick={() => schritt(1.05)}
              >
                +5 %
              </button>
            </div>
          )}
          {art === "markt" && (
            <ZahlFeld
              name={t("werk.mindestpreis")}
              einheit={`${geldEinheit()}/${e}`}
              wert={mindest}
              onWert={setMindest}
              hilfe={t("werk.mindestpreis_hilfe")}
            />
          )}
          {eigenbedarf && (
            <ZahlFeld
              name={t("werk.behalten")}
              einheit={e}
              wert={behalten}
              onWert={setBehalten}
              hilfe={t("werk.behalten_hilfe")}
            />
          )}
        </div>
        <div className="knopfreihe links">
          <button type="submit" className={o ? undefined : "haupt"}>
            {o ? t("werk.uebernehmen") : t("produktion.anbieten")}
          </button>
          {o && (
            <button
              type="button"
              className="schlicht"
              onClick={() =>
                void los(
                  [{ SetSale: { site: s.index, product: produkt, mode: null, keep: 0 } }],
                  t("werk.verkauf_beendet", { produkt: name }),
                )
              }
            >
              {t("werk.verkauf_beenden")}
            </button>
          )}
        </div>
        {fehler && <p className="fehlertext">{fehler}</p>}
        <Rueckmeldung meldung={antwort} />
        {/* Tells the introduction that a price was set. */}
        {antwort && !antwort.fehler && <span data-tour="preis-gesetzt" hidden />}
      </form>
    </article>
  );
}

/**
 * Own price against market price, reference price and unit cost on one scale: is the
 * price above the cost, and where does it stand against the market?
 */
function Preisvergleich({
  einheit: e,
  eigen,
  markt,
  richt,
  stueck,
  teile,
  marge,
}: {
  einheit: string;
  eigen: number | null;
  markt: number;
  richt: number;
  stueck: number | null;
  /** The parts of the unit cost (M27), if the site makes the product. */
  teile: Stueckkosten | null;
  marge: number | null;
}) {
  const zeilen: { key: string; wert: number; klasse: string }[] = [
    ...(eigen !== null ? [{ key: "werk.eigener_preis", wert: eigen, klasse: "eigen" }] : []),
    { key: "werk.marktpreis", wert: markt, klasse: "markt" },
    { key: "werk.richtpreis", wert: richt, klasse: "richt" },
    ...(stueck !== null ? [{ key: "werk.stueckkosten", wert: stueck, klasse: "kosten" }] : []),
  ].filter((z) => z.wert > 0);
  const hoechst = Math.max(...zeilen.map((z) => z.wert), 1e-9);
  return (
    <div className="preisvergleich">
      <dl>
        {zeilen.map((z) => (
          <div key={z.key} className={`balkenzeile ${z.klasse}`}>
            <dt>
              <span>{t(z.key)}</span>
              {z.key === "werk.stueckkosten" && teile && (
                <Erklaerung wert={t("werk.stueckkosten")}>
                  <KostenTeile u={teile} einheit={e} />
                </Erklaerung>
              )}
            </dt>
            <dd>
              <span className="balken" style={{ width: `${(z.wert / hoechst) * 100}%` }} />
              <span className="balkenwert">{formatPreis(z.wert, e)}</span>
            </dd>
          </div>
        ))}
      </dl>
      {marge !== null && (
        <p className={marge < 0 ? "fehlertext" : "erfolgstext"}>
          {t(marge < 0 ? "werk.marge_negativ" : "werk.marge", { marge: formatProzent(marge) })}
        </p>
      )}
    </div>
  );
}

// --- Staff ---

function Personal({ s }: { s: StandortDetail }) {
  const { los, antwort } = useAktion(`personal/${s.index}`);
  const [aufschlag, setAufschlag] = useState(zahlFeld(s.wage_premium * 100, 1));
  const [fehler, setFehler] = useState<string | null>(null);
  const fehlt = s.staff.filter((l) => l.employed + 0.05 < l.needed);
  const max = s.wage_premium_max * 100;
  return (
    <div className="karten" data-tour="werk-personal">
      {s.staff.length === 0 ? (
        <p className="gedaempft">{t("werk.kein_personal")}</p>
      ) : (
        <div className="tabelle">
          <table className="mobil-karten" aria-label={t("werk.arbeitskraefte")}>
            <thead>
              <tr>
                <th>{t("werk.gruppe")}</th>
                <th className="zahl">{t("werk.benoetigt")}</th>
                <th className="zahl">{t("werk.beschaeftigt")}</th>
                <th className="zahl">{t("werk.frei_im_land")}</th>
                <th className="zahl">{t("werk.lohn_stunde")}</th>
              </tr>
            </thead>
            <tbody>
              {s.staff.map((l) => (
                <tr key={l.group}>
                  <td>{gruppenName(l.group)}</td>
                  <td className="zahl" data-spalte={t("werk.benoetigt")}>
                    {formatKoepfe(l.needed)}
                  </td>
                  <td
                    className={`zahl ${l.employed + 0.05 < l.needed ? "zustand-fehlt" : ""}`}
                    data-spalte={t("werk.beschaeftigt")}
                  >
                    {formatKoepfe(l.employed)}
                  </td>
                  <td className="zahl" data-spalte={t("werk.frei_im_land")}>
                    {formatKoepfe(l.free_in_country)}
                  </td>
                  <td className="zahl" data-spalte={t("werk.lohn_stunde")}>
                    <span>
                      {formatGeld(l.wage_usd)}
                      {s.wage_premium > 0 && (
                        <small className="gedaempft"> ({formatGeld(l.country_wage_usd)})</small>
                      )}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      {fehlt.map((l) => (
        <p key={l.group} className="warntext">
          {t(l.free_in_country > 0.05 ? "werk.fehlt_frei" : "werk.fehlt_keine_frei", {
            anzahl: formatKoepfe(l.needed - l.employed),
            gruppe: gruppenName(l.group),
            aufschlag: formatProzent(s.rival_premium_max),
          })}
        </p>
      ))}
      <form
        className="karte"
        aria-label={t("werk.lohnaufschlag")}
        onSubmit={(e) => {
          e.preventDefault();
          const p = zahlLesen(aufschlag);
          if (p === null || p < 0 || p > max + 1e-9) {
            setFehler(t("werk.aufschlag_bereich", { max: formatZahl(max) }));
            return;
          }
          setFehler(null);
          void los(
            [{ SetWagePremium: { site: s.index, premium: p / 100 } }],
            t("werk.aufschlag_gesetzt", { prozent: formatZahl(p, 1) }),
          );
        }}
      >
        <h3>{t("werk.lohnaufschlag")}</h3>
        <p>{t("werk.lohnaufschlag_erklaerung")}</p>
        <dl className="werte">
          <dt>{t("werk.aufschlag_jetzt")}</dt>
          <dd>{formatProzent(s.wage_premium)}</dd>
          <dt>{t("werk.aufschlag_konkurrenz")}</dt>
          <dd>{formatProzent(s.rival_premium_max)}</dd>
          <dt>{t("werk.lohnkosten_tag")}</dt>
          <dd>{formatGeld(s.wage_cost_per_day_usd)}</dd>
        </dl>
        <div className="formular-zeile">
          <ZahlFeld
            name={t("werk.lohnaufschlag")}
            einheit="%"
            wert={aufschlag}
            onWert={setAufschlag}
            hilfe={t("werk.aufschlag_hilfe", { max: formatZahl(max) })}
            gruppieren={false}
          />
          <button type="submit">{t("werk.uebernehmen")}</button>
        </div>
        {fehler && <p className="fehlertext">{fehler}</p>}
        <Rueckmeldung meldung={antwort} />
      </form>
    </div>
  );
}

// --- Costs ---

/** The parts of a unit cost (M27): what each unit of the product costs and why. */
function KostenTeile({ u, einheit }: { u: Stueckkosten; einheit: string }) {
  const zeilen = KOSTENARTEN.filter((k) => Math.abs(u[k.key] as number) >= 0.005);
  return (
    <>
      <dl className="rechnung">
        {zeilen.map((k) => (
          <Fragment key={k.key}>
            <dt>{t(k.text)}</dt>
            <dd>{formatPreis(u[k.key] as number, einheit)}</dd>
          </Fragment>
        ))}
        <dt className="summe">{t("werk.stueckkosten")}</dt>
        <dd className="summe">= {formatPreis(u.total_usd, einheit)}</dd>
      </dl>
      <p className="gedaempft">
        {t("erklaerung.stueckkosten", {
          variabel: formatPreis(u.variable_usd, einheit),
          menge: `${formatZahl(u.output_per_day, u.output_per_day < 10 ? 1 : 0)} ${einheit}`,
        })}
      </p>
    </>
  );
}

const KOSTENARTEN: { key: keyof Stueckkosten; text: string }[] = [
  { key: "material_usd", text: "kostenart.material" },
  { key: "labor_usd", text: "kostenart.personal" },
  { key: "energy_usd", text: "kostenart.energie" },
  { key: "overhead_usd", text: "kostenart.gemeinkosten" },
  { key: "rent_usd", text: "kostenart.pacht" },
  { key: "facility_usd", text: "werk.anlage_kosten" },
];

function Kosten({ s, produktion }: { s: StandortDetail; produktion: Produktion }) {
  const m = s.last_month;
  return (
    <div className="karten">
      <p className="erklaerung">{t("werk.kosten_hinweis")}</p>
      {s.unit_costs.length === 0 && <p className="gedaempft">{t("werk.keine_erzeugung")}</p>}
      {s.unit_costs.map((u) => (
        <StueckkostenKarte
          key={u.product}
          u={u}
          angebot={s.offers.find((o) => o.product === u.product) ?? null}
          produktion={produktion}
        />
      ))}
      <section className="karte" aria-label={t("werk.ergebnis_vormonat")}>
        <h3>
          {t("werk.ergebnis_vormonat")}
          {m && <small>{formatMonat(m.month)}</small>}
        </h3>
        {!m ? (
          <p className="gedaempft">{t("uebersicht.noch_kein_monat")}</p>
        ) : (
          <>
            <div className="tabelle">
              <table className="ergebnis" aria-label={t("werk.ergebnis_vormonat")}>
                <tbody>
                  <tr>
                    <th scope="row">{t("kostenart.umsatz")}</th>
                    <td className="zahl">{formatGeld(m.revenue_usd)}</td>
                  </tr>
                  {m.lines.map((l) => (
                    <tr key={l.key}>
                      <th scope="row">{t(l.key)}</th>
                      <td className={`zahl ${l.usd < 0 ? "negativ" : ""}`}>{formatGeld(l.usd)}</td>
                    </tr>
                  ))}
                  <tr className="summe">
                    <th scope="row">{t("werk.ergebnis")}</th>
                    <td className={`zahl ${m.result_usd < 0 ? "negativ" : ""}`}>
                      {formatGeld(m.result_usd)}
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
            {m.products.length > 0 && (
              <div className="tabelle">
                <table aria-label={t("werk.je_produkt")}>
                  <thead>
                    <tr>
                      <th>{t("uebersicht.produkt")}</th>
                      <th className="zahl">{t("kostenart.umsatz")}</th>
                      <th className="zahl">{t("werk.rohertrag")}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {m.products.map((p) => (
                      <tr key={p.product}>
                        <td>{produktName(p.product)}</td>
                        <td className="zahl">{formatGeld(p.revenue_usd)}</td>
                        <td className={`zahl ${p.margin_usd < 0 ? "negativ" : ""}`}>
                          {formatGeld(p.margin_usd)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                <p className="feld-hilfe">{t("werk.rohertrag_hilfe")}</p>
              </div>
            )}
          </>
        )}
      </section>
    </div>
  );
}

function formatMonat(iso: string): string {
  const [jahr, monat] = iso.split("-");
  return `${t(`monat.${Number(monat)}`)} ${jahr}`;
}

function StueckkostenKarte({
  u,
  angebot,
  produktion,
}: {
  u: Stueckkosten;
  angebot: Angebot | null;
  produktion: Produktion;
}) {
  const e = einheit(produktion, u.product);
  const teile = KOSTENARTEN.map((k) => ({ ...k, wert: u[k.key] as number })).filter(
    (k) => k.wert > 0,
  );
  const skala = Math.max(u.total_usd, angebot?.price_usd ?? 0, 1e-9);
  return (
    <article
      className="karte"
      aria-label={t("werk.stueckkosten_von", { produkt: produktName(u.product) })}
    >
      <h3>
        {produktName(u.product)}
        <small>
          {t("werk.erzeugung_tag", {
            menge: formatMenge(u.output_per_day),
            einheit: e,
          })}
        </small>
      </h3>
      <div className="stapel" aria-hidden="true">
        {teile.map((k, i) => (
          <span
            key={k.key}
            className={`stapel-teil teil-${i}`}
            style={{ width: `${(k.wert / skala) * 100}%` }}
            title={`${t(k.text)}: ${formatPreis(k.wert, e)}`}
          />
        ))}
      </div>
      {angebot && (
        <div className="stapel preislinie" aria-hidden="true">
          <span
            className="stapel-preis"
            style={{ width: `${(angebot.price_usd / skala) * 100}%` }}
          />
        </div>
      )}
      <dl className="werte legende-werte">
        {teile.map((k, i) => (
          <LegendenZeile key={k.key} farbe={i} name={t(k.text)}>
            {formatPreis(k.wert, e)}
          </LegendenZeile>
        ))}
        <dt className="summe">{t("werk.stueckkosten")}</dt>
        <dd className="summe">{formatPreis(u.total_usd, e)}</dd>
        <dt>{t("werk.variable_kosten")}</dt>
        <dd>{formatPreis(u.variable_usd, e)}</dd>
        {angebot && (
          <>
            <dt>{t("werk.eigener_preis")}</dt>
            <dd>{formatPreis(angebot.price_usd, e)}</dd>
          </>
        )}
      </dl>
    </article>
  );
}

function LegendenZeile({
  farbe,
  name,
  children,
}: {
  farbe: number;
  name: string;
  children: ReactNode;
}) {
  return (
    <>
      <dt>
        <span className={`farbpunkt teil-${farbe}`} aria-hidden="true" />
        {name}
      </dt>
      <dd>{children}</dd>
    </>
  );
}

/** Last part of a list: pick another product, then the card for it. */
function WeiteresProdukt({
  titel,
  produkte,
  children,
  tour,
}: {
  titel: string;
  produkte: string[];
  children: (produkt: string) => ReactNode;
  /** Mark for the introduction (`data-tour`). */
  tour?: string;
}) {
  const [produkt, setProdukt] = useState("");
  const id = useId();
  const sortiert = [...produkte].sort((a, b) => produktName(a).localeCompare(produktName(b), "de"));
  return (
    <div className="weiteres" data-tour={tour}>
      <div className="feld">
        <label htmlFor={id}>{titel}</label>
        <select id={id} value={produkt} onChange={(e) => setProdukt(e.target.value)}>
          <option value="">–</option>
          {sortiert.map((p) => (
            <option key={p} value={p}>
              {produktName(p)}
            </option>
          ))}
        </select>
      </div>
      {produkt && children(produkt)}
    </div>
  );
}
