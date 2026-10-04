import { formatDatum, formatGeld, meldungText } from "../format";
import type { Rundenbericht } from "../kern";
import { t } from "../texte";
import { Dialog } from "./Dialog";

export function RundenberichtDialog({
  bericht,
  onZiel,
  onSchliessen,
}: {
  bericht: Rundenbericht;
  onZiel: (ziel: string) => void;
  onSchliessen: () => void;
}) {
  const aenderung = bericht.equity_change_usd;
  return (
    <Dialog titel={t("bericht.titel")} onSchliessen={onSchliessen} breit>
      <p className="gedaempft">
        {t("bericht.zeitraum", {
          von: formatDatum(bericht.from),
          bis: formatDatum(bericht.to),
          tage: bericht.days,
        })}
      </p>
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
      <h3>{t("bericht.meldungen")}</h3>
      {bericht.messages.length === 0 ? (
        <p>{t("bericht.keine_meldungen")}</p>
      ) : (
        <ul className="meldungen">
          {bericht.messages.map((m, i) => (
            <li key={i} className={`meldung meldung-${m.kind}`}>
              <span className="meldungsart">{t(`meldungsart.${m.kind}`)}</span>
              <span className="meldungstext">{meldungText(m)}</span>
              {m.target && (
                <button type="button" className="schlicht" onClick={() => onZiel(m.target!)}>
                  {t("bericht.ansehen")}
                </button>
              )}
            </li>
          ))}
        </ul>
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
