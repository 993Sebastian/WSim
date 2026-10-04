import { useCallback, useState } from "react";
import { formatDatum, formatGeld } from "../format";
import type { Fortschritt, Kern, Meldung, Rundenbericht, Rundenlaenge, Uebersicht } from "../kern";
import { t } from "../texte";
import { Dialog, FehlerText } from "./Dialog";
import { fehlerText } from "./fehler";
import { RundenberichtDialog } from "./Rundenbericht";
import { LadenDialog, SpeichernDialog } from "./SpeichernLaden";
import { WeltereignisDialog } from "./Weltereignis";
import { WeltkarteAnsicht } from "./Weltkarte";

type Ansicht = "uebersicht" | "weltkarte";
const ANSICHTEN: Ansicht[] = ["uebersicht", "weltkarte"];
import { UebersichtAnsicht } from "./Uebersicht";

const LAENGEN: Rundenlaenge[] = ["tag", "woche", "monat", "quartal"];

type Fenster =
  | { art: "keins" }
  | { art: "runde"; fortschritt: Fortschritt | null }
  | { art: "bericht"; bericht: Rundenbericht }
  | { art: "ereignis"; bericht: Rundenbericht; liste: Meldung[]; index: number }
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
  const [ansicht, setAnsicht] = useState<Ansicht>("uebersicht");
  const firma = uebersicht.company;

  const runde = async () => {
    setFehler(null);
    setFenster({ art: "runde", fortschritt: null });
    try {
      const bericht = await kern.rundeBeenden(laenge, (f) =>
        setFenster((alt) => (alt.art === "runde" ? { art: "runde", fortschritt: f } : alt)),
      );
      setUebersicht(await kern.uebersicht());
      // World news first, each in a window of its own (then the report).
      const welt = bericht.messages.filter((m) => m.group === "welt");
      setFenster(
        welt.length > 0
          ? { art: "ereignis", bericht, liste: welt, index: 0 }
          : { art: "bericht", bericht },
      );
    } catch (err) {
      setFehler(fehlerText(err));
      setFenster({ art: "keins" });
    }
  };

  const schliessen = useCallback(() => setFenster({ art: "keins" }), []);

  const springe = (ziel: string) => {
    setFenster({ art: "keins" });
    if ((ANSICHTEN as string[]).includes(ziel)) setAnsicht(ziel as Ansicht);
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
      <nav className="reiter" aria-label={t("spiel.ansichten")}>
        {ANSICHTEN.map((a) => (
          <button
            key={a}
            type="button"
            aria-current={ansicht === a ? "page" : undefined}
            onClick={() => setAnsicht(a)}
          >
            {t(`ansicht.${a}`)}
          </button>
        ))}
        <span className="gedaempft">{t("spiel.weitere_ansichten")}</span>
      </nav>
      <FehlerText fehler={fehler} />
      {uebersicht.game_over && <p className="fehlertext banner">{t("spiel.ende")}</p>}
      {ansicht === "uebersicht" && <UebersichtAnsicht uebersicht={uebersicht} />}
      {ansicht === "weltkarte" && <WeltkarteAnsicht kern={kern} datum={uebersicht.date} />}

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
      {fenster.art === "ereignis" && fenster.liste[fenster.index] && (
        <WeltereignisDialog
          key={fenster.index}
          meldung={fenster.liste[fenster.index]!}
          nummer={fenster.index + 1}
          anzahl={fenster.liste.length}
          onWeiter={() =>
            setFenster(
              fenster.index + 1 < fenster.liste.length
                ? { ...fenster, index: fenster.index + 1 }
                : { art: "bericht", bericht: fenster.bericht },
            )
          }
          onAlle={() => setFenster({ art: "bericht", bericht: fenster.bericht })}
        />
      )}
      {fenster.art === "bericht" && (
        <RundenberichtDialog
          bericht={fenster.bericht}
          onZiel={springe}
          onEreignis={(m) =>
            setFenster({ art: "ereignis", bericht: fenster.bericht, liste: [m], index: 0 })
          }
          onSchliessen={schliessen}
        />
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
