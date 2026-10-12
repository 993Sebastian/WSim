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
