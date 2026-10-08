import { useState } from "react";
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

/** Where a child stands as a manager: not yet, free, employed, retired or dead. */
function kindStatus(k: PersonKind): string {
  if (k.died) return t("person.kind_verstorben", { datum: formatDatum(k.died) });
  if (k.position) return meldungText(k.position);
  if (k.manager !== null) return t("person.kind_frei");
  if (k.card_in !== null) return t("person.kind_karte_in", { jahre: formatZahl(k.card_in) });
  return t("person.kind_im_ruhestand");
}

function Familie({ p }: { p: Person }) {
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
            </tr>
          </thead>
          <tbody>
            {p.children.map((k) => (
              <tr key={`${k.name}-${k.born}`}>
                <td>{k.name}</td>
                <td data-spalte={t("person.geboren")}>{formatDatum(k.born)}</td>
                <td className="zahl" data-spalte={t("person.kind_alter")}>
                  {formatZahl(k.age)}
                </td>
                <td data-spalte={t("person.kind_status")}>{kindStatus(k)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      {p.children_possible && <p className="feld-hilfe">{t("person.kinder_moeglich")}</p>}
    </section>
  );
}

function Rollen({ p }: { p: Person }) {
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
            </tr>
          ))}
        </tbody>
      </table>
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

/** The person's salary as CEO of the main company. */
function Gehalt({ g }: { g: PersonGeld }) {
  const [betrag, setBetrag] = useState(geldFeld(g.salary_usd));
  const { los, antwort } = useAktion("gehalt");
  return (
    <form
      className="karte"
      aria-label={t("person.gehalt")}
      onSubmit={(e) => {
        e.preventDefault();
        const b = zahlLesen(betrag);
        if (b === null || b < 0) return;
        void los(
          [{ SetPersonSalary: { amount: geld(ausAnzeige(b)) } }],
          t("person.gehalt_gesetzt", { betrag: formatGeld(ausAnzeige(b)) }),
        );
      }}
    >
      <h3>{t("person.gehalt")}</h3>
      <p className="feld-hilfe">
        {t(g.salary_paid ? "person.gehalt_hilfe" : "person.gehalt_ruht", {
          vorschlag: formatGeld(g.salary_suggestion_usd),
          steuer: formatProzent(g.income_tax),
        })}
        {g.salary_max_usd !== null &&
          ` ${t("person.gehalt_hoechstens", { max: formatGeld(g.salary_max_usd) })}`}
      </p>
      <div className="formular-zeile">
        <ZahlFeld
          name={t("person.gehalt_jahr")}
          einheit={geldEinheit()}
          wert={betrag}
          onWert={setBetrag}
        />
        <button type="submit">{t("person.gehalt_setzen")}</button>
      </div>
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}

/** Heir, estate and the hand-over while alive (PE6). */
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
      {g && daten.holdings.some((h) => h.person_ceo) && <Gehalt g={g} />}
      <Rollen p={daten} />
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
      {daten.succession && <NachfolgeAbschnitt n={daten.succession} />}
      <Lebenslauf p={daten} />
    </Befehle>
  );
}
