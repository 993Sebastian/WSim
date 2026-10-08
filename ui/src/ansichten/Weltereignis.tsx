import { meldungText, parameterAnzeige } from "../format";
import type { Meldung } from "../kern";
import { t } from "../texte";
import { Dialog } from "./Dialog";

/** The lines naming the effects of a world event (H1), from the same report. */
export function folgenVon(meldungen: Meldung[], ereignis: Meldung): Meldung[] {
  const name = ereignis.params.ereignis;
  if (name?.type !== "text_key") return [];
  return meldungen.filter((m) => {
    const p = m.params.ereignis;
    return m.key.startsWith("meldung.folge.") && p?.type === "text_key" && p.value === name.value;
  });
}

/** World news shown as a window of its own before the round report. */
export function WeltereignisDialog({
  meldung,
  folgen = [],
  nummer,
  anzahl,
  onWeiter,
  onAlle,
}: {
  meldung: Meldung;
  /** Its effects, one line each (H1). */
  folgen?: Meldung[];
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
      : folgen.length > 0
        ? "weltereignis.hinweis_folgen"
        : "weltereignis.hinweis_laenderwerte";
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
      {folgen.length > 0 && (
        <>
          <h3>{t("weltereignis.folgen")}</h3>
          <ul className="folgen">
            {folgen.map((f, i) => (
              <li key={i}>{meldungText(f)}</li>
            ))}
          </ul>
        </>
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
