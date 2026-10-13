// Chronik (MG3): all decision files of the company with their reviews – who recommended
// what, what the player chose, what it brought and what the other way would have brought.
import { useMemo, useState } from "react";
import { formatDatum, formatGeld, landName } from "../format";
import type { ChronikEintrag, Kern } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { useSicht } from "./gemeinsam";

/** The decision a file records, e.g. "So lassen statt Stilllegen". */
export function entscheidungText(e: ChronikEintrag): string {
  return e.against
    ? t("chronik.statt", {
        gewaehlt: t(`option.${e.chosen}`),
        empfohlen: t(`option.${e.recommended}`),
      })
    : t(`option.${e.chosen}`);
}

export function urteilText(e: ChronikEintrag): string {
  if (e.hit === null) return t("chronik.offen", { datum: formatDatum(e.due) });
  return e.hit ? t("chronik.treffer") : t("chronik.fehlschlag");
}

function Zeile({ e }: { e: ChronikEintrag }) {
  const ort = e.site_name ?? landName(e.country);
  return (
    <tr className={e.against ? "gegen-empfehlung" : undefined}>
      <td>{formatDatum(e.created)}</td>
      <td>{t(`thema.${e.topic}`)}</td>
      <td>{ort}</td>
      <td>{e.manager}</td>
      <td>{entscheidungText(e)}</td>
      <td className="zahl">{e.forecast_usd == null ? "–" : formatGeld(e.forecast_usd)}</td>
      <td className="zahl">{e.actual_usd == null ? "–" : formatGeld(e.actual_usd)}</td>
      <td className="zahl">{e.alternative_usd == null ? "–" : formatGeld(e.alternative_usd)}</td>
      <td>{urteilText(e)}</td>
    </tr>
  );
}

/** Chronicle of the decisions, filterable by topic, manager, period and against the advice. */
export function ChronikAnsicht({ kern, stand }: { kern: Kern; stand: string }) {
  const { daten, fehler } = useSicht(() => kern.organisation(), stand);
  const [thema, setThema] = useState("");
  const [manager, setManager] = useState("");
  const [zeitraum, setZeitraum] = useState("");
  const [gegen, setGegen] = useState(false);
  const eintraege = useMemo(() => daten?.chronicle ?? [], [daten]);
  const themen = useMemo(() => [...new Set(eintraege.map((e) => e.topic))].sort(), [eintraege]);
  const manager_liste = useMemo(
    () => [...new Map(eintraege.map((e) => [e.manager_id, e.manager])).entries()],
    [eintraege],
  );
  if (!daten) return <FehlerText fehler={fehler} />;
  const heute = eintraege[0]?.created ?? "";
  const grenze =
    zeitraum && heute
      ? new Date(Date.parse(heute) - Number(zeitraum) * 31 * 864e5).toISOString().slice(0, 10)
      : "";
  const gefiltert = eintraege.filter(
    (e) =>
      (!thema || e.topic === thema) &&
      (!manager || String(e.manager_id) === manager) &&
      (!gegen || e.against) &&
      (!grenze || e.created >= grenze),
  );
  return (
    <section aria-labelledby="chronik-titel">
      <h2 id="chronik-titel">{t("chronik.titel")}</h2>
      <p className="feld-hilfe">{t("chronik.hilfe")}</p>
      <div className="formular-zeile">
        <div className="feld">
          <label htmlFor="chronik-thema">{t("chronik.thema")}</label>
          <select id="chronik-thema" value={thema} onChange={(e) => setThema(e.target.value)}>
            <option value="">{t("chronik.alle")}</option>
            {themen.map((k) => (
              <option key={k} value={k}>
                {t(`thema.${k}`)}
              </option>
            ))}
          </select>
        </div>
        <div className="feld">
          <label htmlFor="chronik-manager">{t("chronik.manager")}</label>
          <select id="chronik-manager" value={manager} onChange={(e) => setManager(e.target.value)}>
            <option value="">{t("chronik.alle")}</option>
            {manager_liste.map(([id, name]) => (
              <option key={id} value={String(id)}>
                {name}
              </option>
            ))}
          </select>
        </div>
        <div className="feld">
          <label htmlFor="chronik-zeit">{t("chronik.zeitraum")}</label>
          <select id="chronik-zeit" value={zeitraum} onChange={(e) => setZeitraum(e.target.value)}>
            <option value="">{t("chronik.alle")}</option>
            <option value="12">{t("chronik.letzte_monate", { monate: 12 })}</option>
            <option value="24">{t("chronik.letzte_monate", { monate: 24 })}</option>
          </select>
        </div>
        <label className="schalter">
          <input type="checkbox" checked={gegen} onChange={(e) => setGegen(e.target.checked)} />{" "}
          {t("chronik.gegen")}
        </label>
      </div>
      {gefiltert.length === 0 ? (
        <p>{t("chronik.leer")}</p>
      ) : (
        <div className="tabelle">
          <table aria-label={t("chronik.titel")}>
            <thead>
              <tr>
                <th>{t("chronik.datum")}</th>
                <th>{t("chronik.thema")}</th>
                <th>{t("chronik.ort")}</th>
                <th>{t("chronik.manager")}</th>
                <th>{t("chronik.entscheidung")}</th>
                <th className="zahl">{t("chronik.prognose")}</th>
                <th className="zahl">{t("chronik.ist")}</th>
                <th className="zahl">{t("chronik.alternative")}</th>
                <th>{t("chronik.urteil")}</th>
              </tr>
            </thead>
            <tbody>
              {gefiltert.map((e) => (
                <Zeile key={`${e.created}/${e.site}/${e.topic}/${e.manager_id}`} e={e} />
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}
