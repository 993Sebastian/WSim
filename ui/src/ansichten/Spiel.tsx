import { useCallback, useEffect, useRef, useState } from "react";
import { formatDatum, formatGeld } from "../format";
import type { Fortschritt, Kern, Meldung, Rundenbericht, Rundenlaenge, Uebersicht } from "../kern";
import { t } from "../texte";
import { Dialog, FehlerText } from "./Dialog";
import { fehlerText } from "./fehler";
import { RundenberichtDialog } from "./Rundenbericht";
import { LadenDialog, SpeichernDialog } from "./SpeichernLaden";
import { WeltereignisDialog } from "./Weltereignis";
import { BerichteAnsicht } from "./Berichte";
import { FinanzenAnsicht } from "./Finanzen";
import { ForschungAnsicht } from "./Forschung";
import { MarktAnsicht } from "./Markt";
import { ProduktionAnsicht } from "./Produktion";
import { Tastenhilfe } from "./Tastenhilfe";
import { UebersichtAnsicht } from "./Uebersicht";
import { WeltkarteAnsicht } from "./Weltkarte";

type Ansicht =
  "uebersicht" | "produktion" | "markt" | "forschung" | "finanzen" | "weltkarte" | "berichte";
export const ANSICHTEN: Ansicht[] = [
  "uebersicht",
  "produktion",
  "markt",
  "forschung",
  "finanzen",
  "weltkarte",
  "berichte",
];
/** Round reports kept for the archive view. */
const ARCHIV = 120;

const LAENGEN: Rundenlaenge[] = ["tag", "woche", "monat", "quartal"];

type Fenster =
  | { art: "keins" }
  | { art: "runde"; fortschritt: Fortschritt | null }
  | { art: "bericht"; bericht: Rundenbericht }
  | { art: "ereignis"; bericht: Rundenbericht; liste: Meldung[]; index: number }
  | { art: "speichern" }
  | { art: "laden" }
  | { art: "hilfe" };

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
  const [berichte, setBerichte] = useState<Rundenbericht[]>([]);
  const firma = uebersicht.company;

  const runde = async () => {
    setFehler(null);
    setFenster({ art: "runde", fortschritt: null });
    try {
      const bericht = await kern.rundeBeenden(laenge, (f) =>
        setFenster((alt) => (alt.art === "runde" ? { art: "runde", fortschritt: f } : alt)),
      );
      setUebersicht(await kern.uebersicht());
      setBerichte((alt) => [bericht, ...alt].slice(0, ARCHIV));
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

  // Keyboard shortcuts (Lastenheft §15): one place, shown in the help window.
  const rundeRef = useRef(runde);
  useEffect(() => {
    rundeRef.current = runde;
  });
  const offen = fenster.art !== "keins";
  useEffect(() => {
    const taste = (e: KeyboardEvent) => {
      const ziel = e.target as HTMLElement | null;
      const eingabe = ziel && /^(INPUT|SELECT|TEXTAREA)$/.test(ziel.tagName);
      if (e.ctrlKey && e.key === "Enter") {
        e.preventDefault();
        if (!offen && !uebersicht.game_over) void rundeRef.current();
      } else if (e.ctrlKey && e.key.toLowerCase() === "s") {
        e.preventDefault();
        if (!offen) setFenster({ art: "speichern" });
      } else if (e.ctrlKey && e.key.toLowerCase() === "o") {
        e.preventDefault();
        if (!offen) setFenster({ art: "laden" });
      } else if (!offen && !eingabe && !e.ctrlKey && !e.altKey && !e.metaKey) {
        const nummer = Number(e.key);
        if (nummer >= 1 && nummer <= ANSICHTEN.length) setAnsicht(ANSICHTEN[nummer - 1]!);
        else if (e.key === "?" || e.key === "F1") {
          e.preventDefault();
          setFenster({ art: "hilfe" });
        }
      }
    };
    window.addEventListener("keydown", taste);
    return () => window.removeEventListener("keydown", taste);
  }, [offen, uebersicht.game_over]);

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
            aria-keyshortcuts="Control+Enter"
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
        {ANSICHTEN.map((a, i) => (
          <button
            key={a}
            type="button"
            aria-current={ansicht === a ? "page" : undefined}
            aria-keyshortcuts={String(i + 1)}
            onClick={() => setAnsicht(a)}
          >
            {t(`ansicht.${a}`)}
          </button>
        ))}
        <button
          type="button"
          className="schlicht"
          aria-keyshortcuts="?"
          onClick={() => setFenster({ art: "hilfe" })}
        >
          {t("tasten.knopf")}
        </button>
      </nav>
      <FehlerText fehler={fehler} />
      {uebersicht.game_over && <p className="fehlertext banner">{t("spiel.ende")}</p>}
      {ansicht === "uebersicht" && <UebersichtAnsicht uebersicht={uebersicht} />}
      {ansicht === "produktion" && (
        <ProduktionAnsicht kern={kern} uebersicht={uebersicht} onGeaendert={setUebersicht} />
      )}
      {ansicht === "markt" && (
        <MarktAnsicht
          kern={kern}
          datum={uebersicht.date}
          heimat={uebersicht.company.headquarters}
        />
      )}
      {ansicht === "forschung" && (
        <ForschungAnsicht kern={kern} uebersicht={uebersicht} onGeaendert={setUebersicht} />
      )}
      {ansicht === "finanzen" && (
        <FinanzenAnsicht kern={kern} uebersicht={uebersicht} onGeaendert={setUebersicht} />
      )}
      {ansicht === "weltkarte" && <WeltkarteAnsicht kern={kern} datum={uebersicht.date} />}
      {ansicht === "berichte" && (
        <BerichteAnsicht
          berichte={berichte}
          onOeffnen={(bericht) => setFenster({ art: "bericht", bericht })}
        />
      )}
      {fenster.art === "hilfe" && <Tastenhilfe onSchliessen={schliessen} />}

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
