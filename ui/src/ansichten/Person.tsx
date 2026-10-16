import { Fragment, useState } from "react";
import {
  ausAnzeige,
  formatDatum,
  formatGeld,
  formatProzent,
  formatZahl,
  geldEinheit,
  geldFeld,
  landName,
  meldungText,
  zahlLesen,
} from "../format";
import {
  geld,
  STUFEN,
  type Geldanlage,
  type Kern,
  type Nachfolge,
  type Person,
  type PersonAnteil,
  type PersonGeld,
  type PersonKind,
  type Uebersicht,
} from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { Befehle, Rueckmeldung, useAktion, useBefehl, useSicht, ZahlFeld } from "./gemeinsam";
import { Verlauf } from "./Grafik";

/** The relationship, waiting events and the family tree (PV6b). */
function Leben({ l }: { l: NonNullable<Person["life"]> }) {
  const { los, antwort } = useAktion("ereignis");
  const senden = (event: number, option: number) => void los([{ AnswerEvent: { event, option } }]);
  return (
    <section className="karte" aria-label={t("person.leben")}>
      <h2>{t("person.leben")}</h2>
      {l.bond !== null && (
        <p>
          {t("person.beziehung", { wert: formatZahl(l.bond) })}
          {l.contract ? ` – ${t("person.ehevertrag_da")}` : ""}
        </p>
      )}
      {l.events.length === 0 ? (
        <p className="gedaempft">{t("person.ereignisse_keine")}</p>
      ) : (
        <ul className="liste">
          {l.events.map((e) => (
            <li key={e.id}>
              <strong>{t(`lebensereignis.${e.key}`)}</strong>{" "}
              <span className="gedaempft">
                {t("person.ereignis_frist", { bis: formatDatum(e.until) })}
              </span>
              <div className="knopfreihe links">
                {e.options.map((o, i) => (
                  <button key={o.key} type="button" onClick={() => senden(e.id, i)}>
                    {t(`lebensereignis.${e.key}.${o.key}`)}
                    {o.cost_usd > 0 ? ` (${formatGeld(o.cost_usd)})` : ""}
                  </button>
                ))}
              </div>
            </li>
          ))}
        </ul>
      )}
      <h3>{t("person.stammbaum")}</h3>
      <ul className="liste" aria-label={t("person.stammbaum")}>
        {l.tree.map((n, i) => (
          <li key={`${n.name}-${i}`} style={{ marginLeft: `${(n.generation + 3) * 1.2}rem` }}>
            {t("person.stammbaum_zeile", {
              name: n.name,
              rolle: t(`person.rolle_${n.role}`),
              geboren: formatDatum(n.born),
            })}
            {n.died ? ` – ${t("person.gestorben_am", { datum: formatDatum(n.died) })}` : ""}
            {n.position ? ` – ${meldungText(n.position)}` : ""}
          </li>
        ))}
      </ul>
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

const ENTWICKLUNG = {
  wohnung: "Apartment",
  mehrfamilienhaus: "Multi",
  buero: "Office",
  halle: "Hall",
} as const;

/** Securities loan and collectors' market (PV7a). */
function Wertpapierkredit({ s }: { s: NonNullable<Person["speculation"]> }) {
  const { los, antwort } = useAktion("wertpapierkredit");
  const [betragText, setBetrag] = useState("");
  const senden = (action: Geldanlage) => void los([{ Invest: { action } }]);
  const betrag = zahlLesen(betragText);
  const wert = betrag === null ? null : geld(ausAnzeige(betrag));
  return (
    <section className="karte" aria-label={t("person.wertpapierkredit")}>
      <h2>{t("person.wertpapierkredit")}</h2>
      <p className="feld-hilfe">
        {t("person.wertpapierkredit_hilfe", {
          fonds: formatProzent(s.lending[0]),
          gold: formatProzent(s.lending[1]),
          sammler: formatProzent(s.lending[2]),
        })}
      </p>
      <dl className="werte">
        <dt>{t("person.wertpapier_schuld")}</dt>
        <dd>
          {formatGeld(s.debt_usd)} ({formatProzent(s.rate)})
        </dd>
        <dt>{t("person.wertpapier_deckung")}</dt>
        <dd>
          {formatGeld(s.capacity_usd)} –{" "}
          {t("person.wertpapier_frei", { frei: formatGeld(s.room_usd) })}
        </dd>
        <dt>{t("person.sammlermarkt")}</dt>
        <dd>{formatProzent(s.collector_level - 1)}</dd>
      </dl>
      {s.call_since && (
        <p className="warnung" role="alert">
          {t("person.nachschuss", {
            seit: formatDatum(s.call_since),
            monate: formatZahl(s.call_months),
          })}
        </p>
      )}
      <div className="formular-zeile">
        <ZahlFeld
          name={t("person.wertpapier_betrag")}
          einheit={geldEinheit()}
          wert={betragText}
          onWert={setBetrag}
        />
        <button
          type="button"
          disabled={wert === null || betrag === null || betrag <= 0}
          onClick={() => wert !== null && senden({ BorrowSecurities: { amount: wert } })}
        >
          {t("person.wertpapier_aufnehmen")}
        </button>
        <button
          type="button"
          disabled={wert === null || betrag === null || betrag <= 0 || s.debt_usd <= 0}
          onClick={() => wert !== null && senden({ RepaySecurities: { amount: wert } })}
        >
          {t("person.wertpapier_tilgen")}
        </button>
      </div>
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

/** Family holding and foundation (PV6c). */
function Bindung({ b }: { b: NonNullable<Person["trust"]> }) {
  const { los, antwort } = useAktion("bindung");
  const [einlage, setEinlage] = useState("");
  const senden = (action: Geldanlage) => void los([{ Invest: { action } }]);
  const betrag = zahlLesen(einlage);
  return (
    <section className="karte" aria-label={t("person.bindung")}>
      <h2>{t("person.bindung")}</h2>
      <p className="feld-hilfe">{t("person.bindung_hilfe")}</p>
      {b.holding ? (
        <p>{t("person.holding_besteht")}</p>
      ) : (
        <button type="button" onClick={() => senden("FoundHolding")}>
          {t("person.holding_gruenden", { kosten: formatGeld(b.holding_cost_usd) })}
        </button>
      )}
      {b.claims.length > 0 && (
        <ul className="liste">
          {b.claims.map(([firma, anteil]) => (
            <li key={firma}>
              {t("person.holding_anspruch", { firma, anteil: formatProzent(anteil) })}
            </li>
          ))}
        </ul>
      )}
      <dl className="werte">
        <dt>{t("person.stiftung_kapital")}</dt>
        <dd>
          {formatGeld(b.capital_usd)} (
          {t("person.stiftung_ertrag", {
            ertrag: formatProzent(b.yield_year),
            ausschuettung: formatGeld(b.payout_year_usd),
            ansehen: formatZahl(b.standing_points),
          })}
          )
        </dd>
      </dl>
      <div className="formular-zeile">
        <ZahlFeld
          name={t("person.stiftung_einlage")}
          einheit={geldEinheit()}
          wert={einlage}
          onWert={setEinlage}
        />
        <button
          type="button"
          disabled={betrag === null || betrag <= 0}
          onClick={() =>
            betrag !== null && senden({ EndowFoundation: { amount: geld(ausAnzeige(betrag)) } })
          }
        >
          {t("person.stiftung_einzahlen", {
            mindest: formatGeld(b.foundation_min_usd),
            steuer: formatProzent(b.gift_tax),
          })}
        </button>
      </div>
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

/** Standing, clubs, contacts and possessions (PV6a). */
function Ansehen({ k }: { k: NonNullable<Person["circle"]> }) {
  const { los, antwort } = useAktion("ansehen");
  const [spende, setSpende] = useState("");
  const senden = (action: Geldanlage) => void los([{ Invest: { action } }]);
  const betrag = zahlLesen(spende);
  return (
    <section className="karte" aria-label={t("person.ansehen")}>
      <h2>{t("person.ansehen")}</h2>
      <p className="feld-hilfe">{t("person.ansehen_hilfe")}</p>
      <dl className="werte">
        <dt>{t("person.ansehen_jetzt")}</dt>
        <dd>
          {formatZahl(k.standing)} / 100 ({t("person.ansehen_ziel", { ziel: formatZahl(k.target) })}
          )
        </dd>
        <dt>{t("person.ansehen_kredit")}</dt>
        <dd>{formatProzent(k.loan_offset)}</dd>
        <dt>{t("person.ansehen_makler")}</dt>
        <dd>{formatProzent(k.broker_discount)}</dd>
      </dl>
      <div className="formular-zeile">
        <ZahlFeld
          name={t("person.spende_betrag")}
          einheit={geldEinheit()}
          wert={spende}
          onWert={setSpende}
        />
        <button
          type="button"
          disabled={betrag === null || betrag <= 0}
          onClick={() =>
            betrag !== null && senden({ Donate: { amount: geld(ausAnzeige(betrag)) } })
          }
        >
          {t("person.spenden")}
        </button>
      </div>
      <h3>{t("person.clubs")}</h3>
      <table className="tabelle" aria-label={t("person.clubs")}>
        <tbody>
          {k.clubs.map((c, i) => (
            <tr key={c.key}>
              <th scope="row">{t(`club.${c.key}`)}</th>
              <td>
                {t("person.club_bedingung", {
                  ansehen: formatZahl(c.standing_min),
                  stufe: t(
                    `person.lebensstil_${["bescheiden", "buergerlich", "gehoben", "luxurioes"][c.lifestyle_min] ?? "bescheiden"}`,
                  ),
                  beitrag: formatGeld(c.fee_usd),
                })}
              </td>
              <td>
                {c.member ? (
                  <button type="button" onClick={() => senden({ LeaveClub: { club: i } })}>
                    {t("person.club_verlassen")}
                  </button>
                ) : (
                  <button
                    type="button"
                    disabled={!c.open}
                    onClick={() => senden({ JoinClub: { club: i } })}
                  >
                    {t("person.club_beitreten")}
                  </button>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <h3>{t("person.kontakte")}</h3>
      {k.contacts.length === 0 ? (
        <p className="gedaempft">{t("person.kontakte_keine")}</p>
      ) : (
        <ul className="liste">
          {k.contacts.map((c) => (
            <li key={`${c.name}-${c.role}`}>
              {t("person.kontakt_zeile", {
                name: c.name,
                rolle: t(`rolle.${c.role}`),
                staerke: formatZahl(c.strength),
                bis: formatDatum(c.until),
              })}
            </li>
          ))}
        </ul>
      )}
      <h3>{t("person.besitz")}</h3>
      <table className="tabelle" aria-label={t("person.besitz")}>
        <tbody>
          {k.goods.map((g, i) => (
            <tr key={g.key}>
              <th scope="row">{t(`besitz.${g.key}`)}</th>
              <td>
                {t("person.besitz_zeile", {
                  preis: formatGeld(g.price_usd),
                  unterhalt: formatProzent(g.upkeep),
                  punkte: formatZahl(g.representation),
                })}
              </td>
              <td>
                <button
                  type="button"
                  disabled={!g.available}
                  onClick={() => senden({ BuyGood: { good: i } })}
                >
                  {t("person.besitz_kaufen")}
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      {k.owned.length > 0 && (
        <ul className="liste">
          {k.owned.map((o, i) => (
            <li key={`${o.key}-${i}`}>
              {t(`besitz.${o.key}`)} – {formatGeld(o.value_usd)}{" "}
              <button type="button" onClick={() => senden({ SellGood: { index: i } })}>
                {t("person.besitz_verkaufen")}
              </button>
            </li>
          ))}
        </ul>
      )}
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

/** Where a child stands as a manager: not yet, free, employed, retired or dead. */
function kindStatus(k: PersonKind): string {
  if (k.died) return t("person.kind_verstorben", { datum: formatDatum(k.died) });
  if (k.position) return meldungText(k.position);
  if (k.manager !== null) return t("person.kind_frei");
  if (k.card_in !== null) return t("person.kind_karte_in", { jahre: formatZahl(k.card_in) });
  return t("person.kind_im_ruhestand");
}

const AUSBILDUNGEN = {
  ingenieur: "Engineer",
  kaufmann: "Merchant",
  jurist: "Lawyer",
  naturwissenschaftler: "Scientist",
  frei: "Free",
} as const;

function Familie({ p }: { p: Person }) {
  const { los, antwort } = useAktion("ausbildung");
  return (
    <section aria-label={t("person.familie")}>
      <h2>{t("person.familie")}</h2>
      {p.children.length === 0 ? (
        <p>{t("person.keine_kinder")}</p>
      ) : (
        <table className="tabelle">
          <thead>
            <tr>
              <th>{t("person.kind")}</th>
              <th>{t("person.geboren")}</th>
              <th className="zahl">{t("person.kind_alter")}</th>
              <th>{t("person.kind_status")}</th>
              <th className="zahl">{t("person.kind_ehrgeiz")}</th>
              <th className="zahl">{t("person.kind_interesse")}</th>
              <th>{t("person.kind_ausbildung")}</th>
            </tr>
          </thead>
          <tbody>
            {p.children.map((k, i) => (
              <tr key={`${k.name}-${k.born}`}>
                <td>{k.name}</td>
                <td data-spalte={t("person.geboren")}>{formatDatum(k.born)}</td>
                <td className="zahl" data-spalte={t("person.kind_alter")}>
                  {formatZahl(k.age)}
                </td>
                <td data-spalte={t("person.kind_status")}>{kindStatus(k)}</td>
                <td className="zahl" data-spalte={t("person.kind_ehrgeiz")}>
                  {k.ambition ? formatZahl(k.ambition) : "–"}
                </td>
                <td className="zahl" data-spalte={t("person.kind_interesse")}>
                  {k.interest ? formatZahl(k.interest) : "–"}
                </td>
                <td data-spalte={t("person.kind_ausbildung")}>
                  {k.can_study ? (
                    <select
                      aria-label={t("person.kind_ausbildung_fuer", { name: k.name })}
                      value={k.education ?? ""}
                      onChange={(e) => {
                        const wahl = AUSBILDUNGEN[e.target.value as keyof typeof AUSBILDUNGEN];
                        if (wahl)
                          void los(
                            [{ SetEducation: { child: i, education: wahl } }],
                            t("person.ausbildung_gesetzt"),
                          );
                      }}
                    >
                      <option value="">{t("person.ausbildung_keine")}</option>
                      {Object.keys(AUSBILDUNGEN).map((a) => (
                        <option key={a} value={a}>
                          {t(`person.ausbildung_${a}`)}
                        </option>
                      ))}
                    </select>
                  ) : k.education ? (
                    t(`person.ausbildung_${k.education}`)
                  ) : (
                    "–"
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      {p.children_possible && <p className="feld-hilfe">{t("person.kinder_moeglich")}</p>}
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

/** Partner search, private schools and health care: buttons, costs a month. */
const FONDS = { gemanagt: "Managed", index: "Index" } as const;

/** Safe investments, gold and funds (PV2). */
function Anlagen({ a }: { a: NonNullable<Person["assets"]> }) {
  const { los, antwort } = useAktion("anlagen");
  const [betrag, setBetrag] = useState("");
  const [monate, setMonate] = useState("12");
  const [anteil, setAnteil] = useState("100");
  const wert = (text: string) => {
    const b = zahlLesen(text);
    return b !== null && b > 0 ? geld(ausAnzeige(b)) : null;
  };
  const senden = (action: Geldanlage) => void los([{ Invest: { action } }]);
  const monateZahl = Math.round(zahlLesen(monate) ?? 0);
  const teil = Math.min(1, Math.max(0, (zahlLesen(anteil) ?? 0) / 100));
  const mit = (f: (b: number) => Geldanlage) => {
    const b = wert(betrag);
    if (b !== null) senden(f(b));
  };
  return (
    <section className="karte" aria-label={t("person.anlagen")}>
      <h2>{t("person.anlagen")}</h2>
      <p className="feld-hilfe">{t("person.anlagen_hilfe")}</p>
      <div className="formular-zeile">
        <ZahlFeld
          name={t("person.anlage_betrag")}
          einheit={geldEinheit()}
          wert={betrag}
          onWert={setBetrag}
        />
        <ZahlFeld name={t("person.anlage_monate")} einheit="" wert={monate} onWert={setMonate} />
        <ZahlFeld name={t("person.anlage_anteil")} einheit="%" wert={anteil} onWert={setAnteil} />
      </div>
      <h3>{t("person.festgeld")}</h3>
      <p className="feld-hilfe">
        {t("person.festgeld_hilfe", {
          zins: formatProzent(a.deposit_rate),
          von: String(a.deposit_months[0]),
          bis: String(a.deposit_months[1]),
          abschlag: formatProzent(a.deposit_penalty),
        })}
      </p>
      <button
        type="button"
        onClick={() => mit((b) => ({ OpenDeposit: { amount: b, months: monateZahl } }))}
      >
        {t("person.festgeld_anlegen")}
      </button>
      {a.deposits.length > 0 && (
        <ul className="liste">
          {a.deposits.map((d, i) => (
            <li key={`${d.until}-${i}`}>
              {t("person.festgeld_zeile", {
                betrag: formatGeld(d.principal_usd),
                zins: formatProzent(d.rate),
                bis: formatDatum(d.until),
              })}{" "}
              <button type="button" onClick={() => senden({ BreakDeposit: { index: i } })}>
                {t("person.kuendigen")}
              </button>
            </li>
          ))}
        </ul>
      )}
      <h3>{t("person.staatsanleihen")}</h3>
      <p className="feld-hilfe">
        {t("person.anleihen_hilfe", {
          kupon: formatProzent(a.bond_coupon),
          von: String(a.bond_months[0]),
          bis: String(a.bond_months[1]),
          spanne: formatProzent(a.bond_spread),
        })}
      </p>
      <button
        type="button"
        onClick={() => mit((b) => ({ BuyBond: { face: b, months: monateZahl } }))}
      >
        {t("person.anleihe_kaufen")}
      </button>
      {a.bonds.length > 0 && (
        <ul className="liste">
          {a.bonds.map((b, i) => (
            <li key={`${b.until}-${i}`}>
              {t("person.anleihe_zeile", {
                nennwert: formatGeld(b.face_usd),
                kupon: formatProzent(b.coupon),
                bis: formatDatum(b.until),
                kurs: formatProzent(b.quote),
                wert: formatGeld(b.value_usd),
              })}{" "}
              <button type="button" onClick={() => senden({ SellBond: { index: i } })}>
                {t("person.verkaufen")}
              </button>
            </li>
          ))}
        </ul>
      )}
      <h3>{t("person.gold")}</h3>
      <p className="feld-hilfe">
        {t("person.gold_hilfe", {
          preis: formatGeld(a.gold_price_usd),
          gebuehr: formatProzent(a.gold_fee),
          spanne: formatProzent(a.gold_spread),
        })}
      </p>
      <p>
        {t("person.gold_bestand", {
          unzen: formatZahl(a.gold_ounces),
          wert: formatGeld(a.gold_value_usd),
        })}{" "}
        <button type="button" onClick={() => mit((b) => ({ BuyGold: { amount: b } }))}>
          {t("person.gold_kaufen")}
        </button>{" "}
        <button
          type="button"
          disabled={a.gold_ounces <= 0}
          onClick={() => senden({ SellGold: { share: teil } })}
        >
          {t("person.gold_verkaufen")}
        </button>
      </p>
      <h3>{t("person.fonds")}</h3>
      {a.funds.map((f) => (
        <div key={f.key}>
          <p>
            <strong>{t(`person.fonds_${f.key}`)}</strong>{" "}
            {f.open
              ? t("person.fonds_zeile", {
                  anteile: formatZahl(f.units),
                  wert: formatGeld(f.value_usd),
                  kosten: formatProzent(f.running_cost),
                  aufschlag: formatProzent(f.load + f.fee),
                })
              : t("person.fonds_ab", { jahr: String(f.from) })}
          </p>
          {f.open && (
            <p>
              <button
                type="button"
                onClick={() =>
                  mit((b) => ({ BuyFund: { fund: FONDS[f.key as keyof typeof FONDS], amount: b } }))
                }
              >
                {t("person.fonds_kaufen")}
              </button>{" "}
              <button
                type="button"
                disabled={f.units <= 0}
                onClick={() =>
                  senden({ SellFund: { fund: FONDS[f.key as keyof typeof FONDS], share: teil } })
                }
              >
                {t("person.fonds_verkaufen")}
              </button>{" "}
              <button
                type="button"
                onClick={() =>
                  senden({
                    SetPlan: {
                      fund: FONDS[f.key as keyof typeof FONDS],
                      amount: wert(betrag) ?? 0,
                    },
                  })
                }
              >
                {t("person.sparplan_setzen")}
              </button>
            </p>
          )}
        </div>
      ))}
      <p className="feld-hilfe">
        {a.plan
          ? t("person.sparplan_aktiv", {
              fonds: t(`person.fonds_${a.plan[0]}`),
              betrag: formatGeld(a.plan[1]),
            })
          : t("person.sparplan_keiner")}
      </p>
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

/** Private real estate: offers, own properties, rent and renovation (PV3). */
function Immobilien({ p }: { p: NonNullable<NonNullable<Person["assets"]>["property"]> }) {
  const { los, antwort } = useAktion("immobilien");
  const [hypothek, setHypothek] = useState("60");
  const [miete, setMiete] = useState("100");
  const senden = (action: Geldanlage) => void los([{ Invest: { action } }]);
  const anteil = Math.min(1 - p.equity_min, Math.max(0, (zahlLesen(hypothek) ?? 0) / 100));
  const faktor = (zahlLesen(miete) ?? 100) / 100;
  return (
    <section className="karte" aria-label={t("person.immobilien")}>
      <h2>{t("person.immobilien")}</h2>
      <p className="feld-hilfe">
        {t("person.immobilien_hilfe", {
          eigenkapital: formatProzent(p.equity_min),
          zins: formatProzent(p.mortgage_rate),
          nebenkosten: formatProzent(p.costs),
          band_von: formatProzent(p.rent_band[0]),
          band_bis: formatProzent(p.rent_band[1]),
        })}
      </p>
      <div className="formular-zeile">
        <ZahlFeld
          name={t("person.hypothek_anteil")}
          einheit="%"
          wert={hypothek}
          onWert={setHypothek}
        />
        <ZahlFeld name={t("person.miete_anteil")} einheit="%" wert={miete} onWert={setMiete} />
      </div>
      <h3>{t("person.immobilien_bestand")}</h3>
      {p.owned.length === 0 ? (
        <p className="gedaempft">{t("person.immobilien_keine")}</p>
      ) : (
        <ul className="liste">
          {p.owned.map((o, i) => (
            <li key={`${o.kind}-${o.country}-${i}`}>
              {t("person.immobilie_zeile", {
                art: t(`person.immobilie_${o.kind}`),
                land: landName(o.country),
                wert: formatGeld(o.value_usd),
                schulden: formatGeld(o.debt_usd),
                miete: formatGeld(o.market_rent_usd * o.rent_factor),
                leerstand: formatProzent(o.vacancy),
                zustand: formatZahl(o.condition),
              })}
              {o.renovating_until
                ? ` – ${t("person.saniert_bis", { datum: formatDatum(o.renovating_until) })}`
                : ""}{" "}
              <button
                type="button"
                onClick={() => senden({ SetRent: { index: i, factor: faktor } })}
              >
                {t("person.miete_setzen")}
              </button>{" "}
              {o.kind !== "grundstueck" && p.sites.length > 0 && (
                <select
                  aria-label={t("person.mieter")}
                  value={o.tenant ?? ""}
                  onChange={(e) =>
                    senden({
                      AssignProperty: {
                        index: i,
                        site: e.target.value === "" ? null : Number(e.target.value),
                      },
                    })
                  }
                >
                  <option value="">{t("person.mieter_keiner")}</option>
                  {p.sites.map((s) => (
                    <option key={s.index} value={s.index}>
                      {s.name ?? `${t("person.mieter_standort")} ${landName(s.country)}`}
                    </option>
                  ))}
                </select>
              )}{" "}
              {o.developing && (
                <span>
                  {t("person.entwicklung_laeuft", {
                    art: t(`person.immobilie_${o.developing.kind}`),
                    datum: formatDatum(o.developing.until),
                  })}{" "}
                </span>
              )}
              {o.kind === "grundstueck" && !o.developing && p.develop && (
                <span>
                  {(["wohnung", "mehrfamilienhaus", "buero", "halle"] as const).map((art) => (
                    <button
                      key={art}
                      type="button"
                      onClick={() => senden({ Develop: { index: i, kind: ENTWICKLUNG[art] } })}
                    >
                      {t("person.entwickeln", {
                        art: t(`person.immobilie_${art}`),
                        kosten: formatProzent(p.develop?.[0] ?? 0),
                        monate: formatZahl(p.develop?.[1] ?? 0),
                      })}
                    </button>
                  ))}{" "}
                </span>
              )}
              {o.kind !== "grundstueck" && (
                <button type="button" onClick={() => senden({ Renovate: { index: i } })}>
                  {t("person.sanieren", { kosten: formatProzent(p.renovation_cost) })}
                </button>
              )}{" "}
              <button type="button" onClick={() => senden({ SellProperty: { index: i } })}>
                {t("person.verkaufen")}
              </button>
            </li>
          ))}
        </ul>
      )}
      <h3>{t("person.immobilien_angebote")}</h3>
      <table className="tabelle" aria-label={t("person.immobilien_angebote")}>
        <tbody>
          {p.offers.map((o) => (
            <tr key={o.id}>
              <th scope="row">
                {t(`person.immobilie_${o.kind}`)} · {landName(o.country)}
              </th>
              <td className="zahl">{formatGeld(o.price_usd)}</td>
              <td className="zahl">
                {t("person.angebot_miete", { miete: formatGeld(o.market_rent_usd) })}
              </td>
              <td className="zahl">
                {t("person.angebot_zustand", { zustand: formatZahl(o.condition) })}
              </td>
              <td>
                <button
                  type="button"
                  onClick={() => senden({ BuyProperty: { offer: o.id, mortgage: anteil } })}
                >
                  {t("person.immobilie_kaufen")}
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

/** Background, time budget and health (PV1b). */
function Wohlbefinden({ w }: { w: NonNullable<Person["wellbeing"]> }) {
  const { los, antwort } = useAktion("zeit");
  const [zeit, setZeit] = useState<[string, string, string, string]>(
    () => w.time.map(String) as [string, string, string, string],
  );
  const werte: number[] = zeit.map((z) => Number(z));
  const summe = werte.reduce((a, b) => a + b, 0);
  const namen = ["firma", "familie", "netzwerk", "erholung"] as const;
  return (
    <section className="karte" aria-label={t("person.zeit_gesundheit")}>
      <h2>{t("person.zeit_gesundheit")}</h2>
      {w.background && (
        <p>
          <strong>{t(`person.hintergrund_${w.background}`)}</strong>
          {" – "}
          {t(`person.hintergrund_hilfe_${w.background}`)}
        </p>
      )}
      <dl className="werte">
        <dt>{t("person.gesundheit")}</dt>
        <dd>
          {formatZahl(w.health)} / 100
          {w.ill_until ? ` – ${t("person.krank_bis", { datum: formatDatum(w.ill_until) })}` : ""}
        </dd>
        <dt>{t("person.sterblichkeit_gesundheit")}</dt>
        <dd>{formatProzent(w.mortality_factor)}</dd>
        <dt>{t("person.mindest_firma")}</dt>
        <dd>{formatZahl(w.min_firm)} %</dd>
      </dl>
      <p className="feld-hilfe">{t("person.zeit_hilfe")}</p>
      <div className="feldreihe">
        {namen.map((n, i) => (
          <label key={n}>
            {t(`person.zeit_${n}`)} (%)
            <input
              type="number"
              min={0}
              max={100}
              value={zeit[i]}
              onChange={(e) => {
                const neu = [...zeit] as [string, string, string, string];
                neu[i] = e.target.value;
                setZeit(neu);
              }}
            />
          </label>
        ))}
      </div>
      {w.delay_days > 0 && (
        <p className="fehlertext">{t("person.zeit_knapp", { tage: formatZahl(w.delay_days) })}</p>
      )}
      <button
        type="button"
        disabled={summe !== 100 || werte.some((x) => !Number.isInteger(x) || x < 0)}
        onClick={() =>
          void los([
            {
              SetTimeBudget: {
                firm: werte[0] ?? 0,
                family: werte[1] ?? 0,
                network: werte[2] ?? 0,
                rest: werte[3] ?? 0,
              },
            },
          ])
        }
      >
        {t("person.zeit_speichern")}
      </button>
      {summe !== 100 && (
        <span className="fehlertext"> {t("person.zeit_summe", { summe: formatZahl(summe) })}</span>
      )}
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

function Privatleben({ p, pl }: { p: Person; pl: NonNullable<Person["private_life"]> }) {
  const { los, antwort } = useAktion("privatleben");
  const schulfaehig = p.children
    .map((k, i) => ({ k, i }))
    .filter(({ k }) => !k.died && k.card_in !== null);
  return (
    <section className="karte" aria-label={t("person.privatleben")}>
      <h2>{t("person.privatleben")}</h2>
      <p className="feld-hilfe">{t("person.privatleben_hilfe")}</p>
      {!p.married && (
        <p>
          {pl.seeking_partner
            ? t("person.partner_sucht", {
                kosten: formatGeld(pl.partner_cost_usd),
                chance: formatProzent(pl.partner_chance),
              })
            : t("person.partner_hilfe", {
                kosten: formatGeld(pl.partner_cost_usd),
                chance: formatProzent(pl.partner_chance),
              })}{" "}
          <button
            type="button"
            onClick={() => void los([{ SeekPartner: { active: !pl.seeking_partner } }])}
          >
            {t(pl.seeking_partner ? "person.partner_beenden" : "person.partner_suchen")}
          </button>
        </p>
      )}
      <h3>{t("person.vorsorge")}</h3>
      <p>
        {t("person.vorsorge_hilfe", {
          kosten: formatGeld(pl.care_cost_usd),
          faktor: formatProzent(pl.care_mortality),
        })}{" "}
        <button
          type="button"
          onClick={() => void los([{ SetHealthCare: { active: !pl.health_care } }])}
        >
          {t(pl.health_care ? "person.vorsorge_aus" : "person.vorsorge_an")}
        </button>
      </p>
      {schulfaehig.length > 0 && (
        <>
          <h3>{t("person.privatschule")}</h3>
          <p className="feld-hilfe">
            {t("person.privatschule_hilfe", {
              kosten: formatGeld(pl.school_cost_usd),
              punkte: formatZahl(pl.school_education),
            })}
          </p>
          <ul className="liste">
            {schulfaehig.map(({ k, i }) => (
              <li key={`${k.name}-${k.born}`}>
                {k.name} ({formatZahl(k.age)})
                {k.private_school && ` – ${t("person.privatschule_besucht")}`}{" "}
                <button
                  type="button"
                  onClick={() =>
                    void los([{ SetPrivateSchool: { child: i, active: !k.private_school } }])
                  }
                >
                  {t(k.private_school ? "person.privatschule_aus" : "person.privatschule_an")}
                </button>
              </li>
            ))}
          </ul>
        </>
      )}
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

function Rollen({ p, g }: { p: Person; g?: PersonGeld | null }) {
  if (p.holdings.length === 0) {
    return (
      <section aria-label={t("person.rollen")}>
        <h2>{t("person.rollen")}</h2>
        <p>{t("person.keine_firma")}</p>
      </section>
    );
  }
  return (
    <section aria-label={t("person.rollen")}>
      <h2>{t("person.rollen")}</h2>
      <table className="tabelle">
        <thead>
          <tr>
            <th>{t("person.firma")}</th>
            <th className="zahl">{t("person.anteil")}</th>
            <th className="zahl">{t("person.wert")}</th>
            <th className="zahl">{t("person.einstand")}</th>
            <th>{t("person.rolle")}</th>
            {g && <th>{t("person.gehalt")}</th>}
          </tr>
        </thead>
        <tbody>
          {p.holdings.map((h) => (
            <tr key={h.company}>
              <td>{h.company}</td>
              <td className="zahl" data-spalte={t("person.anteil")}>
                {formatProzent(h.share)}
              </td>
              <td className="zahl" data-spalte={t("person.wert")}>
                {formatGeld(h.value_usd ?? 0)}
              </td>
              <td className="zahl" data-spalte={t("person.einstand")}>
                {formatGeld(h.cost_basis_usd ?? 0)}
              </td>
              <td data-spalte={t("person.rolle")}>
                {h.person_ceo
                  ? t("person.rolle_ceo")
                  : h.ceo
                    ? t("person.rolle_eigentuemer", { name: h.ceo })
                    : t("person.rolle_eigentuemer_ohne")}
              </td>
              {g && (
                <td data-spalte={t("person.gehalt")}>
                  <GehaltFeld h={h} />
                  <Sonderzahlung h={h} />
                </td>
              )}
            </tr>
          ))}
        </tbody>
      </table>
      {g && (
        <p className="feld-hilfe">
          {t("person.gehalt_tabelle_hilfe", { steuer: formatProzent(g.income_tax) })}
        </p>
      )}
    </section>
  );
}

/** Wealth, its course and the income and spending of the last twelve months (PE3). */
function Vermoegen({ g }: { g: PersonGeld }) {
  return (
    <section aria-label={t("person.vermoegen")}>
      <h2>{t("person.vermoegen")}</h2>
      <dl className="werte">
        <dt>{t("person.privatkonto")}</dt>
        <dd>{formatGeld(g.cash_usd)}</dd>
        <dt>{t("person.anteile")}</dt>
        <dd>{formatGeld(g.shares_usd)}</dd>
        <dt>{t("person.darlehen")}</dt>
        <dd>{formatGeld(g.loans_usd)}</dd>
        {g.classes
          .filter(([k, v]) => k === "startups" && v !== 0)
          .map(([k, v]) => (
            <Fragment key={k}>
              <dt>{t(`privat.vermoegen_${k}`)}</dt>
              <dd>{formatGeld(v)}</dd>
            </Fragment>
          ))}
        <dt>{t("person.vermoegen_gesamt")}</dt>
        <dd>
          <strong>{formatGeld(g.wealth_usd)}</strong>
        </dd>
      </dl>
      <Verlauf
        name={t("person.vermoegen")}
        monate={g.history.map(([d]) => d)}
        werte={g.history.map(([, v]) => v)}
      />
      {g.short && <p className="fehlertext">{t("person.konto_leer")}</p>}
      <h3>{t("person.einnahmen_ausgaben")}</h3>
      {g.flows.length === 0 ? (
        <p className="gedaempft">{t("person.noch_keine_bewegungen")}</p>
      ) : (
        <table className="tabelle" aria-label={t("person.einnahmen_ausgaben")}>
          <tbody>
            {g.flows.map(([k, v]) => (
              <tr key={k}>
                <th scope="row">{t(k)}</th>
                <td className={`zahl${v < 0 ? " negativ" : ""}`}>{formatGeld(v)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

/** The four levels with what each costs and does; changing takes effect next month. */
function Lebensstil({ g }: { g: PersonGeld }) {
  const { los, antwort } = useAktion("lebensstil");
  const wirkung = (wert: number, art: "zins" | "anteil" | "punkte" | "faktor") => {
    if (art === "zins")
      return wert === 0 ? "±0" : `${wert > 0 ? "+" : "−"}${formatZahl(Math.abs(wert) * 100)} Pp.`;
    if (art === "anteil")
      return wert === 0 ? "±0" : `${wert > 0 ? "+" : "−"}${formatProzent(Math.abs(wert))}`;
    if (art === "punkte") return wert === 0 ? "±0" : `+${formatZahl(wert)}`;
    return `× ${formatZahl(wert)}`;
  };
  return (
    <section aria-label={t("person.lebensstil")}>
      <h2>{t("person.lebensstil")}</h2>
      <p className="feld-hilfe">
        {t("person.lebensstil_hilfe", { ab: formatDatum(g.lifestyle_change_from) })}
      </p>
      <table className="tabelle">
        <thead>
          <tr>
            <th>{t("person.stufe")}</th>
            <th className="zahl">{t("person.kosten_monat")}</th>
            <th className="zahl">{t("person.wirkung_zins")}</th>
            <th className="zahl">{t("person.wirkung_gehalt")}</th>
            <th className="zahl">{t("person.wirkung_ausbildung")}</th>
            <th className="zahl">{t("person.wirkung_sterblichkeit")}</th>
            <th />
          </tr>
        </thead>
        <tbody>
          {g.lifestyles.map((l) => {
            const aktuell = l.key === g.lifestyle;
            const geplant = l.key === g.lifestyle_next;
            return (
              <tr key={l.key} aria-current={aktuell ? "true" : undefined}>
                <th scope="row">{t(`lebensstil.${l.key}`)}</th>
                <td className="zahl" data-spalte={t("person.kosten_monat")}>
                  {formatGeld(l.cost_usd)}
                </td>
                <td className="zahl" data-spalte={t("person.wirkung_zins")}>
                  {wirkung(l.interest, "zins")}
                </td>
                <td className="zahl" data-spalte={t("person.wirkung_gehalt")}>
                  {wirkung(l.salary_demand, "anteil")}
                </td>
                <td className="zahl" data-spalte={t("person.wirkung_ausbildung")}>
                  {wirkung(l.education, "punkte")}
                </td>
                <td className="zahl" data-spalte={t("person.wirkung_sterblichkeit")}>
                  {wirkung(l.mortality, "faktor")}
                </td>
                <td>
                  {aktuell ? (
                    <span className="marke">{t("person.lebensstil_aktuell")}</span>
                  ) : geplant ? (
                    <span className="marke">{t("person.lebensstil_geplant")}</span>
                  ) : (
                    <button
                      type="button"
                      onClick={() =>
                        void los(
                          [{ SetLifestyle: { level: STUFEN[l.key] ?? "Middle" } }],
                          t("person.lebensstil_gewaehlt", { stufe: t(`lebensstil.${l.key}`) }),
                        )
                      }
                    >
                      {t("person.lebensstil_waehlen")}
                    </button>
                  )}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

/** The person's salary from one company, in its row of the roles (PE3). */
function GehaltFeld({ h }: { h: PersonAnteil }) {
  const [betrag, setBetrag] = useState(geldFeld(h.salary_usd ?? 0));
  const { los, antwort } = useAktion(`gehalt-${h.index ?? 0}`);
  if (!h.salary_possible) return <>–</>;
  const hilfe = [
    t("person.gehalt_vorschlag", { vorschlag: formatGeld(h.salary_suggestion_usd ?? 0) }),
    h.salary_max_usd != null
      ? t("person.gehalt_hoechstens", { max: formatGeld(h.salary_max_usd) })
      : "",
  ]
    .filter(Boolean)
    .join(" ");
  return (
    <form
      className="formular-zeile"
      aria-label={t("person.gehalt_fuer", { firma: h.company })}
      onSubmit={(e) => {
        e.preventDefault();
        const b = zahlLesen(betrag);
        if (b === null || b < 0) return;
        void los(
          [{ SetPersonSalary: { company: h.index ?? 0, amount: geld(ausAnzeige(b)) } }],
          t("person.gehalt_gesetzt", { betrag: formatGeld(ausAnzeige(b)), firma: h.company }),
        );
      }}
    >
      <ZahlFeld
        name={t("person.gehalt_jahr")}
        einheit={geldEinheit()}
        wert={betrag}
        onWert={setBetrag}
        hilfe={hilfe}
      />
      <button type="submit">{t("person.gehalt_setzen")}</button>
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

/** A one-off payment of the company to the person, taxed as income. */
function Sonderzahlung({ h }: { h: PersonAnteil }) {
  const [betrag, setBetrag] = useState("");
  const { los, antwort } = useAktion(`sonderzahlung-${h.index ?? 0}`);
  if (!h.salary_possible) return null;
  return (
    <form
      className="formular-zeile"
      aria-label={t("person.sonderzahlung_fuer", { firma: h.company })}
      onSubmit={(e) => {
        e.preventDefault();
        const b = zahlLesen(betrag);
        if (b === null || b <= 0) return;
        void los(
          [{ PayBonus: { company: h.index ?? 0, amount: geld(ausAnzeige(b)) } }],
          t("person.sonderzahlung_gezahlt", {
            betrag: formatGeld(ausAnzeige(b)),
            firma: h.company,
          }),
        ).then((ok) => ok && setBetrag(""));
      }}
    >
      <ZahlFeld
        name={t("person.sonderzahlung")}
        einheit={geldEinheit()}
        wert={betrag}
        onWert={setBetrag}
        hilfe={t("person.sonderzahlung_hilfe", { kasse: formatGeld(h.cash_usd ?? 0) })}
      />
      <button type="submit">{t("person.sonderzahlung_auszahlen")}</button>
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

/** Heir, estate and the hand-over while alive (PE6). */
/** The will: shares in percent; the statutory succession while none is set (PV4). */
function Testament({ n }: { n: Nachfolge }) {
  const { los, antwort } = useAktion("testament");
  const zeilen = n.shares ?? [];
  const [prozent, setProzent] = useState<Record<string, string>>({});
  const schluessel = (z: { partner: boolean; child: number | null }) =>
    z.partner ? "partner" : String(z.child);
  const wert = (z: (typeof zeilen)[number]) =>
    prozent[schluessel(z)] ?? String(Math.round(z.percent));
  const summe = zeilen.reduce((a, z) => a + (Number(wert(z)) || 0), 0);
  return (
    <div>
      <h3>{t("person.testament")}</h3>
      <p className="feld-hilfe">
        {n.will_set ? t("person.testament_gesetzt") : t("person.testament_gesetzlich")}
      </p>
      <ul className="liste">
        {zeilen.map((z) => (
          <li key={schluessel(z)}>
            <label>
              {z.partner ? t("person.erbe_partner") : z.name} (%)
              <input
                type="number"
                min={0}
                max={100}
                value={wert(z)}
                onChange={(e) => setProzent({ ...prozent, [schluessel(z)]: e.target.value })}
              />
            </label>
          </li>
        ))}
      </ul>
      <button
        type="button"
        disabled={summe !== 100 || zeilen.length === 0}
        onClick={() =>
          void los(
            [
              {
                SetWill: {
                  shares: zeilen.map((z) => ({
                    heir: z.partner ? "Partner" : { Child: z.child ?? 0 },
                    percent: Number(wert(z)) || 0,
                  })),
                },
              },
            ],
            t("person.testament_gespeichert"),
          )
        }
      >
        {t("person.testament_speichern")}
      </button>{" "}
      {n.will_set && (
        <button
          type="button"
          onClick={() => void los([{ SetWill: { shares: [] } }], t("person.testament_zurueck"))}
        >
          {t("person.testament_gesetzlich_knopf")}
        </button>
      )}
      {summe !== 100 && (
        <span className="fehlertext"> {t("person.zeit_summe", { summe: formatZahl(summe) })}</span>
      )}
      {(n.heir_stakes ?? []).length > 0 && (
        <p className="feld-hilfe">
          {t("person.erbanteile", {
            liste: (n.heir_stakes ?? []).map(([f, q]) => `${f} ${formatProzent(q)}`).join(", "),
          })}
        </p>
      )}
      <Rueckmeldung meldung={antwort} />
    </div>
  );
}

function NachfolgeAbschnitt({ n }: { n: Nachfolge }) {
  const { los, antwort } = useAktion("nachfolge");
  const [fragen, setFragen] = useState(false);
  return (
    <section className="karte" aria-label={t("person.nachfolge")}>
      <h2>{t("person.nachfolge")}</h2>
      <p className="feld-hilfe">{t("person.nachfolge_hilfe")}</p>
      <p>
        {t("person.generation", { n: formatZahl(n.generation) })} ·{" "}
        {t("person.sterbechance", { anteil: formatProzent(n.death_chance_year) })}
      </p>
      <p>{n.heir !== null ? t("person.erbe_jetzt", { name: n.heir }) : t("person.erbe_neffe")}</p>
      <p>
        {t("person.nachlass", {
          nachlass: formatGeld(n.estate_usd),
          satz: formatProzent(n.tax_rate),
          steuer: formatGeld(n.tax_usd),
        })}
      </p>
      {n.shares_enabled && <Testament n={n} />}
      {n.choices.length > 0 && (
        <label className="feld">
          <span>{t("person.erbe_waehlen")}</span>
          <select
            value={n.chosen ?? ""}
            onChange={(e) =>
              void los(
                [{ SetHeir: { child: e.target.value === "" ? null : Number(e.target.value) } }],
                t("person.erbe_gesetzt"),
              )
            }
          >
            <option value="">{t("person.erbe_regel")}</option>
            {n.choices.map((c) => (
              <option key={c.index} value={c.index}>
                {c.name} ({formatZahl(c.age)})
              </option>
            ))}
          </select>
        </label>
      )}
      <div className="knopfreihe links">
        {fragen ? (
          <>
            <p>{t("person.uebergeben_frage")}</p>
            <button
              type="button"
              className="haupt"
              onClick={() =>
                void los([{ HandOver: {} }], t("person.uebergeben_erfolgt")).then(() =>
                  setFragen(false),
                )
              }
            >
              {t("person.uebergeben_bestaetigen")}
            </button>
            <button type="button" onClick={() => setFragen(false)}>
              {t("person.uebergeben_abbrechen")}
            </button>
          </>
        ) : (
          <button type="button" onClick={() => setFragen(true)}>
            {t("person.uebergeben")}
          </button>
        )}
      </div>
      {n.ancestors.length > 0 && (
        <>
          <h3>{t("person.vorfahren")}</h3>
          <ul>
            {n.ancestors.map((a) => (
              <li key={`${a.name}-${a.until}`}>
                {t(a.died ? "person.vorfahr_tot" : "person.vorfahr_uebergabe", {
                  name: a.name,
                  geboren: formatDatum(a.born),
                  bis: formatDatum(a.until),
                })}
                {(a.wealth_usd ?? 0) > 0 &&
                  ` ${t("person.nachruf", {
                    vermoegen: formatGeld(a.wealth_usd ?? 0),
                    firmen: String(a.firms ?? 0),
                    mitarbeiter: formatZahl(a.workers ?? 0),
                    ansehen: formatZahl(a.standing ?? 0),
                    kinder: String(a.children ?? 0),
                  })}`}
              </li>
            ))}
          </ul>
        </>
      )}
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

/** Selling shares to investors and choosing the main company (PE5). */
function Handel({ h }: { h: PersonAnteil }) {
  const [anteil, setAnteil] = useState("");
  const { los, antwort } = useAktion(`handel-${h.index ?? 0}`);
  const firma = h.index ?? 0;
  const prozent = zahlLesen(anteil);
  const erloes =
    prozent !== null && prozent > 0 && h.share > 0
      ? ((h.investors_bid_usd ?? 0) * prozent) / 100 / h.share
      : null;
  return (
    <section className="karte" aria-label={t("person.handel_fuer", { firma: h.company })}>
      <h3>{t("person.handel_fuer", { firma: h.company })}</h3>
      {h.selectable && !h.main && (
        <p>
          <button
            type="button"
            onClick={() =>
              void los([{ SelectCompany: { company: firma } }], t("person.hauptfirma_gewaehlt"))
            }
          >
            {t("person.hauptfirma_waehlen")}
          </button>
        </p>
      )}
      {h.main && <p className="feld-hilfe">{t("person.ist_hauptfirma")}</p>}
      <p className="feld-hilfe">
        {t("person.verkaufen_hilfe", {
          anteil: formatProzent(h.share),
          erloes: formatGeld(h.investors_bid_usd ?? 0),
        })}
      </p>
      <div className="formular-zeile">
        <ZahlFeld
          name={t("person.verkaufen_anteil")}
          einheit="%"
          wert={anteil}
          onWert={setAnteil}
        />
        <button
          type="button"
          onClick={() => {
            if (prozent !== null && prozent > 0)
              void los(
                [{ SellStake: { company: firma, share: Math.min(prozent / 100, h.share) } }],
                t("person.verkauft"),
              ).then((ok) => ok && setAnteil(""));
          }}
        >
          {t("person.verkaufen")}
        </button>
      </div>
      {erloes !== null && (
        <p className="feld-hilfe">{t("person.verkaufen_erloes", { betrag: formatGeld(erloes) })}</p>
      )}
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

/** Capital in, loans to and capital back from a company of the person. */
function Kapital({ h, g }: { h: PersonAnteil; g: PersonGeld }) {
  const [einlage, setEinlage] = useState("");
  const [darlehen, setDarlehen] = useState("");
  const [zins, setZins] = useState("5");
  const [jahre, setJahre] = useState("5");
  const [rueck, setRueck] = useState("");
  const { los, antwort } = useAktion(`kapital-${h.index ?? 0}`);
  const firma = h.index ?? 0;
  const betrag = (text: string) => {
    const b = zahlLesen(text);
    return b !== null && b > 0 ? geld(ausAnzeige(b)) : null;
  };
  return (
    <section className="karte" aria-label={t("person.kapital_fuer", { firma: h.company })}>
      <h3>{t("person.kapital_fuer", { firma: h.company })}</h3>
      <p className="feld-hilfe">
        {t("person.kapital_hilfe", {
          kasse: formatGeld(h.cash_usd ?? 0),
          konto: formatGeld(g.cash_usd),
        })}
      </p>
      <div className="formular-zeile">
        <ZahlFeld
          name={t("person.einlage")}
          einheit={geldEinheit()}
          wert={einlage}
          onWert={setEinlage}
        />
        <button
          type="button"
          onClick={() => {
            const b = betrag(einlage);
            if (b !== null)
              void los(
                [{ ContributeCapital: { company: firma, amount: b } }],
                t("person.eingezahlt"),
              ).then((ok) => ok && setEinlage(""));
          }}
        >
          {t("person.einzahlen")}
        </button>
      </div>
      <div className="formular-zeile">
        <ZahlFeld
          name={t("person.darlehen_betrag")}
          einheit={geldEinheit()}
          wert={darlehen}
          onWert={setDarlehen}
        />
        <ZahlFeld name={t("person.darlehen_zins")} einheit="%" wert={zins} onWert={setZins} />
        <ZahlFeld
          name={t("person.darlehen_jahre")}
          einheit={t("finanzen.jahre")}
          ganzzahlig
          wert={jahre}
          onWert={setJahre}
        />
        <button
          type="button"
          onClick={() => {
            const b = betrag(darlehen);
            const z = zahlLesen(zins);
            const j = zahlLesen(jahre);
            if (b !== null && z !== null && j !== null)
              void los(
                [
                  {
                    LendToCompany: {
                      company: firma,
                      amount: b,
                      rate: z / 100,
                      years: Math.floor(j),
                    },
                  },
                ],
                t("person.geliehen"),
              ).then((ok) => ok && setDarlehen(""));
          }}
        >
          {t("person.leihen")}
        </button>
      </div>
      <p className="feld-hilfe">
        {t("person.darlehen_hilfe", {
          zins: formatZahl(g.loan_max_rate * 100),
          jahre: formatZahl(g.loan_max_years),
        })}
      </p>
      {(h.loans ?? []).length > 0 && (
        <ul className="liste">
          {(h.loans ?? []).map((l) => (
            <li key={l.index}>
              {t("person.darlehen_offen", {
                betrag: formatGeld(l.balance_usd),
                zins: formatProzent(l.rate),
                rate: formatGeld(l.instalment_usd),
              })}
            </li>
          ))}
        </ul>
      )}
      {(h.withdraw_max_usd ?? 0) > 0 && (
        <div className="formular-zeile">
          <ZahlFeld
            name={t("person.rueckzahlung")}
            einheit={geldEinheit()}
            wert={rueck}
            onWert={setRueck}
            hilfe={t("person.rueckzahlung_hilfe", { max: formatGeld(h.withdraw_max_usd ?? 0) })}
          />
          <button
            type="button"
            onClick={() => {
              const b = betrag(rueck);
              if (b !== null)
                void los(
                  [{ WithdrawCapital: { company: firma, amount: b } }],
                  t("person.zurueckgezahlt"),
                ).then((ok) => ok && setRueck(""));
            }}
          >
            {t("person.zurueckholen")}
          </button>
        </div>
      )}
      <Rueckmeldung meldung={antwort} />
    </section>
  );
}

function Lebenslauf({ p }: { p: Person }) {
  return (
    <section aria-label={t("person.lebenslauf")}>
      <h2>{t("person.lebenslauf")}</h2>
      <table className="tabelle">
        <thead>
          <tr>
            <th>{t("person.datum")}</th>
            <th>{t("person.ereignis")}</th>
          </tr>
        </thead>
        <tbody>
          {p.history.map((e, i) => (
            <tr key={`${e.date}-${i}`}>
              <td>{formatDatum(e.date)}</td>
              <td data-spalte={t("person.ereignis")}>{meldungText(e.text)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}

/** The player as a person (PE2) with its money (PE3): profile, wealth, lifestyle,
 * family, roles and chronicle. */
export function PersonAnsicht({
  kern,
  stand,
  onGeaendert,
  onGruenden,
}: {
  kern: Kern;
  stand: string;
  onGeaendert?: (u: Uebersicht) => void;
  onGruenden?: () => void;
}) {
  const { daten, fehler, neu } = useSicht(() => kern.person(), stand);
  const { senden, meldung } = useBefehl(kern, onGeaendert ?? (() => {}), neu);
  if (!daten) return <FehlerText fehler={fehler} />;
  const g = daten.money ?? null;
  return (
    <Befehle senden={senden} meldung={meldung}>
      <p className="feld-hilfe">{t("person.frage")}</p>
      <section aria-label={t("person.steckbrief")}>
        <h2>{daten.name}</h2>
        <dl className="werte">
          <dt>{t("person.geboren")}</dt>
          <dd>
            {t("person.alter_wert", {
              datum: formatDatum(daten.born),
              jahre: formatZahl(daten.age),
            })}
          </dd>
          <dt>{t("person.wohnsitz")}</dt>
          <dd>{landName(daten.home)}</dd>
          <dt>{t("person.familienstand")}</dt>
          <dd>{t(daten.married ? "person.verheiratet" : "person.ledig")}</dd>
        </dl>
      </section>
      {onGruenden && (
        <section className="karte" aria-label={t("gruendung.titel")}>
          <h2>{t("gruendung.titel")}</h2>
          <p>{t("person.gruenden_hilfe")}</p>
          <button type="button" className="haupt" onClick={onGruenden}>
            {t("gruendung.oeffnen")}
          </button>
        </section>
      )}
      {g && <Vermoegen g={g} />}
      {g && <Lebensstil g={g} />}
      {daten.wellbeing && <Wohlbefinden w={daten.wellbeing} />}
      {daten.assets && <Anlagen a={daten.assets} />}
      {daten.circle && <Ansehen k={daten.circle} />}
      {daten.speculation && <Wertpapierkredit s={daten.speculation} />}
      {daten.trust && <Bindung b={daten.trust} />}
      {daten.life && <Leben l={daten.life} />}
      {daten.assets?.property && <Immobilien p={daten.assets.property} />}
      <Rollen p={daten} g={g} />
      {g &&
        daten.holdings
          .filter((h) => h.controlled)
          .map((h) => <Kapital key={h.company} h={h} g={g} />)}
      {daten.holdings
        .filter((h) => h.share > 0)
        .map((h) => (
          <Handel key={`handel-${h.company}`} h={h} />
        ))}
      <Familie p={daten} />
      {daten.private_life && <Privatleben p={daten} pl={daten.private_life} />}
      {daten.succession && <NachfolgeAbschnitt n={daten.succession} />}
      <Lebenslauf p={daten} />
    </Befehle>
  );
}
