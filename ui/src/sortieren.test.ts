import { describe, expect, it } from "vitest";
import { sortierbareTabellen, vergleiche, zahlAus } from "./sortieren";

describe("Tabellen sortieren", () => {
  it("liest Zahlen in deutscher Schreibweise", () => {
    expect(zahlAus("1.234,5 €")).toBe(1234.5);
    expect(zahlAus("−12 %")).toBe(-12);
    expect(zahlAus("3,4 Mio. €")).toBeCloseTo(3.4e6);
    expect(zahlAus("Nägel")).toBeNull();
    expect(vergleiche("9", "10")).toBeLessThan(0);
    expect(vergleiche("Bier", "Apfel")).toBeGreaterThan(0);
  });

  it("sortiert eine Tabelle per Klick auf den Spaltenkopf", () => {
    document.body.innerHTML = `<table><thead><tr><th>Produkt</th><th>Nachfrage</th></tr></thead>
      <tbody><tr><td>A</td><td>1.000</td></tr><tr><td>B</td><td>50.000</td></tr>
      <tr><td>C</td><td>200</td></tr></tbody></table>`;
    const weg = sortierbareTabellen(document.body);
    const kopf = document.querySelectorAll("th")[1] as HTMLElement;
    const reihe = () =>
      Array.from(document.querySelectorAll("tbody tr td:first-child")).map((c) => c.textContent);
    kopf.click();
    expect(reihe()).toEqual(["B", "A", "C"]);
    expect(kopf.getAttribute("aria-sort")).toBe("descending");
    kopf.click();
    expect(reihe()).toEqual(["C", "A", "B"]);
    kopf.click();
    expect(reihe()).toEqual(["A", "B", "C"]);
    weg();
  });
});
