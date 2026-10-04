import { useEffect, useMemo, useState, type FormEvent } from "react";
import { formatGeld, landName } from "../format";
import type { Kern, NeuesSpiel, Optionen, Uebersicht } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { fehlerText } from "./fehler";

/** New game with the settings of Lastenheft §15 that stage 1 knows. */
export function NeuesSpielAnsicht({
  kern,
  onStart,
  onZurueck,
}: {
  kern: Kern;
  onStart: (u: Uebersicht) => void;
  onZurueck: () => void;
}) {
  const [optionen, setOptionen] = useState<Optionen | null>(null);
  const [werte, setWerte] = useState<NeuesSpiel | null>(null);
  const [fehler, setFehler] = useState<string | null>(null);
  const [startet, setStartet] = useState(false);

  useEffect(() => {
    let aktiv = true;
    kern
      .optionen()
      .then((o) => {
        if (!aktiv) return;
        setOptionen(o);
        setWerte({
          seed: Math.floor(Math.random() * 1_000_000_000),
          start_year: o.start_year.default,
          country: o.default_country,
          capital_usd: o.start_capital_usd.default,
          start_form: o.start_forms[0]?.key ?? "",
          company_name: "",
          companies: o.companies.default,
          difficulty: o.default_difficulty,
          research_factor: o.research_factor.default,
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

  if (!optionen || !werte) {
    return (
      <main className="seite">
        <h1>{t("neu.titel")}</h1>
        <FehlerText fehler={fehler} />
      </main>
    );
  }

  const setze = <K extends keyof NeuesSpiel>(feld: K, wert: NeuesSpiel[K]) =>
    setWerte({ ...werte, [feld]: wert });
  const zahl = (s: string) => (s.trim() === "" ? NaN : Number(s));

  const starten = async (e: FormEvent) => {
    e.preventDefault();
    setFehler(null);
    setStartet(true);
    try {
      onStart(await kern.neuesSpiel({ ...werte, company_name: werte.company_name.trim() }));
    } catch (err) {
      setFehler(fehlerText(err));
      setStartet(false);
    }
  };

  return (
    <main className="seite">
      <h1>{t("neu.titel")}</h1>
      <form className="formular" onSubmit={starten}>
        <label>
          {t("neu.firmenname")}
          <input
            id="firmenname"
            required
            maxLength={60}
            value={werte.company_name}
            onChange={(e) => setze("company_name", e.target.value)}
          />
        </label>
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
        <div className="feldreihe">
          <label>
            {t("neu.startjahr")}
            <input
              id="startjahr"
              type="number"
              min={optionen.start_year.min}
              max={optionen.start_year.max}
              value={werte.start_year}
              onChange={(e) => setze("start_year", zahl(e.target.value))}
            />
          </label>
          <label>
            {t("neu.startkapital")}
            <input
              id="startkapital"
              type="number"
              min={optionen.start_capital_usd.min}
              step="any"
              value={werte.capital_usd}
              onChange={(e) => setze("capital_usd", zahl(e.target.value))}
            />
          </label>
        </div>
        <fieldset>
          <legend>{t("neu.startform")}</legend>
          {optionen.start_forms.map((f) => (
            <label key={f.key} className="auswahl">
              <input
                type="radio"
                name="startform"
                value={f.key}
                checked={werte.start_form === f.key}
                onChange={() => setze("start_form", f.key)}
              />
              {t("neu.startform_kosten", {
                form: t(`startform.${f.key}`),
                betrag: formatGeld(f.cost_usd),
              })}
            </label>
          ))}
        </fieldset>
        <div className="feldreihe">
          <label>
            {t("neu.ki_firmen")}
            <input
              id="ki_firmen"
              type="number"
              min={optionen.companies.min}
              max={optionen.companies.max}
              value={werte.companies}
              onChange={(e) => setze("companies", zahl(e.target.value))}
            />
          </label>
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
        </div>
        <div className="feldreihe">
          <label>
            {t("neu.forschungsfaktor")}
            <input
              id="forschungsfaktor"
              type="number"
              min={optionen.research_factor.min}
              max={optionen.research_factor.max}
              step={0.25}
              value={werte.research_factor}
              onChange={(e) => setze("research_factor", zahl(e.target.value))}
            />
          </label>
          <label>
            {t("neu.seed")}
            <input
              id="seed"
              type="number"
              min={0}
              value={werte.seed}
              onChange={(e) => setze("seed", Math.max(0, Math.floor(zahl(e.target.value) || 0)))}
            />
          </label>
        </div>
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
