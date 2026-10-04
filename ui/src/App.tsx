import { useEffect, useState } from "react";
import { Hauptmenue } from "./ansichten/Hauptmenue";
import { NeuesSpielAnsicht } from "./ansichten/NeuesSpiel";
import { Spiel } from "./ansichten/Spiel";
import type { Kern, Uebersicht } from "./kern";

export type KernStatus =
  { art: "laedt" } | { art: "bereit"; version: string } | { art: "fehler"; fehler: string };

type Bildschirm =
  { art: "menue" } | { art: "neu" } | { art: "spiel"; start: Uebersicht; einfuehrung: boolean };

export function App({ kern }: { kern: Kern }) {
  const [status, setStatus] = useState<KernStatus>({ art: "laedt" });
  const [bildschirm, setBildschirm] = useState<Bildschirm>({ art: "menue" });
  const [laufend, setLaufend] = useState<Uebersicht | null>(null);

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

  const spielen = (u: Uebersicht, einfuehrung = false) => {
    setLaufend(u);
    setBildschirm({ art: "spiel", start: u, einfuehrung });
  };

  switch (bildschirm.art) {
    case "menue":
      return (
        <Hauptmenue
          kern={kern}
          status={status}
          laufend={laufend}
          onNeu={() => setBildschirm({ art: "neu" })}
          onSpielen={(u) => spielen(u)}
        />
      );
    case "neu":
      return (
        <NeuesSpielAnsicht
          kern={kern}
          onStart={spielen}
          onZurueck={() => setBildschirm({ art: "menue" })}
        />
      );
    case "spiel":
      return (
        <Spiel
          kern={kern}
          start={bildschirm.start}
          einfuehrung={bildschirm.einfuehrung}
          onMenue={(u) => {
            setLaufend(u);
            setBildschirm({ art: "menue" });
          }}
        />
      );
  }
}
