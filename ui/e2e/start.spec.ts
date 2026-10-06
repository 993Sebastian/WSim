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
  await page.getByText(/Weitere Einstellungen/).click();
  await page.getByLabel("Anzahl KI-Firmen").fill("100");
  await page.getByLabel("Schwierigkeit der KI").selectOption("schwer");
  await page.getByRole("button", { name: "Spiel starten" }).click();

  const kopf = page.locator(".kopfleiste");
  await expect(kopf.getByText("Rheinische Nagelwerke")).toBeVisible();
  await expect(kopf.getByText("01.01.1914")).toBeVisible();
  // The introduction is on by default and leaves the game usable.
  await expect(page.getByRole("complementary", { name: "Einführung" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Standorte" })).toBeVisible();
  await expect(page.getByText(/100 aktive KI-Firmen/)).toBeVisible();

  await page.getByLabel("Rundenlänge").selectOption("woche");
  await page.getByRole("button", { name: "Runde beenden" }).click();
  await expect(page.getByRole("dialog", { name: "Runde läuft" })).toBeVisible();
  const ereignis = page.getByRole("dialog", { name: "Erster Weltkrieg" });
  await expect(ereignis).toBeVisible();
  await expect(ereignis.getByText("Krieg", { exact: true })).toBeVisible();
  await ereignis.getByRole("button", { name: "Weiter" }).click();
  const bericht = page.getByRole("dialog", { name: "Rundenbericht" });
  await expect(bericht).toBeVisible();
  await expect(bericht.getByRole("rowheader", { name: "Umsatz", exact: true })).toBeVisible();
  // What each product brought in the round.
  await expect(
    bericht.getByRole("table", { name: /Was lief/ }).getByRole("cell", { name: "Nägel" }),
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

  // Saving, loading and the main menu sit in the menu of the header.
  await page.getByRole("button", { name: "Menü" }).click();
  await page.getByRole("menuitem", { name: /Speichern/ }).click();
  const speichern = page.getByRole("dialog", { name: "Spiel speichern" });
  await speichern.getByLabel("Name des Spielstands").fill("Probe 1914");
  await speichern.getByRole("button", { name: "Speichern" }).click();
  await expect(speichern.getByText("Gespeichert: Probe 1914")).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(speichern).toBeHidden();

  await page.getByRole("button", { name: "Menü" }).click();
  await page.getByRole("menuitem", { name: "Hauptmenü" }).click();
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
  await page.getByLabel(/Einführung zeigen/).uncheck();
  await page.getByRole("button", { name: "Spiel starten" }).click();

  await page.getByRole("button", { name: "Weltkarte" }).click();
  const karte = page.getByRole("group", { name: "Weltkarte" });
  await expect(karte).toBeVisible();
  // First layer: where a product sells at what price against the reference price.
  await expect(page.getByLabel("Produkt")).toHaveValue("naegel");
  await expect(page.getByRole("list", { name: "Legende" })).toContainText("Nachfrage");
  await page.getByRole("radio", { name: "Lohnniveau" }).click();
  // Amounts in the currency of the headquarters.
  await expect(page.getByRole("list", { name: "Legende" })).toContainText("€/h");

  await page.getByRole("radio", { name: "Rohstoffe" }).click();
  await page.getByLabel("Rohstoff").selectOption("kohle");
  await expect(page.getByRole("list", { name: "Legende" })).toContainText("Konzession frei");

  await karte.getByRole("button", { name: "Deutschland" }).click();
  const detail = page.getByRole("complementary", { name: "Deutschland" });
  await expect(detail.getByRole("heading", { name: "Arbeitskräfte und Löhne" })).toBeVisible();
  await expect(detail.getByText("Ruhrgebiet")).toBeVisible();
  await expect(detail.getByText("Kartenprobe")).toBeVisible();
  // Commercial land: how much is taken, free plots and land prices by location.
  await expect(detail.getByRole("heading", { name: "Gewerbeflächen" })).toBeVisible();
  await expect(detail).toContainText(/[\d.]+ ha, davon [\d,]+ % belegt/);
  await expect(detail.getByRole("cell", { name: "Hafen" })).toBeVisible();
  // The currency of the time with its rate, and the ones that followed.
  await expect(detail).toContainText("Mark (M) · 4,22 M je US-Dollar (1914)");
  await expect(detail).toContainText(
    "Im Lauf der Zeit: Mark, ab Dezember 1923 Reichsmark, ab Juni 1948 D-Mark, ab 1999 Euro",
  );

  await karte.getByRole("button", { name: "Frankreich" }).press("Enter");
  const frankreich = page.getByRole("complementary", { name: "Frankreich" });
  await expect(frankreich.getByRole("alert")).toContainText("Die Vorschau enthält nur");
  // From the map to a new site in that country.
  await frankreich.getByRole("button", { name: "Standort hier gründen" }).click();
  const gruenden = page.getByRole("form", { name: "Neuer Standort" });
  await expect(gruenden.getByLabel("Land")).toHaveValue("FRA");
});
