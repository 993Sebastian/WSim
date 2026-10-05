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
  await expect(page.getByRole("button", { name: "Produktion" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await expect(page.getByRole("heading", { name: /Werk · Deutschland/ })).toBeVisible();
  // A machine at its plan shows no bottleneck; last month's sales are shown.
  await expect(page.getByRole("cell", { name: "läuft" })).toBeVisible();
  const verkauf = page.getByRole("table", { name: "Verkauf" });
  await expect(verkauf.getByRole("cell", { name: "15,5" })).toBeVisible();
  await expect(page.getByRole("combobox", { name: "Weiteres Produkt einkaufen" })).toBeVisible();
  await bild(page, "produktion");

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
  await page.getByRole("button", { name: "Produktion" }).click();

  await page.getByLabel("Auslastung von Nagelmaschine in Prozent").fill("60");
  await page.getByLabel("Auslastung von Nagelmaschine in Prozent").press("Enter");
  await expect(page.getByRole("alert")).toHaveText(/Vorschau im Browser führt keine Befehle aus/);

  const einkauf = page.getByRole("form", { name: "Einkauf von Draht" });
  await einkauf.getByLabel("Ziellager für Draht").fill("30");
  await einkauf.getByLabel("Höchstpreis für Draht in USD").fill("2500");
  await einkauf.getByRole("button", { name: "Ändern" }).click();

  const verkauf = page.getByRole("form", { name: "Verkauf von Nägel" });
  await verkauf.getByLabel("Preisart für Nägel").selectOption("fest");
  await verkauf.getByLabel("Festpreis für Nägel in USD").fill("2400");
  await verkauf.getByRole("button", { name: "Ändern" }).click();

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
    { SetSale: { site: 0, product: "naegel", mode: { Fixed: 24_000_000 }, keep: 0 } },
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
  await expect(einfuehrung.getByRole("heading", { name: "Produktion" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Produktion" })).toHaveAttribute(
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
