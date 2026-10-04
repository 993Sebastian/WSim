import { t } from "../texte";

/** Steps of the introduction (Lastenheft §14.4): the view each one shows and its texts
 * `einfuehrung.<key>.titel` and `.text`. */
export const SCHRITTE: { ansicht: string; key: string }[] = [
  { ansicht: "uebersicht", key: "willkommen" },
  { ansicht: "uebersicht", key: "uebersicht" },
  { ansicht: "produktion", key: "produktion" },
  { ansicht: "markt", key: "markt" },
  { ansicht: "markt", key: "werbung" },
  { ansicht: "forschung", key: "forschung" },
  { ansicht: "finanzen", key: "finanzen" },
  { ansicht: "weltkarte", key: "weltkarte" },
];

/** The introduction as a panel beside the game: it does not block the screen, so the
 * player can try out each step at once. */
export function Einfuehrung({
  schritt,
  onSchritt,
  onEnde,
}: {
  schritt: number;
  onSchritt: (schritt: number) => void;
  onEnde: () => void;
}) {
  const s = SCHRITTE[schritt]!;
  const letzter = schritt === SCHRITTE.length - 1;
  return (
    <aside className="einfuehrung" aria-label={t("einfuehrung.titel")}>
      <p className="gedaempft">
        {t("einfuehrung.schritt", { nummer: schritt + 1, anzahl: SCHRITTE.length })}
      </p>
      <h2>{t(`einfuehrung.${s.key}.titel`)}</h2>
      <p>{t(`einfuehrung.${s.key}.text`)}</p>
      <div className="knopfreihe">
        <button type="button" className="schlicht" onClick={onEnde}>
          {t("einfuehrung.beenden")}
        </button>
        <button type="button" disabled={schritt === 0} onClick={() => onSchritt(schritt - 1)}>
          {t("einfuehrung.zurueck")}
        </button>
        <button
          type="button"
          className="haupt"
          onClick={() => (letzter ? onEnde() : onSchritt(schritt + 1))}
        >
          {letzter ? t("einfuehrung.fertig") : t("einfuehrung.weiter")}
        </button>
      </div>
    </aside>
  );
}
