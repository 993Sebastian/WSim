import { useEffect, useState, type FormEvent } from "react";
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
              <button type="button" onClick={() => laden(s.name)}>
                {t("laden.laden")}
              </button>
            </li>
          ))}
        </ul>
      )}
      <FehlerText fehler={fehler} />
    </Dialog>
  );
}
