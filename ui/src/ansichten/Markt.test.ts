import { describe, expect, it } from "vitest";
import { fortgeschrieben } from "./Markt";

describe("Preisverlauf", () => {
  it("schreibt Monate ohne Verkauf mit dem letzten Preis fort", () => {
    expect(fortgeschrieben([null, 10, null, 12, null], 5)).toEqual([10, 10, 10, 12, 12]);
  });

  it("nimmt ohne jeden Preis den Ersatz", () => {
    expect(fortgeschrieben([null, null], 5)).toEqual([5, 5]);
  });
});
