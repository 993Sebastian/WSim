import { useState } from "react";
import { formatZahl, landName } from "../format";
import type { Kern, Uebersicht } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { Rueckmeldung, useBefehl, useSicht } from "./gemeinsam";
import { LAENDER } from "./laender";

export function ForschungAnsicht({
  kern,
  uebersicht,
  onGeaendert,
}: {
  kern: Kern;
  uebersicht: Uebersicht;
  onGeaendert: (u: Uebersicht) => void;
}) {
  const { daten, fehler, neu } = useSicht(() => kern.forschung(), uebersicht.date);
  const { ausfuehren, meldung } = useBefehl(kern, onGeaendert, neu);
  const [land, setLand] = useState(uebersicht.company.headquarters);
  const [feld, setFeld] = useState("");
  if (!daten) return <FehlerText fehler={fehler} />;
  const felder = [...new Set(daten.technologies.map((x) => x.field))];
  const erforschbar = daten.technologies.filter((x) => x.researchable);
  const technologien = daten.technologies.filter((x) => !feld || x.field === feld);
  return (
    <main className="ansicht" id="forschung">
      <h1 className="unsichtbar">{t("ansicht.forschung")}</h1>
      <Rueckmeldung meldung={meldung} />
      <section aria-labelledby="zentren">
        <h2 id="zentren">{t("forschung.zentren")}</h2>
        {daten.centers.length === 0 && <p className="gedaempft">{t("forschung.keine_zentren")}</p>}
        <ul className="liste">
          {daten.centers.map((z) => (
            <li key={z.site}>
              <strong>{landName(z.country)}</strong>{" "}
              <span className="gedaempft">
                {z.ready
                  ? t("forschung.forscher", { anzahl: formatZahl(z.researchers, 1) })
                  : t("forschung.ohne_labor")}
              </span>{" "}
              <select
                aria-label={t("forschung.projekt_von", { land: landName(z.country) })}
                value={z.project ?? ""}
                disabled={!z.ready}
                onChange={(e) =>
                  void ausfuehren({
                    SetResearch: { site: z.site, technology: e.target.value || null },
                  })
                }
              >
                <option value="">{t("forschung.kein_projekt")}</option>
                {erforschbar.map((x) => (
                  <option key={x.key} value={x.key}>
                    {t(`technologie.${x.key}`)}
                  </option>
                ))}
              </select>
              {!z.ready && daten.laboratory && (
                <button
                  type="button"
                  className="schlicht"
                  onClick={() =>
                    void ausfuehren({
                      BuildFacility: { site: z.site, facility: daten.laboratory!, count: 1 },
                    })
                  }
                >
                  {t("forschung.labor_bauen")}
                </button>
              )}
            </li>
          ))}
        </ul>
        <form
          className="zeilenformular"
          aria-label={t("forschung.gruenden")}
          onSubmit={(e) => {
            e.preventDefault();
            void ausfuehren({ FoundSite: { country: land, kind: "ResearchCenter" } });
          }}
        >
          <label>
            {t("forschung.gruenden")}
            <select id="forschung_land" value={land} onChange={(e) => setLand(e.target.value)}>
              {LAENDER.map((k) => (
                <option key={k} value={k}>
                  {landName(k)}
                </option>
              ))}
            </select>
          </label>
          <button type="submit">{t("produktion.gruenden_knopf")}</button>
        </form>
      </section>

      <section aria-labelledby="technologien">
        <h2 id="technologien">{t("forschung.technologien")}</h2>
        <label>
          {t("forschung.feld")}{" "}
          <select id="forschung_feld" value={feld} onChange={(e) => setFeld(e.target.value)}>
            <option value="">{t("forschung.alle_felder")}</option>
            {felder.map((f) => (
              <option key={f} value={f}>
                {t(`fachrichtung.${f}`)}
              </option>
            ))}
          </select>
        </label>
        <div className="tabelle">
          <table>
            <thead>
              <tr>
                <th>{t("forschung.technologie")}</th>
                <th className="zahl">{t("forschung.erfindung")}</th>
                <th>{t("forschung.stand")}</th>
                <th>{t("forschung.voraussetzungen")}</th>
                <th>{t("forschung.ermoeglicht")}</th>
              </tr>
            </thead>
            <tbody>
              {technologien.map((x) => (
                <tr key={x.key} className={x.known ? "bekannt" : undefined}>
                  <td>{t(`technologie.${x.key}`)}</td>
                  <td className="zahl">{x.invention_year}</td>
                  <td>
                    {x.known
                      ? t("forschung.bekannt")
                      : x.needed !== null
                        ? t("forschung.punkte", {
                            punkte: formatZahl(x.points, 0),
                            bedarf: formatZahl(x.needed, 0),
                          })
                        : t("forschung.gesperrt")}
                  </td>
                  <td>{x.prerequisites.map((v) => t(`technologie.${v}`)).join(", ") || "–"}</td>
                  <td>{x.opens.map((o) => t(o)).join(", ") || "–"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
    </main>
  );
}
