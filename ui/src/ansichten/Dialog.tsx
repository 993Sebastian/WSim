import { useEffect, useId, useRef, type ReactNode } from "react";
import { t } from "../texte";

/** Modal window; Escape or the close button end it when `onSchliessen` is given. */
export function Dialog({
  titel,
  onSchliessen,
  children,
  breit = false,
  tour,
}: {
  titel: string;
  onSchliessen?: () => void;
  children: ReactNode;
  breit?: boolean;
  /** Mark for the introduction (`data-tour`). */
  tour?: string;
}) {
  const id = useId();
  const flaeche = useRef<HTMLDivElement>(null);

  useEffect(() => {
    flaeche.current?.focus();
    if (!onSchliessen) return;
    const taste = (e: KeyboardEvent) => e.key === "Escape" && onSchliessen();
    window.addEventListener("keydown", taste);
    return () => window.removeEventListener("keydown", taste);
  }, [onSchliessen]);

  return (
    <div className="dialog-hintergrund">
      <div
        className={`dialog${breit ? " dialog-breit" : ""}`}
        role="dialog"
        aria-modal="true"
        aria-labelledby={id}
        tabIndex={-1}
        ref={flaeche}
        data-tour={tour}
      >
        <header className="dialog-kopf">
          <h2 id={id}>{titel}</h2>
          {onSchliessen && (
            <button
              type="button"
              className="schlicht"
              onClick={onSchliessen}
              aria-label={t("dialog.schliessen")}
            >
              ×
            </button>
          )}
        </header>
        {children}
      </div>
    </div>
  );
}

/** A refused request as text. */
export function FehlerText({ fehler }: { fehler: string | null }) {
  return fehler ? (
    <p className="fehlertext" role="alert">
      {fehler}
    </p>
  ) : null;
}
