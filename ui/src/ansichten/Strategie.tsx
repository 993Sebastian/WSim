// Strategies (MA4, docs/BEDIENUNG.md): what the managers follow on every level. "Which
// rule holds here, where does it come from, and who carries it out?"
import { useId, useState, type FormEvent, type ReactNode } from "react";
import {
  ausAnzeige,
  formatGeld,
  formatMenge,
  formatProzent,
  formatZahl,
  geldEinheit,
  geldFeld,
  inAnzeige,
  landName,
  zahlFeld,
  zahlLesen,
} from "../format";
import {
  geld,
  type Geltung,
  type Kern,
  type Preisstrategie,
  type Strategie,
  type Bezugsweg,
  type Vorgabe,
  type VorgabeEinheit,
  type VorgabeEintrag,
  type Vorgabefeld,
  type Verkaufsweg,
} from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { Erklaerung, Rueckmeldung, ZahlFeld, useAktion, useSicht } from "./gemeinsam";
import { einheitName, stellenName } from "./stellen";

type Feld = VorgabeEintrag["field"];

const FELDER: Record<Feld, Vorgabefeld> = {
  preis: "Price",
  lager: "Stock",
  personal: "Wages",
  eigenfertigung: "Supply",
  investition: "Investment",
  reserve: "Reserve",
  schulung: "Training",
  umwelt: "Environment",
};

const PREISE: Record<Exclude<Preisstrategie, { MinMargin: number }>, string> = {
  Market: "marktpreis",
  Premium: "premium",
  Fight: "kampfpreis",
};

/** The levels from the top, for the indentation of the units. */
const EBENEN: VorgabeEinheit["level"][] = ["firma", "kontinent", "land", "standort"];

const VERSORGUNGEN: Record<Bezugsweg, string> = {
  OwnFirst: "eigene",
  ByPrice: "preis",
  Buy: "zukauf",
};

function grundTitel(e: VorgabeEinheit): string {
  if (e.level === "firma") return t("ebene.firma");
  if (e.level === "kontinent") return t(`kontinent.${e.continent ?? ""}`);
  if (e.level === "land") return landName(e.country ?? "");
  return einheitName(e);
}

/**
 * "Firma", "Europa", "Deutschland", "Werk · Deutschland"; several sites of the same name
 * are numbered in their order ("Werk · Deutschland (2)").
 */
export function einheitTitel(e: VorgabeEinheit, daten: Strategie): string {
  const titel = grundTitel(e);
  if (e.level !== "standort") return titel;
  const gleich = daten.units.filter((u) => u.level === "standort" && grundTitel(u) === titel);
  return gleich.length > 1 ? `${titel} (${gleich.findIndex((u) => u.key === e.key) + 1})` : titel;
}

/** The scope of a unit as `SetStrategy` takes it. */
function geltung(e: VorgabeEinheit): Geltung {
  if (e.level === "kontinent") return { Continent: e.continent ?? "" };
  if (e.level === "land") return { Country: e.country ?? "" };
  if (e.level === "standort") return { Site: e.site ?? 0 };
  return "Company";
}

/** The value that holds, as text. */
export function wertText(e: VorgabeEintrag): string {
  const v = e.value;
  if (!v) return t("strategie.investition.ohne");
  if ("Price" in v) {
    const p = v.Price;
    return typeof p === "string"
      ? t(`strategie.preis.${PREISE[p]}`)
      : t("strategie.preis.mindestmarge_wert", { marge: formatProzent(p.MinMargin) });
  }
  if ("Stock" in v) {
    return t("strategie.lager.wert", {
      min: formatZahl(v.Stock.input_min_days, 1),
      max: formatZahl(v.Stock.input_max_days, 1),
      ziel: formatZahl(v.Stock.output_days, 1),
    });
  }
  if ("Wages" in v) {
    return t("strategie.personal.wert", {
      min: formatProzent(v.Wages.min),
      max: formatProzent(v.Wages.max),
    });
  }
  if ("Supply" in v) return t(`strategie.versorgung.${VERSORGUNGEN[v.Supply]}`);
  if ("Investment" in v) {
    return t("strategie.investition.wert", { betrag: formatGeld(e.budget_usd ?? 0) });
  }
  if ("Training" in v) return t("strategie.schulung.wert", { ziel: formatProzent(v.Training) });
  if ("Environment" in v) {
    return t(v.Environment ? "strategie.umwelt.uebererfuellen" : "strategie.umwelt.erfuellen");
  }
  if (v.Reserve <= 0) return t("strategie.reserve.keine");
  return t(v.Reserve === 1 ? "strategie.reserve.wert_eins" : "strategie.reserve.wert", {
    monate: formatZahl(v.Reserve, 1),
    betrag: formatGeld(e.reserve_usd ?? 0),
  });
}

/** Where a value comes from: "hier festgelegt", "von Europa", "Standard". */
function herkunft(e: VorgabeEintrag, daten: Strategie): string {
  if (e.own) return t("strategie.hier");
  if (!e.origin) return t("strategie.standard");
  const von = daten.units.find((u) => u.key === e.origin);
  return t("strategie.von", { wo: von ? einheitTitel(von, daten) : e.origin });
}

/** The position that carries a strategy out at a site, with its manager. */
function traeger(e: VorgabeEintrag): string {
  if (!e.carrier) return t("strategie.niemand");
  const p = e.carrier.position;
  const stelle = stellenName(p.role, p.kind_text);
  const wo = p.level === "standort" ? "" : ` · ${einheitName(p)}`;
  return t("strategie.umgesetzt_von", { stelle: `${stelle}${wo} (${e.carrier.manager})` });
}

/** All units with the value of every field and where it comes from. */
function Uebersicht({
  daten,
  auswahl,
  onWahl,
}: {
  daten: Strategie;
  auswahl: string;
  onWahl: (key: string) => void;
}) {
  const felder = daten.units[0]?.entries.map((e) => e.field) ?? [];
  return (
    <div className="tabelle">
      <table className="mobil-karten strategie-tabelle" aria-label={t("strategie.uebersicht")}>
        <thead>
          <tr>
            <th>{t("strategie.einheit")}</th>
            {felder.map((f) => (
              <th key={f}>{t(`strategie.feld.${f}`)}</th>
            ))}
            <th />
          </tr>
        </thead>
        <tbody>
          {daten.units.map((u) => (
            <tr key={u.key} className={u.key === auswahl ? "gewaehlt" : undefined}>
              <td className={`ebene-${u.level}`}>{einheitTitel(u, daten)}</td>
              {u.entries.map((e) => (
                <td key={e.field} data-spalte={t(`strategie.feld.${e.field}`)}>
                  <span className={e.own ? "eigen" : "geerbt"}>{wertText(e)}</span>
                  <small className="feld-hilfe">{herkunft(e, daten)}</small>
                </td>
              ))}
              <td>
                <button
                  type="button"
                  className="schlicht"
                  aria-label={`${t("strategie.bearbeiten")}: ${einheitTitel(u, daten)}`}
                  onClick={() => onWahl(u.key)}
                >
                  {t("strategie.bearbeiten")}
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/** The input of one field: its form fields and the value they make (null: invalid). */
interface Eingabe {
  felder: ReactNode;
  wert: () => Vorgabe | null;
  grenzen: string;
}

function usePreis(e: VorgabeEintrag, daten: Strategie, id: string): Eingabe {
  const start = e.value && "Price" in e.value ? e.value.Price : "Market";
  const [art, setArt] = useState(typeof start === "string" ? PREISE[start] : "mindestmarge");
  const [marge, setMarge] = useState(
    typeof start === "string" ? "" : zahlFeld(start.MinMargin * 100, 1),
  );
  const max = daten.limits.min_margin_max;
  const preis = daten.prices.find((p) => p.kind === art);
  return {
    felder: (
      <>
        <div className="feld">
          <label htmlFor={`${id}-art`}>{t("strategie.preis.art")}</label>
          <select id={`${id}-art`} value={art} onChange={(ev) => setArt(ev.target.value)}>
            {["marktpreis", "premium", "kampfpreis", "mindestmarge"].map((k) => (
              <option key={k} value={k}>
                {t(`strategie.preis.${k}`)}
              </option>
            ))}
          </select>
        </div>
        {art === "mindestmarge" && (
          <ZahlFeld
            name={t("strategie.preis.marge")}
            einheit="%"
            wert={marge}
            onWert={setMarge}
            gruppieren={false}
          />
        )}
        {preis && (
          <p className="feld-hilfe">
            {t("strategie.preis.erklaerung", {
              untergrenze: formatProzent(preis.floor),
              start:
                preis.markup >= 0
                  ? `+${formatProzent(preis.markup)}`
                  : `−${formatProzent(-preis.markup)}`,
            })}
          </p>
        )}
      </>
    ),
    wert: () => {
      if (art !== "mindestmarge") {
        const p = (Object.keys(PREISE) as (keyof typeof PREISE)[]).find((k) => PREISE[k] === art);
        return p ? { Price: p } : null;
      }
      const m = zahlLesen(marge);
      return m === null || m < 0 || m / 100 > max ? null : { Price: { MinMargin: m / 100 } };
    },
    grenzen: t("strategie.grenzen.preis", { max: formatProzent(max) }),
  };
}

function useLager(e: VorgabeEintrag, daten: Strategie): Eingabe {
  const start = e.value && "Stock" in e.value ? e.value.Stock : null;
  const feld = (tage: number | undefined) => (tage === undefined ? "" : zahlFeld(tage, 1));
  const [min, setMin] = useState(feld(start?.input_min_days));
  const [max, setMax] = useState(feld(start?.input_max_days));
  const [ziel, setZiel] = useState(feld(start?.output_days));
  const grenze = daten.limits.stock_days_max;
  const tage = t("strategie.lager.tage");
  return {
    felder: (
      <>
        <ZahlFeld
          name={t("strategie.lager.min")}
          einheit={tage}
          wert={min}
          onWert={setMin}
          gruppieren={false}
        />
        <ZahlFeld
          name={t("strategie.lager.max")}
          einheit={tage}
          wert={max}
          onWert={setMax}
          gruppieren={false}
        />
        <ZahlFeld
          name={t("strategie.lager.ziel")}
          einheit={tage}
          wert={ziel}
          onWert={setZiel}
          gruppieren={false}
        />
      </>
    ),
    wert: () => {
      const [a, b, z] = [zahlLesen(min), zahlLesen(max), zahlLesen(ziel)];
      if (a === null || b === null || z === null) return null;
      if (a <= 0 || a > b || b > grenze || z < 0 || z > grenze) return null;
      return { Stock: { input_min_days: a, input_max_days: b, output_days: z } };
    },
    grenzen: t("strategie.grenzen.lager", { max: formatZahl(grenze) }),
  };
}

function usePersonal(e: VorgabeEintrag, daten: Strategie): Eingabe {
  const start = e.value && "Wages" in e.value ? e.value.Wages : null;
  const feld = (anteil: number | undefined) =>
    anteil === undefined ? "" : zahlFeld(anteil * 100, 1);
  const [min, setMin] = useState(feld(start?.min));
  const [max, setMax] = useState(feld(start?.max));
  const grenze = daten.limits.wage_premium_max;
  return {
    felder: (
      <>
        <ZahlFeld
          name={t("strategie.personal.min")}
          einheit="%"
          wert={min}
          onWert={setMin}
          gruppieren={false}
        />
        <ZahlFeld
          name={t("strategie.personal.max")}
          einheit="%"
          wert={max}
          onWert={setMax}
          gruppieren={false}
        />
      </>
    ),
    wert: () => {
      const [a, b] = [zahlLesen(min), zahlLesen(max)];
      if (a === null || b === null) return null;
      if (a < 0 || a > b || b / 100 > grenze) return null;
      return { Wages: { min: a / 100, max: b / 100 } };
    },
    grenzen: t("strategie.grenzen.personal", { max: formatProzent(grenze) }),
  };
}

function useVersorgung(e: VorgabeEintrag, id: string): Eingabe {
  const start = e.value && "Supply" in e.value ? e.value.Supply : "OwnFirst";
  const [art, setArt] = useState<Bezugsweg>(start);
  return {
    felder: (
      <div className="feld">
        <label htmlFor={`${id}-art`}>{t("strategie.versorgung.art")}</label>
        <select
          id={`${id}-art`}
          value={art}
          onChange={(ev) => setArt(ev.target.value as Bezugsweg)}
        >
          {(Object.keys(VERSORGUNGEN) as Bezugsweg[]).map((k) => (
            <option key={k} value={k}>
              {t(`strategie.versorgung.${VERSORGUNGEN[k]}`)}
            </option>
          ))}
        </select>
      </div>
    ),
    wert: () => ({ Supply: art }),
    grenzen: "",
  };
}

function useInvestition(e: VorgabeEintrag): Eingabe {
  const [betrag, setBetrag] = useState(
    e.budget_usd === null ? "" : zahlFeld(inAnzeige(e.budget_usd), 0),
  );
  return {
    felder: (
      <ZahlFeld
        name={t("strategie.investition.betrag")}
        einheit={geldEinheit()}
        wert={betrag}
        onWert={setBetrag}
        breit
      />
    ),
    wert: () => {
      const b = zahlLesen(betrag);
      return b === null || b < 0 ? null : { Investment: geld(ausAnzeige(b)) };
    },
    grenzen: t("strategie.grenzen.investition"),
  };
}

function useReserve(e: VorgabeEintrag, daten: Strategie): Eingabe {
  const start = e.value && "Reserve" in e.value ? e.value.Reserve : null;
  const [monate, setMonate] = useState(start === null ? "" : zahlFeld(start, 1));
  const grenze = daten.limits.reserve_months_max;
  return {
    felder: (
      <ZahlFeld
        name={t("strategie.reserve.monate")}
        einheit={t("strategie.reserve.einheit")}
        wert={monate}
        onWert={setMonate}
        gruppieren={false}
        hilfe={t("strategie.reserve.hilfe", { betrag: formatGeld(daten.monthly_cost_usd) })}
      />
    ),
    wert: () => {
      const m = zahlLesen(monate);
      return m === null || m < 0 || m > grenze ? null : { Reserve: m };
    },
    grenzen: t("strategie.grenzen.reserve", { max: formatZahl(grenze) }),
  };
}

function useSchulung(e: VorgabeEintrag): Eingabe {
  const start = e.value && "Training" in e.value ? e.value.Training : null;
  const [ziel, setZiel] = useState(start === null ? "" : zahlFeld(start * 100, 0));
  return {
    felder: (
      <ZahlFeld
        name={t("strategie.schulung.ziel")}
        einheit="%"
        wert={ziel}
        onWert={setZiel}
        gruppieren={false}
        hilfe={t("strategie.schulung.hilfe")}
      />
    ),
    wert: () => {
      const z = zahlLesen(ziel);
      return z === null || z < 0 || z > 100 ? null : { Training: z / 100 };
    },
    grenzen: t("strategie.grenzen.schulung"),
  };
}

function useUmwelt(e: VorgabeEintrag, id: string): Eingabe {
  const start = e.value && "Environment" in e.value ? e.value.Environment : false;
  const [mehr, setMehr] = useState(start);
  return {
    felder: (
      <div className="feld">
        <label htmlFor={`${id}-umwelt`}>{t("strategie.umwelt.art")}</label>
        <select
          id={`${id}-umwelt`}
          value={mehr ? "mehr" : "pflicht"}
          onChange={(ev) => setMehr(ev.target.value === "mehr")}
        >
          <option value="pflicht">{t("strategie.umwelt.erfuellen")}</option>
          <option value="mehr">{t("strategie.umwelt.uebererfuellen")}</option>
        </select>
        <p className="feld-hilfe">{t("strategie.umwelt.hilfe")}</p>
      </div>
    ),
    wert: () => ({ Environment: mehr }),
    grenzen: "",
  };
}

interface KarteDaten {
  e: VorgabeEintrag;
  einheit: VorgabeEinheit;
  daten: Strategie;
}

// One component per field, each with its own inputs.
function PreisKarte(p: KarteDaten) {
  const id = useId();
  return <FeldKarte {...p} eingabe={usePreis(p.e, p.daten, id)} />;
}
function LagerKarte(p: KarteDaten) {
  return <FeldKarte {...p} eingabe={useLager(p.e, p.daten)} />;
}
function PersonalKarte(p: KarteDaten) {
  return <FeldKarte {...p} eingabe={usePersonal(p.e, p.daten)} />;
}
function VersorgungKarte(p: KarteDaten) {
  const id = useId();
  return <FeldKarte {...p} eingabe={useVersorgung(p.e, id)} />;
}
function InvestitionKarte(p: KarteDaten) {
  return <FeldKarte {...p} eingabe={useInvestition(p.e)} />;
}
function ReserveKarte(p: KarteDaten) {
  return <FeldKarte {...p} eingabe={useReserve(p.e, p.daten)} />;
}
function SchulungKarte(p: KarteDaten) {
  return <FeldKarte {...p} eingabe={useSchulung(p.e)} />;
}
function UmweltKarte(p: KarteDaten) {
  const id = useId();
  return <FeldKarte {...p} eingabe={useUmwelt(p.e, id)} />;
}

const KARTEN: Record<Feld, (p: KarteDaten) => ReactNode> = {
  preis: PreisKarte,
  lager: LagerKarte,
  personal: PersonalKarte,
  eigenfertigung: VersorgungKarte,
  investition: InvestitionKarte,
  reserve: ReserveKarte,
  schulung: SchulungKarte,
  umwelt: UmweltKarte,
};

/** Name of the unit a budget is set for: "ganze Firma", "Europa" … */
function budgetOrt(daten: Strategie, key: string | null, sonst: VorgabeEinheit): string {
  const u = daten.units.find((x) => x.key === key) ?? sonst;
  return u.level === "firma" ? t("strategie.ganze_firma") : einheitTitel(u, daten);
}

/** One field of the chosen unit: what holds, where from, who follows it, and a form. */
function FeldKarte({ e, einheit, daten, eingabe }: KarteDaten & { eingabe: Eingabe }) {
  const { los, antwort } = useAktion(`strategie:${e.field}`);
  const [fehler, setFehler] = useState<string | null>(null);
  const feld = t(`strategie.feld.${e.field}`);
  const name = einheitTitel(einheit, daten);
  const senden = (value: Vorgabe | null, erfolg: string) =>
    void los([{ SetStrategy: { scope: geltung(einheit), field: FELDER[e.field], value } }], erfolg);
  const festlegen = (ev: FormEvent) => {
    ev.preventDefault();
    const wert = eingabe.wert();
    if (!wert) {
      setFehler(eingabe.grenzen);
      return;
    }
    setFehler(null);
    senden(wert, t("strategie.gesetzt", { feld, einheit: name }));
  };
  return (
    <form className="karte" aria-label={`${feld}: ${name}`} onSubmit={festlegen}>
      <h3>
        {feld}{" "}
        <Erklaerung wert={feld}>
          <p>{t(`strategie.feldhilfe.${e.field}`)}</p>
        </Erklaerung>
      </h3>
      <p>
        <strong>{t("strategie.gilt", { wert: wertText(e) })}</strong> · {herkunft(e, daten)}
      </p>
      {e.field === "investition" && e.left_usd !== null && (
        <p className="feld-hilfe">
          {t("strategie.investition.rest", {
            rest: formatGeld(e.left_usd),
            wo: budgetOrt(daten, e.binding, einheit),
          })}
        </p>
      )}
      {einheit.level === "standort" && <p className="feld-hilfe">{traeger(e)}</p>}
      <div className="formular-zeile">{eingabe.felder}</div>
      {fehler && <p className="fehlertext">{fehler}</p>}
      <div className="knopfreihe links">
        <button type="submit">{t("strategie.festlegen")}</button>
        {e.own && (
          <button
            type="button"
            className="schlicht"
            onClick={() => senden(null, t("strategie.entfernt", { feld, einheit: name }))}
          >
            {einheit.level === "firma" ? t("strategie.zuruecksetzen") : t("strategie.entfernen")}
          </button>
        )}
      </div>
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

/** "ganze Firma", "Deutschland", "Nägel", "Nägel in Deutschland". */
function wegGeltung(w: { product: string | null; country: string | null }): string {
  if (w.product && w.country) {
    return t("verkaufswege.produkt_im_land", {
      produkt: t(`produkt.${w.product}`),
      land: landName(w.country),
    });
  }
  if (w.product) return t(`produkt.${w.product}`);
  if (w.country) return landName(w.country);
  return t("strategie.ganze_firma");
}

function wegScope(product: string, country: string) {
  if (product && country) return { ProductInCountry: [product, country] as [string, string] };
  if (product) return { Product: product };
  if (country) return { Country: country };
  return "Company" as const;
}

const KAEUFER = { haendler: "Traders", firmen: "Companies" } as const;

/** What a rule allows: "erlaubt, ab 12 $/t, höchstens 500 t im Monat" or "gesperrt". */
function wegRegel(w: Verkaufsweg): string {
  if (!w.allowed) return t("verkaufswege.gesperrt");
  const einheit = w.unit ? t(w.unit) : "";
  const teile = [t("verkaufswege.erlaubt")];
  if (w.min_price_usd !== null) {
    teile.push(t("verkaufswege.ab", { preis: `${formatGeld(w.min_price_usd)}/${einheit}` }));
  }
  if (w.max_per_month !== null) {
    teile.push(
      t("verkaufswege.hoechstens", { menge: `${formatMenge(w.max_per_month)} ${einheit}` }),
    );
  }
  return teile.join(", ");
}

/**
 * Sales channels (Lastenheft §9.2, M8): whether traders and other companies may buy the
 * company's goods, for the company, a country, a product or a product in a country.
 */
function Verkaufswege({ daten }: { daten: Strategie }) {
  const { los, antwort } = useAktion("verkaufswege");
  const id = useId();
  const [kaeufer, setKaeufer] = useState<"haendler" | "firmen">("haendler");
  const [land, setLand] = useState("");
  const [produkt, setProdukt] = useState("");
  const [erlaubt, setErlaubt] = useState("ja");
  const [mindestpreis, setMindestpreis] = useState("");
  const [menge, setMenge] = useState("");
  const [fehler, setFehler] = useState<string | null>(null);
  const einheit = daten.sale_products.find((p) => p.product === produkt)?.unit;
  const festlegen = (ev: FormEvent) => {
    ev.preventDefault();
    const preis = mindestpreis.trim() === "" ? null : zahlLesen(mindestpreis);
    const hoechst = menge.trim() === "" ? null : zahlLesen(menge);
    if (
      (mindestpreis.trim() !== "" && (preis === null || preis < 0)) ||
      (menge.trim() !== "" && (hoechst === null || hoechst < 0))
    ) {
      setFehler(t("verkaufswege.grenzen"));
      return;
    }
    setFehler(null);
    const wo = wegGeltung({ product: produkt || null, country: land || null });
    void los(
      [
        {
          SetSalesPolicy: {
            buyer: KAEUFER[kaeufer],
            scope: wegScope(produkt, land),
            rule: {
              allowed: erlaubt === "ja",
              min_price: preis === null ? null : geld(ausAnzeige(preis)),
              max_per_month: hoechst,
            },
          },
        },
      ],
      t("verkaufswege.gesetzt", { kaeufer: t(`verkaufswege.kaeufer.${kaeufer}`), wo }),
    );
  };
  return (
    <section aria-label={t("verkaufswege.titel")}>
      <h2>
        {t("verkaufswege.titel")}{" "}
        <Erklaerung wert={t("verkaufswege.titel")}>
          <p>{t("verkaufswege.erklaerung")}</p>
        </Erklaerung>
      </h2>
      <p className="feld-hilfe">{t("verkaufswege.hilfe")}</p>
      {daten.sales.length === 0 ? (
        <p>{t("verkaufswege.keine")}</p>
      ) : (
        <div className="tabelle">
          <table className="mobil-karten" aria-label={t("verkaufswege.regeln")}>
            <thead>
              <tr>
                <th>{t("verkaufswege.kaeufer_titel")}</th>
                <th>{t("verkaufswege.gilt_fuer")}</th>
                <th>{t("verkaufswege.regel")}</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {daten.sales.map((w) => {
                const wer = t(`verkaufswege.kaeufer.${w.buyer}`);
                const wo = wegGeltung(w);
                return (
                  <tr key={`${w.buyer}/${w.product ?? ""}/${w.country ?? ""}`}>
                    <td>{wer}</td>
                    <td data-spalte={t("verkaufswege.gilt_fuer")}>{wo}</td>
                    <td data-spalte={t("verkaufswege.regel")}>{wegRegel(w)}</td>
                    <td>
                      <button
                        type="button"
                        className="schlicht"
                        aria-label={`${t("verkaufswege.entfernen")}: ${wer}, ${wo}`}
                        onClick={() =>
                          void los(
                            [
                              {
                                SetSalesPolicy: {
                                  buyer: KAEUFER[w.buyer],
                                  scope: wegScope(w.product ?? "", w.country ?? ""),
                                  rule: null,
                                },
                              },
                            ],
                            t("verkaufswege.entfernt", { kaeufer: wer, wo }),
                          )
                        }
                      >
                        {t("verkaufswege.entfernen")}
                      </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
      <form className="karte" aria-label={t("verkaufswege.neu")} onSubmit={festlegen}>
        <h3>{t("verkaufswege.neu")}</h3>
        <div className="formular-zeile">
          <div className="feld">
            <label htmlFor={`${id}-wer`}>{t("verkaufswege.kaeufer_titel")}</label>
            <select
              id={`${id}-wer`}
              value={kaeufer}
              onChange={(e) => setKaeufer(e.target.value as "haendler" | "firmen")}
            >
              <option value="haendler">{t("verkaufswege.kaeufer.haendler")}</option>
              <option value="firmen">{t("verkaufswege.kaeufer.firmen")}</option>
            </select>
          </div>
          <div className="feld">
            <label htmlFor={`${id}-land`}>{t("verkaufswege.land")}</label>
            <select id={`${id}-land`} value={land} onChange={(e) => setLand(e.target.value)}>
              <option value="">{t("verkaufswege.alle_laender")}</option>
              {daten.sale_countries.map((k) => (
                <option key={k} value={k}>
                  {landName(k)}
                </option>
              ))}
            </select>
          </div>
          <div className="feld">
            <label htmlFor={`${id}-produkt`}>{t("verkaufswege.produkt")}</label>
            <select
              id={`${id}-produkt`}
              value={produkt}
              onChange={(e) => setProdukt(e.target.value)}
            >
              <option value="">{t("verkaufswege.alle_produkte")}</option>
              {daten.sale_products.map((p) => (
                <option key={p.product} value={p.product}>
                  {t(`produkt.${p.product}`)}
                </option>
              ))}
            </select>
          </div>
          <div className="feld">
            <label htmlFor={`${id}-erlaubt`}>{t("verkaufswege.kaufen")}</label>
            <select
              id={`${id}-erlaubt`}
              value={erlaubt}
              onChange={(e) => setErlaubt(e.target.value)}
            >
              <option value="ja">{t("verkaufswege.erlaubt")}</option>
              <option value="nein">{t("verkaufswege.gesperrt")}</option>
            </select>
          </div>
        </div>
        {erlaubt === "ja" && (
          <div className="formular-zeile">
            <ZahlFeld
              name={t("verkaufswege.mindestpreis")}
              einheit={einheit ? `${geldEinheit()}/${t(einheit)}` : geldEinheit()}
              wert={mindestpreis}
              onWert={setMindestpreis}
              hilfe={t("verkaufswege.leer_heisst")}
            />
            <ZahlFeld
              name={t("verkaufswege.hoechstmenge")}
              einheit={einheit ? t(einheit) : undefined}
              wert={menge}
              onWert={setMenge}
            />
          </div>
        )}
        {fehler && <p className="fehlertext">{fehler}</p>}
        <div className="knopfreihe links">
          <button type="submit">{t("verkaufswege.festlegen")}</button>
        </div>
      </form>
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

/** "1.000.000" for a money field, "" for none. */
function geldText(usd: number | null): string {
  return usd === null ? "" : geldFeld(usd);
}

/**
 * The policy „Beteiligungen“ (ZA2): a yearly budget for takeovers and licences, the
 * readiness for risks, and up to which amount each department's head decides alone.
 */
function Beteiligungen({ daten }: { daten: Strategie }) {
  const p = daten.participations;
  const { los, antwort } = useAktion("beteiligungen");
  const [budget, setBudget] = useState(geldText(p.budget_usd));
  const [risiko, setRisiko] = useState(zahlFeld(p.risk * 100, 0));
  const [grenzen, setGrenzen] = useState<Record<string, string>>(() =>
    Object.fromEntries(p.limits.map((l) => [l.kind, geldText(l.limit_usd)])),
  );
  const [fehler, setFehler] = useState<string | null>(null);
  const festlegen = (ev: FormEvent) => {
    ev.preventDefault();
    const b = budget.trim() === "" ? null : zahlLesen(budget);
    const r = zahlLesen(risiko);
    const limits: Record<string, number> = {};
    let falsch = (budget.trim() !== "" && (b === null || b < 0)) || r === null || r < 0 || r > 100;
    for (const l of p.limits) {
      const text = grenzen[l.kind] ?? "";
      if (text.trim() === "") continue;
      const g = zahlLesen(text);
      if (g === null || g < 0) falsch = true;
      else limits[l.kind] = geld(ausAnzeige(g));
    }
    if (falsch || r === null) {
      setFehler(t("beteiligungen.werte"));
      return;
    }
    setFehler(null);
    void los(
      [
        {
          SetParticipations: {
            budget: b === null ? null : geld(ausAnzeige(b)),
            risk: r / 100,
            limits,
          },
        },
      ],
      t("beteiligungen.gesetzt"),
    );
  };
  return (
    <section aria-label={t("beteiligungen.titel")}>
      <h2>
        {t("beteiligungen.titel")}{" "}
        <Erklaerung wert={t("beteiligungen.titel")}>
          <p>{t("beteiligungen.erklaerung")}</p>
        </Erklaerung>
      </h2>
      <p className="feld-hilfe">
        {t("beteiligungen.stand", {
          gekauft: formatGeld(p.spent_usd),
          offen: formatGeld(p.open_bids_usd),
        })}{" "}
        {p.left_usd !== null && t("beteiligungen.rest", { rest: formatGeld(p.left_usd) })}
      </p>
      <form className="karte" aria-label={t("beteiligungen.titel")} onSubmit={festlegen}>
        <div className="formular-zeile">
          <ZahlFeld
            name={t("beteiligungen.budget")}
            einheit={geldEinheit()}
            wert={budget}
            onWert={setBudget}
            hilfe={t("beteiligungen.budget_hilfe")}
          />
          <ZahlFeld
            name={t("beteiligungen.risiko")}
            einheit="%"
            wert={risiko}
            onWert={setRisiko}
            hilfe={t("beteiligungen.risiko_hilfe")}
          />
        </div>
        {p.limits.length > 0 && (
          <>
            <h3>{t("beteiligungen.freigabe")}</h3>
            <p className="feld-hilfe">{t("beteiligungen.freigabe_hilfe")}</p>
            <div className="formular-zeile">
              {p.limits.map((l) => (
                <ZahlFeld
                  key={l.kind}
                  name={t(`abteilung.${l.key}`)}
                  einheit={geldEinheit()}
                  wert={grenzen[l.kind] ?? ""}
                  onWert={(text) => setGrenzen((g) => ({ ...g, [l.kind]: text }))}
                />
              ))}
            </div>
          </>
        )}
        {fehler && <p className="fehlertext">{fehler}</p>}
        <div className="knopfreihe links">
          <button type="submit">{t("beteiligungen.festlegen")}</button>
        </div>
      </form>
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

/**
 * The dividend policy (PE4): a share of the net profit or an amount per year, paid at the
 * end of January for the closed year; and a special dividend at once.
 */
function Dividende({ daten }: { daten: Strategie }) {
  const d = daten.dividend;
  const { los, antwort } = useAktion("dividende");
  const [art, setArt] = useState<"anteil" | "betrag">(d.kind);
  const [anteil, setAnteil] = useState(zahlFeld(d.share * 100, 0));
  const [betrag, setBetrag] = useState(geldText(d.amount_usd));
  const [sonder, setSonder] = useState("");
  const [fehler, setFehler] = useState<string | null>(null);
  const festlegen = (ev: FormEvent) => {
    ev.preventDefault();
    const n = zahlLesen(art === "anteil" ? anteil : betrag);
    if (n === null || n < 0 || (art === "anteil" && n > 100)) {
      setFehler(t("dividende.werte"));
      return;
    }
    setFehler(null);
    const policy = art === "anteil" ? { Share: n / 100 } : { Amount: geld(ausAnzeige(n)) };
    void los([{ SetDividendPolicy: { policy } }], t("dividende.gesetzt"));
  };
  const ausschuetten = () => {
    const n = zahlLesen(sonder);
    if (n === null || n <= 0) {
      setFehler(t("dividende.werte"));
      return;
    }
    setFehler(null);
    void los([{ SpecialDividend: { amount: geld(ausAnzeige(n)) } }], t("dividende.ausgeschuettet"));
  };
  return (
    <section aria-label={t("dividende.titel")}>
      <h2>
        {t("dividende.titel")}{" "}
        <Erklaerung wert={t("dividende.titel")}>
          <p>{t("dividende.erklaerung", { steuer: formatProzent(d.tax) })}</p>
        </Erklaerung>
      </h2>
      <p className="feld-hilfe">
        {d.year === null
          ? t("dividende.erstes_jahr")
          : t("dividende.stand", {
              jahr: String(d.year),
              gewinn: formatGeld(d.profit_usd),
              gewollt: formatGeld(d.wanted_usd),
            })}{" "}
        {t("dividende.grenzen", {
          ruecklagen: formatGeld(d.distributable_usd),
          reserve: formatGeld(d.reserve_usd),
          moeglich: formatGeld(d.payable_usd),
        })}{" "}
        {d.last &&
          t("dividende.zuletzt", { jahr: String(d.last[0]), betrag: formatGeld(d.last[1]) })}
      </p>
      {d.ceo && <p className="feld-hilfe">{t("dividende.ceo")}</p>}
      <form className="karte" aria-label={t("dividende.politik")} onSubmit={festlegen}>
        <fieldset>
          <legend>{t("dividende.politik")}</legend>
          <label className="auswahl">
            <input
              type="radio"
              name="dividende-art"
              checked={art === "anteil"}
              onChange={() => setArt("anteil")}
            />
            <span>{t("dividende.anteil")}</span>
          </label>
          <label className="auswahl">
            <input
              type="radio"
              name="dividende-art"
              checked={art === "betrag"}
              onChange={() => setArt("betrag")}
            />
            <span>{t("dividende.betrag")}</span>
          </label>
        </fieldset>
        <div className="formular-zeile">
          {art === "anteil" ? (
            <ZahlFeld
              name={t("dividende.anteil_feld")}
              einheit="%"
              wert={anteil}
              onWert={setAnteil}
              hilfe={t("dividende.anteil_hilfe")}
            />
          ) : (
            <ZahlFeld
              name={t("dividende.betrag_feld")}
              einheit={geldEinheit()}
              wert={betrag}
              onWert={setBetrag}
            />
          )}
        </div>
        <div className="knopfreihe links">
          <button type="submit">{t("dividende.festlegen")}</button>
        </div>
      </form>
      <div className="karte formular-zeile">
        <ZahlFeld
          name={t("dividende.sonder")}
          einheit={geldEinheit()}
          wert={sonder}
          onWert={setSonder}
          hilfe={t("dividende.sonder_hilfe", { max: formatGeld(d.payable_usd) })}
        />
        <button type="button" onClick={ausschuetten} disabled={d.payable_usd <= 0}>
          {t("dividende.ausschuetten")}
        </button>
      </div>
      {fehler && <p className="fehlertext">{fehler}</p>}
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

/**
 * The strategies of the company (MA4): an overview of every unit, then the fields of the
 * chosen unit to set or remove, the sales channels and the participations (ZA2).
 */
export function StrategieAnsicht({ kern, stand }: { kern: Kern; stand: string }) {
  const { daten, fehler } = useSicht(() => kern.strategie(), stand);
  const [auswahl, setAuswahl] = useState("firma");
  const auswahlId = useId();
  if (!daten) return <FehlerText fehler={fehler} />;
  if (!daten.enabled) return <p>{t("organisation.aus")}</p>;
  const einheit = daten.units.find((u) => u.key === auswahl) ?? daten.units[0];
  if (!einheit) return null;
  return (
    <section aria-label={t("strategie.titel")}>
      <p className="feld-hilfe">
        {t("strategie.hilfe")}{" "}
        <Erklaerung wert={t("strategie.titel")}>
          <p>{t("strategie.erklaerung")}</p>
        </Erklaerung>
      </p>
      <Uebersicht daten={daten} auswahl={einheit.key} onWahl={setAuswahl} />
      <section aria-label={t("strategie.vorgaben_fuer", { einheit: einheitTitel(einheit, daten) })}>
        <div className="werk-kopf">
          <h2>{t("strategie.vorgaben_fuer", { einheit: einheitTitel(einheit, daten) })}</h2>
          <div className="feld">
            <label htmlFor={auswahlId}>{t("strategie.einheit")}</label>
            <select id={auswahlId} value={einheit.key} onChange={(e) => setAuswahl(e.target.value)}>
              {daten.units.map((u) => (
                <option key={u.key} value={u.key}>
                  {`${"\u00a0".repeat(2 * EBENEN.indexOf(u.level))}${einheitTitel(u, daten)}`}
                </option>
              ))}
            </select>
          </div>
        </div>
        <p className="feld-hilfe">
          {einheit.level === "firma" ? t("strategie.vererbung_firma") : t("strategie.vererbung")}{" "}
          {einheit.level !== "standort" &&
            t("strategie.standorte", { anzahl: formatZahl(einheit.sites) })}
        </p>
        <div className="vorgaben-karten">
          {einheit.entries.map((e) => {
            const Karte = KARTEN[e.field];
            return (
              <Karte key={`${einheit.key}/${e.field}`} e={e} einheit={einheit} daten={daten} />
            );
          })}
        </div>
      </section>
      <Verkaufswege daten={daten} />
      <Beteiligungen daten={daten} />
      <Dividende daten={daten} />
    </section>
  );
}
