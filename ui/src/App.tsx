import { useEffect, useState } from "react";
import type { Kern } from "./kern";
import { t } from "./texte";

type KernStatus =
  { art: "laedt" } | { art: "bereit"; version: string } | { art: "fehler"; fehler: string };

export function App({ kern }: { kern: Kern }) {
  const [status, setStatus] = useState<KernStatus>({ art: "laedt" });

  useEffect(() => {
    let aktiv = true;
    kern
      .info()
      .then((info) => aktiv && setStatus({ art: "bereit", version: info.version }))
      .catch((fehler: unknown) => aktiv && setStatus({ art: "fehler", fehler: String(fehler) }));
    return () => {
      aktiv = false;
    };
  }, [kern]);

  return (
    <main className="start">
      <h1>{t("app.titel")}</h1>
      <p className="untertitel">{t("app.untertitel")}</p>
      <p className={`kernstatus kernstatus-${status.art}`} role="status">
        {kernStatusText(status, kern.echt)}
      </p>
    </main>
  );
}

function kernStatusText(status: KernStatus, echt: boolean): string {
  switch (status.art) {
    case "laedt":
      return t("kern.laedt");
    case "bereit":
      return echt ? t("kern.bereit", { version: status.version }) : t("kern.vorschau");
    case "fehler":
      return t("kern.fehler", { fehler: status.fehler });
  }
}
