import { meldungText, parameterAnzeige } from "../format";
import type { Meldung } from "../kern";
import { t } from "../texte";
import { Dialog } from "./Dialog";

/** World news shown as a window of its own before the round report. */
export function WeltereignisDialog({
  meldung,
  nummer,
  anzahl,
  onWeiter,
  onAlle,
}: {
  meldung: Meldung;
  nummer: number;
  anzahl: number;
  onWeiter: () => void;
  onAlle: () => void;
}) {
  const laender = parameterAnzeige(meldung, "laender");
  const datum = parameterAnzeige(meldung, "datum");
  // Historical events bring a description; a change of currency is its own text.
  const text = parameterAnzeige(meldung, "beschreibung") || meldungText(meldung);
  const hinweis =
    meldung.key === "meldung.waehrungsreform"
      ? "weltereignis.hinweis_waehrung"
      : "weltereignis.hinweis_stufe1";
  return (
    <Dialog titel={parameterAnzeige(meldung, "ereignis")} onSchliessen={onWeiter} tour="ereignis">
      <p className="ereignis-kopf">
        <span className="marke">{parameterAnzeige(meldung, "art")}</span>
        {datum && <time>{datum}</time>}
        {anzahl > 1 && (
          <span className="gedaempft">{t("weltereignis.zaehler", { nummer, anzahl })}</span>
        )}
      </p>
      <p className="ereignis-text">{text}</p>
      {laender && (
        <p className="gedaempft">
          <strong>{t("weltereignis.laender")}:</strong> {laender}
        </p>
      )}
      <p className="hinweis-links">{t(hinweis)}</p>
      <div className="knopfreihe">
        {anzahl > nummer && (
          <button type="button" onClick={onAlle}>
            {t("weltereignis.alle_ueberspringen")}
          </button>
        )}
        <button type="button" className="haupt" onClick={onWeiter} data-tour="ereignis-weiter">
          {t("bericht.weiter")}
        </button>
      </div>
    </Dialog>
  );
}
