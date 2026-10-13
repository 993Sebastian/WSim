import { describe, expect, it } from "vitest";
import type { StandortDetail } from "../kern";
import { laufendeDauern } from "./Produktion";

const anlage = (teil: Record<string, unknown>) =>
  ({
    index: 0,
    facility: "schmelzofen",
    ready: "1900-01-01",
    operation: "laeuft",
    ...teil,
  }) as never;

describe("laufendeDauern", () => {
  it("nennt Bau, Überholung und Wiederanlauf mit Datum und Resttagen", () => {
    const s = {
      deposit_ready: null,
      slots: [
        anlage({ cause: { key: "ursache.im_bau" }, ready: "1907-08-01" }),
        anlage({ down_until: "1907-07-25", down_overhaul: true }),
        anlage({ operation: "wiederanlauf", operation_date: "1907-07-20" }),
        anlage({}),
      ],
    } as unknown as StandortDetail;
    const zeilen = laufendeDauern(s, "1907-07-18");
    expect(zeilen).toHaveLength(3);
    expect(zeilen[0]).toContain("im Bau bis 01.08.1907 – noch 14 Tage");
    expect(zeilen[1]).toContain("Überholung bis 25.07.1907 – noch 7 Tage");
    expect(zeilen[2]).toContain("Wiederanlauf bis 20.07.1907 – noch 2 Tage");
  });

  it("zeigt nichts, wenn nichts läuft", () => {
    const s = { deposit_ready: null, slots: [anlage({})] } as unknown as StandortDetail;
    expect(laufendeDauern(s, "1907-07-18")).toEqual([]);
  });
});

describe("laufendeDauern mit Ausschreibung (MAA)", () => {
  it("zeigt Tag X von 28 und offene Angebote", () => {
    const s = { index: 2, deposit_ready: null, slots: [] } as unknown as StandortDetail;
    const lauf = {
      site: 2,
      kind: "bauen",
      product: "stahl",
      count: 2,
      day: 5,
      days: 28,
      ends: "1907-08-10",
      offers: [],
    } as never;
    const fertig = { ...(lauf as object), day: 29, offers: [{}, {}] } as never;
    expect(laufendeDauern(s, "1907-07-18", [lauf])[0]).toContain("Tag 5 von 28, bis 10.08.1907");
    expect(laufendeDauern(s, "1907-07-18", [fertig])[0]).toContain("2 Angebot(e) warten");
    expect(laufendeDauern({ ...s, index: 3 } as StandortDetail, "1907-07-18", [lauf])).toEqual([]);
  });
});
