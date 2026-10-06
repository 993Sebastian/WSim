// The guided start (Lastenheft §14.4; M20): it leads a new player to the first sale.
// Each step highlights the button to use – or, from elsewhere, the way to it – and goes
// on by itself once it was used. A short tour of the other views follows.
import { useEffect, useRef, useState } from "react";
import { formatGeld } from "../format";
import type { Rundenbericht } from "../kern";
import { t, type TextParameter } from "../texte";

/** The way to the first sale depends on the start form: produce or trade. */
export type Pfad = "werkstatt" | "handel";

export interface Schritt {
  /** Texts `einfuehrung.<key>.titel` and `.text`. */
  key: string;
  /** Marked elements (`data-tour`) to highlight; the first one on screen wins, so the
   * step shows the button to use or, from elsewhere, the way to it. */
  ziele?: string[];
  /** Done once one of these is on screen; "!x": once x was there and is gone. */
  fertig?: string[];
  /** View the step shows (the tour). */
  ansicht?: string;
}

const ZUM_WERK = ["werk-oeffnen", "reiter-produktion"];
const imWerk = (bereich: string) => [`bereich-${bereich}`, ...ZUM_WERK];

const abschluss = (runde: string): Schritt[] => [
  { key: runde, ziele: ["runde"], fertig: ["bericht", "ereignis"] },
  { key: "bericht", ziele: ["was-lief", "ereignis-weiter"], fertig: ["!bericht"] },
  { key: "geschafft", ansicht: "uebersicht", ziele: ["zu-erledigen"] },
  { key: "etappen", ansicht: "uebersicht", ziele: ["etappen"] },
  { key: "grundstuecke", ansicht: "produktion", ziele: ["grundstueck", "reiter-produktion"] },
  { key: "markt", ansicht: "markt", ziele: ["reiter-markt"] },
  { key: "werbung", ansicht: "markt", ziele: ["bereich-marke"] },
  { key: "forschung", ansicht: "forschung", ziele: ["reiter-forschung"] },
  { key: "weiterentwicklung", ansicht: "forschung", ziele: ["bereich-entwicklung"] },
  { key: "finanzen", ansicht: "finanzen", ziele: ["reiter-finanzen"] },
  { key: "weltkarte", ansicht: "weltkarte", ziele: ["reiter-weltkarte"] },
];

export const PFADE: Record<Pfad, Schritt[]> = {
  werkstatt: [
    { key: "willkommen" },
    { key: "standorte", ziele: ["reiter-produktion"], fertig: ["standorte", "werk"] },
    { key: "werk_oeffnen", ziele: ZUM_WERK, fertig: ["werk"] },
    { key: "anlage", ziele: ["anlage", ...imWerk("anlagen")] },
    { key: "einkauf_oeffnen", ziele: imWerk("einkauf"), fertig: ["werk-einkauf"] },
    { key: "einkauf", ziele: ["einkauf-auftrag", ...imWerk("einkauf")] },
    { key: "verkauf_oeffnen", ziele: imWerk("verkauf"), fertig: ["werk-verkauf"] },
    { key: "preis", ziele: ["preis", ...imWerk("verkauf")], fertig: ["preis-gesetzt"] },
    { key: "personal", ziele: imWerk("personal"), fertig: ["werk-personal"] },
    ...abschluss("runde"),
  ],
  handel: [
    { key: "willkommen_handel" },
    { key: "standorte", ziele: ["reiter-produktion"], fertig: ["standorte", "werk"] },
    { key: "niederlassung_oeffnen", ziele: ZUM_WERK, fertig: ["werk"] },
    { key: "einkauf_oeffnen", ziele: imWerk("einkauf"), fertig: ["werk-einkauf"] },
    {
      key: "einkauf_handel",
      ziele: ["einkauf-neu", ...imWerk("einkauf")],
      fertig: ["einkauf-auftrag"],
    },
    { key: "verkauf_oeffnen", ziele: imWerk("verkauf"), fertig: ["werk-verkauf"] },
    {
      key: "angebot_handel",
      ziele: ["angebot-neu", ...imWerk("verkauf")],
      fertig: ["angebot-aktiv"],
    },
    ...abschluss("runde_handel"),
  ],
};

const markiert = (id: string) => document.querySelector<HTMLElement>(`[data-tour="${id}"]`);

/** Revenue of a round report, null without one. */
export function umsatz(bericht: Rundenbericht | null): number | null {
  return bericht ? bericht.products.reduce((summe, p) => summe + p.revenue_usd, 0) : null;
}

/** Index of the next step: after a report without a sale, the round again. */
export function naechster(pfad: Pfad, schritt: number, bericht: Rundenbericht | null): number {
  const schritte = PFADE[pfad];
  if (schritte[schritt]?.key === "bericht" && (umsatz(bericht) ?? 0) <= 0) {
    return schritte.findIndex((s) => s.key.startsWith("runde"));
  }
  return schritt + 1;
}

interface Rahmen {
  top: number;
  left: number;
  width: number;
  height: number;
}

const ABSTAND = 6;

/** Whether the middle of an element can be seen: on screen and not covered (by the
 * panel, a dialog) or scrolled out of its box. */
function sichtbar(el: HTMLElement): boolean {
  const r = el.getBoundingClientRect();
  const x = r.left + r.width / 2;
  const y = r.top + Math.min(r.height / 2, 20);
  if (x < 0 || y < 0 || x > window.innerWidth || y > window.innerHeight) return false;
  const oben = document.elementFromPoint(x, y);
  return oben !== null && (el === oben || el.contains(oben));
}

/** The introduction as a panel beside the game: it does not block the screen, the
 * highlighted element stays clickable. */
export function Einfuehrung({
  pfad,
  schritt,
  bericht,
  parameter,
  onSchritt,
  onEnde,
}: {
  pfad: Pfad;
  schritt: number;
  /** The last round report since the introduction started. */
  bericht: Rundenbericht | null;
  /** Values for the texts (company, country). */
  parameter: TextParameter;
  onSchritt: (schritt: number) => void;
  onEnde: () => void;
}) {
  const schritte = PFADE[pfad];
  const s = schritte[schritt]!;
  const letzter = schritt === schritte.length - 1;
  const [rahmen, setRahmen] = useState<Rahmen | null>(null);
  // Which of the step's targets is shown: 0 is the element to use, more is the way.
  const [zielNr, setZielNr] = useState(0);
  const [oben, setOben] = useState(false);
  // Folded to its first line, to see more of the screen (phones).
  const [klein, setKlein] = useState(false);
  // Went back by hand: the step waits for "Weiter" even if it is already done.
  const [zurueck, setZurueck] = useState(false);
  const panel = useRef<HTMLElement>(null);

  const weiter = () => {
    setZurueck(false);
    if (letzter) onEnde();
    else onSchritt(naechster(pfad, schritt, bericht));
  };
  const weiterRef = useRef(weiter);
  useEffect(() => {
    weiterRef.current = weiter;
  });

  // Follows the page: finds the target, moves the highlight with it and notices when
  // the step is done (the views load and change asynchronously, so it looks often).
  useEffect(() => {
    let gesehen = false;
    let gerollt = -1;
    const pruefe = () => {
      if (panel.current) {
        document.documentElement.style.setProperty(
          "--einfuehrung-hoehe",
          `${panel.current.offsetHeight + 16}px`,
        );
      }
      if (!zurueck && s.fertig) {
        const da = s.fertig.some((f) => !f.startsWith("!") && markiert(f));
        const weg = s.fertig
          .filter((f) => f.startsWith("!"))
          .some((f) => {
            if (markiert(f.slice(1))) gesehen = true;
            else return gesehen;
            return false;
          });
        if (da || weg) {
          weiterRef.current();
          return;
        }
      }
      let el: HTMLElement | null = null;
      let nr = 0;
      for (const [i, z] of (s.ziele ?? []).entries()) {
        const e = markiert(z);
        if (e && e.getClientRects().length > 0) {
          el = e;
          nr = i;
          break;
        }
      }
      if (el && gerollt !== nr) {
        // Once per target: into view, also inside a scrolling dialog or tab bar.
        gerollt = nr;
        const r = el.getBoundingClientRect();
        const halb = r.bottom > window.innerHeight || r.right > window.innerWidth;
        if (el.closest(".kopfbereich")) {
          el.scrollIntoView?.({ block: "nearest", inline: "nearest" });
        } else if (!sichtbar(el) || halb) {
          el.scrollIntoView?.({
            block: r.height < window.innerHeight / 2 ? "center" : "start",
            inline: "nearest",
          });
        }
      }
      // Hidden behind a dialog or scrolled away: no highlight and no dimming.
      if (!el || !sichtbar(el)) {
        setRahmen((r) => (r === null ? r : null));
        return;
      }
      const r = el.getBoundingClientRect();
      const neu = {
        top: r.top - ABSTAND,
        left: r.left - ABSTAND,
        width: r.width + 2 * ABSTAND,
        height: r.height + 2 * ABSTAND,
      };
      setRahmen((alt) =>
        alt &&
        Math.abs(alt.top - neu.top) < 0.5 &&
        Math.abs(alt.left - neu.left) < 0.5 &&
        Math.abs(alt.width - neu.width) < 0.5 &&
        Math.abs(alt.height - neu.height) < 0.5
          ? alt
          : neu,
      );
      setZielNr(nr);
      // The panel moves to the top when it would cover the target at the bottom (dialogs
      // and wide screens leave room for it instead).
      const p = panel.current;
      if (p) {
        const schmal = window.innerWidth <= 700;
        const breite = schmal ? window.innerWidth : p.offsetWidth + 16;
        const hoehe = p.offsetHeight + 16;
        const ueberlappt =
          neu.top + neu.height > window.innerHeight - hoehe &&
          neu.left + neu.width > window.innerWidth - breite;
        setOben(ueberlappt && !document.querySelector(".dialog-hintergrund"));
      }
    };
    const raf = requestAnimationFrame(pruefe);
    // On every change of the page (a dialog that opens and closes at once is seen too),
    // and regularly for scrolling and resizing.
    const beobachter = new MutationObserver(pruefe);
    beobachter.observe(document.body, { childList: true, subtree: true });
    const takt = window.setInterval(pruefe, 200);
    return () => {
      cancelAnimationFrame(raf);
      beobachter.disconnect();
      window.clearInterval(takt);
    };
  }, [s, zurueck]);
  useEffect(
    () => () => {
      document.documentElement.style.removeProperty("--einfuehrung-hoehe");
    },
    [],
  );

  const verkauft = umsatz(bericht);
  const key =
    s.key.startsWith("runde") && verkauft !== null && verkauft <= 0
      ? "runde_nochmal"
      : s.key === "bericht"
        ? verkauft !== null && verkauft > 0
          ? "bericht_verkauft"
          : "bericht_nichts"
        : s.key;
  const werte = { ...parameter, umsatz: formatGeld(verkauft ?? 0) };
  // Waits for the player to use the highlighted element (not to close something).
  const aktion = s.fertig !== undefined && !s.fertig.some((f) => f.startsWith("!"));
  const titel = t(`einfuehrung.${key}.titel`, werte);
  return (
    <>
      {rahmen && (
        <div
          className="einfuehrung-rahmen"
          style={{
            top: rahmen.top,
            left: rahmen.left,
            width: rahmen.width,
            height: rahmen.height,
          }}
          aria-hidden="true"
        />
      )}
      <aside
        ref={panel}
        className={`einfuehrung${oben ? " oben" : ""}${klein ? " klein" : ""}`}
        aria-label={t("einfuehrung.titel")}
      >
        <div className="einfuehrung-kopf">
          <span className="gedaempft">
            {t("einfuehrung.schritt", { nummer: schritt + 1, anzahl: schritte.length })}
            {klein && ` · ${titel}`}
          </span>
          <button
            type="button"
            className="schlicht"
            aria-expanded={!klein}
            aria-label={klein ? t("einfuehrung.aufklappen") : t("einfuehrung.zuklappen")}
            onClick={() => setKlein((k) => !k)}
          >
            {klein ? "▴" : "▾"}
          </button>
          <button
            type="button"
            className="schlicht"
            aria-label={t("einfuehrung.beenden")}
            onClick={onEnde}
          >
            ×
          </button>
        </div>
        {!klein && (
          <>
            <div className="einfuehrung-text" aria-live="polite">
              <h2>{titel}</h2>
              <p>{t(`einfuehrung.${key}.text`, werte)}</p>
              {aktion && rahmen && (
                <p className="einfuehrung-tipp">
                  {zielNr === 0 ? t("einfuehrung.tipp_klick") : t("einfuehrung.tipp_weg")}
                </p>
              )}
            </div>
            <div className="knopfreihe">
              <button
                type="button"
                disabled={schritt === 0}
                onClick={() => {
                  setZurueck(true);
                  onSchritt(schritt - 1);
                }}
              >
                {t("einfuehrung.zurueck")}
              </button>
              <button type="button" className={aktion ? undefined : "haupt"} onClick={weiter}>
                {letzter
                  ? t("einfuehrung.fertig")
                  : aktion
                    ? t("einfuehrung.ueberspringen")
                    : t("einfuehrung.weiter")}
              </button>
            </div>
          </>
        )}
      </aside>
    </>
  );
}
