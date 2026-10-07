// Start-ups (SU1, SU2; docs/BEDIENUNG.md "Beteiligungen"): "Who works on which
// technology, how far along is it, how likely is it to succeed, who owns it – and where do
// I put my money?"
import { useState } from "react";
import {
  ausAnzeige,
  formatDatum,
  formatGeld,
  formatProzent,
  formatZahl,
  geldEinheit,
  landName,
  zahlLesen,
} from "../format";
import type { Ausgruendung, Befehl, Kern, StartUp, StartUps, Tempo, Uebersicht } from "../kern";
import { geld } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import {
  Befehle,
  Rueckmeldung,
  Unterreiter,
  ZahlFeld,
  useAktion,
  useBefehl,
  useSicht,
} from "./gemeinsam";

type Reiter = "laufend" | "eigene" | "ausgruenden" | "beendet";

/** The pace keys of the data and the core's names. */
const TEMPI: Record<string, Tempo> = { normal: "Normal", zuegig: "Fast", gruendlich: "Thorough" };

/** What a start-up works on: a new technology with its lead, or a product's next level. */
function ziel(s: StartUp): string {
  if (s.kind === "technologie") {
    const name = t(`technologie.${s.target}`);
    return s.lead >= 1
      ? t("beteiligungen.ziel_neu", { technologie: name, jahre: formatZahl(s.lead) })
      : name;
  }
  return t("beteiligungen.ziel_verbesserung", {
    produkt: t(`produkt.${s.target}`),
    stufe: formatZahl(s.level ?? 0),
  });
}

/** The owners with their shares, largest first. */
function eigner(s: StartUp): string {
  return [...s.owners]
    .sort((a, b) => b.share - a.share)
    .map((o) => {
      const wer =
        o.holder === "firma"
          ? (o.company ?? t("beteiligungen.eigner.firma"))
          : t(`beteiligungen.eigner.${o.holder}`);
      return `${wer} ${formatProzent(o.share)}`;
    })
    .join(", ");
}

/** The open round or the funded phase. */
function finanzierung(s: StartUp): string {
  if (s.round_until) {
    return t("beteiligungen.runde_offen", {
      bedarf: formatGeld(s.capital_usd - s.raised_usd),
      datum: formatDatum(s.round_until),
    });
  }
  return t("beteiligungen.finanziert", {
    bedarf: formatGeld(s.capital_usd),
    datum: s.phase_until ? formatDatum(s.phase_until) : "–",
  });
}

function chanceText(s: StartUp, geschaetzt: boolean): string {
  return geschaetzt && s.chance !== null
    ? formatProzent(s.chance)
    : t(`beteiligungen.stufe.${s.chance_level ?? "gering"}`);
}

/** The player's part: share and pledge. */
function eigenerTeil(s: StartUp): string {
  const teile: string[] = [];
  if (s.own_share > 0) teile.push(formatProzent(s.own_share));
  if (s.own_pledge_usd > 0) {
    teile.push(t("beteiligungen.zugesagt", { betrag: formatGeld(s.own_pledge_usd) }));
  }
  return teile.length > 0 ? teile.join(", ") : "–";
}

function Name({ s }: { s: StartUp }) {
  return (
    <td>
      <strong>{s.name}</strong>
      {s.inventor && (
        <span className="marke" title={t("beteiligungen.erfinder_hilfe")}>
          {t("beteiligungen.erfinder")}
        </span>
      )}
      {s.subsidiary && <span className="marke">{t("beteiligungen.tochter")}</span>}
      {s.origin && (
        <span className="marke" title={t("beteiligungen.herkunft", { firma: s.origin })}>
          {t("beteiligungen.ausgruendung_marke")}
        </span>
      )}
    </td>
  );
}

function Laufende({
  liste,
  geschaetzt,
  onWahl,
}: {
  liste: StartUp[];
  geschaetzt: boolean;
  onWahl: (id: number) => void;
}) {
  if (liste.length === 0) return <p className="gedaempft">{t("beteiligungen.keine_laufenden")}</p>;
  return (
    <div className="tabelle">
      <table className="mobil-karten" aria-label={t("beteiligungen.laufend")}>
        <thead>
          <tr>
            <th>{t("beteiligungen.name")}</th>
            <th>{t("beteiligungen.land")}</th>
            <th>{t("beteiligungen.ziel")}</th>
            <th>{t("beteiligungen.phase")}</th>
            <th>{t("beteiligungen.finanzierung")}</th>
            <th className="zahl">{t("beteiligungen.chance")}</th>
            <th>{t("beteiligungen.dein_anteil")}</th>
            <th>
              <span className="unsichtbar">{t("beteiligungen.handeln")}</span>
            </th>
          </tr>
        </thead>
        <tbody>
          {liste.map((s) => (
            <tr key={s.id} className={s.own_share > 0 || s.subsidiary ? "eigen" : undefined}>
              <Name s={s} />
              <td data-spalte={t("beteiligungen.land")}>{landName(s.country)}</td>
              <td data-spalte={t("beteiligungen.ziel")}>{ziel(s)}</td>
              <td data-spalte={t("beteiligungen.phase")}>
                {t("beteiligungen.phase_von", {
                  phase: t(`startup.phase.${s.phase ?? ""}`),
                  nummer: formatZahl(s.phase_number),
                  alle: formatZahl(s.phases),
                })}
              </td>
              <td data-spalte={t("beteiligungen.finanzierung")}>{finanzierung(s)}</td>
              <td className="zahl" data-spalte={t("beteiligungen.chance")}>
                {chanceText(s, geschaetzt)}
              </td>
              <td data-spalte={t("beteiligungen.dein_anteil")}>{eigenerTeil(s)}</td>
              <td>
                <button
                  type="button"
                  onClick={() => onWahl(s.id)}
                  aria-label={t("beteiligungen.handeln_fuer", { name: s.name })}
                >
                  {t("beteiligungen.handeln")}
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function Beendete({ liste }: { liste: StartUp[] }) {
  if (liste.length === 0) return <p className="gedaempft">{t("beteiligungen.keine_beendeten")}</p>;
  return (
    <div className="tabelle">
      <table className="mobil-karten" aria-label={t("beteiligungen.beendet")}>
        <thead>
          <tr>
            <th>{t("beteiligungen.name")}</th>
            <th>{t("beteiligungen.land")}</th>
            <th>{t("beteiligungen.ziel")}</th>
            <th>{t("beteiligungen.ergebnis")}</th>
            <th>{t("beteiligungen.ende")}</th>
          </tr>
        </thead>
        <tbody>
          {liste.map((s) => (
            <tr key={s.id} className={s.status === "erfolg" ? "eigen" : undefined}>
              <Name s={s} />
              <td data-spalte={t("beteiligungen.land")}>{landName(s.country)}</td>
              <td data-spalte={t("beteiligungen.ziel")}>{ziel(s)}</td>
              <td data-spalte={t("beteiligungen.ergebnis")}>
                <span title={t(`beteiligungen.status_hilfe.${s.status}`)}>
                  {t(`beteiligungen.status.${s.status}`)}
                </span>
                {s.exit && (
                  <span className="gedaempft">
                    {" "}
                    {s.exit_company
                      ? t(`beteiligungen.ausgang.${s.exit}`, { firma: s.exit_company })
                      : t(`beteiligungen.ausgang.${s.exit}_ohne`)}
                  </span>
                )}
              </td>
              <td data-spalte={t("beteiligungen.ende")}>{s.ended ? formatDatum(s.ended) : "–"}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/** A form with one amount of money. */
function BetragsForm({
  ort,
  titel,
  knopf,
  hilfe,
  max,
  befehl,
  erfolg,
  feld = t("beteiligungen.betrag"),
}: {
  ort: string;
  titel: string;
  knopf: string;
  hilfe: string;
  max: number | null;
  befehl: (usd: number) => Befehl;
  erfolg: (usd: number) => string;
  feld?: string;
}) {
  const [betrag, setBetrag] = useState("");
  const [fehler, setFehler] = useState<string | null>(null);
  const { los, antwort } = useAktion(ort);
  return (
    <form
      className="karte"
      aria-label={titel}
      onSubmit={(e) => {
        e.preventDefault();
        const b = zahlLesen(betrag);
        if (b === null || b <= 0) {
          setFehler(t("feld.keine_zahl"));
          return;
        }
        setFehler(null);
        const usd = ausAnzeige(b);
        void los([befehl(usd)], erfolg(usd)).then((ok) => ok && setBetrag(""));
      }}
    >
      <h4>{titel}</h4>
      <p className="feld-hilfe">{hilfe}</p>
      <div className="formular-zeile">
        <ZahlFeld
          name={feld}
          einheit={geldEinheit()}
          wert={betrag}
          onWert={setBetrag}
          hilfe={
            max !== null ? t("beteiligungen.hoechstens", { betrag: formatGeld(max) }) : undefined
          }
        />
        <button type="submit">{knopf}</button>
      </div>
      {fehler && <p className="fehlertext">{fehler}</p>}
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

function Verkauf({ s, d }: { s: StartUp; d: StartUps }) {
  const [prozent, setProzent] = useState("");
  const [fehler, setFehler] = useState<string | null>(null);
  const { los, antwort } = useAktion("verkauf");
  const titel = t("beteiligungen.verkaufen");
  return (
    <form
      className="karte"
      aria-label={titel}
      onSubmit={(e) => {
        e.preventDefault();
        const p = zahlLesen(prozent);
        if (p === null || p <= 0 || p > 100) {
          setFehler(t("feld.keine_zahl"));
          return;
        }
        setFehler(null);
        const share = (s.own_share * p) / 100;
        void los(
          [{ SellVentureStake: { venture: s.id, share } }],
          t("beteiligungen.verkauft", {
            anteil: formatProzent(share),
            betrag: formatGeld(s.sale_value_usd * share),
          }),
        ).then((ok) => ok && setProzent(""));
      }}
    >
      <h4>{titel}</h4>
      <p className="feld-hilfe">
        {t("beteiligungen.verkaufen_hilfe", {
          abschlag: formatProzent(d.sale_discount),
          wert: formatGeld(s.sale_value_usd * s.own_share),
        })}
      </p>
      <div className="formular-zeile">
        <ZahlFeld
          name={t("beteiligungen.anteil_deines")}
          einheit="%"
          wert={prozent}
          onWert={setProzent}
        />
        <button type="submit">{t("beteiligungen.verkaufen_knopf")}</button>
      </div>
      {fehler && <p className="fehlertext">{fehler}</p>}
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

/** All of the player's shares offered to the companies from a minimum price (ZA4). */
function AnFirmen({ s, d }: { s: StartUp; d: StartUps }) {
  const titel = t("beteiligungen.an_firmen");
  const { los, antwort } = useAktion("an_firmen");
  const datum = formatDatum(d.offers_settle);
  if (s.own_offer_usd !== null) {
    return (
      <div className="karte" role="group" aria-label={titel}>
        <h4>{titel}</h4>
        <p>{t("beteiligungen.angeboten", { preis: formatGeld(s.own_offer_usd), datum })}</p>
        <button
          type="button"
          onClick={() =>
            void los(
              [{ OfferVentureStake: { venture: s.id, minimum: null } }],
              t("beteiligungen.zurueckgezogen"),
            )
          }
        >
          {t("beteiligungen.zurueckziehen")}
        </button>
        <Rueckmeldung meldung={antwort} />
      </div>
    );
  }
  return (
    <BetragsForm
      ort="an_firmen"
      titel={titel}
      knopf={t("beteiligungen.anbieten_knopf")}
      feld={t("beteiligungen.mindestpreis")}
      hilfe={t("beteiligungen.an_firmen_hilfe", {
        anteil: formatProzent(s.own_share),
        wert: formatGeld(s.own_value_usd),
        aufschlag: formatProzent(d.company_premium_max),
        datum,
      })}
      max={null}
      befehl={(usd) => ({ OfferVentureStake: { venture: s.id, minimum: geld(usd) } })}
      erfolg={(usd) => t("beteiligungen.angeboten_meldung", { preis: formatGeld(usd), datum })}
    />
  );
}

function Lenkung({ s, d }: { s: StartUp; d: StartUps }) {
  const { los, antwort } = useAktion("lenkung");
  const titel = t("beteiligungen.lenken");
  return (
    <div className="karte" role="group" aria-label={titel}>
      <h4>{titel}</h4>
      <p className="feld-hilfe">{t("beteiligungen.lenken_hilfe")}</p>
      <label>
        {t("beteiligungen.tempo")}
        <select
          value={s.pace}
          onChange={(e) => {
            const tempo = TEMPI[e.target.value];
            if (tempo) {
              void los(
                [{ SteerVenture: { venture: s.id, pace: tempo } }],
                t("beteiligungen.gelenkt", { tempo: t(`startup.lenkung.${e.target.value}`) }),
              );
            }
          }}
        >
          {d.paces.map((p) => (
            <option key={p} value={p}>
              {t(`startup.lenkung.${p}`)}
            </option>
          ))}
        </select>
      </label>
      <Rueckmeldung meldung={antwort} />
    </div>
  );
}

function Eingliedern({ s }: { s: StartUp }) {
  const { los, antwort } = useAktion("eingliedern");
  const titel = t("beteiligungen.eingliedern");
  return (
    <div className="karte" role="group" aria-label={titel}>
      <h4>{titel}</h4>
      <p className="feld-hilfe">
        {t("beteiligungen.eingliedern_hilfe", { preis: formatGeld(s.integration_usd ?? 0) })}
      </p>
      <button
        type="button"
        onClick={() =>
          void los(
            [{ IntegrateVenture: { venture: s.id } }],
            t("beteiligungen.eingegliedert", { name: s.name }),
          )
        }
      >
        {t("beteiligungen.eingliedern_knopf", { preis: formatGeld(s.integration_usd ?? 0) })}
      </button>
      <Rueckmeldung meldung={antwort} />
    </div>
  );
}

/** One start-up with what the player can do about it. */
function Detail({ s, d, onZurueck }: { s: StartUp; d: StartUps; onZurueck: () => void }) {
  const runde = s.invest_mode === "runde";
  return (
    <section aria-label={s.name}>
      <button type="button" className="schlicht" onClick={onZurueck}>
        {t("beteiligungen.zurueck")}
      </button>
      <h3>
        {s.name} – {ziel(s)}
      </h3>
      <dl className="werte">
        <dt>{t("beteiligungen.land")}</dt>
        <dd>{landName(s.country)}</dd>
        <dt>{t("beteiligungen.phase")}</dt>
        <dd>
          {t("beteiligungen.phase_von", {
            phase: t(`startup.phase.${s.phase ?? ""}`),
            nummer: formatZahl(s.phase_number),
            alle: formatZahl(s.phases),
          })}
          {" · "}
          {finanzierung(s)}
        </dd>
        <dt>{t("beteiligungen.chance")}</dt>
        <dd>
          {chanceText(s, d.estimated)}
          {s.expected_return !== null &&
            ` · ${t("beteiligungen.ertrag", { faktor: formatZahl(s.expected_return, 1) })}`}
        </dd>
        <dt>{t("beteiligungen.wert")}</dt>
        <dd>
          {t("beteiligungen.wert_text", {
            heute: formatGeld(s.value_usd),
            erfolg: formatGeld(s.success_value_usd),
          })}
        </dd>
        <dt>{t("beteiligungen.eigner")}</dt>
        <dd>{eigner(s)}</dd>
        <dt>{t("beteiligungen.dein_anteil")}</dt>
        <dd>
          {eigenerTeil(s)}
          {s.own_book_usd > 0 &&
            ` · ${t("beteiligungen.buchwert", { betrag: formatGeld(s.own_book_usd) })}`}
          {s.own_grants_usd > 0 &&
            ` · ${t("beteiligungen.gefoerdert", { betrag: formatGeld(s.own_grants_usd) })}`}
        </dd>
      </dl>
      <p className="feld-hilfe">
        {t("beteiligungen.rechte", {
          sperr: formatProzent(d.blocking),
          mehr: formatProzent(d.majority),
        })}
      </p>
      {s.origin && <p>{t("beteiligungen.herkunft", { firma: s.origin })}</p>}
      {s.parent && (
        <p className="gedaempft">{t("beteiligungen.fremde_tochter", { firma: s.parent })}</p>
      )}
      {s.subsidiary && <p>{t("beteiligungen.tochter_hilfe")}</p>}
      <div className="karten-raster">
        {s.invest_mode && (
          <BetragsForm
            ort="beteiligen"
            titel={runde ? t("beteiligungen.zusagen") : t("beteiligungen.aufstocken")}
            knopf={runde ? t("beteiligungen.zusagen_knopf") : t("beteiligungen.kaufen_knopf")}
            hilfe={
              runde
                ? t("beteiligungen.zusagen_hilfe")
                : t("beteiligungen.aufstocken_hilfe", { aufschlag: formatProzent(d.buy_premium) })
            }
            max={s.invest_max_usd}
            befehl={(usd) => ({ InvestInVenture: { venture: s.id, amount: geld(usd) } })}
            erfolg={(usd) =>
              t("beteiligungen.beteiligt", { betrag: formatGeld(usd), name: s.name })
            }
          />
        )}
        {s.status === "aktiv" && (
          <BetragsForm
            ort="foerdern"
            titel={t("beteiligungen.foerdern")}
            knopf={t("beteiligungen.foerdern_knopf")}
            hilfe={t("beteiligungen.foerdern_hilfe", { wirkung: formatProzent(d.grant_effect) })}
            max={null}
            befehl={(usd) => ({ GrantVenture: { venture: s.id, amount: geld(usd) } })}
            erfolg={(usd) => t("beteiligungen.gefoerdert_meldung", { betrag: formatGeld(usd) })}
          />
        )}
        {s.own_share > 0 && s.status === "aktiv" && <Verkauf s={s} d={d} />}
        {s.own_share > 0 && s.status === "aktiv" && <AnFirmen s={s} d={d} />}
        {s.majority && s.status === "aktiv" && <Lenkung s={s} d={d} />}
        {s.integration_usd !== null && <Eingliedern s={s} />}
        {s.blocked && <p className="gedaempft">{t("beteiligungen.gesperrt")}</p>}
      </div>
    </section>
  );
}

/** A target as words: the technology, or the product with its level. */
function projektZiel(p: Ausgruendung): string {
  return p.kind === "technologie"
    ? t(`technologie.${p.target}`)
    : t("beteiligungen.ziel_verbesserung", {
        produkt: t(`produkt.${p.target}`),
        stufe: formatZahl(p.level ?? 0),
      });
}

/** One research project and, where possible, the form to spin it off. */
function Projekt({ p, d }: { p: Ausgruendung; d: StartUps }) {
  const [prozent, setProzent] = useState("40");
  const [fehler, setFehler] = useState<string | null>(null);
  const { los, antwort } = useAktion(`ausgruenden-${p.site}`);
  const ziel = projektZiel(p);
  const titel = t("beteiligungen.projekt", { ziel });
  return (
    <form
      className="karte"
      aria-label={titel}
      onSubmit={(e) => {
        e.preventDefault();
        const wert = zahlLesen(prozent);
        if (wert === null || wert < 0 || wert >= 100) {
          setFehler(t("beteiligungen.verkauf_ungueltig"));
          return;
        }
        setFehler(null);
        void los(
          [{ SpinOff: { site: p.site, sell: wert / 100 } }],
          t("beteiligungen.ausgegruendet", { ziel }),
        );
      }}
    >
      <h4>{titel}</h4>
      <p>
        {t("beteiligungen.projekt_stand", {
          land: landName(p.country),
          fortschritt: formatProzent(p.progress),
        })}
      </p>
      {p.reason ? (
        <p className="gedaempft">
          {t(`beteiligungen.grund.${p.reason}`, { min: formatProzent(d.spin_off_min) })}
        </p>
      ) : (
        <>
          <p className="feld-hilfe">
            {t("beteiligungen.start_als", {
              phase: t(`startup.phase.${p.phase ?? ""}`),
              wert: formatGeld(p.value_usd ?? 0),
              alles: formatGeld(p.sale_value_usd ?? 0),
            })}
          </p>
          {p.months !== null && (
            <p className="feld-hilfe">
              {p.lead !== null
                ? t("beteiligungen.dauer_vorlauf", {
                    monate: formatZahl(p.months),
                    jahre: formatZahl(p.lead),
                  })
                : t("beteiligungen.dauer", { monate: formatZahl(p.months) })}
            </p>
          )}
          {p.lead !== null && p.months !== null && p.lead * 12 < p.months && (
            <p className="warntext">{t("beteiligungen.zu_spaet")}</p>
          )}
          {p.rivals > 0 && (
            <p className="warntext">
              {t("beteiligungen.rivalen", { anzahl: formatZahl(p.rivals) })}
            </p>
          )}
          <div className="formular-zeile">
            <ZahlFeld
              name={t("beteiligungen.verkauf_anteil")}
              einheit="%"
              wert={prozent}
              onWert={setProzent}
            />
            <button type="submit">{t("beteiligungen.ausgruenden_knopf")}</button>
          </div>
        </>
      )}
      {fehler && <p className="fehlertext">{fehler}</p>}
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

/** The player's research projects that could become start-ups (SU3). */
function Ausgruenden({ d }: { d: StartUps }) {
  if (d.spin_offs.length === 0) {
    return <p className="gedaempft">{t("beteiligungen.keine_projekte")}</p>;
  }
  return (
    <>
      <p>
        {t("beteiligungen.ausgruenden_einleitung", {
          mehr: formatProzent(d.majority),
          min: formatProzent(d.spin_off_min),
        })}
      </p>
      <div className="karten-raster">
        {d.spin_offs.map((p) => (
          <Projekt key={p.site} p={p} d={d} />
        ))}
      </div>
    </>
  );
}

function Inhalt({
  d,
  kern,
  onGeaendert,
  neu,
}: {
  d: StartUps;
  kern: Kern;
  onGeaendert: (u: Uebersicht) => void;
  neu: () => void;
}) {
  const [reiter, setReiter] = useState<Reiter>("laufend");
  const [gewaehlt, setGewaehlt] = useState<number | null>(null);
  const { senden, meldung } = useBefehl(kern, onGeaendert, neu);
  const titel = d.label ? t(`startup.bezeichnung.${d.label}`) : t("ansicht.beteiligungen");
  const eigene = d.active.filter((s) => s.own_share > 0 || s.own_pledge_usd > 0 || s.subsidiary);
  const auswahl = gewaehlt === null ? null : (d.active.find((s) => s.id === gewaehlt) ?? null);
  return (
    <Befehle senden={senden} meldung={meldung}>
      <h2>{titel}</h2>
      <p>{t("beteiligungen.einleitung")}</p>
      <p className="gedaempft">
        {d.per_year > 0
          ? t("beteiligungen.zahlen", {
              je_jahr: formatZahl(d.per_year),
              gegruendet: formatZahl(d.founded),
              erfolg: formatZahl(d.succeeded),
              gescheitert: formatZahl(d.failed),
              jahre: formatZahl(d.keep_years),
            })
          : t("beteiligungen.aus")}
      </p>
      <p className="feld-hilfe">
        {d.estimated ? t("beteiligungen.schaetzung_genau") : t("beteiligungen.schaetzung_grob")}
      </p>
      <p>
        {d.holdings > 0
          ? t("beteiligungen.portfolio", {
              anzahl: formatZahl(d.holdings),
              buch: formatGeld(d.portfolio_book_usd),
              wert: formatGeld(d.portfolio_value_usd),
            })
          : t("beteiligungen.kein_portfolio")}
      </p>
      {auswahl ? (
        <Detail s={auswahl} d={d} onZurueck={() => setGewaehlt(null)} />
      ) : (
        <>
          <Unterreiter
            name={titel}
            bereiche={[
              { key: "laufend", text: t("beteiligungen.laufend"), zaehler: d.active.length },
              { key: "eigene", text: t("beteiligungen.eigene"), zaehler: eigene.length },
              {
                key: "ausgruenden",
                text: t("beteiligungen.ausgruenden"),
                zaehler: d.spin_offs.filter((p) => p.reason === null).length,
              },
              { key: "beendet", text: t("beteiligungen.beendet") },
            ]}
            aktiv={reiter}
            onWahl={setReiter}
          />
          {reiter === "laufend" && (
            <Laufende liste={d.active} geschaetzt={d.estimated} onWahl={setGewaehlt} />
          )}
          {reiter === "eigene" &&
            (eigene.length > 0 ? (
              <Laufende liste={eigene} geschaetzt={d.estimated} onWahl={setGewaehlt} />
            ) : (
              <p className="gedaempft">{t("beteiligungen.keine_eigenen")}</p>
            ))}
          {reiter === "ausgruenden" && <Ausgruenden d={d} />}
          {reiter === "beendet" && <Beendete liste={d.closed} />}
        </>
      )}
    </Befehle>
  );
}

/** The start-ups of the world and the player's stakes (SU1, SU2). */
export function BeteiligungenAnsicht({
  kern,
  uebersicht,
  onGeaendert,
}: {
  kern: Kern;
  uebersicht: Uebersicht;
  onGeaendert: (u: Uebersicht) => void;
}) {
  const [zaehler, setZaehler] = useState(0);
  const { daten, fehler } = useSicht(() => kern.startups(), `${uebersicht.date}/${zaehler}`);
  return (
    <main className="ansicht" id="beteiligungen">
      <h1 className="unsichtbar">{t("ansicht.beteiligungen")}</h1>
      {daten ? (
        <Inhalt
          d={daten}
          kern={kern}
          onGeaendert={onGeaendert}
          neu={() => setZaehler((z) => z + 1)}
        />
      ) : (
        <FehlerText fehler={fehler} />
      )}
    </main>
  );
}
