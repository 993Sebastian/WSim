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
  // Amounts in the currency of the headquarters; at the prices of 1900 that is the Mark.
  await expect(page.locator(".geld-hinweis")).toHaveText(/^Beträge in Euro mit der Kaufkraft/);
  await page.getByRole("button", { name: "Menü" }).click();
  await page.getByRole("menuitemradio", { name: "Preise der Zeit (mit Inflation)" }).click();
  await expect(page.locator(".geld-hinweis")).toHaveText(
    /^Beträge in Mark zu den Preisen von 1900\./,
  );
  await expect(page.locator(".kopf-firma")).toContainText(" M");

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

  // Months at a stretch up to the next warning or world event (M26), real core.
  await page.getByLabel("Rundenlänge").selectOption("meldung");
  await page.getByRole("button", { name: "Runde beenden" }).click();
  // World events come first, each in a window of its own.
  const ereignis = page.locator('[data-tour="ereignis"]');
  await expect(ereignis.or(bericht).first()).toBeVisible({ timeout: 120_000 });
  while (await ereignis.isVisible())
    await ereignis.locator('[data-tour="ereignis-weiter"]').click();
  await expect(bericht).toBeVisible();
  await expect(bericht.locator(".halt-grund")).toBeVisible();
  await bericht.getByRole("button", { name: "Weiter" }).click();

  // The production chains from the real core (M25).
  await page.getByRole("button", { name: "Markt", exact: true }).click();
  await page.getByRole("button", { name: "Produktionsketten", exact: true }).click();
  const ketten = page.getByRole("region", { name: "Produktionsketten" });
  await expect(ketten.getByLabel("Kette von")).toHaveValue("naegel");
  await expect(ketten.getByText(/Nagelmaschine · Stückkosten ≈/)).toBeVisible();
});

// Research with the real core: found a center, build a laboratory, pick a technology.
test("Forschungszentrum gründen und eine Technologie erforschen", async ({ page }, info) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Neues Spiel" }).click();
  await page.getByLabel("Name der Firma").fill("Forschung AG");
  await page.getByLabel(/Einführung zeigen/).uncheck();
  await page.getByText(/Weitere Einstellungen/).click();
  await page.getByLabel("Anzahl KI-Firmen").fill("8");
  await page.getByLabel("Startkapital in USD").fill("5000000");
  await page.getByRole("button", { name: "Spiel starten" }).click();
  await expect(page.locator(".kopfleiste")).toContainText("Forschung AG");

  await page.getByRole("button", { name: "Forschung", exact: true }).click();
  await page.getByRole("button", { name: "Forschungszentren", exact: true }).click();
  // Every site but a mine stands on a plot (M35): take the cheapest free one.
  const neu = page.getByRole("form", { name: "Neues Forschungszentrum" });
  await neu.getByRole("table", { name: "Freie Grundstücke" }).getByRole("radio").first().check();
  await neu.getByRole("button", { name: "Gründen" }).click();
  const zentrum = page.getByRole("article", { name: "Forschungszentrum in Deutschland" });
  await zentrum.getByRole("button", { name: /Labor bauen/ }).click();
  await expect(zentrum).toContainText("Labor im Bau bis");

  await page.getByRole("button", { name: "Technologiebaum", exact: true }).click();
  await page.getByLabel("Liste").check();
  await page.getByRole("button", { name: /Turbogenerator/ }).click();
  const turbo = page.getByRole("region", { name: "Turbogenerator" });
  await expect(turbo.getByText("Turbinenkraftwerk", { exact: true })).toBeVisible();
  await turbo.getByRole("button", { name: "Forschung starten" }).click();
  await expect(turbo).toContainText("Das Zentrum forscht jetzt an Turbogenerator.");
  await expect(turbo).toContainText("Forscht daran");
  if (bilder) await page.screenshot({ path: `${bilder}/web-${info.project.name}-forschung.png` });
});

// The introduction with the real core: from the start to the first sale.
test("Die Einführung führt bis zum ersten Verkauf", async ({ page }, info) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Neues Spiel" }).click();
  await page.getByLabel("Name der Firma").fill("Lehrling AG");
  await page.getByText(/Weitere Einstellungen/).click();
  await page.getByLabel("Anzahl KI-Firmen").fill("8");
  await page.getByRole("button", { name: "Spiel starten" }).click();
  const einfuehrung = page.getByRole("complementary", { name: "Einführung" });
  const titel = (name: string) => einfuehrung.getByRole("heading", { name });
  const markiert = (id: string) => page.locator(`[data-tour="${id}"]`).first();

  await expect(titel("Willkommen bei Lehrling AG")).toBeVisible();
  await einfuehrung.getByRole("button", { name: "Weiter" }).click();
  await markiert("reiter-produktion").click();
  await expect(titel("Die Werkstatt öffnen")).toBeVisible();
  await markiert("werk-oeffnen").click();
  await expect(titel("Die Anlage")).toBeVisible();
  await expect(page.locator(".einfuehrung-rahmen")).toBeVisible();
  await einfuehrung.getByRole("button", { name: "Weiter" }).click();
  await markiert("bereich-einkauf").click();
  await expect(titel("Draht einkaufen")).toBeVisible();
  await einfuehrung.getByRole("button", { name: "Weiter" }).click();
  await markiert("bereich-verkauf").click();
  await expect(titel("Dein Preis")).toBeVisible();
  await page.getByRole("button", { name: "Preis für Nägel um 5 % senken" }).click();
  await expect(titel("Personal")).toBeVisible();
  await markiert("bereich-personal").click();
  await expect(titel("Runde beenden")).toBeVisible();
  await page.getByLabel("Rundenlänge").selectOption("woche");
  await page.getByRole("button", { name: "Runde beenden" }).click();
  await expect(titel("Dein erster Umsatz")).toBeVisible({ timeout: 60_000 });
  if (bilder) await page.screenshot({ path: `${bilder}/web-${info.project.name}-einfuehrung.png` });
  // The first goal reached, reported by the core (M23).
  const bericht = page.getByRole("dialog", { name: /Rundenbericht/ });
  await expect(bericht.getByRole("heading", { name: "Erreicht" })).toBeVisible();
  await expect(bericht.getByText("Etappe erreicht: Erster Verkauf")).toBeVisible();
  await bericht.getByRole("button", { name: "Weiter" }).click();
  await expect(titel("Deine Firma läuft")).toBeVisible();
  await einfuehrung.getByRole("button", { name: "Weiter" }).click();
  await expect(titel("Etappen")).toBeVisible();
  const etappen = page.getByRole("region", { name: /Etappen/ });
  await expect(etappen).toContainText("1 von 8 erreicht");
  await expect(etappen).toContainText("Nächste Etappe: Erster Monat mit Gewinn");
});

// Shutting a machine down and starting it up again with the real core (M22).
test("Anlage stilllegen und wieder anfahren", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Neues Spiel" }).click();
  await page.getByLabel("Name der Firma").fill("Stillstand AG");
  await page.getByLabel(/Einführung zeigen/).uncheck();
  await page.getByText(/Weitere Einstellungen/).click();
  await page.getByLabel("Anzahl KI-Firmen").fill("8");
  await page.getByRole("button", { name: "Spiel starten" }).click();
  await expect(page.locator(".kopfleiste")).toContainText("Stillstand AG");

  await page.getByRole("button", { name: "Standorte" }).click();
  await page.getByRole("button", { name: "Werk · Deutschland öffnen" }).click();
  const anlage = page.getByRole("article", { name: "Nagelmaschine" });
  await anlage.getByText("Stilllegen oder verkaufen").click();
  await anlage.getByRole("button", { name: "Stilllegen" }).click();
  await expect(anlage).toContainText("stillgelegt seit 01.01.1900");
  await anlage.getByRole("button", { name: /Wieder anfahren/ }).click();
  await expect(anlage).toContainText("fährt wieder an bis 31.01.1900");
});

test("Wettbewerber und ihre Standorte mit dem echten Kern", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Neues Spiel" }).click();
  await page.getByLabel("Name der Firma").fill("Bieter AG");
  await page.getByLabel(/Einführung zeigen/).uncheck();
  await page.getByText(/Weitere Einstellungen/).click();
  await page.getByLabel("Anzahl KI-Firmen").fill("8");
  await page.getByRole("button", { name: "Spiel starten" }).click();
  await expect(page.locator(".kopfleiste")).toContainText("Bieter AG");

  await page.getByRole("button", { name: "Wettbewerb" }).click();
  await expect(page.getByText("Im Moment liegen keine offenen Angebote vor.")).toBeVisible();
  await page.getByRole("button", { name: "Firmen", exact: true }).click();
  const tabelle = page.getByRole("table", { name: "Firmen" });
  await expect(tabelle.getByRole("row")).toHaveCount(10);
  await tabelle.getByRole("row").nth(1).getByRole("button").click();
  // At the start the sites are too young to buy; the date when they can be bought shows.
  await expect(page.getByText(/Erst ab 01\.01\.1901 zu kaufen\./).first()).toBeVisible();
});
