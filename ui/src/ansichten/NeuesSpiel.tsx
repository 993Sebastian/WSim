import { useEffect, useMemo, useState, type FormEvent } from "react";
import { formatGeld, formatProzent, formatZahl, landName, zahlFeld, zahlLesen } from "../format";
import type { Kern, NeuesSpiel, Optionen, Uebersicht } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { fehlerText } from "./fehler";
import { ZahlEingabe } from "./gemeinsam";

/** The number fields: they keep the typed text until the game starts. */
type Zahlfeld = "start_year" | "companies" | "capital_usd" | "research_factor" | "seed";
const ZAHLFELDER: Zahlfeld[] = [
  "start_year",
  "companies",
  "capital_usd",
  "research_factor",
  "seed",
];

/** New game with the settings of Lastenheft §15 that stage 1 knows. */
export function NeuesSpielAnsicht({
  kern,
  onStart,
  onZurueck,
}: {
  kern: Kern;
  onStart: (u: Uebersicht, einfuehrung: boolean) => void;
  onZurueck: () => void;
}) {
  const [optionen, setOptionen] = useState<Optionen | null>(null);
  const [werte, setWerte] = useState<Omit<NeuesSpiel, Zahlfeld> | null>(null);
  const [zahlen, setZahlen] = useState<Record<Zahlfeld, string> | null>(null);
  // Part of the interface only: the core does not know the introduction.
  const [einfuehrung, setEinfuehrung] = useState(true);
  const [fehler, setFehler] = useState<string | null>(null);
  const [startet, setStartet] = useState(false);
  // Empty: the start year minus the default age (PE2).
  const [geburtsjahr, setGeburtsjahr] = useState("");

  useEffect(() => {
    let aktiv = true;
    kern
      .optionen()
      .then((o) => {
        if (!aktiv) return;
        setOptionen(o);
        setWerte({
          country: o.default_country,
          difficulty: o.default_difficulty,
          startups: o.default_startups,
          tariffs: o.default_tariffs ?? null,
          event_effects: true,
          person_name: "",
          married: true,
          children: 0,
        });
        // Year and seed name something: no thousands separators.
        setZahlen({
          start_year: String(o.start_year.default),
          companies: zahlFeld(o.companies.default, 0),
          capital_usd: zahlFeld(o.start_capital_usd.default),
          research_factor: zahlFeld(o.research_factor.default),
          seed: String(Math.floor(Math.random() * 1_000_000_000)),
        });
      })
      .catch((e: unknown) => aktiv && setFehler(fehlerText(e)));
    return () => {
      aktiv = false;
    };
  }, [kern]);

  const laender = useMemo(
    () =>
      (optionen?.countries ?? [])
        .map((k) => ({ k, name: landName(k) }))
        .sort((a, b) => a.name.localeCompare(b.name, "de")),
    [optionen],
  );

  if (!optionen || !werte || !zahlen) {
    return (
      <main className="seite">
        <h1>{t("neu.titel")}</h1>
        <FehlerText fehler={fehler} />
      </main>
    );
  }

  const setze = <K extends keyof typeof werte>(feld: K, wert: (typeof werte)[K]) =>
    setWerte({ ...werte, [feld]: wert });
  const zahlFeldProps = (feld: Zahlfeld) => ({
    wert: zahlen[feld],
    onWert: (text: string) => setZahlen({ ...zahlen, [feld]: text }),
  });

  const starten = async (e: FormEvent) => {
    e.preventDefault();
    const zahl = (feld: Zahlfeld) => zahlLesen(zahlen[feld]) ?? NaN;
    const jahr = geburtsjahr.trim() === "" ? null : zahlLesen(geburtsjahr);
    if (jahr !== null && !Number.isInteger(jahr)) {
      setFehler(t("feld.keine_zahl"));
      return;
    }
    const einstellungen: NeuesSpiel = {
      ...werte,
      person_name: (werte.person_name ?? "").trim(),
      birth_year: jahr,
      // The company is founded in the game (PE3).
      found_at_start: false,
      start_year: zahl("start_year"),
      companies: zahl("companies"),
      capital_usd: zahl("capital_usd"),
      research_factor: zahl("research_factor"),
      seed: zahl("seed"),
    };
    // Whether the numbers fit the ranges is the core's to say.
    if (ZAHLFELDER.some((feld) => Number.isNaN(einstellungen[feld]))) {
      setFehler(t("feld.keine_zahl"));
      return;
    }
    setFehler(null);
    setStartet(true);
    try {
      onStart(await kern.neuesSpiel(einstellungen), einfuehrung);
    } catch (err) {
      setFehler(fehlerText(err));
      setStartet(false);
    }
  };

  return (
    <main className="seite">
      <h1>{t("neu.titel")}</h1>
      <form className="formular" onSubmit={starten}>
        <p className="feld-hilfe">{t("neu.person_zuerst")}</p>
        <div className="feldreihe">
          <label>
            {t("neu.startland")}
            <select
              id="startland"
              value={werte.country}
              onChange={(e) => setze("country", e.target.value)}
            >
              {laender.map(({ k, name }) => (
                <option key={k} value={k}>
                  {name}
                </option>
              ))}
            </select>
          </label>
          <label>
            {t("neu.startjahr")}
            <ZahlEingabe
              id="startjahr"
              ganzzahlig
              gruppieren={false}
              {...zahlFeldProps("start_year")}
            />
          </label>
        </div>
        <label>
          {t("neu.startgeld")}
          <ZahlEingabe id="startgeld" {...zahlFeldProps("capital_usd")} />
          <small className="feld-hilfe">
            {t("neu.startgeld_hilfe", {
              mindestens: formatGeld(optionen.start_money_min_usd ?? 0),
              einlage: formatProzent(optionen.capital_suggestion ?? 0.9),
            })}
          </small>
        </label>
        <Personfelder
          optionen={optionen}
          startjahr={zahlLesen(zahlen.start_year)}
          name={werte.person_name ?? ""}
          onName={(n) => setze("person_name", n)}
          geburtsjahr={geburtsjahr}
          onGeburtsjahr={setGeburtsjahr}
          verheiratet={werte.married ?? true}
          onVerheiratet={(v) => setze("married", v)}
          kinder={werte.children ?? 0}
          onKinder={(k) => setze("children", k)}
        />
        <label>
          {t("neu.schwierigkeit")}
          <select
            id="schwierigkeit"
            value={werte.difficulty}
            onChange={(e) => setze("difficulty", e.target.value)}
          >
            {optionen.difficulties.map((d) => (
              <option key={d.key} value={d.key}>
                {t(`schwierigkeit.${d.key}`)}
              </option>
            ))}
          </select>
        </label>
        <label className="auswahl">
          <input
            id="einfuehrung"
            type="checkbox"
            checked={einfuehrung}
            onChange={(e) => setEinfuehrung(e.target.checked)}
          />
          {t("neu.einfuehrung")}
        </label>
        <details className="aufklapper">
          <summary>{t("neu.weitere")}</summary>
          <div className="formular">
            <div className="feldreihe">
              <label>
                {t("neu.ki_firmen")}
                <ZahlEingabe id="ki_firmen" ganzzahlig {...zahlFeldProps("companies")} />
                <small className="feld-hilfe">{t("neu.ki_firmen_hilfe")}</small>
              </label>
            </div>
            <div className="feldreihe">
              <label>
                {t("neu.forschungsfaktor")}
                <ZahlEingabe
                  id="forschungsfaktor"
                  gruppieren={false}
                  {...zahlFeldProps("research_factor")}
                />
              </label>
              <label>
                {t("neu.seed")}
                <ZahlEingabe id="seed" ganzzahlig gruppieren={false} {...zahlFeldProps("seed")} />
              </label>
            </div>
            {optionen.startups.length > 0 && (
              <label>
                {t("neu.startups")}
                <select
                  id="startups"
                  value={werte.startups ?? ""}
                  onChange={(e) => setze("startups", e.target.value)}
                >
                  {optionen.startups.map((f) => (
                    <option key={f.key} value={f.key}>
                      {t("neu.startups_wahl", {
                        wahl: t(`startup.haeufigkeit.${f.key}`),
                        je_jahr: formatZahl(f.per_year),
                      })}
                    </option>
                  ))}
                </select>
                <small className="feld-hilfe">{t("neu.startups_hilfe")}</small>
              </label>
            )}
            {(optionen.tariffs ?? []).length > 0 && (
              <label>
                {t("neu.zoelle")}
                <select
                  id="zoelle"
                  value={werte.tariffs ?? ""}
                  onChange={(e) => setze("tariffs", e.target.value)}
                >
                  {(optionen.tariffs ?? []).map((k) => (
                    <option key={k} value={k}>
                      {t(`zoll.dynamik.${k}`)}
                    </option>
                  ))}
                </select>
                <small className="feld-hilfe">{t("neu.zoelle_hilfe")}</small>
              </label>
            )}
            <label className="auswahl">
              <input
                id="folgen"
                type="checkbox"
                checked={werte.event_effects ?? true}
                onChange={(e) => setze("event_effects", e.target.checked)}
              />
              {t("neu.folgen")}
            </label>
            <small className="feld-hilfe">{t("neu.folgen_hilfe")}</small>
          </div>
        </details>
        <FehlerText fehler={fehler} />
        <div className="knopfreihe">
          <button type="button" onClick={onZurueck} disabled={startet}>
            {t("neu.zurueck")}
          </button>
          <button type="submit" className="haupt" disabled={startet}>
            {startet ? t("neu.startet") : t("neu.starten")}
          </button>
        </div>
      </form>
    </main>
  );
}

/** The player as a person (PE2): name, year of birth, family. */
function Personfelder(p: {
  optionen: Optionen;
  startjahr: number | null;
  name: string;
  onName: (n: string) => void;
  geburtsjahr: string;
  onGeburtsjahr: (j: string) => void;
  verheiratet: boolean;
  onVerheiratet: (v: boolean) => void;
  kinder: number;
  onKinder: (k: number) => void;
}) {
  const alter = p.optionen.person_age ?? { min: 18, max: 60, default: 30 };
  const start = p.startjahr ?? p.optionen.start_year.default;
  const hoechstens = p.optionen.children_max ?? 4;
  return (
    <fieldset>
      <legend>{t("neu.person")}</legend>
      <div className="feldreihe">
        <label>
          {t("neu.person_name")}
          <input
            id="person_name"
            maxLength={60}
            value={p.name}
            placeholder={t("neu.person_name_hilfe")}
            onChange={(e) => p.onName(e.target.value)}
          />
        </label>
        <label>
          {t("neu.geburtsjahr")}
          <ZahlEingabe
            id="geburtsjahr"
            ganzzahlig
            gruppieren={false}
            wert={p.geburtsjahr}
            onWert={p.onGeburtsjahr}
            placeholder={String(start - alter.default)}
          />
          <small className="feld-hilfe">
            {t("neu.geburtsjahr_hilfe", {
              von: String(start - alter.max),
              bis: String(start - alter.min),
              alter_von: formatZahl(alter.min),
              alter_bis: formatZahl(alter.max),
            })}
          </small>
        </label>
      </div>
      <div className="feldreihe">
        <label className="auswahl">
          <input
            id="verheiratet"
            type="checkbox"
            checked={p.verheiratet}
            onChange={(e) => p.onVerheiratet(e.target.checked)}
          />
          {t("neu.verheiratet")}
        </label>
        <label>
          {t("neu.kinder")}
          <select id="kinder" value={p.kinder} onChange={(e) => p.onKinder(Number(e.target.value))}>
            {Array.from({ length: hoechstens + 1 }, (_, k) => (
              <option key={k} value={k}>
                {k}
              </option>
            ))}
          </select>
        </label>
      </div>
    </fieldset>
  );
}
