import { parameterAnzeige } from "../format";
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
  return (
    <Dialog titel={parameterAnzeige(meldung, "ereignis")} onSchliessen={onWeiter} tour="ereignis">
      <p className="ereignis-kopf">
        <span className="marke">{parameterAnzeige(meldung, "art")}</span>
        <time>{parameterAnzeige(meldung, "datum")}</time>
        {anzahl > 1 && (
          <span className="gedaempft">{t("weltereignis.zaehler", { nummer, anzahl })}</span>
        )}
      </p>
      <p className="ereignis-text">{parameterAnzeige(meldung, "beschreibung")}</p>
      {laender && (
        <p className="gedaempft">
          <strong>{t("weltereignis.laender")}:</strong> {laender}
        </p>
      )}
      <p className="hinweis-links">{t("weltereignis.hinweis_stufe1")}</p>
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
