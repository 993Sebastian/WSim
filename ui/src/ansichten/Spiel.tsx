import { useCallback, useState } from "react";
import { formatDatum, formatGeld } from "../format";
import type { Fortschritt, Kern, Rundenbericht, Rundenlaenge, Uebersicht } from "../kern";
import { t } from "../texte";
import { Dialog, FehlerText } from "./Dialog";
import { fehlerText } from "./fehler";
import { RundenberichtDialog } from "./Rundenbericht";
import { LadenDialog, SpeichernDialog } from "./SpeichernLaden";
import { UebersichtAnsicht } from "./Uebersicht";

const LAENGEN: Rundenlaenge[] = ["tag", "woche", "monat", "quartal"];

type Fenster =
  | { art: "keins" }
  | { art: "runde"; fortschritt: Fortschritt | null }
  | { art: "bericht"; bericht: Rundenbericht }
  | { art: "speichern" }
  | { art: "laden" };

export function Spiel({
  kern,
  start,
  onMenue,
}: {
  kern: Kern;
  start: Uebersicht;
  onMenue: (u: Uebersicht) => void;
}) {
  const [uebersicht, setUebersicht] = useState(start);
  const [laenge, setLaenge] = useState<Rundenlaenge>("monat");
  const [fenster, setFenster] = useState<Fenster>({ art: "keins" });
  const [fehler, setFehler] = useState<string | null>(null);
  const firma = uebersicht.company;

  const runde = async () => {
    setFehler(null);
    setFenster({ art: "runde", fortschritt: null });
    try {
      const bericht = await kern.rundeBeenden(laenge, (f) =>
        setFenster((alt) => (alt.art === "runde" ? { art: "runde", fortschritt: f } : alt)),
      );
      setUebersicht(await kern.uebersicht());
      setFenster({ art: "bericht", bericht });
    } catch (err) {
      setFehler(fehlerText(err));
      setFenster({ art: "keins" });
    }
  };

  const schliessen = useCallback(() => setFenster({ art: "keins" }), []);

  const springe = (ziel: string) => {
    setFenster({ art: "keins" });
    document.getElementById(ziel)?.scrollIntoView({ behavior: "smooth", block: "start" });
  };

  return (
    <div className="spiel">
      <header className="kopfleiste">
        <div className="kopf-firma">
          <strong>{firma.name}</strong>
          <span>
            {t("spiel.datum")}:{" "}
            <time dateTime={uebersicht.date}>{formatDatum(uebersicht.date)}</time>
          </span>
          <span>
            {t("spiel.kasse")}:{" "}
            <span className={firma.cash_usd < 0 ? "negativ" : ""}>
              {formatGeld(firma.cash_usd)}
            </span>
          </span>
        </div>
        <div className="kopf-runde">
          <label>
            {t("spiel.rundenlaenge")}
            <select
              id="rundenlaenge"
              value={laenge}
              onChange={(e) => setLaenge(e.target.value as Rundenlaenge)}
            >
              {LAENGEN.map((l) => (
                <option key={l} value={l}>
                  {t(`rundenlaenge.${l}`)}
                </option>
              ))}
            </select>
          </label>
          <button
            type="button"
            className="haupt"
            onClick={runde}
            disabled={uebersicht.game_over || fenster.art === "runde"}
          >
            {t("spiel.runde_beenden")}
          </button>
        </div>
        <nav className="kopf-menue">
          <button type="button" onClick={() => setFenster({ art: "speichern" })}>
            {t("spiel.speichern")}
          </button>
          <button type="button" onClick={() => setFenster({ art: "laden" })}>
            {t("spiel.laden")}
          </button>
          <button type="button" onClick={() => onMenue(uebersicht)}>
            {t("menue.hauptmenue")}
          </button>
        </nav>
      </header>
      <FehlerText fehler={fehler} />
      {uebersicht.game_over && <p className="fehlertext banner">{t("spiel.ende")}</p>}
      <UebersichtAnsicht uebersicht={uebersicht} />

      {fenster.art === "runde" && (
        <Dialog titel={t("fortschritt.titel")}>
          <progress
            className="fortschritt"
            max={fenster.fortschritt?.total ?? 1}
            value={fenster.fortschritt?.done ?? 0}
            aria-label={t("fortschritt.titel")}
          />
          <p className="gedaempft">
            {fenster.fortschritt ? t("fortschritt.tage", { ...fenster.fortschritt }) : "…"}
          </p>
        </Dialog>
      )}
      {fenster.art === "bericht" && (
        <RundenberichtDialog bericht={fenster.bericht} onZiel={springe} onSchliessen={schliessen} />
      )}
      {fenster.art === "speichern" && (
        <SpeichernDialog
          kern={kern}
          vorschlag={`${firma.name} ${formatDatum(uebersicht.date)}`}
          onSchliessen={schliessen}
        />
      )}
      {fenster.art === "laden" && (
        <LadenDialog
          kern={kern}
          onGeladen={(u) => {
            setUebersicht(u);
            setFenster({ art: "keins" });
          }}
          onSchliessen={schliessen}
        />
      )}
    </div>
  );
}
