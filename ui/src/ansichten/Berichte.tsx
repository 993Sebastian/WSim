import { useState } from "react";
import { formatDatum, formatGeld, meldungText } from "../format";
import type { Rundenbericht } from "../kern";
import { t } from "../texte";

const GRUPPEN = ["alle", "warnung", "welt", "wettbewerb", "forschung", "allgemein"] as const;

/** Archive of the round reports of this session, newest first. */
export function BerichteAnsicht({
  berichte,
  onOeffnen,
}: {
  berichte: Rundenbericht[];
  onOeffnen: (b: Rundenbericht) => void;
}) {
  const [gruppe, setGruppe] = useState<(typeof GRUPPEN)[number]>("alle");
  return (
    <main className="ansicht" id="berichte">
      <h1 className="unsichtbar">{t("ansicht.berichte")}</h1>
      <div className="zeilenformular">
        <label>
          {t("berichte.gruppe")}
          <select
            id="berichte_gruppe"
            value={gruppe}
            onChange={(e) => setGruppe(e.target.value as (typeof GRUPPEN)[number])}
          >
            {GRUPPEN.map((g) => (
              <option key={g} value={g}>
                {t(`berichte.gruppe.${g}`)}
              </option>
            ))}
          </select>
        </label>
        <span className="gedaempft">{t("berichte.hinweis")}</span>
      </div>
      {berichte.length === 0 && <p className="gedaempft">{t("berichte.leer")}</p>}
      {berichte.map((b) => {
        const meldungen = b.messages.filter((m) => gruppe === "alle" || m.group === gruppe);
        return (
          <article key={b.to} className="standort" aria-label={formatDatum(b.to)}>
            <h2>
              {formatDatum(b.from)} – {formatDatum(b.to)}
              <small>{t("berichte.ergebnis", { betrag: formatGeld(b.period.result_usd) })}</small>
              <button type="button" className="schlicht" onClick={() => onOeffnen(b)}>
                {t("berichte.oeffnen")}
              </button>
            </h2>
            {meldungen.length === 0 ? (
              <p className="gedaempft">{t("berichte.keine_meldungen")}</p>
            ) : (
              <ul className="meldungen">
                {meldungen.map((m, i) => (
                  <li key={i} className={`meldung meldung-${m.kind}`}>
                    {meldungText(m)}
                  </li>
                ))}
              </ul>
            )}
          </article>
        );
      })}
    </main>
  );
}
