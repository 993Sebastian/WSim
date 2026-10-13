import { useEffect, useState } from "react";
import type { KernStatus } from "../App";
import type { Kern, Optionen, Uebersicht } from "../kern";
import { t } from "../texte";
import { LadenDialog } from "./SpeichernLaden";

export function Hauptmenue({
  kern,
  status,
  laufend,
  onNeu,
  onSpielen,
}: {
  kern: Kern;
  status: KernStatus;
  laufend: Uebersicht | null;
  onNeu: () => void;
  onSpielen: (u: Uebersicht) => void;
}) {
  const [laden, setLaden] = useState(false);
  const bereit = status.art === "bereit";
  const [optionen, setOptionen] = useState<Optionen | null>(null);
  useEffect(() => {
    if (!bereit) return;
    let aktiv = true;
    kern
      .optionen()
      .then((o) => aktiv && setOptionen(o))
      .catch(() => undefined);
    return () => {
      aktiv = false;
    };
  }, [bereit, kern]);
  const aufbau = optionen?.market_build_up;

  return (
    <main className="start">
      <h1>{t("app.titel")}</h1>
      <p className="untertitel">{t("app.untertitel")}</p>
      <nav className="menue" aria-label={t("menue.hauptmenue")}>
        {laufend && (
          <button type="button" className="haupt" onClick={() => onSpielen(laufend)}>
            {t("menue.fortsetzen")}
          </button>
        )}
        <button type="button" className={laufend ? "" : "haupt"} onClick={onNeu} disabled={!bereit}>
          {t("menue.neues_spiel")}
        </button>
        <button type="button" onClick={() => setLaden(true)} disabled={!bereit}>
          {t("menue.laden")}
        </button>
      </nav>
      <p className={`kernstatus kernstatus-${status.art}`} role="status">
        {kernStatusText(status, kern.echt)}
      </p>
      {!kern.echt && <p className="hinweis">{t("menue.vorschau_hinweis")}</p>}
      {aufbau && (
        <p className="hinweis" data-testid="marktaufbau">
          {t("menue.marktaufbau", {
            jahr: aufbau.from_year,
            anteil: Math.round(aufbau.start_share * 100),
            jahre: Math.round(aufbau.years),
            bis: aufbau.from_year + Math.round(aufbau.years),
          })}
        </p>
      )}
      {laden && (
        <LadenDialog
          kern={kern}
          onGeladen={(u) => {
            setLaden(false);
            onSpielen(u);
          }}
          onSchliessen={() => setLaden(false)}
        />
      )}
    </main>
  );
}

function kernStatusText(status: KernStatus, echt: boolean): string {
  switch (status.art) {
    case "laedt":
      return t("kern.laedt");
    case "bereit":
      return echt ? t("kern.bereit", { version: status.version }) : t("kern.vorschau");
    case "fehler":
      return t("kern.fehler", { fehler: status.fehler });
  }
}
