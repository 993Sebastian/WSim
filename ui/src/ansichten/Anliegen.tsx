// Concerns (MA2, docs/BEDIENUNG.md): what the player's positions ask beyond their budget.
// "What does my management want from me, what does it recommend, and what does each way
// cost and bring?"
import { formatDatum, formatGeld, landName, meldungText } from "../format";
import type { Anliegen, AnliegenGruppe, AnliegenOption, Anliegenantwort, Kern } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { Rueckmeldung, useAktion, useSicht } from "./gemeinsam";
import { einheitName, stellenName } from "./stellen";

/** Where the answers appear: at the top of the list (an answered concern leaves it). */
const ORT = "anliegen";

/** An effect with its sign: "+1.200 $". */
function mitVorzeichen(usd: number): string {
  return usd > 0 ? `+${formatGeld(usd)}` : formatGeld(usd);
}

function wirkung(o: AnliegenOption): string {
  // Keeping things as they are is what the forecasts are measured against.
  if (o.kind === "beibehalten") return t("anliegen.unveraendert");
  if (!o.forecast_usd) return t("anliegen.keine_schaetzung");
  const [von, bis] = o.forecast_usd;
  return t("anliegen.spanne", { von: mitVorzeichen(von), bis: mitVorzeichen(bis) });
}

/** "Werksleitung · Werk · Deutschland", "Landesleitung · Land · Deutschland" */
export function herkunft(a: Anliegen): string {
  return `${stellenName(a.asker.role, a.asker.kind_text)} · ${einheitName(a.asker)}`;
}

/** The way of a concern (MA3): each position with its recommendation. */
function weg(a: Anliegen): string {
  return a.path
    .map(
      (h) =>
        `${stellenName(h.position.role, h.position.kind_text)} (${einheitName(h.position)}, ${t("anliegen.empfiehlt", { option: t(`option.${h.recommended}`) })})`,
    )
    .join(" → ");
}

/** Where an investment budget is set (MA4): "ganze Firma", "Europa", "Werk · Deutschland". */
function budgetOrt(a: Anliegen): string {
  const [ebene, wert = ""] = (a.strategy_scope ?? "").split(":");
  if (ebene === "firma") return t("strategie.ganze_firma");
  if (ebene === "kontinent") return t(`kontinent.${wert}`);
  if (ebene === "land") return landName(wert);
  if (ebene === "standort" && a.site_kind_text && a.site_country) {
    return `${t(a.site_kind_text)} · ${landName(a.site_country)}`;
  }
  return "";
}

/** "Überkapazität" or "Einkauf · Roheisen". */
function thema(a: Anliegen): string {
  const text = t(`thema.${a.topic}`);
  return a.product ? `${text} · ${t(`produkt.${a.product}`)}` : text;
}

function optionName(a: Anliegen, i: number): string {
  const o = a.options[i];
  return o ? t(`option.${o.kind}`) : "";
}

function AnliegenKarte({ a, ruhetage }: { a: Anliegen; ruhetage: number }) {
  const { los } = useAktion(ORT);
  const antworten = (answer: Anliegenantwort, erfolg: string) =>
    void los([{ AnswerConcern: { concern: a.id, answer } }], erfolg);
  const stelle = stellenName(a.asker.role, a.asker.kind_text);
  return (
    <article className="karte anliegen" aria-label={`${thema(a)} – ${herkunft(a)}`}>
      <h3>
        {thema(a)}
        {a.important && <span className="marke">{t("anliegen.wichtig")}</span>}
      </h3>
      <p className="feld-hilfe">
        {herkunft(a)} · {t("anliegen.fragt", { name: a.manager })} ·{" "}
        <strong>{t("anliegen.frist", { datum: formatDatum(a.deadline) })}</strong>
      </p>
      {a.site_kind_text && a.site_country && a.asker.level !== "standort" && (
        <p className="feld-hilfe">
          {t("anliegen.ort", { ort: `${t(a.site_kind_text)} · ${landName(a.site_country)}` })}
        </p>
      )}
      {a.path.length > 1 && <p className="feld-hilfe">{t("anliegen.weg", { weg: weg(a) })}</p>}
      <p>
        {t(`anliegen.grund.${a.reason}`, {
          budget: formatGeld(a.per_decision_usd),
          rest: formatGeld(
            a.reason === "investition" || a.reason === "verschuldung" || a.reason === "beteiligung"
              ? (a.strategy_limit_usd ?? 0)
              : a.left_usd,
          ),
          reserve: formatGeld(a.strategy_limit_usd ?? 0),
          grenze: formatGeld(a.strategy_limit_usd ?? 0),
          wo: budgetOrt(a),
        })}{" "}
        {a.site_result_usd !== null &&
          t("anliegen.lage", { betrag: formatGeld(a.site_result_usd) })}
      </p>
      {a.parts.length > 0 && (
        <div className="tabelle">
          <table className="mobil-karten" aria-label={t("anliegen.teile")}>
            <thead>
              <tr>
                <th>{t("anliegen.teil")}</th>
                <th className="zahl">{t("anliegen.kosten")}</th>
                <th className="zahl">{t("anliegen.prognose")}</th>
              </tr>
            </thead>
            <tbody>
              {a.parts.map((teil, i) => (
                <tr key={i}>
                  <td>
                    <strong>
                      {teil.kind_text && teil.country
                        ? `${t(teil.kind_text)} · ${landName(teil.country)}`
                        : t("anliegen.ohne_ort")}
                    </strong>
                    {teil.product && ` · ${t(`produkt.${teil.product}`)}`}
                    {teil.option.steps.length > 0 && (
                      <ul className="schritte">
                        {teil.option.steps.map((s, j) => (
                          <li key={j}>{meldungText(s)}</li>
                        ))}
                      </ul>
                    )}
                  </td>
                  <td className="zahl" data-spalte={t("anliegen.kosten")}>
                    {formatGeld(teil.option.amount_usd)}
                  </td>
                  <td className="zahl" data-spalte={t("anliegen.prognose")}>
                    {wirkung(teil.option)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      <div className="tabelle">
        <table className="mobil-karten" aria-label={t("anliegen.optionen")}>
          <thead>
            <tr>
              <th>{t("anliegen.option")}</th>
              <th className="zahl">{t("anliegen.kosten")}</th>
              <th className="zahl">{t("anliegen.prognose")}</th>
              <th className="zahl">{t("anliegen.einmalig")}</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {a.options.map((o, i) => (
              <tr key={i} className={i === a.recommended ? "empfohlen" : undefined}>
                <td>
                  <strong>{t(`option.${o.kind}`)}</strong>
                  {i === a.recommended && <span className="marke">{t("anliegen.empfohlen")}</span>}
                  {o.steps.length > 0 && (
                    <ul className="schritte">
                      {o.steps.map((s, j) => (
                        <li key={j}>{meldungText(s)}</li>
                      ))}
                    </ul>
                  )}
                </td>
                <td className="zahl" data-spalte={t("anliegen.kosten")}>
                  {formatGeld(o.amount_usd)}
                </td>
                <td className="zahl" data-spalte={t("anliegen.prognose")}>
                  {wirkung(o)}
                </td>
                <td className="zahl" data-spalte={t("anliegen.einmalig")}>
                  {Math.abs(o.once_usd) >= 0.005 ? mitVorzeichen(o.once_usd) : "–"}
                </td>
                <td>
                  <button
                    type="button"
                    aria-label={`${t("anliegen.waehlen")}: ${optionName(a, i)} (${thema(a)}, ${herkunft(a)})`}
                    onClick={() =>
                      antworten(
                        { Choose: i },
                        t("anliegen.erfolg.gewaehlt", { option: optionName(a, i) }),
                      )
                    }
                  >
                    {t("anliegen.waehlen")}
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p>
        <strong>{t("anliegen.empfehlung", { option: optionName(a, a.recommended) })}</strong>{" "}
        {meldungText(a.because)}
      </p>
      <div className="knopfreihe links">
        <button
          type="button"
          className="haupt"
          onClick={() =>
            antworten(
              "Delegate",
              t("anliegen.erfolg.delegiert", { option: optionName(a, a.recommended) }),
            )
          }
        >
          {t("anliegen.selbst")}
        </button>
        <button
          type="button"
          className="schlicht"
          title={t("anliegen.nie_hilfe")}
          onClick={() =>
            antworten(
              "NeverAsk",
              t("anliegen.erfolg.nie", { stelle, thema: t(`thema.${a.topic}`) }),
            )
          }
        >
          {t("anliegen.nie")}
        </button>
        <button
          type="button"
          className="schlicht"
          title={t("anliegen.ablehnen_hilfe", { tage: ruhetage })}
          onClick={() => antworten("Decline", t("anliegen.erfolg.abgelehnt", { tage: ruhetage }))}
        >
          {t("anliegen.ablehnen")}
        </button>
      </div>
    </article>
  );
}

/** Equal concerns of several sites: one answer for all of them. */
function Gruppe({ g, ruhetage }: { g: AnliegenGruppe; ruhetage: number }) {
  const { los } = useAktion(ORT);
  if (g.concerns.length === 1) return <AnliegenKarte a={g.concerns[0]!} ruhetage={ruhetage} />;
  const titel = t("anliegen.gruppe", {
    anzahl: g.concerns.length,
    thema: t(`thema.${g.topic}`),
    option: t(`option.${g.kind}`),
  });
  return (
    <section className="anliegen-gruppe" aria-label={titel}>
      <div className="werk-kopf">
        <h3>{titel}</h3>
        <button
          type="button"
          onClick={() =>
            void los(
              g.concerns.map((a) => ({
                AnswerConcern: { concern: a.id, answer: "Delegate" as const },
              })),
              t("anliegen.alle_erfolg", { anzahl: g.concerns.length }),
            )
          }
        >
          {t("anliegen.alle_uebernehmen")}
        </button>
      </div>
      {g.concerns.map((a) => (
        <AnliegenKarte key={a.id} a={a} ruhetage={ruhetage} />
      ))}
    </section>
  );
}

/** How a closed concern ended. */
export function ausgang(a: Anliegen): string {
  const option = a.carried_out !== null ? optionName(a, a.carried_out) : "";
  return t(`anliegen.status.${a.status}`, { option });
}

/** The inbox of the concerns: the open ones, then those closed in the last year. */
export function AnliegenListe({ kern, stand }: { kern: Kern; stand: string }) {
  const { daten, fehler } = useSicht(() => kern.anliegen(), stand);
  const { antwort } = useAktion(ORT);
  if (!daten) return <FehlerText fehler={fehler} />;
  return (
    <section aria-label={t("anliegen.titel")}>
      <p className="feld-hilfe">{t("anliegen.hilfe", { tage: daten.deadline_days })}</p>
      <Rueckmeldung meldung={antwort} />
      {daten.open.length === 0 ? (
        <p>{t("anliegen.leer")}</p>
      ) : (
        daten.open.map((g) => (
          <Gruppe key={`${g.topic}/${g.kind}`} g={g} ruhetage={daten.block_days} />
        ))
      )}
      {daten.closed.length > 0 && (
        <details className="anliegen-erledigt">
          <summary>{t("anliegen.erledigte", { anzahl: daten.closed.length })}</summary>
          <ul>
            {daten.closed.map((a) => (
              <li key={a.id}>
                {formatDatum(a.closed ?? a.created)} · <strong>{thema(a)}</strong> · {herkunft(a)}:{" "}
                {ausgang(a)}
              </li>
            ))}
          </ul>
        </details>
      )}
    </section>
  );
}
