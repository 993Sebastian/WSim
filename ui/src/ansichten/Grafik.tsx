// Small charts of a course over the months (overview, finances). They only draw the
// numbers of the core; a text alternative names the first and the last value.
import { formatGeld } from "../format";
import { t } from "../texte";

const BREITE = 240;
const HOEHE = 56;
const RAND = 3;

export function formatMonatKurz(iso: string): string {
  const [jahr, monat] = iso.split("-");
  return `${t(`monat.${Number(monat)}`).slice(0, 3)} ${jahr}`;
}

/**
 * A course as line (cash, revenue) or bars around zero (result). With fewer than two
 * values there is nothing to see yet.
 */
export function Verlauf({
  name,
  monate,
  werte,
  art = "linie",
  bezug,
  format = formatGeld,
}: {
  name: string;
  monate: string[];
  werte: number[];
  art?: "linie" | "balken";
  /** A value to compare with, drawn as a dashed line (e.g. the reference price). */
  bezug?: number;
  /** How the text alternative writes a value (amounts by default). */
  format?: (wert: number) => string;
}) {
  if (werte.length < 2) {
    return <p className="gedaempft verlauf-leer">{t("grafik.zu_wenig")}</p>;
  }
  // Bars stand on the zero line; a line uses the range of its values, so that the
  // course shows (cash from 40 to 48 thousand is no flat line at the top).
  const roh = [
    ...(art === "balken" ? [0, ...werte] : werte),
    ...(bezug === undefined ? [] : [bezug]),
  ];
  const tief = Math.min(...roh);
  const hoch = Math.max(...roh);
  const rand = art === "balken" ? 0 : (hoch - tief) * 0.1 || Math.abs(hoch) * 0.05 || 1;
  const min = tief - rand;
  const max = hoch + rand;
  const spanne = max - min || 1;
  const y = (v: number) => RAND + (HOEHE - 2 * RAND) * (1 - (v - min) / spanne);
  const schritt = (BREITE - 2 * RAND) / Math.max(werte.length - 1, 1);
  const x = (i: number) => RAND + i * schritt;
  const beschreibung = t("grafik.beschreibung", {
    name,
    von: formatMonatKurz(monate[0]!),
    anfang: format(werte[0]!),
    bis: formatMonatKurz(monate.at(-1)!),
    ende: format(werte.at(-1)!),
  });
  return (
    <svg
      className={`verlauf verlauf-${art}`}
      viewBox={`0 0 ${BREITE} ${HOEHE}`}
      role="img"
      aria-label={beschreibung}
      preserveAspectRatio="none"
    >
      <title>{beschreibung}</title>
      {min <= 0 && max >= 0 && (
        <line className="nulllinie" x1={0} x2={BREITE} y1={y(0)} y2={y(0)} />
      )}
      {bezug !== undefined && (
        <line className="bezugslinie" x1={0} x2={BREITE} y1={y(bezug)} y2={y(bezug)} />
      )}
      {art === "linie" ? (
        <polyline points={werte.map((v, i) => `${x(i)},${y(v)}`).join(" ")} />
      ) : (
        werte.map((v, i) => {
          // One slot per month, the bar in its middle.
          const fach = (BREITE - 2 * RAND) / werte.length;
          return (
            <rect
              key={i}
              className={v < 0 ? "minus" : "plus"}
              x={RAND + i * fach + fach * 0.15}
              y={Math.min(y(v), y(0))}
              width={fach * 0.7}
              height={Math.max(Math.abs(y(v) - y(0)), 1)}
            />
          );
        })
      )}
    </svg>
  );
}
