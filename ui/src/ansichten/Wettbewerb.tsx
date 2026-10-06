// Competition (M30/M31, docs/BEDIENUNG.md): the offers to buy and sell sites, areas and
// licences, and the companies with what they own. "Who wants what of mine, and what
// could I buy?"
import { useState, type FormEvent } from "react";
import {
  ausAnzeige,
  formatDatum,
  formatGeld,
  formatProzent,
  formatZahl,
  geldEinheit,
  geldFeld,
  geldSchluessel,
  inAnzeige,
  landName,
  zahlLesen,
} from "../format";
import {
  geld,
  type Angebot,
  type Befehl,
  type Geschaeftsbereich,
  type Firmendetail,
  type Gegenstandssicht,
  type Kern,
  type Standortwert,
  type Uebersicht,
} from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import {
  Befehle,
  Erklaerung,
  Rueckmeldung,
  Unterreiter,
  ZahlFeld,
  useAktion,
  useBefehl,
  useSicht,
} from "./gemeinsam";

type Reiter = "angebote" | "firmen";

/** "Werk in Deutschland (Nägel, Draht)", "Bereich Metallwaren (…)" or "Lizenz auf …". */
export function gegenstandText(g: Gegenstandssicht): string {
  if (g.kind === "lizenz")
    return t("wettbewerb.lizenz_auf", { technologie: t(`technologie.${g.technology}`) });
  if (g.kind === "bereich")
    return t("wettbewerb.bereich_mit", {
      gruppe: t(`warengruppe.${g.group}`),
      anzahl: formatZahl(g.site_count),
      produkte: g.products.map((p) => t(`produkt.${p}`)).join(", "),
    });
  const art = t(g.site_type ?? "standorttyp.werk");
  const land = landName(g.country ?? "");
  const produkte = g.products.map((p) => t(`produkt.${p}`)).join(", ");
  return produkte
    ? t("wettbewerb.standort_mit", { art, land, produkte })
    : t("wettbewerb.standort", { art, land });
}

/** The parts of a site's base value (docs/FORMELN.md, M30), or an area's (M31). */
function Wertteile({ w, bereich }: { w: Standortwert; bereich: boolean }) {
  if (bereich)
    return (
      <dl className="rechnung">
        <dt>{t("wettbewerb.standorte_summe")}</dt>
        <dd>{formatGeld(w.base_usd - w.brand_usd)}</dd>
        <dt>{t("wettbewerb.markenwert")}</dt>
        <dd>{formatGeld(w.brand_usd)}</dd>
        <dt className="summe">{t("wettbewerb.grundwert")}</dt>
        <dd className="summe">{formatGeld(w.base_usd)}</dd>
        <dt>{t("wettbewerb.buchwert")}</dt>
        <dd>{formatGeld(w.book_usd)}</dd>
      </dl>
    );
  return (
    <dl className="rechnung">
      <dt>
        {w.result_year_usd === null
          ? t("wettbewerb.ertrag_fehlt")
          : t("wettbewerb.ertragswert", {
              ergebnis: formatGeld(w.result_year_usd),
              jahre: formatZahl(w.earnings_years, 1),
            })}
      </dt>
      <dd>{formatGeld(w.earnings_value_usd)}</dd>
      <dt>{t("wettbewerb.restwert")}</dt>
      <dd>{formatGeld(w.liquidation_usd)}</dd>
      {w.under_construction_usd > 0 && (
        <>
          <dt>{t("wettbewerb.im_bau")}</dt>
          <dd>{formatGeld(w.under_construction_usd)}</dd>
        </>
      )}
      <dt>{t("wettbewerb.lager")}</dt>
      <dd>{formatGeld(w.inventory_usd)}</dd>
      {w.land_usd > 0 && (
        <>
          <dt>{t("wettbewerb.grundstueck")}</dt>
          <dd>{formatGeld(w.land_usd)}</dd>
        </>
      )}
      <dt className="summe">{t("wettbewerb.grundwert")}</dt>
      <dd className="summe">{formatGeld(w.base_usd)}</dd>
      <dt>{t("wettbewerb.buchwert")}</dt>
      <dd>{formatGeld(w.book_usd)}</dd>
    </dl>
  );
}

/** Base value with its explanation. */
function Grundwert({ w, bereich = false }: { w: Standortwert; bereich?: boolean }) {
  return (
    <span className="mit-erklaerung">
      {formatGeld(w.base_usd)}
      <Erklaerung wert={t("wettbewerb.grundwert")}>
        <p>{t(bereich ? "wettbewerb.grundwert_bereich_hilfe" : "wettbewerb.grundwert_hilfe")}</p>
        <Wertteile w={w} bereich={bereich} />
      </Erklaerung>
    </span>
  );
}

/** A suggested price, in the shown currency to three significant digits. */
function vorschlagRunden(usd: number): number {
  const wert = inAnzeige(usd);
  return wert > 0 ? ausAnzeige(Number(wert.toPrecision(3))) : 0;
}

/** A price field in the shown currency with a button; sends `befehl(price)`. */
function Preisformular({
  ort,
  vorschlag,
  knopf,
  erfolg,
  befehl,
}: {
  ort: string;
  vorschlag: number;
  knopf: string;
  erfolg: string;
  befehl: (preis: number) => Befehl;
}) {
  const { los, antwort } = useAktion(ort);
  const [preis, setPreis] = useState(geldFeld(vorschlagRunden(vorschlag)));
  const [fehler, setFehler] = useState<string | null>(null);
  const absenden = (ev: FormEvent) => {
    ev.preventDefault();
    const p = zahlLesen(preis);
    if (p === null || p <= 0) {
      setFehler(t("wettbewerb.preis_fehlt"));
      return;
    }
    setFehler(null);
    void los([befehl(geld(ausAnzeige(p)))], erfolg);
  };
  return (
    <form className="formular-zeile" onSubmit={absenden}>
      <ZahlFeld
        name={t("wettbewerb.preis")}
        einheit={geldEinheit()}
        wert={preis}
        onWert={setPreis}
        breit
      />
      <button type="submit">{knopf}</button>
      {fehler && <p className="fehlertext">{fehler}</p>}
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

function AngebotKarte({ a }: { a: Angebot }) {
  const { los, antwort } = useAktion(`angebot/${a.id}`);
  const kaeufer = a.role === "kaeufer";
  const titel = a.answer
    ? t(a.counter ? "wettbewerb.gegenangebot_von" : "wettbewerb.angebot_von", { firma: a.company })
    : t(kaeufer ? "wettbewerb.dein_angebot_an" : "wettbewerb.dein_gegenangebot_an", {
        firma: a.company,
      });
  const antworten = (antwort: "Accept" | "Decline", erfolg: string) =>
    void los([{ AnswerOffer: { offer: a.id, answer: antwort } }], erfolg);
  return (
    <article
      className="karte angebot"
      aria-label={titel}
      data-tour={a.answer ? "angebot-antwort" : undefined}
    >
      <h3>{titel}</h3>
      <p>{gegenstandText(a.object)}</p>
      <dl className="kennzahlen kompakt">
        <div className="kennzahl">
          <dt>{t("wettbewerb.preis")}</dt>
          <dd>{formatGeld(a.price_usd)}</dd>
        </div>
        {a.value && (
          <div className="kennzahl">
            <dt>{t("wettbewerb.grundwert")}</dt>
            <dd>
              <Grundwert w={a.value} bereich={a.object.kind === "bereich"} />
            </dd>
          </div>
        )}
        {a.license_value_usd !== null && (
          <div className="kennzahl">
            <dt>{t("wettbewerb.lizenzwert")}</dt>
            <dd>{formatGeld(a.license_value_usd)}</dd>
          </div>
        )}
        <div className="kennzahl">
          <dt>{t("wettbewerb.frist")}</dt>
          <dd>{formatDatum(a.deadline)}</dd>
        </div>
      </dl>
      {a.answer && (
        <>
          <p className="gedaempft">
            {t(kaeufer ? "wettbewerb.hilfe_kauf" : "wettbewerb.hilfe_verkauf")}
          </p>
          <div className="knopfreihe">
            <button
              type="button"
              className="haupt"
              onClick={() => antworten("Accept", t("wettbewerb.angenommen"))}
            >
              {t("wettbewerb.annehmen")}
            </button>
            <button type="button" onClick={() => antworten("Decline", t("wettbewerb.abgelehnt"))}>
              {t("wettbewerb.ablehnen")}
            </button>
          </div>
          {a.can_counter && (
            <Preisformular
              key={`${a.id}/${geldSchluessel()}`}
              ort={`gegenangebot/${a.id}`}
              vorschlag={Math.max(a.price_usd, a.value?.base_usd ?? 0) * 1.2}
              knopf={t("wettbewerb.gegenangebot")}
              erfolg={t("wettbewerb.gegenangebot_gesendet")}
              befehl={(price) => ({ AnswerOffer: { offer: a.id, answer: { Counter: { price } } } })}
            />
          )}
        </>
      )}
      {!a.answer && a.status === "offen" && (
        <p className="gedaempft">
          {t("wettbewerb.wartet", { datum: formatDatum(a.deadline) })}{" "}
          {a.can_withdraw && (
            <button
              type="button"
              className="schlicht"
              onClick={() =>
                void los([{ WithdrawOffer: { offer: a.id } }], t("wettbewerb.zurueckgezogen"))
              }
            >
              {t("wettbewerb.zurueckziehen")}
            </button>
          )}
        </p>
      )}
      <Rueckmeldung meldung={antwort} />
    </article>
  );
}

function AngeboteListe({ kern, stand }: { kern: Kern; stand: string }) {
  const { daten, fehler } = useSicht(() => kern.angebote(), stand);
  if (!daten) return <FehlerText fehler={fehler} />;
  const offen = daten.offers.filter((a) => a.status === "offen");
  const fertig = daten.offers.filter((a) => a.status !== "offen");
  return (
    <>
      {offen.length === 0 ? (
        <p>{t("wettbewerb.keine_angebote")}</p>
      ) : (
        <div className="karten">
          {offen.map((a) => (
            <AngebotKarte key={a.id} a={a} />
          ))}
        </div>
      )}
      <p className="gedaempft">{t("wettbewerb.angebote_hilfe")}</p>
      {fertig.length > 0 && (
        <section aria-labelledby="angebote-fertig">
          <h3 id="angebote-fertig">{t("wettbewerb.abgeschlossen")}</h3>
          <ul className="meldungen">
            {fertig.map((a) => (
              <li key={a.id}>
                {t(`wettbewerb.status.${a.status}`, {
                  firma: a.company,
                  gegenstand: gegenstandText(a.object),
                  preis: formatGeld(a.price_usd),
                  datum: formatDatum(a.closed ?? a.date),
                })}
              </li>
            ))}
          </ul>
        </section>
      )}
    </>
  );
}

function FirmenListe({
  kern,
  stand,
  onFirma,
}: {
  kern: Kern;
  stand: string;
  onFirma: (index: number) => void;
}) {
  const { daten, fehler } = useSicht(() => kern.firmen(), stand);
  if (!daten) return <FehlerText fehler={fehler} />;
  return (
    <div className="tabelle">
      <table className="mobil-karten" aria-label={t("wettbewerb.firmen")}>
        <thead>
          <tr>
            <th>{t("uebersicht.firma")}</th>
            <th>{t("uebersicht.sitz")}</th>
            <th className="zahl">{t("uebersicht.eigenkapital")}</th>
            <th className="zahl">{t("wettbewerb.umsatz")}</th>
            <th className="zahl">{t("wettbewerb.standorte")}</th>
          </tr>
        </thead>
        <tbody>
          {daten.companies.map((c) => (
            <tr key={c.index} className={c.player ? "eigene-zeile" : undefined}>
              <td>
                {c.player ? (
                  <strong>{c.name}</strong>
                ) : (
                  <button type="button" className="schlicht" onClick={() => onFirma(c.index)}>
                    {c.name}
                  </button>
                )}{" "}
                {c.real && <span className="marke">{t("uebersicht.real")}</span>}
              </td>
              <td data-spalte={t("uebersicht.sitz")}>{landName(c.headquarters)}</td>
              <td className="zahl" data-spalte={t("uebersicht.eigenkapital")}>
                {formatGeld(c.equity_usd)}
              </td>
              <td className="zahl" data-spalte={t("wettbewerb.umsatz")}>
                {formatGeld(c.revenue_year_usd)}
              </td>
              <td className="zahl" data-spalte={t("wettbewerb.standorte")}>
                {formatZahl(c.sites)}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/** An area of another company: its sites, brand and value, with a price form. */
function BereichKarte({ b, d }: { b: Geschaeftsbereich; d: Firmendetail }) {
  const gruppe = t(`warengruppe.${b.group}`);
  const standorte = b.sites
    .map((id) => d.sites.find((s) => s.site === id))
    .filter((s) => s !== undefined)
    .map((s) => t("wettbewerb.standort", { art: t(s.site_type), land: landName(s.country) }));
  return (
    <article className="karte" aria-label={t("wettbewerb.bereich", { gruppe })}>
      <h4>{gruppe}</h4>
      <p>
        {t("wettbewerb.bereich_standorte", {
          anzahl: formatZahl(b.sites.length),
          liste: standorte.join(", "),
        })}
      </p>
      <p className="gedaempft">
        {t("wettbewerb.marke")}:{" "}
        {b.brand.length === 0
          ? t("wettbewerb.marke_keine")
          : b.brand
              .map(([land, anteil]) => `${landName(land)} ${formatProzent(anteil)}`)
              .join(", ")}
      </p>
      <p>
        {t("wettbewerb.grundwert")}: <Grundwert w={b.value} bereich />
      </p>
      <p>
        {t("wettbewerb.neubau")}: {formatGeld(b.new_build_usd)}
      </p>
      {b.blocked === null ? (
        <Preisformular
          key={`${b.group}/${geldSchluessel()}`}
          ort={`bereich/${d.company.index}/${b.group}`}
          vorschlag={b.value.base_usd * 1.1}
          knopf={t("wettbewerb.anbieten")}
          erfolg={t("wettbewerb.angebot_gesendet")}
          befehl={(price) => ({
            MakeOffer: { seller: d.company.index, object: { Area: b.group }, price },
          })}
        />
      ) : (
        <p className="gedaempft">
          {t(`wettbewerb.gesperrt.${b.blocked}`, {
            datum: b.blocked_until ? formatDatum(b.blocked_until) : "",
          })}
        </p>
      )}
    </article>
  );
}

function FirmaDetail({ d, onZurueck }: { d: Firmendetail; onZurueck: () => void }) {
  const c = d.company;
  return (
    <section aria-labelledby="firma-titel">
      <button type="button" className="schlicht" onClick={onZurueck}>
        ← {t("wettbewerb.alle_firmen")}
      </button>
      <h2 id="firma-titel">{c.name}</h2>
      <p className="gedaempft">
        {t("wettbewerb.firma_kopf", {
          land: landName(c.headquarters),
          eigenkapital: formatGeld(c.equity_usd),
          umsatz: formatGeld(c.revenue_year_usd),
        })}
      </p>
      <h3>{t("wettbewerb.standorte")}</h3>
      <p className="gedaempft">
        {t("wettbewerb.kaufen_hilfe")}
        {d.min_age_months > 0 && ` ${t("wettbewerb.mindestalter", { monate: d.min_age_months })}`}
      </p>
      <div className="karten">
        {d.sites.map((s) => (
          <article
            key={s.site}
            className="karte"
            aria-label={t("wettbewerb.standort", {
              art: t(s.site_type),
              land: landName(s.country),
            })}
          >
            <h4>
              {t(s.site_type)} · {landName(s.country)}
            </h4>
            {s.products.length > 0 && <p>{s.products.map((p) => t(`produkt.${p}`)).join(", ")}</p>}
            <p className="gedaempft">
              {s.facilities.map(([f, n]) => `${n} × ${t(`anlage.${f}`)}`).join(", ")}
              {" · "}
              {t("wettbewerb.beschaeftigte", { anzahl: formatZahl(s.workers) })}
            </p>
            <p>
              {t("wettbewerb.grundwert")}: <Grundwert w={s.value} />
            </p>
            <p>
              {t("wettbewerb.neubau")}: {formatGeld(s.new_build_usd)}
            </p>
            {s.needed && <p className="gedaempft">{t("wettbewerb.selbst_gebraucht")}</p>}
            {s.blocked === null ? (
              <Preisformular
                key={`${s.site}/${geldSchluessel()}`}
                ort={`kauf/${s.site}`}
                vorschlag={
                  s.needed
                    ? Math.max(s.value.base_usd * 1.1, s.new_build_usd)
                    : s.value.base_usd * 1.1
                }
                knopf={t("wettbewerb.anbieten")}
                erfolg={t("wettbewerb.angebot_gesendet")}
                befehl={(price) => ({
                  MakeOffer: { seller: c.index, object: { Site: s.site }, price },
                })}
              />
            ) : (
              <p className="gedaempft">
                {t(`wettbewerb.gesperrt.${s.blocked}`, {
                  datum: s.blocked_until ? formatDatum(s.blocked_until) : "",
                })}
              </p>
            )}
          </article>
        ))}
      </div>
      {d.areas.length > 0 && (
        <>
          <h3>{t("wettbewerb.bereiche")}</h3>
          <p className="gedaempft">{t("wettbewerb.bereiche_hilfe")}</p>
          <div className="karten">
            {d.areas.map((b) => (
              <BereichKarte key={b.group} b={b} d={d} />
            ))}
          </div>
        </>
      )}
      <h3>{t("wettbewerb.lizenzen")}</h3>
      {d.licenses.length === 0 ? (
        <p className="gedaempft">{t("wettbewerb.keine_lizenzen")}</p>
      ) : (
        <div className="karten">
          {d.licenses.map((l) => (
            <article
              key={l.technology}
              className="karte"
              aria-label={t(`technologie.${l.technology}`)}
            >
              <h4>{t(`technologie.${l.technology}`)}</h4>
              <p>
                {t("wettbewerb.lizenzwert")}: {formatGeld(l.value_usd)}
              </p>
              {l.open_offer !== null ? (
                <p className="gedaempft">{t("wettbewerb.gesperrt.angebot_offen")}</p>
              ) : l.blocked_until ? (
                <p className="gedaempft">
                  {t("wettbewerb.gesperrt.gesperrt", { datum: formatDatum(l.blocked_until) })}
                </p>
              ) : (
                <Preisformular
                  key={`${l.technology}/${geldSchluessel()}`}
                  ort={`lizenz/${l.technology}`}
                  vorschlag={l.value_usd * 0.5}
                  knopf={t("wettbewerb.lizenz_anfragen")}
                  erfolg={t("wettbewerb.angebot_gesendet")}
                  befehl={(price) => ({
                    MakeOffer: { seller: c.index, object: { License: l.technology }, price },
                  })}
                />
              )}
            </article>
          ))}
        </div>
      )}
    </section>
  );
}

function FirmaLaden({
  kern,
  stand,
  index,
  onZurueck,
}: {
  kern: Kern;
  stand: string;
  index: number;
  onZurueck: () => void;
}) {
  const { daten, fehler } = useSicht(() => kern.firma(index), `${stand}/${index}`);
  if (!daten) return <FehlerText fehler={fehler} />;
  return <FirmaDetail d={daten} onZurueck={onZurueck} />;
}

export function WettbewerbAnsicht({
  kern,
  uebersicht,
  onGeaendert,
  offene,
}: {
  kern: Kern;
  uebersicht: Uebersicht;
  onGeaendert: (u: Uebersicht) => void;
  /** Offers waiting for the player's answer (the counter on the sub-tab). */
  offene: number;
}) {
  const [reiter, setReiter] = useState<Reiter>("angebote");
  const [firma, setFirma] = useState<number | null>(null);
  // Every command (offer, answer) reloads the lists; the overview keeps the counters.
  const [zaehler, setZaehler] = useState(0);
  const { senden, meldung } = useBefehl(kern, onGeaendert, () => setZaehler((z) => z + 1));
  const stand = `${uebersicht.date}/${zaehler}`;
  return (
    <main className="ansicht" id="wettbewerb">
      <h1 className="unsichtbar">{t("ansicht.wettbewerb")}</h1>
      <Unterreiter
        name={t("ansicht.wettbewerb")}
        bereiche={[
          { key: "angebote", text: t("wettbewerb.angebote"), zaehler: offene },
          { key: "firmen", text: t("wettbewerb.firmen") },
        ]}
        aktiv={reiter}
        onWahl={(r) => {
          setReiter(r);
          setFirma(null);
        }}
      />
      <Befehle senden={senden} meldung={meldung}>
        {reiter === "angebote" && <AngeboteListe kern={kern} stand={stand} />}
        {reiter === "firmen" &&
          (firma === null ? (
            <FirmenListe kern={kern} stand={stand} onFirma={setFirma} />
          ) : (
            <FirmaLaden kern={kern} stand={stand} index={firma} onZurueck={() => setFirma(null)} />
          ))}
      </Befehle>
    </main>
  );
}
