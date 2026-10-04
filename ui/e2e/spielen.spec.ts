import { expect, test, type Page } from "@playwright/test";

const bilder = process.env.WSIM_BILDER;

async function starten(page: Page) {
  await page.goto("/");
  await page.getByRole("button", { name: "Neues Spiel" }).click();
  await page.getByLabel("Name der Firma").fill("Rheinische Nagelwerke");
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
  await expect(page.getByText(/Zu wenig Angelernte/)).toBeVisible();
  await bild(page, "produktion");

  await page.keyboard.press("3");
  await expect(page.getByRole("table", { name: "Markt Deutschland" })).toBeVisible();
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
  ]);
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

  await page.getByLabel("Meldungen").selectOption("welt");
  await expect(page.getByText(/Erster Weltkrieg/).first()).toBeVisible();
  await page.getByRole("button", { name: "Bericht öffnen" }).click();
  await expect(bericht).toBeVisible();
});
