import { t } from "../texte";
import { Dialog } from "./Dialog";

/** All keyboard shortcuts of the game screen (see `Spiel`). */
export const TASTEN: [string, string][] = [
  ["Strg + Enter", "tasten.runde"],
  ["1 … 7", "tasten.ansichten"],
  ["Strg + S", "tasten.speichern"],
  ["Strg + O", "tasten.laden"],
  ["? / F1", "tasten.hilfe"],
  ["Esc", "tasten.schliessen"],
];

export function Tastenhilfe({
  onSchliessen,
  onEinfuehrung,
}: {
  onSchliessen: () => void;
  onEinfuehrung: () => void;
}) {
  return (
    <Dialog titel={t("tasten.titel")} onSchliessen={onSchliessen}>
      <dl className="tasten">
        {TASTEN.map(([taste, text]) => (
          <div key={taste}>
            <dt>
              <kbd>{taste}</kbd>
            </dt>
            <dd>{t(text)}</dd>
          </div>
        ))}
      </dl>
      <div className="knopfreihe">
        <button type="button" onClick={onEinfuehrung}>
          {t("einfuehrung.starten")}
        </button>
        <button type="button" className="haupt" onClick={onSchliessen}>
          {t("dialog.schliessen")}
        </button>
      </div>
    </Dialog>
  );
}
