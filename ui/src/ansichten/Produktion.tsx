// Sites (docs/BEDIENUNG.md): one card per site with what matters at a glance – last
// month's result, staff, bottlenecks – and the plant view behind it.
import { useEffect, useId, useMemo, useRef, useState } from "react";
import { formatGeld, landName } from "../format";
import type { Befehl, Kern, Produktion, StandortDetail, Uebersicht } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { Befehle, Rueckmeldung, useAktion, useBefehl, useSicht } from "./gemeinsam";
import { GrundstueckWahl, useGewerbeflaeche, type GrundstueckWahlWert } from "./Grundstuecke";
import { formatKoepfe, standortTitel, ursacheText, Werk } from "./Werk";

export function ProduktionAnsicht({
  kern,
  uebersicht,
  onGeaendert,
  werk,
  werkBereich = null,
  gruendenIn = null,
  onWerk,
}: {
  kern: Kern;
  uebersicht: Uebersicht;
  onGeaendert: (u: Uebersicht) => void;
  /** Site shown in the plant view, or null for the list of sites. */
  werk: number | null;
  /** Area of the plant view to show first (e.g. from a hint). */
  werkBereich?: string | null;
  /** Country preselected for a new site (from the map). */
  gruendenIn?: string | null;
  onWerk: (site: number | null) => void;
}) {
  const { daten, fehler, neu } = useSicht(() => kern.produktion(), uebersicht.date);
  const { senden, meldung } = useBefehl(kern, onGeaendert, neu);
  if (!daten) return <FehlerText fehler={fehler} />;
  const offen = werk === null ? undefined : daten.sites.find((s) => s.index === werk);
  return (
    <main className="ansicht" id="produktion">
      <h1 className="unsichtbar">{t("ansicht.produktion")}</h1>
      <Befehle senden={senden} meldung={meldung}>
        {offen ? (
          <Werk
            standort={offen}
            produktion={daten}
            bereich={werkBereich}
            onZurueck={() => onWerk(null)}
          />
        ) : (
          <>
            {daten.sites.length === 0 && (
              <p className="gedaempft">{t("uebersicht.keine_standorte")}</p>
            )}
            <div className="karten-raster" data-tour="standorte">
              {daten.sites.map((s) => (
                <StandortKarte key={s.index} s={s} onOeffnen={() => onWerk(s.index)} />
              ))}
            </div>
            <StandortGruenden
              key={gruendenIn ?? "heimat"}
              kern={kern}
              produktion={daten}
              heimat={gruendenIn ?? uebersicht.company?.headquarters ?? ""}
              hervorheben={gruendenIn !== null}
            />
          </>
        )}
      </Befehle>
    </main>
  );
}

function StandortKarte({ s, onOeffnen }: { s: StandortDetail; onOeffnen: () => void }) {
  const titel = standortTitel(s);
  const probleme = s.slots.filter((a) => a.cause && a.cause.key !== "ursache.im_bau");
  const imBau = s.slots.filter((a) => a.cause?.key === "ursache.im_bau").length;
  const produkte = [...new Set(s.slots.map((a) => a.product).filter((p): p is string => !!p))];
  const benoetigt = s.staff.reduce((summe, l) => summe + l.needed, 0);
  const m = s.last_month;
  return (
    <article className="karte standortkarte" aria-label={titel}>
      <h2>
        {titel}
        {s.deposit && <small>{t(`lagerstaette.${s.deposit}`)}</small>}
      </h2>
      {produkte.length > 0 && (
        <p className="gedaempft">{produkte.map((p) => t(`produkt.${p}`)).join(", ")}</p>
      )}
      <dl className="werte">
        <dt>{t("uebersicht.umsatz_monat")}</dt>
        <dd>{m ? formatGeld(m.revenue_usd) : "–"}</dd>
        <dt>{t("uebersicht.ergebnis_monat")}</dt>
        <dd className={m && m.result_usd < 0 ? "negativ" : ""}>
          {m ? formatGeld(m.result_usd) : t("uebersicht.noch_kein_monat")}
        </dd>
        <dt>{t("werk.beschaeftigte")}</dt>
        <dd>
          {formatKoepfe(s.workers)}
          {benoetigt > s.workers + 0.05 && (
            <small className="warntext">
              {" "}
              {t("werk.von_benoetigt", { anzahl: formatKoepfe(benoetigt) })}
            </small>
          )}
        </dd>
      </dl>
      {probleme.length > 0 ? (
        <ul className="probleme">
          {probleme.map((a) => (
            <li key={a.index}>
              <strong>{t(`anlage.${a.facility}`)}:</strong> {ursacheText(a)}
            </li>
          ))}
        </ul>
      ) : (
        s.slots.length > imBau && <p className="erfolgstext">{t("werk.alles_laeuft")}</p>
      )}
      {imBau > 0 && <p className="gedaempft">{t("werk.anlagen_im_bau", { anzahl: imBau })}</p>}
      <div className="knopfreihe links">
        <button
          type="button"
          className="haupt"
          aria-label={t("werk.oeffnen_von", { standort: titel })}
          data-tour="werk-oeffnen"
          onClick={onOeffnen}
        >
          {t("werk.oeffnen")}
        </button>
      </div>
    </article>
  );
}

function StandortGruenden({
  kern,
  produktion,
  heimat,
  hervorheben,
}: {
  kern: Kern;
  produktion: Produktion;
  heimat: string;
  /** Scroll to the form (the country was chosen on the map). */
  hervorheben: boolean;
}) {
  const [land, setLand] = useState(heimat);
  const ref = useRef<HTMLFormElement>(null);
  useEffect(() => {
    if (hervorheben) ref.current?.scrollIntoView({ block: "center" });
  }, [hervorheben]);
  const [art, setArt] = useState(produktion.site_types[1]?.kind ?? "Factory");
  const { los, antwort } = useAktion("gruenden");
  const id = useId();
  const laender = useMemo(
    () =>
      Object.keys(import.meta.glob("../../../data/laender/*.yaml"))
        .map((p) => p.split("/").pop()!.replace(".yaml", ""))
        .map((k) => ({ k, name: landName(k) }))
        .sort((a, b) => a.name.localeCompare(b.name, "de")),
    [],
  );
  const flaeche = useGewerbeflaeche(kern, land, produktion.date);
  const [wahl, setWahl] = useState<GrundstueckWahlWert>({ plot: null, lease: false });
  const typ = produktion.site_types.find((s) => s.kind === art);
  // Extraction sites stand on their concession; without plots the core picks none.
  const mitGrundstueck = art !== "Extraction" && flaeche !== null;
  const grundstueck = flaeche?.free.find((g) => g.id === wahl.plot) ?? null;
  const bereit = !mitGrundstueck || grundstueck !== null;
  return (
    <form
      ref={ref}
      className={`karte${hervorheben ? " hervorgehoben" : ""}`}
      aria-label={t("produktion.gruenden")}
      onSubmit={(e) => {
        e.preventDefault();
        if (!bereit) return;
        const befehl: Befehl = grundstueck
          ? { FoundSiteOnPlot: { plot: grundstueck.id, kind: art, lease: wahl.lease } }
          : { FoundSite: { country: land, kind: art } };
        void los(
          [befehl],
          t("werk.gegruendet", { art: typ ? t(typ.kind_text) : art, land: landName(land) }),
        );
      }}
    >
      <h2>{t("produktion.gruenden")}</h2>
      <p className="erklaerung">{t("werk.gruenden_hinweis")}</p>
      <div className="formular-zeile">
        <div className="feld">
          <label htmlFor={`${id}-land`}>{t("produktion.land")}</label>
          <select
            id={`${id}-land`}
            value={land}
            onChange={(e) => {
              setLand(e.target.value);
              setWahl({ ...wahl, plot: null });
            }}
          >
            {laender.map(({ k, name }) => (
              <option key={k} value={k}>
                {name}
              </option>
            ))}
          </select>
        </div>
        <div className="feld">
          <label htmlFor={`${id}-art`}>{t("produktion.standorttyp")}</label>
          <select id={`${id}-art`} value={art} onChange={(e) => setArt(e.target.value)}>
            {produktion.site_types.map((s) => (
              <option key={s.kind} value={s.kind}>
                {t(s.kind_text)} ({formatGeld(s.cost_usd)})
              </option>
            ))}
          </select>
        </div>
      </div>
      {mitGrundstueck && flaeche && (
        <GrundstueckWahl flaeche={flaeche} wert={wahl} onWert={setWahl} />
      )}
      <div className="knopfreihe links">
        <button type="submit" disabled={!bereit}>
          {t("produktion.gruenden_knopf")}
        </button>
        {typ && (
          <span className="gedaempft">
            {t("grundstueck.kosten_jetzt", {
              betrag: formatGeld(
                typ.cost_usd + (grundstueck && !wahl.lease ? grundstueck.value_usd : 0),
              ),
            })}
          </span>
        )}
      </div>
      <Rueckmeldung meldung={antwort} />
    </form>
  );
}
