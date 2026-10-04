import { useCallback, useEffect, useState } from "react";
import type { Befehl, Kern, Uebersicht } from "../kern";
import { t } from "../texte";
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

/** Sends decisions to the core; reports the answer and the new overview. */
export function useBefehl(kern: Kern, onGeaendert: (u: Uebersicht) => void, neu: () => void) {
  const [meldung, setMeldung] = useState<{ fehler: boolean; text: string } | null>(null);
  const ausfuehren = useCallback(
    async (...befehle: Befehl[]) => {
      try {
        let u: Uebersicht | null = null;
        for (const b of befehle) u = await kern.befehl(b);
        if (u) onGeaendert(u);
        setMeldung({ fehler: false, text: t("befehl.ausgefuehrt") });
        neu();
        return true;
      } catch (e) {
        setMeldung({ fehler: true, text: fehlerText(e) });
        return false;
      }
    },
    [kern, onGeaendert, neu],
  );
  return { ausfuehren, meldung };
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
