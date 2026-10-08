// Markets (docs/BEDIENUNG.md): which products sell where at what price, where a
// newcomer has a chance, and who rules each market. A click opens one product market.
import { useId, useState } from "react";
import {
  ausAnzeige,
  formatGeld,
  formatPreis,
  formatProzent,
  formatZahl,
  formatZahlKurz,
  geldEinheit,
  geldFeld,
  geldSchluessel,
  landName,
  zahlLesen,
} from "../format";
import { geld, type Kern, type Markt, type ProduktMarkt, type Uebersicht } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { formatMonatKurz, Verlauf } from "./Grafik";
import { KettenAnsicht } from "./Ketten";
import { LogistikAnsicht } from "./Logistik";
import { VertraegeAnsicht } from "./Vertraege";
import {
  Befehle,
  Erklaerung,
  Rueckmeldung,
  Unterreiter,
  useAktion,
  useBefehl,
  useSicht,
  ZahlFeld,
} from "./gemeinsam";
import { LAENDER } from "./laender";

type Zeile = Markt["lines"][number];
type Marke = Markt["brands"][number];
type Filter = "alle" | "eigene" | "chancen";
type Sortierung = "name" | "nachfrage" | "preis" | "versorgung";

const einheit = (units: Record<string, string>, p: string) => t(`einheit.${units[p] ?? "t"}`);

/** Supply of consumers and government as a traffic light. */
function versorgungsZustand(anteil: number | null): "gut" | "knapp" | "fehlt" | null {
  if (anteil === null) return null;
  return anteil >= 0.99 ? "gut" : anteil >= 0.9 ? "knapp" : "fehlt";
}

/** Price against last month's average, as an arrow with percent. */
function Trend({ jetzt, vorher }: { jetzt: number; vorher: number | null }) {
  if (vorher === null || vorher <= 0) return null;
  const d = jetzt / vorher - 1;
  if (Math.abs(d) < 0.005) return <span className="trend gedaempft"> ▶</span>;
  return (
    <span className={`trend ${d > 0 ? "steigt" : "faellt"}`} title={t("markt.trend_hilfe")}>
      {" "}
      {d > 0 ? "▲" : "▼"} {formatProzent(Math.abs(d))}
    </span>
  );
}

/** Price against the reference price: a bar to 200 %, the reference at its middle. */
function PreisZuRichtpreis({ preis, richt }: { preis: number; richt: number }) {
  if (richt <= 0) return <span className="gedaempft">–</span>;
  const r = preis / richt;
  return (
    <span className="richtbalken" title={t("markt.richtpreis_hilfe")}>
      <span className="richtbalken-spur">
        <span
          className={`richtbalken-wert ${r > 1.05 ? "ueber" : r < 0.95 ? "unter" : ""}`}
          style={{ width: `${Math.min(r, 2) * 50}%` }}
        />
      </span>
      <span>{formatProzent(r)}</span>
    </span>
  );
}

function Chancen({ chancen }: { chancen: string[] }) {
  return (
    <>
      {chancen.map((c) => (
        <span key={c} className={`chip chance-${c}`} title={t(`markt.chance.${c}.hilfe`)}>
          {t(`markt.chance.${c}`)}
        </span>
      ))}
    </>
  );
}

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
  const [bereich, setBereich] = useState<
    "produkte" | "marke" | "ketten" | "vertraege" | "logistik"
  >("produkte");
  const [produkt, setProdukt] = useState<string | null>(null);
  const { daten, fehler, neu } = useSicht(() => kern.markt(land), `${uebersicht.date}/${land}`);
  const { senden, meldung } = useBefehl(kern, onGeaendert, neu);
  const id = useId();
  return (
    <main className="ansicht" id="markt">
      <h1 className="unsichtbar">{t("ansicht.markt")}</h1>
      <div className="formular-zeile">
        <div className="feld">
          <label htmlFor={`${id}-land`}>{t("markt.land")}</label>
          <select
            id={`${id}-land`}
            value={land}
            onChange={(e) => {
              setLand(e.target.value);
              setProdukt(null);
            }}
          >
            {LAENDER.map((k) => (
              <option key={k} value={k}>
                {landName(k)}
              </option>
            ))}
          </select>
        </div>
      </div>
      {!daten ? (
        <FehlerText fehler={fehler} />
      ) : (
        <Befehle senden={senden} meldung={meldung}>
          {produkt ? (
            <ProduktmarktAnsicht
              kern={kern}
              land={daten.country}
              produkt={produkt}
              stand={uebersicht.date}
              onZurueck={() => setProdukt(null)}
            />
          ) : (
            <>
              <Unterreiter
                name={t("markt.bereiche")}
                bereiche={[
                  { key: "produkte", text: t("markt.produkte") },
                  { key: "marke", text: t("markt.marke_titel") },
                  { key: "ketten", text: t("markt.ketten") },
                  { key: "vertraege", text: t("markt.vertraege") },
                  { key: "logistik", text: t("markt.logistik") },
                ]}
                aktiv={bereich}
                onWahl={setBereich}
              />
              {bereich === "produkte" && <Produkte daten={daten} onProdukt={setProdukt} />}
              {bereich === "marke" && <MarkeUndWerbung daten={daten} />}
              {bereich === "vertraege" && <VertraegeAnsicht kern={kern} stand={uebersicht.date} />}
              {bereich === "logistik" && <LogistikAnsicht kern={kern} stand={uebersicht.date} />}
              {bereich === "ketten" && (
                <KettenAnsicht
                  kern={kern}
                  stand={uebersicht.date}
                  onProdukt={(p) => {
                    // Prices and costs of the chains are those of the home country.
                    setLand(uebersicht.company.headquarters);
                    setProdukt(p);
                  }}
                />
              )}
            </>
          )}
        </Befehle>
      )}
    </main>
  );
}

function Produkte({ daten, onProdukt }: { daten: Markt; onProdukt: (p: string) => void }) {
  const [filter, setFilter] = useState<Filter>("alle");
  const [sortierung, setSortierung] = useState<Sortierung>("name");
  const [gruppe, setGruppe] = useState("alle");
  const [suche, setSuche] = useState("");
  const id = useId();
  // Products nobody asks for or offers here yet (e.g. cars in 1900) are left out.
  const aktiv = daten.lines.filter(
    (z) => z.demand_last_month > 0 || z.sellers > 0 || z.sold_last_month > 0,
  );
  const ohneMarkt = daten.lines.length - aktiv.length;
  const gruppen = [...new Set(aktiv.map((z) => z.group))].sort((a, b) =>
    t(`warengruppe.${a}`).localeCompare(t(`warengruppe.${b}`), "de"),
  );
  const gesucht = suche.trim().toLocaleLowerCase("de");
  const gefiltert = aktiv.filter(
    (z) =>
      (filter === "alle" ||
        (filter === "eigene" && (z.own_price_usd !== null || z.own_share > 0)) ||
        (filter === "chancen" && z.chances.length > 0)) &&
      (gruppe === "alle" || z.group === gruppe) &&
      (gesucht === "" || t(`produkt.${z.product}`).toLocaleLowerCase("de").includes(gesucht)),
  );
  const wert = (z: Zeile): number => {
    switch (sortierung) {
      case "nachfrage":
        return -z.demand_last_month;
      case "preis":
        return -(z.reference_usd > 0 ? z.price_usd / z.reference_usd : 0);
      case "versorgung":
        return z.supply ?? 2;
      default:
        return 0;
    }
  };
  const zeilen = [...gefiltert].sort(
    (a, b) =>
      wert(a) - wert(b) || t(`produkt.${a.product}`).localeCompare(t(`produkt.${b.product}`), "de"),
  );
  return (
    <>
      <p className="erklaerung">{t("markt.erklaerung")}</p>
      <div className="formular-zeile">
        <fieldset className="auswahlgruppe waagrecht">
          <legend>{t("markt.zeigen")}</legend>
          {(["alle", "eigene", "chancen"] as const).map((f) => (
            <label key={f}>
              <input
                type="radio"
                name={`${id}-filter`}
                checked={filter === f}
                onChange={() => setFilter(f)}
              />
              {t(`markt.filter.${f}`)}
            </label>
          ))}
        </fieldset>
        <div className="feld">
          <label htmlFor={`${id}-sort`}>{t("markt.sortieren")}</label>
          <select
            id={`${id}-sort`}
            value={sortierung}
            onChange={(e) => setSortierung(e.target.value as Sortierung)}
          >
            {(["name", "nachfrage", "preis", "versorgung"] as const).map((s) => (
              <option key={s} value={s}>
                {t(`markt.sort.${s}`)}
              </option>
            ))}
          </select>
        </div>
        <div className="feld">
          <label htmlFor={`${id}-gruppe`}>{t("markt.warengruppe")}</label>
          <select id={`${id}-gruppe`} value={gruppe} onChange={(e) => setGruppe(e.target.value)}>
            <option value="alle">{t("markt.alle_gruppen")}</option>
            {gruppen.map((g) => (
              <option key={g} value={g}>
                {t(`warengruppe.${g}`)} ({aktiv.filter((z) => z.group === g).length})
              </option>
            ))}
          </select>
        </div>
        <div className="feld">
          <label htmlFor={`${id}-suche`}>{t("markt.suchen")}</label>
          <input
            id={`${id}-suche`}
            type="search"
            value={suche}
            placeholder={t("markt.suchen_platzhalter")}
            onChange={(e) => setSuche(e.target.value)}
          />
        </div>
      </div>
      {zeilen.length === 0 && <p className="gedaempft">{t("markt.keine_produkte")}</p>}
      <div className="tabelle">
        <table
          className="mobil-karten marktliste"
          aria-label={t("markt.titel", { land: landName(daten.country) })}
        >
          <thead>
            <tr>
              <th>{t("uebersicht.produkt")}</th>
              <th className="zahl">{t("markt.preis")}</th>
              <th>{t("markt.zum_richtpreis")}</th>
              <th className="zahl">{t("markt.nachfrage_monat")}</th>
              <th className="zahl">{t("markt.versorgung")}</th>
              <th className="zahl spalte-breit">{t("markt.anbieter")}</th>
              <th className="spalte-breit">{t("markt.fuehrer")}</th>
              <th className="zahl spalte-breit">{t("markt.anteil")}</th>
            </tr>
          </thead>
          <tbody>
            {zeilen.map((z) => {
              const e = einheit(daten.units, z.product);
              const zustand = versorgungsZustand(z.supply);
              return (
                <tr key={z.product} className={z.own_price_usd !== null ? "eigen" : undefined}>
                  <td>
                    <button
                      type="button"
                      className="verweis"
                      aria-label={t("markt.oeffnen", { produkt: t(`produkt.${z.product}`) })}
                      onClick={() => onProdukt(z.product)}
                    >
                      {t(`produkt.${z.product}`)}
                    </button>
                    <Chancen chancen={z.chances} />
                  </td>
                  <td className="zahl" data-spalte={t("markt.preis")}>
                    <span>
                      {formatPreis(z.price_usd, e)}
                      <Trend jetzt={z.price_usd} vorher={z.price_last_month_usd} />
                    </span>
                  </td>
                  <td data-spalte={t("markt.zum_richtpreis")}>
                    <PreisZuRichtpreis preis={z.price_usd} richt={z.reference_usd} />
                  </td>
                  <td className="zahl" data-spalte={t("markt.nachfrage_monat")}>
                    {formatZahl(z.demand_last_month, z.demand_last_month < 10 ? 1 : 0)} {e}
                  </td>
                  <td
                    className={`zahl ${zustand ? `zustand-${zustand}` : ""}`}
                    data-spalte={t("markt.versorgung")}
                  >
                    {z.supply === null ? "–" : formatProzent(z.supply)}
                  </td>
                  <td className="zahl spalte-breit" data-spalte={t("markt.anbieter")}>
                    {z.sellers}
                  </td>
                  <td
                    className="name-kurz spalte-breit"
                    title={z.leader ?? undefined}
                    data-spalte={t("markt.fuehrer")}
                  >
                    {z.leader === null ? "–" : `${formatProzent(z.leader_share)} ${z.leader}`}
                  </td>
                  <td className="zahl spalte-breit" data-spalte={t("markt.anteil")}>
                    {z.own_price_usd === null && z.own_share === 0
                      ? "–"
                      : formatProzent(z.own_share)}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      {ohneMarkt > 0 && (
        <p className="feld-hilfe">{t("markt.ohne_markt", { anzahl: ohneMarkt })}</p>
      )}
      <p className="feld-hilfe">{t("markt.hinweis")}</p>
    </>
  );
}

function ProduktmarktAnsicht({
  kern,
  land,
  produkt,
  stand,
  onZurueck,
}: {
  kern: Kern;
  land: string;
  produkt: string;
  stand: string;
  onZurueck: () => void;
}) {
  const { daten, fehler } = useSicht(
    () => kern.produktmarkt(land, produkt),
    `${stand}/${land}/${produkt}`,
  );
  const name = t(`produkt.${produkt}`);
  const titel = t("markt.produktmarkt_titel", { produkt: name, land: landName(land) });
  return (
    <section className="werk" aria-label={titel}>
      <div className="werk-kopf">
        <button type="button" className="schlicht" onClick={onZurueck}>
          ← {t("markt.alle_produkte")}
        </button>
        <h2>{titel}</h2>
      </div>
      {!daten ? <FehlerText fehler={fehler} /> : <Produktmarkt m={daten} />}
    </section>
  );
}

/** How the market price comes about (M27): reference price × price level × situation. */
function PreisTeile({
  p,
  einheit,
  land,
}: {
  p: NonNullable<ProduktMarkt["price_parts"]>;
  einheit: string;
  land: string;
}) {
  const faktor = (f: number) => formatZahl(f, 2);
  const abweichung = p.situation - 1;
  return (
    <>
      <dl className="rechnung">
        <dt>{t("erklaerung.richtpreis_welt")}</dt>
        <dd>{formatPreis(p.world_reference_usd, einheit)}</dd>
        <dt>
          {t("erklaerung.preisniveau", {
            land: landName(land),
            niveau: faktor(p.price_level),
            anteil: formatProzent(p.level_share),
          })}
        </dt>
        <dd>× {faktor(p.level_factor)}</dd>
        <dt>{t("werk.richtpreis")}</dt>
        <dd>= {formatPreis(p.reference_usd, einheit)}</dd>
        <dt>
          {t(
            Math.abs(abweichung) < 0.005
              ? "erklaerung.marktlage_gleich"
              : abweichung > 0
                ? "erklaerung.marktlage_hoch"
                : "erklaerung.marktlage_tief",
            { prozent: formatProzent(Math.abs(abweichung)) },
          )}
        </dt>
        <dd>× {faktor(p.situation)}</dd>
        <dt className="summe">{t("markt.preis")}</dt>
        <dd className="summe">= {formatPreis(p.price_usd, einheit)}</dd>
      </dl>
      <p>
        {t("erklaerung.vormonat", {
          versorgung: p.supply === null ? "–" : formatProzent(p.supply),
          offen: formatProzent(p.unmet_share),
          einfuhr: formatProzent(p.import_share),
          anbieter: p.sellers,
        })}
      </p>
      <p className="gedaempft">
        {t("erklaerung.preisregel", { hoechst: formatZahl(p.price_max_factor) })}
      </p>
    </>
  );
}

/** How the consumer demand comes about (M27): people, income, price and saturation. */
function NachfrageTeile({
  d,
  einheit,
}: {
  d: NonNullable<ProduktMarkt["demand_parts"]>;
  einheit: string;
}) {
  const je = (q: number) => `${formatZahl(q, q < 10 ? 3 : 0)} ${einheit}`;
  const basis =
    d.kind === "verbrauch"
      ? "erklaerung.basis_verbrauch"
      : d.kind === "gebrauch"
        ? "erklaerung.basis_gebrauch"
        : "erklaerung.basis_ergaenzung";
  return (
    <>
      <p>
        {t("erklaerung.menschen", {
          menschen: formatZahlKurz(d.population),
          bip: formatGeld(d.gdp_per_capita_usd),
        })}{" "}
        {t("erklaerung.preisvergleich", {
          preis: formatPreis(d.price_usd, einheit),
          bezug: formatPreis(d.reference_usd, einheit),
        })}
        {d.grid !== null && ` ${t("erklaerung.netz", { anteil: formatProzent(d.grid) })}`}
        {Math.abs(d.season - 1) > 0.005 &&
          ` ${t("erklaerung.saison", { faktor: formatZahl(d.season, 2) })}`}
      </p>
      <div className="tabelle">
        <table className="rechnung-tabelle">
          <thead>
            <tr>
              <th>{t("erklaerung.fuenftel")}</th>
              <th className="zahl">{t("erklaerung.einkommen")}</th>
              <th className="zahl">{t("erklaerung.kaufneigung")}</th>
              <th className="zahl">
                {t(basis, { produkt: d.complement_of ? t(`produkt.${d.complement_of}`) : "" })}
              </th>
              {d.kind === "gebrauch" && <th className="zahl">{t("erklaerung.besitz")}</th>}
              <th className="zahl">{t("erklaerung.je_kopf")}</th>
            </tr>
          </thead>
          <tbody>
            {d.fifths.map((f, i) => (
              <tr key={i}>
                <td>{t(`markt.fuenftel.${i + 1}`)}</td>
                <td className="zahl">{formatGeld(f.income_usd)}</td>
                <td className="zahl">{formatProzent(f.propensity)}</td>
                <td className="zahl">{formatZahl(f.base, f.base < 10 ? 3 : 0)}</td>
                {d.kind === "gebrauch" && <td className="zahl">{formatZahl(f.owned ?? 0, 3)}</td>}
                <td className="zahl">{je(f.per_head_year)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p className="gedaempft">{t(`erklaerung.regel_${d.kind}`)}</p>
    </>
  );
}

/** Months without sales keep the last price paid, so the line does not drop to zero. */
export function fortgeschrieben(werte: (number | null)[], ersatz: number): number[] {
  const erster = werte.find((w) => w !== null) ?? ersatz;
  return werte.reduce<number[]>((liste, w) => [...liste, w ?? liste.at(-1) ?? erster], []);
}

/** Price, sales and the own share over the last months (M24). */
function MarktVerlauf({ m, einheit }: { m: ProduktMarkt; einheit: string }) {
  const verlauf = m.history ?? [];
  if (verlauf.length < 2) return null;
  const monate = verlauf.map((v) => v.month);
  const preise = fortgeschrieben(
    verlauf.map((v) => v.price_usd),
    m.reference_usd,
  );
  const eigen = verlauf.some((v) => v.own_share !== null);
  const zeitraum = (
    <small className="gedaempft">
      {formatMonatKurz(monate[0]!)} – {formatMonatKurz(monate.at(-1)!)}
    </small>
  );
  return (
    <section className="karte" aria-label={t("markt.verlauf_titel")}>
      <h3>{t("markt.verlauf_titel")}</h3>
      <div className="verlaeufe">
        <figure className="verlaufskarte">
          <figcaption>
            {t("markt.verlauf_preis")} {zeitraum}
          </figcaption>
          <Verlauf
            name={t("markt.verlauf_preis")}
            monate={monate}
            werte={preise}
            bezug={m.reference_usd}
            format={(v) => formatPreis(v, einheit)}
          />
          <p className="feld-hilfe">{t("markt.verlauf_richtpreis")}</p>
        </figure>
        <figure className="verlaufskarte">
          <figcaption>
            {t("markt.verlauf_absatz")} {zeitraum}
          </figcaption>
          <Verlauf
            name={t("markt.verlauf_absatz")}
            monate={monate}
            werte={verlauf.map((v) => v.sold)}
            art="balken"
            format={(v) => `${formatZahl(v, v < 10 ? 1 : 0)} ${einheit}`}
          />
        </figure>
        {eigen && (
          <figure className="verlaufskarte">
            <figcaption>
              {t("markt.verlauf_anteil")} {zeitraum}
            </figcaption>
            <Verlauf
              name={t("markt.verlauf_anteil")}
              monate={monate}
              werte={verlauf.map((v) => v.own_share ?? 0)}
              format={(v) => (v > 0 && v < 0.1 ? `${formatZahl(v * 100, 1)} %` : formatProzent(v))}
            />
          </figure>
        )}
      </div>
    </section>
  );
}

function Produktmarkt({ m }: { m: ProduktMarkt }) {
  const e = t(`einheit.${m.unit}`);
  const menge = (q: number) => `${formatZahl(q, q < 10 ? 1 : 0)} ${e}`;
  const verbraucher = m.consumers_per_month.reduce((s, q) => s + q, 0);
  const industrie = Math.max(m.demand_last_month - m.outside_demand_last_month, 0);
  const groesste = Math.max(...m.consumers_per_month, m.state_per_month, industrie, 1e-9);
  const zustand = versorgungsZustand(m.supply);
  return (
    <>
      <dl className="kennzahlen">
        <div>
          <dt>
            <span>{t("markt.preis")}</span>
            {m.price_parts && (
              <Erklaerung wert={t("markt.preis")}>
                <PreisTeile p={m.price_parts} einheit={e} land={m.country} />
              </Erklaerung>
            )}
          </dt>
          <dd>
            {formatPreis(m.price_usd, e)}
            <Trend jetzt={m.price_usd} vorher={m.price_last_month_usd} />
          </dd>
        </div>
        <div>
          <dt>{t("werk.richtpreis")}</dt>
          <dd>{formatPreis(m.reference_usd, e)}</dd>
        </div>
        <div>
          <dt>{t("markt.nachfrage_monat")}</dt>
          <dd>{menge(m.demand_last_month)}</dd>
        </div>
        <div>
          <dt>{t("markt.versorgung")}</dt>
          <dd className={zustand ? `zustand-${zustand}` : ""}>
            {m.supply === null ? "–" : formatProzent(m.supply)}
          </dd>
        </div>
      </dl>
      {m.chances.length > 0 && (
        <ul className="chancenliste">
          {m.chances.map((c) => (
            <li key={c}>
              <span className={`chip chance-${c}`}>{t(`markt.chance.${c}`)}</span>{" "}
              {t(`markt.chance.${c}.hilfe`)}
            </li>
          ))}
        </ul>
      )}

      <MarktVerlauf m={m} einheit={e} />

      {m.nameable && <Produktname m={m} />}

      <section className="karte" aria-label={t("markt.anbieter_titel")}>
        <h3>{t("markt.anbieter_titel")}</h3>
        {m.sellers.length === 0 ? (
          <p className="gedaempft">{t("markt.keine_anbieter")}</p>
        ) : (
          <div className="tabelle">
            <table className="mobil-karten" aria-label={t("markt.anbieter_titel")}>
              <thead>
                <tr>
                  <th>{t("uebersicht.firma")}</th>
                  {m.nameable && <th>{t("markt.produktname")}</th>}
                  <th className="zahl">{t("markt.preis_anbieter")}</th>
                  <th className="zahl">{t("markt.verkauft_vormonat")}</th>
                  <th>{t("markt.anteil_markt")}</th>
                  <th className="zahl" title={t("markt.stufe_hilfe")}>
                    {t("markt.stufe")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {m.sellers.map((s) => (
                  <tr key={s.company} className={s.own ? "eigen" : undefined}>
                    <td>
                      {s.company}
                      {s.own && <span className="marke">{t("markt.du")}</span>}
                      {s.real && <span className="marke">{t("uebersicht.real")}</span>}
                    </td>
                    {m.nameable && (
                      <td data-spalte={t("markt.produktname")}>
                        {s.product_name ?? (
                          <span className="gedaempft">{t("markt.ohne_namen")}</span>
                        )}
                      </td>
                    )}
                    <td className="zahl" data-spalte={t("markt.preis_anbieter")}>
                      {formatPreis(s.price_usd, e)}
                    </td>
                    <td className="zahl" data-spalte={t("markt.verkauft_vormonat")}>
                      {menge(s.sold_last_month)}
                    </td>
                    <td data-spalte={t("markt.anteil_markt")}>
                      <span className="anteilbalken">
                        <span style={{ width: `${s.share * 100}%` }} />
                      </span>{" "}
                      {formatProzent(s.share)}
                    </td>
                    <td className="zahl" data-spalte={t("markt.stufe")}>
                      {s.level > 0 ? s.level : "–"}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
        <dl className="werte">
          {m.imported_last_month > 0 && (
            <>
              <dt>{t("markt.einfuhr")}</dt>
              <dd>
                {menge(m.imported_last_month)}
                {m.import_price_usd !== null && ` (${formatPreis(m.import_price_usd, e)})`}
              </dd>
            </>
          )}
          {m.exported_last_month > 0 && (
            <>
              <dt>{t("markt.ausfuhr")}</dt>
              <dd>{menge(m.exported_last_month)}</dd>
            </>
          )}
          {m.state_price_usd !== null && (
            <>
              <dt>{t("markt.staatsmarkt")}</dt>
              <dd>{formatPreis(m.state_price_usd, e)}</dd>
            </>
          )}
          <dt>{t("markt.deine_bekanntheit")}</dt>
          <dd>
            {formatProzent(m.own_awareness)} ({t(`warengruppe.${m.group}`)})
          </dd>
        </dl>
      </section>

      <section className="karte" aria-label={t("markt.nachfrage_titel")}>
        <h3>
          <span>{t("markt.nachfrage_titel")}</span>
          {m.demand_parts && (
            <Erklaerung wert={t("erklaerung.nachfrage")}>
              <NachfrageTeile d={m.demand_parts} einheit={e} />
            </Erklaerung>
          )}
        </h3>
        <p className="feld-hilfe">{t("markt.nachfrage_hilfe")}</p>
        <dl className="nachfrage">
          {m.consumers_per_month.map((q, i) => (
            <div key={i} className="balkenzeile">
              <dt>{t(`markt.fuenftel.${i + 1}`)}</dt>
              <dd>
                <span className="balken" style={{ width: `${(q / groesste) * 100}%` }} />
                <span className="balkenwert">{menge(q)}</span>
              </dd>
            </div>
          ))}
          <div className="balkenzeile">
            <dt>{t("markt.staat_nachfrage")}</dt>
            <dd>
              <span
                className="balken"
                style={{ width: `${(m.state_per_month / groesste) * 100}%` }}
              />
              <span className="balkenwert">{menge(m.state_per_month)}</span>
            </dd>
          </div>
          <div className="balkenzeile">
            <dt>{t("markt.industrie")}</dt>
            <dd>
              <span className="balken" style={{ width: `${(industrie / groesste) * 100}%` }} />
              <span className="balkenwert">{menge(industrie)}</span>
            </dd>
          </div>
        </dl>
        <p className="feld-hilfe">{t("markt.verbraucher_summe", { menge: menge(verbraucher) })}</p>
      </section>
    </>
  );
}

/** The player's own name for an end product (M42), with suggestions from the core. */
function Produktname({ m }: { m: ProduktMarkt }) {
  const produkt = t(`produkt.${m.product}`);
  const [gespeichert, setGespeichert] = useState(m.own_name ?? null);
  const [name, setName] = useState(m.own_name ?? m.name_suggestions?.[0] ?? "");
  const { los, antwort } = useAktion(`produktname/${m.product}`);
  const id = useId();
  const speichern = async (neu: string | null) => {
    const sauber = neu === null ? null : neu.trim().replace(/\s+/g, " ");
    const ok = await los(
      [{ NameProduct: { product: m.product, name: sauber } }],
      sauber === null
        ? t("markt.produktname_entfernt", { produkt })
        : t("markt.produktname_gesetzt", { produkt, name: sauber }),
    );
    if (ok) setGespeichert(sauber);
  };
  return (
    <form
      className="karte"
      aria-label={t("markt.produktname_titel")}
      onSubmit={(e) => {
        e.preventDefault();
        void speichern(name);
      }}
    >
      <h3>{t("markt.produktname_titel")}</h3>
      <p>
        {gespeichert === null
          ? t("markt.produktname_ohne", { produkt })
          : t("markt.produktname_aktuell", { produkt, name: gespeichert })}
      </p>
      <div className="formular-zeile">
        <div className="feld">
          <label htmlFor={`${id}-name`}>{t("markt.produktname")}</label>
          <input
            id={`${id}-name`}
            value={name}
            maxLength={40}
            onChange={(e) => setName(e.target.value)}
          />
        </div>
        <button type="submit">{t("markt.produktname_speichern")}</button>
        {gespeichert !== null && (
          <button type="button" className="schlicht" onClick={() => void speichern(null)}>
            {t("markt.produktname_entfernen")}
          </button>
        )}
      </div>
      {(m.name_suggestions?.length ?? 0) > 0 && (
        <p className="vorschlaege">
          {t("markt.produktname_vorschlaege")}{" "}
          {m.name_suggestions?.map((v) => (
            <button key={v} type="button" onClick={() => setName(v)}>
              {v}
            </button>
          ))}
        </p>
      )}
      <p className="gedaempft">{t("markt.produktname_hilfe")}</p>
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

function MarkeUndWerbung({ daten }: { daten: Markt }) {
  return (
    <>
      <p className="erklaerung">{t("markt.marke_hinweis")}</p>
      <p>
        {daten.medium === null
          ? t("markt.kein_werbemittel")
          : t("markt.werbemittel", {
              mittel: t(`werbemittel.${daten.medium}`),
              betrag: formatGeld(daten.reach_usd),
            })}
      </p>
      <div className="karten-raster">
        {daten.brands.map((m) => (
          <Werbung
            key={`${daten.country}/${m.group}/${m.budget_usd}/${geldSchluessel()}`}
            land={daten.country}
            marke={m}
          />
        ))}
      </div>
    </>
  );
}

function Werbung({ land, marke }: { land: string; marke: Marke }) {
  const gruppe = t(`warengruppe.${marke.group}`);
  const [betrag, setBetrag] = useState(geldFeld(marke.budget_usd));
  const [fehler, setFehler] = useState<string | null>(null);
  const { los, antwort } = useAktion(`werbung/${land}/${marke.group}`);
  return (
    <form
      className="karte"
      aria-label={t("markt.werbung_fuer", { gruppe })}
      onSubmit={(e) => {
        e.preventDefault();
        const b = zahlLesen(betrag || "0");
        if (b === null || b < 0) {
          setFehler(t("markt.budget_ungueltig"));
          return;
        }
        setFehler(null);
        void los(
          [{ SetAdvertising: { country: land, group: marke.group, budget: geld(ausAnzeige(b)) } }],
          b > 0
            ? t("markt.budget_gesetzt", { betrag: formatGeld(ausAnzeige(b)) })
            : t("markt.werbung_beendet"),
        );
      }}
    >
      <h3>{gruppe}</h3>
      <dl className="bekanntheit">
        <div className="balkenzeile eigen">
          <dt>{t("markt.bekanntheit")}</dt>
          <dd>
            <span className="balken" style={{ width: `${marke.own_awareness * 100}%` }} />
            <span className="balkenwert">{formatProzent(marke.own_awareness)}</span>
          </dd>
        </div>
        <div className="balkenzeile">
          <dt>{marke.top ?? t("markt.konkurrenz")}</dt>
          <dd>
            <span className="balken" style={{ width: `${marke.top_awareness * 100}%` }} />
            <span className="balkenwert">
              {marke.top === null ? "–" : formatProzent(marke.top_awareness)}
            </span>
          </dd>
        </div>
      </dl>
      <div className="formular-zeile">
        <ZahlFeld
          name={t("markt.werbebudget")}
          einheit={t("markt.je_monat", { waehrung: geldEinheit() })}
          wert={betrag}
          onWert={setBetrag}
        />
        <button type="submit">{t("markt.budget_setzen")}</button>
      </div>
      {fehler && <p className="fehlertext">{fehler}</p>}
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}
