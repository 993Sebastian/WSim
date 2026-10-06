import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useId,
  useRef,
  useState,
  type InputHTMLAttributes,
  type ReactNode,
} from "react";
import { zahlLesen } from "../format";
import type { Befehl, Kern, Uebersicht } from "../kern";
import { t } from "../texte";
import { zahlEingeben, zahlGlaetten, type Eingabeart, type Zahlart } from "../zahleingabe";
import { fehlerText } from "./fehler";

/** Loads a view of the core and reloads it on `neu()` or when `stand` changes. */
export function useSicht<T>(laden: () => Promise<T>, stand: string) {
  const [daten, setDaten] = useState<T | null>(null);
  const [fehler, setFehler] = useState<string | null>(null);
  const [zaehler, setZaehler] = useState(0);
  useEffect(() => {
    let aktiv = true;
    laden()
      .then((d) => {
        if (!aktiv) return;
        setDaten(d);
        setFehler(null);
      })
      .catch((e: unknown) => aktiv && setFehler(fehlerText(e)));
    return () => {
      aktiv = false;
    };
    // `laden` changes with every render; `stand` and the counter decide when to reload.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [stand, zaehler]);
  const neu = useCallback(() => setZaehler((z) => z + 1), []);
  return { daten, fehler, neu };
}

export interface Antwort {
  fehler: boolean;
  text: string;
  /** Where the decision was taken (shown next to that form only). */
  ort: string | null;
}

/** Sends decisions to the core; reports the answer and the new overview. */
export function useBefehl(kern: Kern, onGeaendert: (u: Uebersicht) => void, neu: () => void) {
  const [meldung, setMeldung] = useState<Antwort | null>(null);
  const senden = useCallback(
    async (ort: string | null, erfolg: string | null, befehle: Befehl[]) => {
      let antwort: Antwort;
      try {
        let u: Uebersicht | null = null;
        for (const b of befehle) u = await kern.befehl(b);
        if (u) onGeaendert(u);
        antwort = { fehler: false, text: erfolg ?? t("befehl.ausgefuehrt"), ort };
        neu();
      } catch (e) {
        antwort = { fehler: true, text: fehlerText(e), ort };
      }
      setMeldung(antwort);
      return antwort;
    },
    [kern, onGeaendert, neu],
  );
  const ausfuehren = useCallback(
    async (...befehle: Befehl[]) => !(await senden(null, null, befehle)).fehler,
    [senden],
  );
  return { ausfuehren, senden, meldung };
}

export type Senden = ReturnType<typeof useBefehl>["senden"];

const BefehlKontext = createContext<{ senden: Senden; meldung: Antwort | null } | null>(null);

/** Gives the forms below a view the means to send decisions (see `useAktion`). */
export function Befehle({
  senden,
  meldung,
  children,
}: {
  senden: Senden;
  meldung: Antwort | null;
  children: ReactNode;
}) {
  return <BefehlKontext.Provider value={{ senden, meldung }}>{children}</BefehlKontext.Provider>;
}

/**
 * Decisions of one form: the answer of the core appears next to this form's buttons
 * (docs/BEDIENUNG.md, rule 4), not at the top of the page.
 */
export function useAktion(ort: string) {
  const kontext = useContext(BefehlKontext);
  if (!kontext) throw new Error("useAktion außerhalb von <Befehle>");
  const { senden, meldung } = kontext;
  const los = useCallback(
    async (befehle: Befehl[], erfolg?: string) =>
      !(await senden(ort, erfolg ?? null, befehle)).fehler,
    [senden, ort],
  );
  return { los, antwort: meldung?.ort === ort ? meldung : null };
}

export function Rueckmeldung({ meldung }: { meldung: { fehler: boolean; text: string } | null }) {
  if (!meldung) return null;
  return (
    <p
      className={meldung.fehler ? "fehlertext" : "erfolgstext"}
      role={meldung.fehler ? "alert" : "status"}
    >
      {meldung.text}
    </p>
  );
}

/** A number field's text changed: the new text and the number it means (null: none). */
export type ZahlMeldung = (text: string, zahl: number | null) => void;

/**
 * The input of a number field for German input: it sets thousands separators while one
 * types ("200.000.000", see `zahlEingeben`) and accepts the comma for decimals where
 * the field allows them. The parent keeps the text; `onWert` also gets the number.
 */
export function ZahlEingabe({
  wert,
  onWert,
  ganzzahlig = false,
  negativ = false,
  gruppieren = true,
  onBlur,
  ...rest
}: Zahlart & {
  wert: string;
  onWert: ZahlMeldung;
} & Omit<
    InputHTMLAttributes<HTMLInputElement>,
    "value" | "defaultValue" | "onChange" | "type" | "inputMode"
  >) {
  const feld = useRef<HTMLInputElement>(null);
  // The selection an edit replaces, as the browser reports it just before the edit.
  const vorher = useRef<Eingabeart | null>(null);
  useEffect(() => {
    const el = feld.current;
    if (!el) return;
    const merken = (e: InputEvent) => {
      vorher.current = {
        auswahl: [el.selectionStart ?? 0, el.selectionEnd ?? 0],
        typ: e.inputType,
      };
    };
    el.addEventListener("beforeinput", merken);
    return () => el.removeEventListener("beforeinput", merken);
  }, []);
  const art: Zahlart = { ganzzahlig, negativ, gruppieren };
  const melden = (text: string) => onWert(text, zahlLesen(text));
  return (
    <input
      type="text"
      // Phone keyboards: digits, with the decimal separator where decimals are allowed
      // (their number pads have no minus).
      inputMode={negativ ? "text" : ganzzahlig ? "numeric" : "decimal"}
      autoComplete="off"
      aria-invalid={(wert.trim() !== "" && zahlLesen(wert) === null) || undefined}
      {...rest}
      ref={feld}
      value={wert}
      onChange={(e) => {
        const el = e.currentTarget;
        const nativ = e.nativeEvent as Partial<InputEvent>;
        // Only the record of this very edit (the same kind of input) is used.
        const eingabe: Eingabeart =
          vorher.current && vorher.current.typ === nativ.inputType
            ? vorher.current
            : { typ: nativ.inputType };
        vorher.current = null;
        // While an input method composes, the text stays as it is (shaped on leaving).
        if (nativ.isComposing) {
          melden(el.value);
          return;
        }
        const neu = zahlEingeben(
          wert,
          el.value,
          el.selectionStart ?? el.value.length,
          art,
          eingabe,
        );
        // Set here, before React renders the same text, so that the caret stays put.
        if (el.value !== neu.text) el.value = neu.text;
        if (document.activeElement === el) el.setSelectionRange(neu.cursor, neu.cursor);
        melden(neu.text);
      }}
      onBlur={(e) => {
        const glatt = zahlGlaetten(wert, art);
        if (glatt !== wert) melden(glatt);
        onBlur?.(e);
      }}
    />
  );
}

/**
 * A labelled number field (see `ZahlEingabe`), with its unit next to it and an optional
 * explanation below.
 */
export function ZahlFeld({
  name,
  einheit,
  wert,
  onWert,
  hilfe,
  breit = false,
  ...art
}: Zahlart & {
  name: string;
  einheit?: string;
  wert: string;
  onWert: ZahlMeldung;
  hilfe?: ReactNode;
  breit?: boolean;
}) {
  const id = useId();
  const ungueltig = wert.trim() !== "" && zahlLesen(wert) === null;
  return (
    <div className="feld">
      <label htmlFor={id}>{name}</label>
      <span className="feld-eingabe">
        <ZahlEingabe
          id={id}
          className={breit ? undefined : "schmal"}
          wert={wert}
          onWert={onWert}
          aria-describedby={hilfe ? `${id}-hilfe` : undefined}
          {...art}
        />
        {einheit && <span className="einheit">{einheit}</span>}
      </span>
      {hilfe && (
        <small id={`${id}-hilfe`} className="feld-hilfe">
          {hilfe}
        </small>
      )}
      {ungueltig && <small className="fehlertext">{t("feld.keine_zahl")}</small>}
    </div>
  );
}

/** Sub-tabs of a view (e.g. the areas of a plant). */
export function Unterreiter<K extends string>({
  name,
  bereiche,
  aktiv,
  onWahl,
}: {
  name: string;
  bereiche: { key: K; text: string; zaehler?: number }[];
  aktiv: K;
  onWahl: (k: K) => void;
}) {
  return (
    <nav className="unterreiter" aria-label={name}>
      {bereiche.map((b) => (
        <button
          key={b.key}
          type="button"
          aria-current={aktiv === b.key ? "page" : undefined}
          data-tour={`bereich-${b.key}`}
          onClick={() => onWahl(b.key)}
        >
          {b.text}
          {b.zaehler ? <span className="zaehler">{b.zaehler}</span> : null}
        </button>
      ))}
    </nav>
  );
}

/**
 * "Where does this value come from?" (M27): a small ⓘ that opens the parts of a value.
 * A details element, so it works with the keyboard and on a phone alike.
 */
export function Erklaerung({ wert, children }: { wert: string; children: ReactNode }) {
  // Drawn only while open: closed, its texts would stand twice on the page.
  const [offen, setOffen] = useState(false);
  return (
    <details className="erklaerung-knopf" onToggle={(e) => setOffen(e.currentTarget.open)}>
      <summary aria-label={t("erklaerung.wie", { wert })} title={t("erklaerung.wie", { wert })}>
        ⓘ
      </summary>
      {offen && (
        <div className="erklaerung-inhalt" role="note" aria-label={t("erklaerung.wie", { wert })}>
          {children}
        </div>
      )}
    </details>
  );
}
