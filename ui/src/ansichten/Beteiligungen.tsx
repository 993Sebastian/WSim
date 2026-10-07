// Start-ups (SU1, docs/BEDIENUNG.md "Beteiligungen"): "Who works on which technology,
// how far along is it, how likely is it to succeed, and who owns it?"
import { useState } from "react";
import { formatDatum, formatGeld, formatProzent, formatZahl, landName } from "../format";
import type { Kern, StartUp, StartUps, Uebersicht } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { Unterreiter, useSicht } from "./gemeinsam";

type Reiter = "laufend" | "beendet";

/** What a start-up works on: a new technology with its lead, or a product's next level. */
function ziel(s: StartUp): string {
  if (s.kind === "technologie") {
    const name = t(`technologie.${s.target}`);
    return s.lead >= 1
      ? t("beteiligungen.ziel_neu", { technologie: name, jahre: formatZahl(s.lead) })
      : name;
  }
  return t("beteiligungen.ziel_verbesserung", {
    produkt: t(`produkt.${s.target}`),
    stufe: formatZahl(s.level ?? 0),
  });
}

/** The owners with their shares, largest first. */
function eigner(s: StartUp): string {
  return [...s.owners]
    .sort((a, b) => b.share - a.share)
    .map((o) => {
      const wer =
        o.holder === "firma"
          ? (o.company ?? t("beteiligungen.eigner.firma"))
          : t(`beteiligungen.eigner.${o.holder}`);
      return `${wer} ${formatProzent(o.share)}`;
    })
    .join(", ");
}

/** The open round or the funded phase. */
function finanzierung(s: StartUp): string {
  if (s.round_until) {
    return t("beteiligungen.runde_offen", {
      bedarf: formatGeld(s.capital_usd),
      datum: formatDatum(s.round_until),
    });
  }
  return t("beteiligungen.finanziert", {
    bedarf: formatGeld(s.capital_usd),
    datum: s.phase_until ? formatDatum(s.phase_until) : "–",
  });
}

function Name({ s }: { s: StartUp }) {
  return (
    <td>
      <strong>{s.name}</strong>
      {s.inventor && (
        <span className="marke" title={t("beteiligungen.erfinder_hilfe")}>
          {t("beteiligungen.erfinder")}
        </span>
      )}
    </td>
  );
}

function Laufende({ liste, geschaetzt }: { liste: StartUp[]; geschaetzt: boolean }) {
  if (liste.length === 0) return <p className="gedaempft">{t("beteiligungen.keine_laufenden")}</p>;
  return (
    <div className="tabelle">
      <table className="mobil-karten" aria-label={t("beteiligungen.laufend")}>
        <thead>
          <tr>
            <th>{t("beteiligungen.name")}</th>
            <th>{t("beteiligungen.land")}</th>
            <th>{t("beteiligungen.ziel")}</th>
            <th>{t("beteiligungen.phase")}</th>
            <th>{t("beteiligungen.finanzierung")}</th>
            <th className="zahl">{t("beteiligungen.chance")}</th>
            <th>{t("beteiligungen.eigner")}</th>
          </tr>
        </thead>
        <tbody>
          {liste.map((s) => (
            <tr key={s.id}>
              <Name s={s} />
              <td data-spalte={t("beteiligungen.land")}>{landName(s.country)}</td>
              <td data-spalte={t("beteiligungen.ziel")}>{ziel(s)}</td>
              <td data-spalte={t("beteiligungen.phase")}>
                {t("beteiligungen.phase_von", {
                  phase: t(`startup.phase.${s.phase ?? ""}`),
                  nummer: formatZahl(s.phase_number),
                  alle: formatZahl(s.phases),
                })}
              </td>
              <td data-spalte={t("beteiligungen.finanzierung")}>{finanzierung(s)}</td>
              <td className="zahl" data-spalte={t("beteiligungen.chance")}>
                {geschaetzt && s.chance !== null
                  ? formatProzent(s.chance)
                  : t(`beteiligungen.stufe.${s.chance_level ?? "gering"}`)}
              </td>
              <td data-spalte={t("beteiligungen.eigner")}>{eigner(s)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function Beendete({ liste }: { liste: StartUp[] }) {
  if (liste.length === 0) return <p className="gedaempft">{t("beteiligungen.keine_beendeten")}</p>;
  return (
    <div className="tabelle">
      <table className="mobil-karten" aria-label={t("beteiligungen.beendet")}>
        <thead>
          <tr>
            <th>{t("beteiligungen.name")}</th>
            <th>{t("beteiligungen.land")}</th>
            <th>{t("beteiligungen.ziel")}</th>
            <th>{t("beteiligungen.ergebnis")}</th>
            <th>{t("beteiligungen.ende")}</th>
          </tr>
        </thead>
        <tbody>
          {liste.map((s) => (
            <tr key={s.id} className={s.status === "erfolg" ? "eigene-zeile" : undefined}>
              <Name s={s} />
              <td data-spalte={t("beteiligungen.land")}>{landName(s.country)}</td>
              <td data-spalte={t("beteiligungen.ziel")}>{ziel(s)}</td>
              <td data-spalte={t("beteiligungen.ergebnis")}>
                <span title={t(`beteiligungen.status_hilfe.${s.status}`)}>
                  {t(`beteiligungen.status.${s.status}`)}
                </span>
              </td>
              <td data-spalte={t("beteiligungen.ende")}>{s.ended ? formatDatum(s.ended) : "–"}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function Inhalt({ d }: { d: StartUps }) {
  const [reiter, setReiter] = useState<Reiter>("laufend");
  const titel = d.label ? t(`startup.bezeichnung.${d.label}`) : t("ansicht.beteiligungen");
  return (
    <>
      <h2>{titel}</h2>
      <p>{t("beteiligungen.einleitung")}</p>
      <p className="gedaempft">
        {d.per_year > 0
          ? t("beteiligungen.zahlen", {
              je_jahr: formatZahl(d.per_year),
              gegruendet: formatZahl(d.founded),
              erfolg: formatZahl(d.succeeded),
              gescheitert: formatZahl(d.failed),
              jahre: formatZahl(d.keep_years),
            })
          : t("beteiligungen.aus")}
      </p>
      <p className="feld-hilfe">
        {d.estimated ? t("beteiligungen.schaetzung_genau") : t("beteiligungen.schaetzung_grob")}
      </p>
      <Unterreiter
        name={titel}
        bereiche={[
          { key: "laufend", text: t("beteiligungen.laufend"), zaehler: d.active.length },
          { key: "beendet", text: t("beteiligungen.beendet") },
        ]}
        aktiv={reiter}
        onWahl={setReiter}
      />
      {reiter === "laufend" ? (
        <Laufende liste={d.active} geschaetzt={d.estimated} />
      ) : (
        <Beendete liste={d.closed} />
      )}
    </>
  );
}

/** The start-ups of the world (SU1). */
export function BeteiligungenAnsicht({ kern, uebersicht }: { kern: Kern; uebersicht: Uebersicht }) {
  const { daten, fehler } = useSicht(() => kern.startups(), uebersicht.date);
  return (
    <main className="ansicht" id="beteiligungen">
      <h1 className="unsichtbar">{t("ansicht.beteiligungen")}</h1>
      {daten ? <Inhalt d={daten} /> : <FehlerText fehler={fehler} />}
    </main>
  );
}
