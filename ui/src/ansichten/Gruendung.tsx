import { useMemo, useState, type FormEvent } from "react";
import {
  ausAnzeige,
  formatGeld,
  formatProzent,
  geldEinheit,
  geldFeld,
  landName,
  zahlLesen,
} from "../format";
import { geld, STARTFORMEN, type Kern, type Uebersicht } from "../kern";
import { t } from "../texte";
import { Dialog, FehlerText } from "./Dialog";
import { fehlerText } from "./fehler";
import { useSicht, ZahlFeld } from "./gemeinsam";

/**
 * Founding the person's company (PE3): name, start form, seat and capital paid in from
 * the private account. Opens right after the start and from the person view.
 */
export function GruendungDialog({
  kern,
  stand,
  onGegruendet,
  onSchliessen,
}: {
  kern: Kern;
  stand: string;
  onGegruendet: (u: Uebersicht) => void;
  onSchliessen: () => void;
}) {
  const { daten, fehler: ladefehler } = useSicht(() => kern.person(), stand);
  const { daten: optionen } = useSicht(() => kern.optionen(), "optionen");
  const [name, setName] = useState("");
  const [form, setForm] = useState<string | null>(null);
  const [land, setLand] = useState<string | null>(null);
  const [einlage, setEinlage] = useState<string | null>(null);
  const [fehler, setFehler] = useState<string | null>(null);
  const [laeuft, setLaeuft] = useState(false);
  const g = daten?.founding ?? null;
  const konto = daten?.money?.cash_usd ?? 0;
  // Every country of the data, the home first suggested.
  const laender = useMemo(
    () =>
      (optionen?.countries ?? (g ? [g.country] : []))
        .map((k) => ({ k, name: landName(k) }))
        .sort((a, b) => a.name.localeCompare(b.name, "de")),
    [optionen, g],
  );
  if (!daten) {
    return (
      <Dialog titel={t("gruendung.titel")} onSchliessen={onSchliessen}>
        <FehlerText fehler={ladefehler} />
      </Dialog>
    );
  }
  if (!g) return null;
  const formWahl = form ?? g.forms[0]?.key ?? "werkstatt";
  const sitz = land ?? g.country;
  const betragText = einlage ?? geldFeld(g.capital_suggestion_usd);
  const betrag = zahlLesen(betragText);
  const kosten = g.forms.find((f) => f.key === formWahl)?.cost_usd ?? 0;

  const gruenden = async (e: FormEvent) => {
    e.preventDefault();
    if (betrag === null || betrag <= 0) {
      setFehler(t("feld.keine_zahl"));
      return;
    }
    setFehler(null);
    setLaeuft(true);
    try {
      const u = await kern.befehl({
        FoundCompany: {
          name: name.trim(),
          form: STARTFORMEN[formWahl] ?? "Workshop",
          country: sitz,
          capital: geld(ausAnzeige(betrag)),
        },
      });
      onGegruendet(u);
    } catch (err) {
      setFehler(fehlerText(err));
      setLaeuft(false);
    }
  };

  return (
    <Dialog titel={t("gruendung.titel")} onSchliessen={onSchliessen} tour="gruendung">
      <form className="formular" onSubmit={gruenden}>
        <p className="feld-hilfe">{t("gruendung.frage", { konto: formatGeld(konto) })}</p>
        <label>
          {t("gruendung.name")}
          <input
            id="firmenname"
            required
            maxLength={60}
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </label>
        <fieldset>
          <legend>{t("neu.startform")}</legend>
          {g.forms.map((f) => (
            <label key={f.key} className="auswahl">
              <input
                type="radio"
                name="startform"
                value={f.key}
                checked={formWahl === f.key}
                onChange={() => setForm(f.key)}
              />
              <span>
                {t("neu.startform_kosten", {
                  form: t(`startform.${f.key}`),
                  betrag: formatGeld(f.cost_usd),
                })}
                <small className="feld-hilfe">{t(`neu.startform_hilfe.${f.key}`)}</small>
              </span>
            </label>
          ))}
        </fieldset>
        <label>
          {t("gruendung.sitz")}
          <select id="hauptsitz" value={sitz} onChange={(e) => setLand(e.target.value)}>
            {laender.map(({ k, name: n }) => (
              <option key={k} value={k}>
                {n}
              </option>
            ))}
          </select>
        </label>
        <ZahlFeld
          name={t("gruendung.einlage")}
          einheit={geldEinheit()}
          wert={betragText}
          onWert={setEinlage}
          breit
          hilfe={t("gruendung.einlage_hilfe", {
            mindestens: formatGeld(kosten),
            anteil: formatProzent(g.cost_share),
            kosten_min: formatGeld(g.cost_min_usd),
          })}
        />
        <FehlerText fehler={fehler} />
        <div className="knopfreihe">
          <button type="button" onClick={onSchliessen} disabled={laeuft}>
            {t("gruendung.spaeter")}
          </button>
          <button type="submit" className="haupt" disabled={laeuft}>
            {t("gruendung.gruenden")}
          </button>
        </div>
      </form>
    </Dialog>
  );
}
