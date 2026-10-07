// The mandate to the board and the CEO's strategy reviews (MA5, docs/BEDIENUNG.md):
// "What is the board to aim at, how did the last period go, and what does the CEO
// propose?"
import { useId, useState, type FormEvent } from "react";
import {
  formatDatum,
  formatGeld,
  formatProzent,
  formatZahl,
  landName,
  zahlFeld,
  zahlLesen,
} from "../format";
import type {
  Auftrag,
  AuftragSicht,
  Kern,
  Leitlinie,
  Ruecksprache,
  Ruecksprachen,
  Ruecksprachetakt,
  Zielstand,
} from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { Erklaerung, Rueckmeldung, ZahlFeld, useAktion, useSicht } from "./gemeinsam";

/** Where the answers to the mandate appear. */
const ORT = "auftrag";

const LEITLINIEN: Record<Exclude<AuftragSicht["guideline"], "marktfuehrung">, Leitlinie> = {
  wachstum: "Growth",
  ertrag: "Profit",
  sicherheit: "Safety",
};

const TAKTE: Record<AuftragSicht["review"], Ruecksprachetakt> = {
  monatlich: "Monthly",
  quartalsweise: "Quarterly",
  halbjaehrlich: "HalfYearly",
  jaehrlich: "Yearly",
};

/** A goal's value: a place for the rank, else a share. */
function zielWert(z: Pick<Zielstand, "goal">, wert: number): string {
  return z.goal === "rang"
    ? t("ruecksprache.platz", { platz: formatZahl(wert) })
    : formatProzent(wert);
}

/** A share as the form shows it: percent with one decimal, empty for none. */
function prozentFeld(anteil: number | null): string {
  return anteil === null ? "" : zahlFeld(anteil * 100, 1);
}

/**
 * A share from a percent field: `null` for an empty field, `undefined` for a value outside
 * the bounds or no number.
 */
function anteilLesen(text: string, von: number, bis: number): number | null | undefined {
  if (text.trim() === "") return null;
  const wert = zahlLesen(text);
  if (wert === null || wert < von || wert > bis) return undefined;
  return wert / 100;
}

/** Keys to choose from and those chosen, as removable chips with a list to add more. */
function Auswahl({
  name,
  alle,
  gewaehlt,
  text,
  onWahl,
}: {
  name: string;
  alle: string[];
  gewaehlt: string[];
  text: (k: string) => string;
  onWahl: (neu: string[]) => void;
}) {
  const id = useId();
  const frei = alle.filter((k) => !gewaehlt.includes(k));
  return (
    <div className="feld">
      <label htmlFor={id}>{name}</label>
      <select
        id={id}
        value=""
        onChange={(e) => e.target.value && onWahl([...gewaehlt, e.target.value])}
      >
        <option value="">{t("auftrag.hinzufuegen")}</option>
        {frei
          .map((k) => ({ k, name: text(k) }))
          .sort((a, b) => a.name.localeCompare(b.name, "de"))
          .map(({ k, name }) => (
            <option key={k} value={k}>
              {name}
            </option>
          ))}
      </select>
      {gewaehlt.length > 0 && (
        <ul className="chips" aria-label={name}>
          {gewaehlt.map((k) => (
            <li key={k}>
              {text(k)}{" "}
              <button
                type="button"
                className="schlicht"
                aria-label={`${t("auftrag.entfernen")}: ${text(k)}`}
                onClick={() => onWahl(gewaehlt.filter((x) => x !== k))}
              >
                ×
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

/** The mandate to the board: guideline, goals, limits and how often the CEO reports. */
function AuftragFormular({ daten }: { daten: Ruecksprachen }) {
  const id = useId();
  const { los, antwort } = useAktion(ORT);
  const m = daten.mandate;
  const [leitlinie, setLeitlinie] = useState(m.guideline);
  const [gruppe, setGruppe] = useState(m.leading_group ?? daten.groups[0] ?? "");
  const [wachstum, setWachstum] = useState(prozentFeld(m.growth));
  const [rendite, setRendite] = useState(prozentFeld(m.margin));
  const [quote, setQuote] = useState(prozentFeld(m.equity_ratio));
  const [rang, setRang] = useState(m.rank === null ? "" : String(m.rank));
  const [schulden, setSchulden] = useState(prozentFeld(m.max_debt));
  const [laender, setLaender] = useState<string[]>(m.blocked_countries);
  const [gruppen, setGruppen] = useState<string[]>(m.blocked_groups);
  const [takt, setTakt] = useState(m.review);
  const [fehler, setFehler] = useState<string | null>(null);
  const uebernehmen = (ev: FormEvent) => {
    ev.preventDefault();
    const growth = anteilLesen(wachstum, -100, 1000);
    const margin = anteilLesen(rendite, -100, 100);
    const equity = anteilLesen(quote, 0, 100);
    const debt = anteilLesen(schulden, 0, 100);
    const platz = rang.trim() === "" ? null : zahlLesen(rang);
    if (
      growth === undefined ||
      margin === undefined ||
      equity === undefined ||
      debt === undefined ||
      (platz !== null && (!Number.isInteger(platz) || platz < 1))
    ) {
      setFehler(t("auftrag.grenzen"));
      return;
    }
    setFehler(null);
    const mandate: Auftrag = {
      guideline: leitlinie === "marktfuehrung" ? { Leadership: gruppe } : LEITLINIEN[leitlinie],
      goals: { growth, margin, equity_ratio: equity, rank: platz },
      max_debt: debt,
      blocked_countries: laender,
      blocked_groups: gruppen,
      review: TAKTE[takt],
    };
    void los([{ SetMandate: { mandate } }], t("auftrag.gesetzt"));
  };
  return (
    <form className="karte" aria-label={t("auftrag.titel")} onSubmit={uebernehmen}>
      <h2>
        {t("auftrag.titel")}{" "}
        <Erklaerung wert={t("auftrag.titel")}>
          <p>{t("auftrag.erklaerung")}</p>
        </Erklaerung>
      </h2>
      <p className="feld-hilfe">{t("auftrag.hilfe")}</p>
      <div className="formular-zeile">
        <div className="feld">
          <label htmlFor={`${id}-leitlinie`}>{t("auftrag.leitlinie_titel")}</label>
          <select
            id={`${id}-leitlinie`}
            value={leitlinie}
            onChange={(e) => setLeitlinie(e.target.value as AuftragSicht["guideline"])}
          >
            {daten.guidelines.map((g) => (
              <option key={g.key} value={g.key}>
                {t(`auftrag.leitlinie.${g.key}`)}
              </option>
            ))}
          </select>
          <small className="feld-hilfe">
            {t(`auftrag.leitlinie_hilfe.${leitlinie}`, {
              wert: formatProzent(
                daten.guidelines.find((g) => g.key === leitlinie)?.aggressiveness ?? 0,
              ),
            })}
          </small>
        </div>
        {leitlinie === "marktfuehrung" && (
          <div className="feld">
            <label htmlFor={`${id}-gruppe`}>{t("auftrag.warengruppe")}</label>
            <select id={`${id}-gruppe`} value={gruppe} onChange={(e) => setGruppe(e.target.value)}>
              {daten.groups.map((g) => (
                <option key={g} value={g}>
                  {t(`warengruppe.${g}`)}
                </option>
              ))}
            </select>
          </div>
        )}
        <div className="feld">
          <label htmlFor={`${id}-takt`}>{t("auftrag.ruecksprache_titel")}</label>
          <select
            id={`${id}-takt`}
            value={takt}
            onChange={(e) => setTakt(e.target.value as AuftragSicht["review"])}
          >
            {daten.intervals.map((k) => (
              <option key={k} value={k}>
                {t(`auftrag.ruecksprache.${k}`)}
              </option>
            ))}
          </select>
        </div>
      </div>
      <fieldset>
        <legend>{t("auftrag.ziele")}</legend>
        <p className="feld-hilfe">{t("auftrag.ziele_hilfe")}</p>
        <div className="formular-zeile">
          <ZahlFeld
            name={t("ziel.wachstum")}
            einheit="%"
            wert={wachstum}
            onWert={setWachstum}
            negativ
            gruppieren={false}
          />
          <ZahlFeld
            name={t("ziel.rendite")}
            einheit="%"
            wert={rendite}
            onWert={setRendite}
            negativ
            gruppieren={false}
          />
          <ZahlFeld
            name={t("ziel.eigenkapitalquote")}
            einheit="%"
            wert={quote}
            onWert={setQuote}
            gruppieren={false}
          />
          <ZahlFeld
            name={t("ziel.rang")}
            wert={rang}
            onWert={setRang}
            ganzzahlig
            gruppieren={false}
            hilfe={t("auftrag.rang_hilfe")}
          />
        </div>
      </fieldset>
      <fieldset>
        <legend>{t("auftrag.grenzen_titel")}</legend>
        <div className="formular-zeile">
          <ZahlFeld
            name={t("auftrag.verschuldung")}
            einheit="%"
            wert={schulden}
            onWert={setSchulden}
            gruppieren={false}
            hilfe={t("auftrag.verschuldung_hilfe")}
          />
          <Auswahl
            name={t("auftrag.gesperrte_laender")}
            alle={daten.countries}
            gewaehlt={laender}
            text={landName}
            onWahl={setLaender}
          />
          <Auswahl
            name={t("auftrag.gesperrte_gruppen")}
            alle={daten.groups}
            gewaehlt={gruppen}
            text={(g) => t(`warengruppe.${g}`)}
            onWahl={setGruppen}
          />
        </div>
        <p className="feld-hilfe">{t("auftrag.sperren_hilfe")}</p>
      </fieldset>
      {fehler && <p className="fehlertext">{fehler}</p>}
      <div className="knopfreihe links">
        <button type="submit">{t("auftrag.uebernehmen")}</button>
      </div>
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

/** "erreicht", "verfehlt" or "noch kein Wert". */
function erreicht(z: Zielstand): string {
  if (z.met === null) return t("ruecksprache.kein_wert");
  return z.met ? t("ruecksprache.erreicht") : t("ruecksprache.verfehlt");
}

function Ziele({ ziele, name }: { ziele: Zielstand[]; name: string }) {
  if (ziele.length === 0) return <p className="feld-hilfe">{t("ruecksprache.keine_ziele")}</p>;
  return (
    <div className="tabelle">
      <table className="mobil-karten" aria-label={name}>
        <thead>
          <tr>
            <th>{t("ruecksprache.ziel")}</th>
            <th className="zahl">{t("ruecksprache.vorgabe")}</th>
            <th className="zahl">{t("ruecksprache.ist")}</th>
            <th>{t("ruecksprache.stand")}</th>
          </tr>
        </thead>
        <tbody>
          {ziele.map((z) => (
            <tr key={z.goal}>
              <td>{t(`ziel.${z.goal}`)}</td>
              <td className="zahl" data-spalte={t("ruecksprache.vorgabe")}>
                {zielWert(z, z.target)}
              </td>
              <td className="zahl" data-spalte={t("ruecksprache.ist")}>
                {z.actual === null ? "–" : zielWert(z, z.actual)}
              </td>
              <td data-spalte={t("ruecksprache.stand")}>
                <span className={z.met === false ? "negativ" : undefined}>{erreicht(z)}</span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/** A risk in words. */
function risikoText(r: Ruecksprache["risks"][number]): string {
  switch (r.kind) {
    case "verlust":
      return t("ruecksprache.risiko.verlust", {
        standort: `${t(r.site_kind_text ?? "")} · ${landName(r.country ?? "")}`,
        betrag: formatGeld(r.amount_usd ?? 0),
      });
    case "ziel":
      return t("ruecksprache.risiko.ziel", { ziel: t(`ziel.${r.goal ?? ""}`) });
    case "reserve":
      return t("ruecksprache.risiko.reserve", {
        kasse: formatGeld(r.amount_usd ?? 0),
        reserve: formatGeld(r.limit_usd ?? 0),
      });
    case "verschuldung":
      return t("ruecksprache.risiko.verschuldung", {
        anteil: formatProzent(r.share ?? 0),
        grenze: formatProzent(r.max ?? 0),
      });
  }
}

/** A chance in words. */
function chanceText(c: Ruecksprache["chances"][number]): string {
  if (c.kind === "produkt") {
    return t("ruecksprache.chance.produkt", {
      produkt: t(`produkt.${c.product ?? ""}`),
      marge: formatGeld(c.margin_usd),
      anteil: formatProzent(c.revenue_usd > 0 ? c.margin_usd / c.revenue_usd : 0),
    });
  }
  const thema = t(`thema.${c.topic ?? ""}`);
  const was = c.product ? `${thema} · ${t(`produkt.${c.product}`)}` : thema;
  return t(c.open ? "ruecksprache.chance.antrag_offen" : "ruecksprache.chance.antrag", {
    antrag: was,
  });
}

/** One review: figures, goals, chances and risks, the proposals. */
function Bericht({ r, onAnliegen }: { r: Ruecksprache; onAnliegen: () => void }) {
  const zeitraum = t("ruecksprache.zeitraum", {
    von: formatDatum(r.from),
    bis: formatDatum(r.to),
  });
  const offen = r.chances.filter((c) => c.kind === "antrag" && c.open).length;
  return (
    <>
      <p className="feld-hilfe">
        {t("ruecksprache.von", {
          name: r.manager,
          takt: t(`auftrag.ruecksprache.${r.interval}`),
        })}
      </p>
      <div className="tabelle">
        <table className="mobil-karten" aria-label={t("ruecksprache.zahlen", { zeitraum })}>
          <thead>
            <tr>
              <th>{t("ruecksprache.teil")}</th>
              <th className="zahl">{t("ruecksprache.umsatz")}</th>
              <th className="zahl">{t("ruecksprache.ergebnis")}</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>
                <strong>{t("ebene.firma")}</strong>
              </td>
              <td className="zahl" data-spalte={t("ruecksprache.umsatz")}>
                {formatGeld(r.revenue_usd)}
              </td>
              <td className="zahl" data-spalte={t("ruecksprache.ergebnis")}>
                {formatGeld(r.result_usd)}
              </td>
            </tr>
            {r.continents.map((k) => (
              <tr key={`k/${k.key}`}>
                <td>{t(`kontinent.${k.key}`)}</td>
                <td className="zahl" data-spalte={t("ruecksprache.umsatz")}>
                  {formatGeld(k.revenue_usd)}
                </td>
                <td className="zahl" data-spalte={t("ruecksprache.ergebnis")}>
                  {formatGeld(k.result_usd)}
                </td>
              </tr>
            ))}
            <tr>
              <td>{t("ruecksprache.gemeinkosten")}</td>
              <td className="zahl" data-spalte={t("ruecksprache.umsatz")}>
                –
              </td>
              <td className="zahl" data-spalte={t("ruecksprache.ergebnis")}>
                {formatGeld(r.overhead_usd)}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      {r.groups.length > 0 && (
        <div className="tabelle">
          <table className="mobil-karten" aria-label={t("ruecksprache.gruppen", { zeitraum })}>
            <thead>
              <tr>
                <th>{t("auftrag.warengruppe")}</th>
                <th className="zahl">{t("ruecksprache.umsatz")}</th>
                <th className="zahl">{t("ruecksprache.marge")}</th>
              </tr>
            </thead>
            <tbody>
              {r.groups.map((g) => (
                <tr key={g.key}>
                  <td>{t(`warengruppe.${g.key}`)}</td>
                  <td className="zahl" data-spalte={t("ruecksprache.umsatz")}>
                    {formatGeld(g.revenue_usd)}
                  </td>
                  <td className="zahl" data-spalte={t("ruecksprache.marge")}>
                    {formatGeld(g.result_usd)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      <h4>{t("ruecksprache.ziele")}</h4>
      <Ziele ziele={r.goals} name={t("ruecksprache.ziele_zeitraum", { zeitraum })} />
      <div className="formular-zeile">
        <section aria-label={t("ruecksprache.chancen")}>
          <h4>{t("ruecksprache.chancen")}</h4>
          {r.chances.length === 0 ? (
            <p className="feld-hilfe">{t("ruecksprache.keine_chancen")}</p>
          ) : (
            <ul>
              {r.chances.map((c, i) => (
                <li key={i}>{chanceText(c)}</li>
              ))}
            </ul>
          )}
        </section>
        <section aria-label={t("ruecksprache.risiken")}>
          <h4>{t("ruecksprache.risiken")}</h4>
          {r.risks.length === 0 ? (
            <p className="feld-hilfe">{t("ruecksprache.keine_risiken")}</p>
          ) : (
            <ul>
              {r.risks.map((x, i) => (
                <li key={i}>{risikoText(x)}</li>
              ))}
            </ul>
          )}
        </section>
      </div>
      {offen > 0 && (
        <div className="knopfreihe links">
          <button type="button" onClick={onAnliegen}>
            {t("ruecksprache.zu_antraegen", { anzahl: formatZahl(offen) })}
          </button>
        </div>
      )}
    </>
  );
}

/** The reviews of the CEO, the newest open, the older ones folded. */
function Berichte({ daten, onAnliegen }: { daten: Ruecksprachen; onAnliegen: () => void }) {
  if (!daten.ceo) {
    return <p className="feld-hilfe">{t("ruecksprache.ohne_ceo")}</p>;
  }
  const [neueste, ...aeltere] = daten.reviews;
  return (
    <>
      <p>
        {t("ruecksprache.naechste", {
          name: daten.ceo,
          datum: formatDatum(daten.next_review ?? ""),
        })}
      </p>
      <h3>{t("ruecksprache.ziele_heute")}</h3>
      <Ziele ziele={daten.goals_now} name={t("ruecksprache.ziele_heute")} />
      {!neueste && <p className="feld-hilfe">{t("ruecksprache.keine")}</p>}
      {neueste && (
        <article
          className="karte"
          aria-label={t("ruecksprache.titel", {
            von: formatDatum(neueste.from),
            bis: formatDatum(neueste.to),
          })}
        >
          <h3>
            {t("ruecksprache.titel", {
              von: formatDatum(neueste.from),
              bis: formatDatum(neueste.to),
            })}
          </h3>
          <Bericht r={neueste} onAnliegen={onAnliegen} />
        </article>
      )}
      {aeltere.map((r) => (
        <details key={r.date} className="karte">
          <summary>
            {t("ruecksprache.titel", { von: formatDatum(r.from), bis: formatDatum(r.to) })} ·{" "}
            {t("ruecksprache.kurz", {
              umsatz: formatGeld(r.revenue_usd),
              ergebnis: formatGeld(r.result_usd),
            })}
          </summary>
          <Bericht r={r} onAnliegen={onAnliegen} />
        </details>
      ))}
    </>
  );
}

/** The mandate and the reviews (MA5). */
export function RuecksprachenAnsicht({
  kern,
  stand,
  onAnliegen,
}: {
  kern: Kern;
  stand: string;
  onAnliegen: () => void;
}) {
  const { daten, fehler } = useSicht(() => kern.ruecksprache(), stand);
  if (!daten) return <FehlerText fehler={fehler} />;
  if (!daten.enabled) return <p>{t("organisation.aus")}</p>;
  return (
    <section aria-label={t("ruecksprache.ansicht")}>
      {/* A new key after every change keeps the form in step with the core. */}
      <AuftragFormular key={JSON.stringify(daten.mandate)} daten={daten} />
      <section aria-label={t("ruecksprache.berichte")}>
        <h2>{t("ruecksprache.berichte")}</h2>
        <Berichte daten={daten} onAnliegen={onAnliegen} />
      </section>
    </section>
  );
}
