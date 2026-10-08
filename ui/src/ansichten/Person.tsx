import { formatDatum, formatProzent, formatZahl, landName, meldungText } from "../format";
import type { Kern, Person, PersonKind } from "../kern";
import { t } from "../texte";
import { FehlerText } from "./Dialog";
import { useSicht } from "./gemeinsam";

/** Where a child stands as a manager: not yet, free, employed, retired or dead. */
function kindStatus(k: PersonKind): string {
  if (k.died) return t("person.kind_verstorben", { datum: formatDatum(k.died) });
  if (k.position) return meldungText(k.position);
  if (k.manager !== null) return t("person.kind_frei");
  if (k.card_in !== null) return t("person.kind_karte_in", { jahre: formatZahl(k.card_in) });
  return t("person.kind_im_ruhestand");
}

function Familie({ p }: { p: Person }) {
  return (
    <section aria-label={t("person.familie")}>
      <h2>{t("person.familie")}</h2>
      {p.children.length === 0 ? (
        <p>{t("person.keine_kinder")}</p>
      ) : (
        <table className="tabelle">
          <thead>
            <tr>
              <th>{t("person.kind")}</th>
              <th>{t("person.geboren")}</th>
              <th className="zahl">{t("person.kind_alter")}</th>
              <th>{t("person.kind_status")}</th>
            </tr>
          </thead>
          <tbody>
            {p.children.map((k) => (
              <tr key={`${k.name}-${k.born}`}>
                <td>{k.name}</td>
                <td data-spalte={t("person.geboren")}>{formatDatum(k.born)}</td>
                <td className="zahl" data-spalte={t("person.kind_alter")}>
                  {formatZahl(k.age)}
                </td>
                <td data-spalte={t("person.kind_status")}>{kindStatus(k)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      {p.children_possible && <p className="feld-hilfe">{t("person.kinder_moeglich")}</p>}
    </section>
  );
}

function Rollen({ p }: { p: Person }) {
  return (
    <section aria-label={t("person.rollen")}>
      <h2>{t("person.rollen")}</h2>
      <table className="tabelle">
        <thead>
          <tr>
            <th>{t("person.firma")}</th>
            <th className="zahl">{t("person.anteil")}</th>
            <th>{t("person.rolle")}</th>
          </tr>
        </thead>
        <tbody>
          {p.holdings.map((h) => (
            <tr key={h.company}>
              <td>{h.company}</td>
              <td className="zahl" data-spalte={t("person.anteil")}>
                {formatProzent(h.share)}
              </td>
              <td data-spalte={t("person.rolle")}>
                {h.person_ceo
                  ? t("person.rolle_ceo")
                  : h.ceo
                    ? t("person.rolle_eigentuemer", { name: h.ceo })
                    : t("person.rolle_eigentuemer_ohne")}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}

function Lebenslauf({ p }: { p: Person }) {
  return (
    <section aria-label={t("person.lebenslauf")}>
      <h2>{t("person.lebenslauf")}</h2>
      <table className="tabelle">
        <thead>
          <tr>
            <th>{t("person.datum")}</th>
            <th>{t("person.ereignis")}</th>
          </tr>
        </thead>
        <tbody>
          {p.history.map((e, i) => (
            <tr key={`${e.date}-${i}`}>
              <td>{formatDatum(e.date)}</td>
              <td data-spalte={t("person.ereignis")}>{meldungText(e.text)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}

/** The player as a person (PE2): profile, family, roles and chronicle. */
export function PersonAnsicht({ kern, stand }: { kern: Kern; stand: string }) {
  const { daten, fehler } = useSicht(() => kern.person(), stand);
  if (!daten) return <FehlerText fehler={fehler} />;
  return (
    <>
      <p className="feld-hilfe">{t("person.frage")}</p>
      <section aria-label={t("person.steckbrief")}>
        <h2>{daten.name}</h2>
        <dl className="werte">
          <dt>{t("person.geboren")}</dt>
          <dd>
            {t("person.alter_wert", {
              datum: formatDatum(daten.born),
              jahre: formatZahl(daten.age),
            })}
          </dd>
          <dt>{t("person.wohnsitz")}</dt>
          <dd>{landName(daten.home)}</dd>
          <dt>{t("person.familienstand")}</dt>
          <dd>{t(daten.married ? "person.verheiratet" : "person.ledig")}</dd>
        </dl>
      </section>
      <Familie p={daten} />
      <Rollen p={daten} />
      <Lebenslauf p={daten} />
    </>
  );
}
