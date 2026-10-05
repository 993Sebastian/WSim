import { useCallback, useEffect, useRef, useState } from "react";
import { formatDatum, formatGeld, landName } from "../format";
import type {
  Fortschritt,
  Hinweis,
  Kern,
  Meldung,
  Rundenbericht,
  Rundenlaenge,
  Uebersicht,
} from "../kern";
import { t } from "../texte";
import { Dialog, FehlerText } from "./Dialog";
import { fehlerText } from "./fehler";
import { RundenberichtDialog } from "./Rundenbericht";
import { LadenDialog, SpeichernDialog } from "./SpeichernLaden";
import { WeltereignisDialog } from "./Weltereignis";
import { BerichteAnsicht } from "./Berichte";
import { Einfuehrung, PFADE, type Pfad } from "./Einfuehrung";
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

/** The introduction's way to the first sale: a start with facilities produces. */
function pfadVon(u: Uebersicht): Pfad {
  return u.company.sites.some((s) => s.facilities.length > 0) ? "werkstatt" : "handel";
}

/** The running introduction and the number of rounds played when it started. */
interface Fuehrung {
  pfad: Pfad;
  schritt: number;
  ab: number;
}

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
  einfuehrung = false,
  onMenue,
}: {
  kern: Kern;
  start: Uebersicht;
  /** Show the introduction (chosen in the new game dialog). */
  einfuehrung?: boolean;
  onMenue: (u: Uebersicht) => void;
}) {
  const [uebersicht, setUebersicht] = useState(start);
  const [fuehrung, setFuehrung] = useState<Fuehrung | null>(
    einfuehrung ? { pfad: pfadVon(start), schritt: 0, ab: 0 } : null,
  );
  // Rounds played in this session (the introduction looks at reports after its start).
  const [runden, setRunden] = useState(0);
  const [laenge, setLaenge] = useState<Rundenlaenge>("monat");
  const [fenster, setFenster] = useState<Fenster>({ art: "keins" });
  const [fehler, setFehler] = useState<string | null>(null);
  const [ansicht, setAnsicht] = useState<Ansicht>("uebersicht");
  // Site open in the plant view of the sites tab (null: the list of sites), and the
  // area of the plant view to show first.
  const [werk, setWerk] = useState<number | null>(null);
  const [werkBereich, setWerkBereich] = useState<string | null>(null);
  const [menue, setMenue] = useState(false);
  // Country chosen on the map for a new site.
  const [gruendenIn, setGruendenIn] = useState<string | null>(null);
  // Every view starts at its top (and the game screen at its first view): the page
  // kept the scroll position of the previous screen and hid the tabs.
  useEffect(() => window.scrollTo(0, 0), [ansicht]);
  const [berichte, setBerichte] = useState<Rundenbericht[]>([]);
  // Counts loaded games: the views start afresh, even if the date is the same.
  const [ladung, setLadung] = useState(0);
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
      setRunden((n) => n + 1);
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

  const zeigeSchritt = (n: number) => {
    if (!fuehrung) return;
    setFuehrung({ ...fuehrung, schritt: n });
    const a = PFADE[fuehrung.pfad][n]?.ansicht;
    if (a) setAnsicht(a as Ansicht);
  };

  const springe = (ziel: string) => {
    setFenster({ art: "keins" });
    if ((ANSICHTEN as string[]).includes(ziel)) setAnsicht(ziel as Ansicht);
  };

  /** Opens the place a hint points to: a plant (with its area) or a view. */
  const zuHinweis = (h: Hinweis) => {
    setFenster({ art: "keins" });
    if (h.site !== null) {
      setWerk(h.site);
      setWerkBereich(h.area);
      setAnsicht("produktion");
    } else if (h.message.target) springe(h.message.target);
  };

  const oeffneWerk = (site: number) => {
    setWerk(site);
    setWerkBereich(null);
    setAnsicht("produktion");
  };

  const warnungen = uebersicht.hints.filter(
    (h) => h.message.kind === "warning" || h.message.kind === "crisis",
  ).length;
  const vormonat = uebersicht.history.at(-1);
  const trend = vormonat ? firma.cash_usd - vormonat.cash_usd : null;
  const menuePunkt = (aktion: () => void) => () => {
    setMenue(false);
    aktion();
  };

  return (
    <div className={fuehrung ? "spiel mit-einfuehrung" : "spiel"}>
      {/* Header and tabs stay on screen together while the view scrolls. */}
      <div className="kopfbereich">
        <header className="kopfleiste">
          <div className="kopf-firma">
            <strong>{firma.name}</strong>
            <span>
              <span className="nur-breit">{t("spiel.datum")}: </span>
              <time dateTime={uebersicht.date}>{formatDatum(uebersicht.date)}</time>
            </span>
            <span>
              {t("spiel.kasse")}:{" "}
              <span className={firma.cash_usd < 0 ? "negativ" : ""}>
                {formatGeld(firma.cash_usd)}
              </span>
              {trend !== null && Math.abs(trend) >= 0.5 && (
                <span
                  className={`trend ${trend < 0 ? "negativ" : "positiv"}`}
                  title={t("spiel.kasse_trend")}
                >
                  {" "}
                  {trend < 0 ? "▼" : "▲"}
                  <span className="nur-breit"> {formatGeld(Math.abs(trend))}</span>
                </span>
              )}
            </span>
          </div>
          <div className="kopf-runde" data-tour="runde">
            <label>
              <span className="nur-breit">{t("spiel.rundenlaenge")}</span>
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
          <div className="kopf-menue">
            <button
              type="button"
              aria-haspopup="menu"
              aria-expanded={menue}
              aria-label={t("spiel.menue")}
              onClick={() => setMenue((m) => !m)}
            >
              ☰
            </button>
            {menue && (
              <div className="menue-liste" role="menu" aria-label={t("spiel.menue")}>
                <button
                  type="button"
                  role="menuitem"
                  onClick={menuePunkt(() => setFenster({ art: "speichern" }))}
                >
                  {t("spiel.speichern")} <kbd>Strg+S</kbd>
                </button>
                <button
                  type="button"
                  role="menuitem"
                  onClick={menuePunkt(() => setFenster({ art: "laden" }))}
                >
                  {t("spiel.laden")} <kbd>Strg+O</kbd>
                </button>
                <button
                  type="button"
                  role="menuitem"
                  onClick={menuePunkt(() => setFenster({ art: "hilfe" }))}
                >
                  {t("tasten.titel")} <kbd>?</kbd>
                </button>
                <button
                  type="button"
                  role="menuitem"
                  onClick={menuePunkt(() => onMenue(uebersicht))}
                >
                  {t("menue.hauptmenue")}
                </button>
              </div>
            )}
          </div>
        </header>
        <nav className="reiter" aria-label={t("spiel.ansichten")}>
          {ANSICHTEN.map((a, i) => (
            <button
              key={a}
              type="button"
              aria-current={ansicht === a ? "page" : undefined}
              aria-keyshortcuts={String(i + 1)}
              data-tour={`reiter-${a}`}
              onClick={() => {
                if (a === "produktion" && ansicht === "produktion") setWerk(null);
                setGruendenIn(null);
                setAnsicht(a);
              }}
            >
              {t(`ansicht.${a}`)}
              {a === "uebersicht" && warnungen > 0 && (
                <span className="zaehler" title={t("spiel.offene_hinweise")}>
                  {warnungen}
                </span>
              )}
            </button>
          ))}
        </nav>
      </div>
      <FehlerText fehler={fehler} />
      {uebersicht.game_over && <p className="fehlertext banner">{t("spiel.ende")}</p>}
      {/* A text key: with the number alone the views were drawn twice after a round. */}
      <div key={`ladung-${ladung}`} className="ansichtsbereich">
        {ansicht === "uebersicht" && (
          <UebersichtAnsicht uebersicht={uebersicht} onHinweis={zuHinweis} onWerk={oeffneWerk} />
        )}
        {ansicht === "produktion" && (
          <ProduktionAnsicht
            kern={kern}
            uebersicht={uebersicht}
            onGeaendert={setUebersicht}
            werk={werk}
            werkBereich={werkBereich}
            gruendenIn={gruendenIn}
            onWerk={(site) => {
              setWerk(site);
              setWerkBereich(null);
            }}
          />
        )}
        {ansicht === "markt" && (
          <MarktAnsicht kern={kern} uebersicht={uebersicht} onGeaendert={setUebersicht} />
        )}
        {ansicht === "forschung" && (
          <ForschungAnsicht
            kern={kern}
            uebersicht={uebersicht}
            onGeaendert={setUebersicht}
            onStandorte={() => {
              setWerk(null);
              setAnsicht("produktion");
            }}
          />
        )}
        {ansicht === "finanzen" && (
          <FinanzenAnsicht kern={kern} uebersicht={uebersicht} onGeaendert={setUebersicht} />
        )}
        {ansicht === "weltkarte" && (
          <WeltkarteAnsicht
            kern={kern}
            datum={uebersicht.date}
            onGruenden={(land) => {
              setGruendenIn(land);
              setWerk(null);
              setAnsicht("produktion");
            }}
          />
        )}
        {ansicht === "berichte" && (
          <BerichteAnsicht
            berichte={berichte}
            onOeffnen={(bericht) => setFenster({ art: "bericht", bericht })}
          />
        )}
      </div>
      {fuehrung && (
        <Einfuehrung
          pfad={fuehrung.pfad}
          schritt={fuehrung.schritt}
          bericht={runden > fuehrung.ab ? (berichte[0] ?? null) : null}
          parameter={{ firma: firma.name, land: landName(firma.headquarters) }}
          onSchritt={zeigeSchritt}
          onEnde={() => setFuehrung(null)}
        />
      )}
      {fenster.art === "hilfe" && (
        <Tastenhilfe
          onSchliessen={schliessen}
          onEinfuehrung={() => {
            schliessen();
            setFuehrung({ pfad: pfadVon(uebersicht), schritt: 0, ab: runden });
            setAnsicht("uebersicht");
          }}
        />
      )}

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
          onHinweis={zuHinweis}
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
            setWerk(null);
            setLadung((n) => n + 1);
            setBerichte([]);
            setFenster({ art: "keins" });
          }}
          onSchliessen={schliessen}
        />
      )}
    </div>
  );
}
