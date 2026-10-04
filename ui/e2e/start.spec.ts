import { expect, test } from "@playwright/test";

test("Hauptmenü lädt im Browser", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveTitle("WSim – Wirtschaftssimulation 1900–2100");
  await expect(page.getByRole("heading", { name: "WSim" })).toBeVisible();
  await expect(page.getByRole("status")).toHaveText(/Vorschau im Browser/);
});

test("Neues Spiel, Runde, Bericht, Speichern und Laden", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Neues Spiel" }).click();

  await page.getByLabel("Name der Firma").fill("Rheinische Nagelwerke");
  await expect(page.getByLabel("Startland (Firmensitz)")).toHaveValue("DEU");
  await page.getByLabel("Anzahl KI-Firmen").fill("100");
  await page.getByLabel("Schwierigkeit der KI").selectOption("schwer");
  await page.getByRole("button", { name: "Spiel starten" }).click();

  const kopf = page.locator(".kopfleiste");
  await expect(kopf.getByText("Rheinische Nagelwerke")).toBeVisible();
  await expect(kopf.getByText("01.01.1914")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Standorte" })).toBeVisible();
  await expect(page.getByText("Carnegie Steel Company")).toBeVisible();

  await page.getByLabel("Rundenlänge").selectOption("woche");
  await page.getByRole("button", { name: "Runde beenden" }).click();
  await expect(page.getByRole("dialog", { name: "Runde läuft" })).toBeVisible();
  const ereignis = page.getByRole("dialog", { name: "Erster Weltkrieg" });
  await expect(ereignis).toBeVisible();
  await expect(ereignis.getByText("Krieg", { exact: true })).toBeVisible();
  await ereignis.getByRole("button", { name: "Weiter" }).click();
  const bericht = page.getByRole("dialog", { name: "Rundenbericht" });
  await expect(bericht).toBeVisible();
  await expect(
    bericht
      .getByRole("cell", { name: "Umsatz" })
      .or(bericht.getByRole("rowheader", { name: "Umsatz" })),
  ).toBeVisible();
  // The world news can be opened again from the report.
  await bericht.getByRole("button", { name: "Ansehen" }).first().click();
  await page
    .getByRole("dialog", { name: "Erster Weltkrieg" })
    .getByRole("button", { name: "Weiter" })
    .click();
  await page
    .getByRole("dialog", { name: "Rundenbericht" })
    .getByRole("button", { name: "Weiter" })
    .click();
  await expect(kopf.getByText("01.08.1914")).toBeVisible();

  await page.getByRole("button", { name: "Speichern" }).click();
  const speichern = page.getByRole("dialog", { name: "Spiel speichern" });
  await speichern.getByLabel("Name des Spielstands").fill("Probe 1914");
  await speichern.getByRole("button", { name: "Speichern" }).click();
  await expect(speichern.getByText("Gespeichert: Probe 1914")).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(speichern).toBeHidden();

  await page.getByRole("button", { name: "Hauptmenü" }).click();
  await page.getByRole("button", { name: "Spiel laden" }).click();
  const laden = page.getByRole("dialog", { name: "Spiel laden" });
  await expect(laden.getByText("Probe 1914")).toBeVisible();
  await laden.getByRole("button", { name: "Laden" }).click();
  await expect(kopf.getByText("Rheinische Nagelwerke")).toBeVisible();
});

test("Ein leerer Firmenname wird nicht abgeschickt", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Neues Spiel" }).click();
  await page.getByRole("button", { name: "Spiel starten" }).click();
  // The empty company name is refused by the browser before anything is sent.
  await expect(page.getByRole("heading", { name: "Neues Spiel" })).toBeVisible();
});

test("Weltkarte mit Ebenen und Länderdetail", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Neues Spiel" }).click();
  await page.getByLabel("Name der Firma").fill("Kartenprobe");
  await page.getByRole("button", { name: "Spiel starten" }).click();

  await page.getByRole("button", { name: "Weltkarte" }).click();
  const karte = page.getByRole("group", { name: "Weltkarte" });
  await expect(karte).toBeVisible();
  await expect(page.getByRole("list", { name: "Legende" })).toContainText("USD/h");

  await page.getByRole("radio", { name: "Rohstoffe" }).click();
  await page.getByLabel("Rohstoff").selectOption("kohle");
  await expect(page.getByRole("list", { name: "Legende" })).toContainText("Konzession frei");

  await karte.getByRole("button", { name: "Deutschland" }).click();
  const detail = page.getByRole("complementary", { name: "Deutschland" });
  await expect(detail.getByRole("heading", { name: "Arbeitskräfte und Löhne" })).toBeVisible();
  await expect(detail.getByText("Ruhrgebiet")).toBeVisible();
  await expect(detail.getByText("Kartenprobe")).toBeVisible();

  await karte.getByRole("button", { name: "Frankreich" }).press("Enter");
  await expect(
    page.getByRole("complementary", { name: "Frankreich" }).getByRole("alert"),
  ).toContainText("Die Vorschau enthält nur");
});
