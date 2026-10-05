import { useEffect, useState, type ChangeEvent, type FormEvent } from "react";
import { formatDatum } from "../format";
import type { Kern, Spielstand, Uebersicht } from "../kern";
import { t } from "../texte";
import { Dialog, FehlerText } from "./Dialog";
import { fehlerText } from "./fehler";

export function SpeichernDialog({
  kern,
  vorschlag,
  onSchliessen,
}: {
  kern: Kern;
  vorschlag: string;
  onSchliessen: () => void;
}) {
  const [name, setName] = useState(vorschlag);
  const [fehler, setFehler] = useState<string | null>(null);
  const [gespeichert, setGespeichert] = useState<string | null>(null);

  const speichern = async (e: FormEvent) => {
    e.preventDefault();
    setFehler(null);
    try {
      const stand = await kern.speichern(name);
      setGespeichert(stand.name);
    } catch (err) {
      setFehler(fehlerText(err));
    }
  };

  return (
    <Dialog titel={t("speichern.titel")} onSchliessen={onSchliessen}>
      <form className="formular" onSubmit={speichern}>
        <label>
          {t("speichern.name")}
          <input
            id="spielstand_name"
            value={name}
            maxLength={60}
            onChange={(e) => setName(e.target.value)}
          />
        </label>
        <FehlerText fehler={fehler} />
        {gespeichert && (
          <p className="erfolgstext" role="status">
            {t("speichern.gespeichert", { name: gespeichert })}
          </p>
        )}
        <div className="knopfreihe">
          <button type="button" onClick={onSchliessen}>
            {t("dialog.schliessen")}
          </button>
          <button type="submit" className="haupt">
            {t("speichern.speichern")}
          </button>
        </div>
      </form>
    </Dialog>
  );
}

export function LadenDialog({
  kern,
  onGeladen,
  onSchliessen,
}: {
  kern: Kern;
  onGeladen: (u: Uebersicht) => void;
  onSchliessen: () => void;
}) {
  const [staende, setStaende] = useState<Spielstand[] | null>(null);
  const [fehler, setFehler] = useState<string | null>(null);

  useEffect(() => {
    let aktiv = true;
    kern
      .spielstaende()
      .then((s) => aktiv && setStaende(s))
      .catch((e: unknown) => aktiv && setFehler(fehlerText(e)));
    return () => {
      aktiv = false;
    };
  }, [kern]);

  const laden = async (name: string) => {
    setFehler(null);
    try {
      onGeladen(await kern.laden(name));
    } catch (err) {
      setFehler(fehlerText(err));
    }
  };

  // Browser version: saves as files, to move them between devices.
  const dateien = kern.spielstandDatei !== undefined && kern.spielstandEinlesen !== undefined;
  const [hinweis, setHinweis] = useState<string | null>(null);

  const herunterladen = async (name: string) => {
    setFehler(null);
    try {
      const bytes = await kern.spielstandDatei!(name);
      const url = URL.createObjectURL(
        new Blob([bytes as BlobPart], { type: "application/octet-stream" }),
      );
      const a = document.createElement("a");
      a.href = url;
      a.download = `${name}.wsim`;
      a.click();
      setTimeout(() => URL.revokeObjectURL(url), 1000);
    } catch (err) {
      setFehler(fehlerText(err));
    }
  };

  const hochladen = async (e: ChangeEvent<HTMLInputElement>) => {
    const datei = e.target.files?.[0];
    e.target.value = "";
    if (!datei) return;
    setFehler(null);
    // Names of saves allow letters, digits, spaces, points, hyphens and underscores.
    const name =
      datei.name
        .replace(/\.wsim$/i, "")
        .replace(/[^\p{L}\p{N} ._-]/gu, "_")
        .slice(0, 60)
        .trim() || "Hochgeladen";
    try {
      const bytes = new Uint8Array(await datei.arrayBuffer());
      setStaende(await kern.spielstandEinlesen!(name, bytes));
      setHinweis(t("laden.hochgeladen", { name }));
    } catch (err) {
      setFehler(fehlerText(err));
    }
  };

  return (
    <Dialog titel={t("laden.titel")} onSchliessen={onSchliessen}>
      {staende && staende.length === 0 && <p>{t("laden.keine")}</p>}
      {staende && staende.length > 0 && (
        <ul className="liste">
          {staende.map((s) => (
            <li key={s.name}>
              <span>
                <strong>{s.name}</strong>
                <small>{t("laden.stand", { firma: s.company, datum: formatDatum(s.date) })}</small>
              </span>
              <span className="knopfreihe">
                {dateien && (
                  <button
                    type="button"
                    className="schlicht"
                    aria-label={t("laden.datei_von", { name: s.name })}
                    onClick={() => void herunterladen(s.name)}
                  >
                    {t("laden.datei")}
                  </button>
                )}
                <button type="button" onClick={() => laden(s.name)}>
                  {t("laden.laden")}
                </button>
              </span>
            </li>
          ))}
        </ul>
      )}
      {dateien && (
        <div className="feld hochladen">
          <label htmlFor="spielstand_datei">{t("laden.hochladen")}</label>
          <input
            id="spielstand_datei"
            type="file"
            accept=".wsim,application/octet-stream"
            onChange={(e) => void hochladen(e)}
          />
          <small className="feld-hilfe">{t("laden.hochladen_hilfe")}</small>
        </div>
      )}
      {hinweis && (
        <p className="erfolgstext" role="status">
          {hinweis}
        </p>
      )}
      <FehlerText fehler={fehler} />
    </Dialog>
  );
}
