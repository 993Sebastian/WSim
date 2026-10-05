import { formatDatum, formatGeld, formatProzent, meldungText } from "../format";
import type { Hinweis, Meldung, MeldungsGruppe, Periode, Rundenbericht } from "../kern";
import { t } from "../texte";
import { Dialog } from "./Dialog";

const GRUPPEN: MeldungsGruppe[] = [
  "erfolg",
  "welt",
  "warnung",
  "wettbewerb",
  "forschung",
  "allgemein",
];

function Veraenderung({ jetzt, vorher }: { jetzt: number; vorher: number | undefined }) {
  if (vorher === undefined) return <td className="zahl gedaempft">–</td>;
  const d = jetzt - vorher;
  return (
    <td className={`zahl ${d < 0 ? "negativ" : "positiv"}`}>
      {d > 0 ? "+" : ""}
      {formatGeld(d)}
    </td>
  );
}

function Ergebnis({ periode, vorher }: { periode: Periode; vorher: Periode | null }) {
  // Without the change of the stock the lines would not add up to the result.
  const lager =
    Math.abs(periode.inventory_change_usd) >= 0.5 ||
    Math.abs(vorher?.inventory_change_usd ?? 0) >= 0.5;
  const zeilen: [string, number, number | undefined][] = [
    [t("bericht.umsatz"), periode.revenue_usd, vorher?.revenue_usd],
    [t("bericht.kosten"), -periode.costs_usd, vorher ? -vorher.costs_usd : undefined],
    ...(lager
      ? [
          [t("bericht.lager"), periode.inventory_change_usd, vorher?.inventory_change_usd] as [
            string,
            number,
            number | undefined,
          ],
        ]
      : []),
    [t("bericht.ergebnis"), periode.result_usd, vorher?.result_usd],
  ];
  return (
    <div className="tabelle">
      <table className="ergebnis">
        <thead>
          <tr>
            <th />
            <th className="zahl">{t("bericht.diese_runde")}</th>
            <th className="zahl">{t("bericht.vorrunde")}</th>
            <th className="zahl">{t("bericht.veraenderung")}</th>
          </tr>
        </thead>
        <tbody>
          {zeilen.map(([name, jetzt, alt]) => (
            <tr key={name}>
              <th scope="row">{name}</th>
              <td className={`zahl ${jetzt < 0 ? "negativ" : ""}`}>{formatGeld(jetzt)}</td>
              <td className="zahl">{alt === undefined ? "–" : formatGeld(alt)}</td>
              <Veraenderung jetzt={jetzt} vorher={alt} />
            </tr>
          ))}
        </tbody>
      </table>
      {lager && <p className="feld-hilfe">{t("bericht.lager_hinweis")}</p>}
      {periode.lines.length > 0 && (
        <details className="aufschluesselung">
          <summary>{t("bericht.nach_kostenart")}</summary>
          <table>
            <tbody>
              {periode.lines.map(([schluessel, betrag]) => (
                <tr key={schluessel}>
                  <td>{t(schluessel)}</td>
                  <td className={`zahl ${betrag < 0 ? "negativ" : ""}`}>{formatGeld(betrag)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </details>
      )}
    </div>
  );
}

function MeldungZeile({ m, onZiel }: { m: Meldung; onZiel: (ziel: string) => void }) {
  return (
    <li className={`meldung meldung-${m.kind}`}>
      <span className="meldungsart">{t(`meldungsart.${m.kind}`)}</span>
      <span className="meldungstext">{meldungText(m)}</span>
      {m.target && (
        <button type="button" className="schlicht" onClick={() => onZiel(m.target!)}>
          {t("bericht.ansehen")}
        </button>
      )}
    </li>
  );
}

export function RundenberichtDialog({
  bericht,
  onZiel,
  onHinweis,
  onEreignis,
  onSchliessen,
}: {
  bericht: Rundenbericht;
  onZiel: (ziel: string) => void;
  onHinweis: (h: Hinweis) => void;
  onEreignis: (m: Meldung) => void;
  onSchliessen: () => void;
}) {
  const aenderung = bericht.equity_change_usd;
  const gruppen = GRUPPEN.map((g) => [g, bericht.messages.filter((m) => m.group === g)] as const);
  return (
    <Dialog titel={t("bericht.titel")} onSchliessen={onSchliessen} breit tour="bericht">
      <p className="gedaempft">
        {t("bericht.zeitraum", {
          von: formatDatum(bericht.from),
          bis: formatDatum(bericht.to),
          tage: bericht.days,
        })}
        {(bericht.rounds ?? 1) > 1 && ` · ${t("bericht.runden", { anzahl: bericht.rounds! })}`}
      </p>
      {bericht.stop && <p className="halt-grund">{t(`bericht.halt.${bericht.stop}`)}</p>}
      <dl className="kennzahlen">
        <div>
          <dt>{t("bericht.kasse")}</dt>
          <dd>
            {formatGeld(bericht.cash_before_usd)} → {formatGeld(bericht.cash_after_usd)}
          </dd>
        </div>
        <div>
          <dt>{t("bericht.eigenkapital_aenderung")}</dt>
          <dd className={aenderung < 0 ? "negativ" : "positiv"}>
            {aenderung > 0 ? "+" : ""}
            {formatGeld(aenderung)}
          </dd>
        </div>
      </dl>

      <h3>{t("bericht.finanzergebnis")}</h3>
      <Ergebnis periode={bericht.period} vorher={bericht.previous} />

      <div data-tour="was-lief">
        <h3>{t("bericht.was_lief")}</h3>
        {bericht.products.length === 0 ? (
          <p className="gedaempft">{t("bericht.nichts_verkauft")}</p>
        ) : (
          <div className="tabelle">
            <table className="mobil-karten" aria-label={t("bericht.was_lief")}>
              <thead>
                <tr>
                  <th>{t("uebersicht.produkt")}</th>
                  <th className="zahl">{t("kostenart.umsatz")}</th>
                  <th className="zahl">{t("werk.rohertrag")}</th>
                </tr>
              </thead>
              <tbody>
                {bericht.products.map((p) => (
                  <tr key={p.product}>
                    <td>{t(`produkt.${p.product}`)}</td>
                    <td className="zahl" data-spalte={t("kostenart.umsatz")}>
                      {formatGeld(p.revenue_usd)}
                    </td>
                    <td
                      className={`zahl ${p.margin_usd < 0 ? "negativ" : ""}`}
                      data-spalte={t("werk.rohertrag")}
                    >
                      {formatGeld(p.margin_usd)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {bericht.hints.length > 0 && (
        <section>
          <h3>{t("bericht.jetzt_tun")}</h3>
          <ul className="meldungen">
            {bericht.hints.map((h, i) => (
              <li key={i} className={`meldung meldung-${h.message.kind}`}>
                <span className="meldungsart">{t(`meldungsart.${h.message.kind}`)}</span>
                <span className="meldungstext">{meldungText(h.message)}</span>
                <button type="button" className="schlicht" onClick={() => onHinweis(h)}>
                  {t("uebersicht.hingehen")}
                </button>
              </li>
            ))}
          </ul>
        </section>
      )}

      {gruppen.map(([gruppe, liste]) =>
        gruppe === "forschung"
          ? (bericht.research.length > 0 || liste.length > 0) && (
              <section key={gruppe}>
                <h3>{t("bericht.gruppe.forschung")}</h3>
                {bericht.research.map((p) => (
                  <p key={p.technology} className="forschungszeile">
                    <span>{t(`technologie.${p.technology}`)}</span>
                    <progress max={p.needed} value={Math.min(p.points, p.needed)} />
                    <span className="zahl">
                      {formatProzent(p.needed > 0 ? p.points / p.needed : 0)}
                    </span>
                  </p>
                ))}
                <ul className="meldungen">
                  {liste.map((m, i) => (
                    <MeldungZeile key={i} m={m} onZiel={onZiel} />
                  ))}
                </ul>
              </section>
            )
          : liste.length > 0 && (
              <section key={gruppe}>
                <h3>{t(`bericht.gruppe.${gruppe}`)}</h3>
                <ul className="meldungen">
                  {liste.map((m, i) =>
                    gruppe === "welt" ? (
                      <li key={i} className="meldung meldung-world_event">
                        <span className="meldungsart">{t("meldungsart.world_event")}</span>
                        <span className="meldungstext">{meldungText(m)}</span>
                        <button type="button" className="schlicht" onClick={() => onEreignis(m)}>
                          {t("bericht.ansehen")}
                        </button>
                      </li>
                    ) : (
                      <MeldungZeile key={i} m={m} onZiel={onZiel} />
                    ),
                  )}
                </ul>
              </section>
            ),
      )}
      {bericht.messages.length === 0 && bericht.research.length === 0 && (
        <p>{t("bericht.keine_meldungen")}</p>
      )}
      {bericht.game_over && <p className="fehlertext">{t("spiel.ende")}</p>}
      <div className="knopfreihe">
        <button type="button" className="haupt" onClick={onSchliessen}>
          {t("bericht.weiter")}
        </button>
      </div>
    </Dialog>
  );
}
