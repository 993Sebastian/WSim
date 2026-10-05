import { expect, test } from "@playwright/test";

const bilder = process.env.WSIM_BILDER;

// A whole short game against the real core in the browser: start, play a week, save,
// reload the page and continue from the save kept in the browser's storage.
test("Spielen, speichern und nach dem Neuladen weiterspielen", async ({ page }, info) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Neues Spiel" }).click();
  await page.getByLabel("Name der Firma").fill("Browser AG");
  await page.getByText(/Weitere Einstellungen/).click();
  await page.getByLabel("Anzahl KI-Firmen").fill("8");
  await page.getByLabel(/Einführung zeigen/).uncheck();
  await page.getByRole("button", { name: "Spiel starten" }).click();
  await expect(page.locator(".kopfleiste")).toContainText("Browser AG");
  if (bilder) await page.screenshot({ path: `${bilder}/web-${info.project.name}-start.png` });

  await page.getByLabel("Rundenlänge").selectOption("woche");
  await page.getByRole("button", { name: "Runde beenden" }).click();
  const bericht = page.getByRole("dialog", { name: /Rundenbericht/ });
  await expect(bericht).toBeVisible();
  await expect(bericht).toContainText("01.01.1900 bis 07.01.1900");
  await bericht.getByRole("button", { name: "Weiter" }).click();

  await page.getByRole("button", { name: "Menü" }).click();
  await page.getByRole("menuitem", { name: /Speichern/ }).click();
  const speichern = page.getByRole("dialog", { name: "Spiel speichern" });
  await speichern.getByLabel("Name des Spielstands").fill("Probe");
  await speichern.getByRole("button", { name: "Speichern" }).click();
  await expect(speichern).toContainText("Gespeichert: Probe");

  await page.reload();
  await page.getByRole("button", { name: "Spiel laden" }).click();
  const laden = page.getByRole("dialog", { name: "Spiel laden" });
  await expect(laden).toContainText("Browser AG");

  // A save as a file and back (to move it between devices).
  const [datei] = await Promise.all([
    page.waitForEvent("download"),
    laden.getByRole("button", { name: "Spielstand Probe als Datei herunterladen" }).click(),
  ]);
  expect(datei.suggestedFilename()).toBe("Probe.wsim");
  const pfad = info.outputPath("Kopie.wsim");
  await datei.saveAs(pfad);
  await laden.getByLabel("Spielstand aus einer Datei").setInputFiles(pfad);
  await expect(laden).toContainText("„Kopie“ eingelesen");

  await laden
    .getByRole("listitem")
    .filter({ hasText: "Kopie" })
    .getByRole("button", { name: "Laden", exact: true })
    .click();
  await expect(page.locator(".kopfleiste")).toContainText("08.01.1900");
  await expect(page.locator(".kopfleiste")).toContainText("Browser AG");
});
