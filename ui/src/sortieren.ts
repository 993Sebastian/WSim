// Every table sorts by a click on a column head (ascending, descending, back to the
// order of the view). One listener for the whole page: the views need not know about it.
// The rows are moved in the DOM; after React renders a sorted table again, an observer
// sorts it anew.

import { t } from "./texte";

type Richtung = "ascending" | "descending";

/** A number in German writing ("1.234,5 €", "−12 %", "3,4 Mio."), else null. */
export function zahlAus(text: string): number | null {
  const s = text.replace(/−/g, "-").replace(/\s/g, " ").trim();
  const m = s.match(/-?\d[\d.]*(,\d+)?/);
  if (!m) return null;
  let n = Number(m[0].replace(/\./g, "").replace(",", "."));
  if (!Number.isFinite(n)) return null;
  const rest = s.slice((m.index ?? 0) + m[0].length).toLowerCase();
  if (/^\s*(mrd|bn)/.test(rest)) n *= 1e9;
  else if (/^\s*mio/.test(rest)) n *= 1e6;
  else if (/^\s*(tsd|k\b)/.test(rest)) n *= 1e3;
  return n;
}

/** Compares two cell texts: as numbers when both are, else alphabetically. */
export function vergleiche(a: string, b: string): number {
  const x = zahlAus(a);
  const y = zahlAus(b);
  if (x !== null && y !== null) return x - y;
  if (x !== null) return -1;
  if (y !== null) return 1;
  return a.localeCompare(b, "de", { numeric: true });
}

const ursprung = new WeakMap<HTMLTableRowElement, number>();
let beobachter: MutationObserver | null = null;
let ordnet = false;

function zeilen(tabelle: HTMLTableElement): HTMLTableRowElement[] {
  const koerper = tabelle.tBodies[0];
  return koerper ? Array.from(koerper.rows) : [];
}

/** Whether the table can be sorted: one body, no cells spanning rows. */
function sortierbar(tabelle: HTMLTableElement): boolean {
  if (tabelle.tBodies.length !== 1 || !tabelle.tHead) return false;
  return !zeilen(tabelle).some((z) => Array.from(z.cells).some((c) => c.rowSpan > 1));
}

function ordne(tabelle: HTMLTableElement) {
  const spalte = Number(tabelle.dataset.sortSpalte);
  const richtung = tabelle.dataset.sortRichtung as Richtung | undefined;
  const koerper = tabelle.tBodies[0];
  if (!koerper || Number.isNaN(spalte)) return;
  const liste = zeilen(tabelle);
  liste.forEach((z, i) => {
    if (!ursprung.has(z)) ursprung.set(z, i);
  });
  const text = (z: HTMLTableRowElement) => z.cells[spalte]?.textContent ?? "";
  const sortiert = [...liste].sort((a, b) => {
    if (!richtung) return (ursprung.get(a) ?? 0) - (ursprung.get(b) ?? 0);
    const v = vergleiche(text(a), text(b));
    return richtung === "ascending" ? v : -v;
  });
  if (sortiert.every((z, i) => z === liste[i])) return;
  ordnet = true;
  for (const z of sortiert) koerper.appendChild(z);
  ordnet = false;
}

function klick(e: MouseEvent) {
  const ziel = e.target as HTMLElement | null;
  if (!ziel || ziel.closest("button, a, input, select, label")) return;
  const kopf = ziel.closest("th");
  const tabelle = kopf?.closest("table");
  if (!kopf || !tabelle || !kopf.closest("thead") || !sortierbar(tabelle)) return;
  const spalte = Array.from(kopf.parentElement?.children ?? []).indexOf(kopf);
  const vorher = tabelle.dataset.sortSpalte === String(spalte) ? tabelle.dataset.sortRichtung : "";
  const neu: Richtung | "" =
    vorher === "" ? "descending" : vorher === "descending" ? "ascending" : "";
  tabelle.querySelectorAll("thead th[aria-sort]").forEach((th) => th.removeAttribute("aria-sort"));
  if (neu) {
    tabelle.dataset.sortSpalte = String(spalte);
    tabelle.dataset.sortRichtung = neu;
    kopf.setAttribute("aria-sort", neu);
  } else {
    tabelle.dataset.sortSpalte = String(spalte);
    delete tabelle.dataset.sortRichtung;
  }
  ordne(tabelle);
}

/** Marks the heads of sortable tables (pointer, tooltip). */
function markiere(wurzel: ParentNode) {
  wurzel
    .querySelectorAll<HTMLTableCellElement>("table thead th:not([data-sortierbar])")
    .forEach((th) => {
      th.dataset.sortierbar = "";
      if (!th.title) th.title = t("tabelle.sortieren");
    });
}

/** Installs sorting for every table of the page; returns how to remove it. */
export function sortierbareTabellen(wurzel: HTMLElement = document.body): () => void {
  wurzel.addEventListener("click", klick);
  markiere(wurzel);
  beobachter = new MutationObserver((aenderungen) => {
    if (ordnet) return;
    const tabellen = new Set<HTMLTableElement>();
    for (const a of aenderungen) {
      const t = (a.target as HTMLElement).closest?.("table");
      if (t?.dataset.sortRichtung) tabellen.add(t);
    }
    tabellen.forEach(ordne);
    markiere(wurzel);
  });
  beobachter.observe(wurzel, { childList: true, subtree: true, characterData: true });
  return () => {
    wurzel.removeEventListener("click", klick);
    beobachter?.disconnect();
    beobachter = null;
  };
}
