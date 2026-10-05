import { expect, test, type Page } from "@playwright/test";

const bilder = process.env.WSIM_BILDER;

async function starten(page: Page, einfuehrung = false) {
  await page.goto("/");
  await page.getByRole("button", { name: "Neues Spiel" }).click();
  await page.getByLabel("Name der Firma").fill("Rheinische Nagelwerke");
  const haken = page.getByLabel(/Einführung zeigen/);
  await expect(haken).toBeChecked();
  if (!einfuehrung) await haken.uncheck();
  await page.getByRole("button", { name: "Spiel starten" }).click();
  await expect(page.locator(".kopfleiste")).toBeVisible();
}

const befehle = (page: Page) =>
  page.evaluate(() => (globalThis as { __wsimBefehle?: unknown[] }).__wsimBefehle ?? []);

async function bild(page: Page, name: string) {
  if (bilder) await page.screenshot({ path: `${bilder}/${name}.png`, fullPage: true });
}

test("Ansichten über Reiter und Zifferntasten", async ({ page }) => {
  await starten(page);
  await page.keyboard.press("2");
  await expect(page.getByRole("button", { name: "Standorte" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  // One card per site; it opens the plant view.
  const karte = page.getByRole("article", { name: "Werk · Deutschland" });
  await expect(karte.getByText("Alle Anlagen laufen nach Plan.")).toBeVisible();
  await bild(page, "standorte");
  await karte.getByRole("button", { name: "Werk · Deutschland öffnen" }).click();
  const werk = page.getByRole("region", { name: "Werk · Deutschland" });
  await expect(werk.getByRole("heading", { name: /Werk · Deutschland/ })).toBeVisible();
  // A machine at its plan shows no bottleneck.
  const maschine = werk.getByRole("article", { name: "Nagelmaschine" });
  await expect(maschine.getByText("läuft nach Plan")).toBeVisible();
  await expect(maschine.getByText(/0,5 t Draht → 0,5 t Nägel/)).toBeVisible();
  await bild(page, "werk_anlagen");

  // Sales: last month's sales, the price beside market price and unit cost.
  await werk.getByRole("button", { name: "Verkauf" }).click();
  const naegel = werk.getByRole("article", { name: "Verkauf von Nägel" });
  await expect(naegel.getByText("15,5 t")).toBeVisible();
  await expect(naegel.getByText("Stückkosten", { exact: true })).toBeVisible();
  await expect(naegel.getByText(/Marge 18 %/)).toBeVisible();
  await bild(page, "werk_verkauf");

  await werk.getByRole("button", { name: "Einkauf" }).click();
  await expect(werk.getByRole("combobox", { name: "Weiteres Produkt einkaufen" })).toBeVisible();
  await werk.getByRole("button", { name: "Personal" }).click();
  await expect(werk.getByRole("table", { name: "Arbeitskräfte" })).toBeVisible();
  await werk.getByRole("button", { name: "Kosten und Ergebnis" }).click();
  await expect(werk.getByRole("article", { name: "Stückkosten von Nägel" })).toBeVisible();
  await expect(werk.getByRole("table", { name: "Ergebnis des Vormonats" })).toBeVisible();
  await bild(page, "werk_kosten");

  await page.keyboard.press("3");
  await expect(page.getByRole("table", { name: "Markt Deutschland" })).toBeVisible();
  // Newcomer against the established companies: leader, own share and brands.
  await expect(
    page.getByRole("cell", { name: "99 % Maschinenfabrik Lehmann & Söhne" }),
  ).toBeVisible();
  const marken = page.getByRole("table", { name: "Marke und Werbung Deutschland" });
  await expect(marken.getByRole("cell", { name: "Metallwaren", exact: true })).toBeVisible();
  await expect(page.getByText(/Bestes Werbemittel: Zeitungsanzeigen/)).toBeVisible();
  await bild(page, "markt");

  await page.keyboard.press("4");
  await expect(page.getByRole("heading", { name: "Technologien" })).toBeVisible();
  await expect(page.getByRole("cell", { name: "Bessemer-Verfahren" })).toBeVisible();
  await bild(page, "forschung");

  await page.keyboard.press("5");
  await expect(page.getByRole("heading", { name: "Bilanz" })).toBeVisible();
  await expect(page.getByRole("cell", { name: "Sachanlagen" })).toBeVisible();
  await bild(page, "finanzen");

  await page.keyboard.press("?");
  const hilfe = page.getByRole("dialog", { name: "Tastaturkürzel" });
  await expect(hilfe).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(hilfe).toBeHidden();

  await page.keyboard.press("Control+s");
  await expect(page.getByRole("dialog", { name: "Spiel speichern" })).toBeVisible();
});

test("Formulare schicken die richtigen Befehle", async ({ page }) => {
  await starten(page);
  await page.getByRole("button", { name: "Standorte" }).click();
  await page.getByRole("button", { name: "Werk · Deutschland öffnen" }).click();

  const anlage = page.getByRole("form", { name: "Nagelmaschine steuern" });
  await anlage.getByLabel("Geplante Auslastung").fill("60");
  await anlage.getByRole("button", { name: "Übernehmen" }).click();
  // The answer appears at the form, not at the top of the page.
  await expect(anlage.getByRole("alert")).toHaveText(/Vorschau im Browser führt keine Befehle aus/);

  await page.getByRole("button", { name: "Einkauf" }).click();
  const einkauf = page.getByRole("form", { name: "Einkauf von Draht festlegen" });
  await einkauf.getByLabel("Ziellager").fill("30");
  // German input: the point separates thousands.
  await einkauf.getByLabel("Höchstpreis").fill("2.500");
  await einkauf.getByRole("button", { name: /Einkauf (ändern|starten)/ }).click();

  await page.getByRole("button", { name: "Verkauf" }).click();
  const verkauf = page.getByRole("form", { name: "Preis für Nägel festlegen" });
  await verkauf.getByRole("button", { name: "Preis für Nägel um 5 % senken" }).click();
  await verkauf.getByLabel("Fester Preis").check();
  await verkauf.getByLabel("Preis", { exact: true }).fill("2.400,50");
  await verkauf.getByRole("button", { name: "Übernehmen" }).click();

  await page.getByRole("button", { name: "Personal" }).click();
  const lohn = page.getByRole("form", { name: "Lohnaufschlag" });
  await lohn.getByLabel("Lohnaufschlag").fill("12,5");
  await lohn.getByRole("button", { name: "Übernehmen" }).click();

  await page.getByRole("button", { name: "Finanzen" }).click();
  await page.getByLabel("Betrag (USD)").fill("20000");
  await page.getByLabel("Laufzeit (Jahre)").fill("8");
  await page.getByRole("button", { name: "Aufnehmen" }).click();

  await page.getByRole("button", { name: "Markt" }).click();
  const werbung = page.getByRole("form", { name: "Werbung für Metallwaren" });
  await werbung.getByLabel("Werbebudget je Monat für Metallwaren in USD").fill("5000");
  await werbung.getByRole("button", { name: "Setzen" }).click();

  expect(await befehle(page)).toEqual([
    { SetProduction: { site: 0, slot: 0, recipe: "naegel_maschine", utilization: 0.6 } },
    {
      SetPurchase: {
        site: 0,
        product: "draht",
        target: 30,
        max_price: 25_000_000,
        min_quality: 0,
      },
    },
    // 1 932,4557 USD − 5 %, rounded to cents
    { SetPrice: { site: 0, product: "naegel", price: 18_358_300 } },
    { SetSale: { site: 0, product: "naegel", mode: { Fixed: 24_005_000 }, keep: 0 } },
    { SetWagePremium: { site: 0, premium: 0.125 } },
    { TakeLoan: { amount: 200_000_000, years: 8 } },
    { SetAdvertising: { country: "DEU", group: "metallwaren", budget: 50_000_000 } },
  ]);
});

test("Die Einführung führt durch die Ansichten und lässt sich neu starten", async ({ page }) => {
  await starten(page, true);
  const einfuehrung = page.getByRole("complementary", { name: "Einführung" });
  await expect(einfuehrung.getByRole("heading", { name: "Willkommen" })).toBeVisible();
  await expect(einfuehrung.getByText("Schritt 1 von 8")).toBeVisible();
  await bild(page, "einfuehrung");

  const weiter = einfuehrung.getByRole("button", { name: "Weiter" });
  await weiter.click();
  await weiter.click();
  await expect(einfuehrung.getByRole("heading", { name: "Standorte" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Standorte" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await weiter.click();
  await weiter.click();
  await expect(einfuehrung.getByRole("heading", { name: "Marke und Werbung" })).toBeVisible();
  await expect(page.getByRole("table", { name: "Marke und Werbung Deutschland" })).toBeVisible();
  await einfuehrung.getByRole("button", { name: "Zurück" }).click();
  await expect(einfuehrung.getByRole("heading", { name: "Markt" })).toBeVisible();

  await einfuehrung.getByRole("button", { name: "Einführung beenden" }).click();
  await expect(einfuehrung).toBeHidden();

  await page.keyboard.press("?");
  await page
    .getByRole("dialog", { name: "Tastaturkürzel" })
    .getByRole("button", { name: "Einführung starten" })
    .click();
  await expect(einfuehrung.getByText("Schritt 1 von 8")).toBeVisible();
  for (let i = 0; i < 7; i++) await weiter.click();
  await expect(page.getByRole("button", { name: "Weltkarte" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await einfuehrung.getByRole("button", { name: "Fertig" }).click();
  await expect(einfuehrung).toBeHidden();
});

test("Berichte sammeln die Runden der Sitzung", async ({ page }) => {
  await starten(page);
  await page.keyboard.press("7");
  await expect(page.getByText("Noch keine Runde beendet.")).toBeVisible();

  await page.keyboard.press("Control+Enter");
  const ereignis = page.getByRole("dialog", { name: "Erster Weltkrieg" });
  await ereignis.getByRole("button", { name: "Weiter" }).click();
  const bericht = page.getByRole("dialog", { name: "Rundenbericht" });
  await expect(bericht).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(bericht).toBeHidden();

  await page.locator("#berichte_gruppe").selectOption("welt");
  await expect(page.getByText(/Erster Weltkrieg/).first()).toBeVisible();
  await page.getByRole("button", { name: "Bericht öffnen" }).click();
  await expect(bericht).toBeVisible();
});
