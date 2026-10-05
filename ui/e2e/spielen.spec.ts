import { expect, test, type Page } from "@playwright/test";

const bilder = process.env.WSIM_BILDER;

/**
 * Starts a game in the preview. The expectations are written in game dollars, so the
 * amounts are shown in US dollars unless a test looks at the default (the currency of
 * the headquarters).
 */
async function starten(page: Page, einfuehrung = false, dollar = true) {
  if (dollar)
    await page.addInitScript(() =>
      localStorage.setItem(
        "wsim-geldanzeige",
        JSON.stringify({ waehrung: "dollar", preise: "basis" }),
      ),
    );
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
  await expect(maschine.getByText(/0,51 t Draht → 0,5 t Nägel/)).toBeVisible();
  await bild(page, "werk_anlagen");

  // Sales: last month's sales, the price beside market price and unit cost.
  await werk.getByRole("button", { name: "Verkauf" }).click();
  const naegel = werk.getByRole("article", { name: "Verkauf von Nägel" });
  await expect(naegel.getByText("15,5 t")).toBeVisible();
  await expect(naegel.getByText("Stückkosten", { exact: true })).toBeVisible();
  await expect(naegel.getByText(/Marge 16 %/)).toBeVisible();
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
  const markt = page.getByRole("table", { name: "Markt Deutschland" });
  await expect(markt).toBeVisible();
  // Newcomer against the established companies: the leader and its share.
  await expect(
    markt.getByRole("cell", { name: "99 % Maschinenfabrik Lehmann & Söhne" }),
  ).toBeVisible();
  // Openings for a newcomer.
  await page.getByLabel("Chancen").check();
  await expect(markt.getByRole("button", { name: "Markt für Eisenerz öffnen" })).toBeVisible();
  await expect(markt.getByRole("button", { name: "Markt für Blech öffnen" })).toBeHidden();
  await page.getByLabel("alle Produkte").check();
  await bild(page, "markt");
  // One product market: sellers with shares and who buys.
  await markt.getByRole("button", { name: "Markt für Nägel öffnen" }).click();
  const naegelMarkt = page.getByRole("region", { name: "Nägel in Deutschland" });
  await expect(
    naegelMarkt
      .getByRole("table", { name: "Anbieter im Vormonat" })
      .getByText("Maschinenfabrik Lehmann & Söhne"),
  ).toBeVisible();
  await expect(naegelMarkt.getByText("Ärmstes Fünftel")).toBeVisible();
  // The last months: price against the reference price, sales and the own share (M24).
  const verlauf = naegelMarkt.getByRole("region", { name: "Verlauf" });
  await expect(verlauf.getByText("Gestrichelt: Richtpreis im Land.")).toBeVisible();
  await expect(
    verlauf.getByRole("img", {
      name: /^Bezahlter Preis je Monat: 1\.837 USD\/t \(Jan 1914\) bis 1\.905 USD\/t \(Jul 1914\)$/,
    }),
  ).toBeVisible();
  await expect(
    verlauf.getByRole("img", {
      name: /^Dein Anteil am Absatz: 0 % \(Jan 1914\) bis 0,7 % \(Jul 1914\)$/,
    }),
  ).toBeVisible();
  await bild(page, "produktmarkt");
  await naegelMarkt.getByRole("button", { name: /Alle Produkte/ }).click();
  await page.getByRole("button", { name: "Marke und Werbung", exact: true }).click();
  await expect(page.getByRole("form", { name: "Werbung für Metallwaren" })).toBeVisible();
  await expect(page.getByText(/Bestes Werbemittel: Zeitungsanzeigen/)).toBeVisible();
  await bild(page, "marke");

  await page.keyboard.press("4");
  // The technology tree: a node opens what the technology costs and opens.
  const baum = page.getByRole("group", { name: "Technologiebaum" });
  await expect(baum).toBeVisible();
  await baum.getByRole("button", { name: /^Fließband, 1913/ }).click();
  const fliessband = page.getByRole("region", { name: "Fließband" });
  await expect(fliessband.getByText("Schaltet frei")).toBeVisible();
  await expect(fliessband.getByText(/57 t Stahl → 300 Stück Motor/)).toBeVisible();
  await fliessband.getByRole("button", { name: "Automobil" }).click();
  await expect(page.getByRole("region", { name: "Automobil" })).toBeVisible();
  await bild(page, "forschung");
  await page.getByLabel("Liste").check();
  // Picking from the list moves the focus out of the input, so digits switch views again.
  await page.getByRole("button", { name: /Bessemer-Verfahren/ }).click();
  await expect(page.getByRole("region", { name: "Bessemer-Verfahren" })).toBeVisible();

  await page.keyboard.press("5");
  await expect(page.getByRole("heading", { name: "Bilanz" })).toBeVisible();
  await expect(page.getByRole("cell", { name: "Sachanlagen" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Womit verdienst du Geld?" })).toBeVisible();
  await expect(page.getByRole("table", { name: "Je Produkt" }).getByText("Nägel")).toBeVisible();
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

  await page.getByRole("button", { name: "Finanzen", exact: true }).click();
  const kredit = page.getByRole("form", { name: "Kredit aufnehmen" });
  await kredit.getByLabel("Betrag").fill("20.000");
  await kredit.getByLabel("Laufzeit").fill("8");
  await kredit.getByRole("button", { name: "Aufnehmen" }).click();

  await page.getByRole("button", { name: "Markt", exact: true }).click();
  await page.getByRole("button", { name: "Marke und Werbung", exact: true }).click();
  const werbung = page.getByRole("form", { name: "Werbung für Metallwaren" });
  await werbung.getByLabel("Werbebudget").fill("5.000");
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
    // 1 932,48 USD − 5 %, rounded to cents
    { SetPrice: { site: 0, product: "naegel", price: 18_358_600 } },
    { SetSale: { site: 0, product: "naegel", mode: { Fixed: 24_005_000 }, keep: 0 } },
    { SetWagePremium: { site: 0, premium: 0.125 } },
    { TakeLoan: { amount: 200_000_000, years: 8 } },
    { SetAdvertising: { country: "DEU", group: "metallwaren", budget: 50_000_000 } },
  ]);
});

test("Etappen zeigen das nächste Ziel und lassen sich ausblenden", async ({ page }) => {
  await starten(page);
  const etappen = page.getByRole("region", { name: /Etappen/ });
  await expect(etappen).toContainText("0 von 8 erreicht");
  await expect(etappen).toContainText("Nächste Etappe: Erster Verkauf");
  await expect(etappen).toContainText("So geht's: Biete im Werk unter „Verkauf“");

  // After a round (the preview's example: January 1914 brought the first sale and
  // the first month with a profit).
  await page.keyboard.press("Control+Enter");
  await page
    .getByRole("dialog", { name: "Erster Weltkrieg" })
    .getByRole("button", { name: "Weiter" })
    .click();
  await page.keyboard.press("Escape");
  await expect(etappen).toContainText("2 von 8 erreicht");
  await expect(etappen).toContainText("Nächste Etappe: Zwei Anlagen in Betrieb");
  await expect(etappen).toContainText("1 von 2");
  await expect(etappen).toContainText("So geht's: Unter „Produktion“ → „Anlage bauen“");
  await etappen.getByText("Alle Etappen").click();
  await expect(etappen.getByText(/erreicht am 31\.01\.1914/)).toBeVisible();
  await expect(etappen.getByText("0 % von 30 %")).toBeVisible();
  await bild(page, "etappen");

  await etappen.getByRole("button", { name: "Etappen ausblenden" }).click();
  await expect(etappen).toBeHidden();
  // Kept in the browser: hidden in the next game too, until the menu shows them again.
  await starten(page);
  await expect(page.locator(".kopfleiste")).toBeVisible();
  await expect(etappen).toBeHidden();
  await page.getByRole("button", { name: "Menü" }).click();
  const schalter = page.getByRole("menuitemcheckbox", { name: "Etappen zeigen" });
  await expect(schalter).toHaveAttribute("aria-checked", "false");
  await schalter.click();
  await expect(etappen).toBeVisible();
});

test("Anlagen lassen sich stilllegen und verkaufen", async ({ page }) => {
  await starten(page);
  await page.getByRole("button", { name: "Standorte" }).click();
  await page.getByRole("button", { name: "Werk · Deutschland öffnen" }).click();
  const anlage = page.getByRole("article", { name: "Nagelmaschine" });
  await anlage.getByText("Stilllegen oder verkaufen").click();
  // What the machine is worth and what shutting it down saves.
  await expect(anlage).toContainText("Restbuchwert");
  await expect(anlage).toContainText("38.451 USD");
  await expect(anlage).toContainText("19.226 USD");
  await expect(anlage).toContainText("stillgelegt nur 25 USD");
  await anlage.getByRole("button", { name: "Stilllegen" }).click();
  // Selling asks first and names the loss against the book value.
  await anlage.getByRole("button", { name: "Verkaufen …" }).click();
  await expect(anlage.getByText(/weniger als der Restbuchwert/)).toBeVisible();
  await anlage.getByRole("button", { name: "Abbrechen" }).click();
  await anlage.getByRole("button", { name: "Verkaufen …" }).click();
  await anlage.getByRole("button", { name: "Ja, verkaufen" }).click();
  expect(await befehle(page)).toEqual([
    { MothballFacility: { site: 0, slot: 0, count: 1 } },
    { SellFacility: { site: 0, slot: 0, count: 1 } },
  ]);
});

test("Beträge in der Landeswährung, in US-Dollar und zu Preisen der Zeit", async ({ page }) => {
  await starten(page, false, false);
  const kasse = page.locator(".kopf-firma");
  const hinweis = page.locator(".geld-hinweis");
  // By default: the currency of the headquarters at the purchasing power of 2026.
  await expect(hinweis).toHaveText(/^Beträge in Euro mit der Kaufkraft von 2026\./);
  await expect(kasse).toContainText("€");

  const waehle = async (name: string) => {
    await page.getByRole("button", { name: "Menü" }).click();
    await page.getByRole("menuitemradio", { name }).click();
  };
  await waehle("US-Dollar");
  await expect(hinweis).toHaveText(/^Beträge in US-Dollar mit der Kaufkraft von 2026\./);
  await expect(kasse).toContainText("USD");
  await waehle("Preise der Zeit (mit Inflation)");
  await expect(hinweis).toHaveText(/^Beträge in US-Dollar zu den Preisen von 1914\./);
  // In 1914 Germany paid in Mark.
  await waehle("Mark (Firmensitz)");
  await expect(hinweis).toHaveText(/^Beträge in Mark zu den Preisen von 1914\./);
  await expect(kasse).toContainText(" M");
  await page.getByRole("button", { name: "Menü" }).click();
  await expect(page.getByRole("menuitemradio", { name: "Mark (Firmensitz)" })).toHaveAttribute(
    "aria-checked",
    "true",
  );
  await page.getByRole("button", { name: "Menü" }).click();
  // The choice is kept for the next game.
  expect(await page.evaluate(() => localStorage.getItem("wsim-geldanzeige"))).toBe(
    JSON.stringify({ waehrung: "heimat", preise: "zeit" }),
  );

  // Typed amounts are in the shown currency: 20.000 € are 20.000 / 0,87 game dollars.
  await waehle("Kaufkraft 2026 (ohne Inflation)");
  await page.getByRole("button", { name: "Finanzen", exact: true }).click();
  const kredit = page.getByRole("form", { name: "Kredit aufnehmen" });
  await expect(kredit.getByText("€", { exact: true })).toBeVisible();
  await kredit.getByLabel("Betrag").fill("20.000");
  await kredit.getByLabel("Laufzeit").fill("8");
  await kredit.getByRole("button", { name: "Aufnehmen" }).click();
  expect(await befehle(page)).toEqual([{ TakeLoan: { amount: 229_885_057, years: 8 } }]);
});

test("Die Einführung führt bis zum ersten Verkauf und lässt sich neu starten", async ({ page }) => {
  await starten(page, true);
  const einfuehrung = page.getByRole("complementary", { name: "Einführung" });
  const titel = (name: string | RegExp) => einfuehrung.getByRole("heading", { name });
  const ring = page.locator(".einfuehrung-rahmen");
  await expect(titel(/^Willkommen bei/)).toBeVisible();
  await expect(einfuehrung.getByText("Schritt 1 von 18")).toBeVisible();
  await bild(page, "einfuehrung");
  // Folded to one line, to see more of the screen.
  await einfuehrung.getByRole("button", { name: "Einführung verkleinern" }).click();
  await expect(einfuehrung.getByRole("button", { name: "Weiter" })).toBeHidden();
  await expect(einfuehrung).toContainText("Schritt 1 von 18 · Willkommen bei");
  await einfuehrung.getByRole("button", { name: "Einführung aufklappen" }).click();
  await einfuehrung.getByRole("button", { name: "Weiter" }).click();

  // Each step highlights the button to use and goes on once it was used.
  await expect(titel("Deine Standorte")).toBeVisible();
  await expect(ring).toBeVisible();
  await expect(einfuehrung.getByText(/Klicke auf das hervorgehobene Feld/)).toBeVisible();
  await page.getByRole("button", { name: "Standorte", exact: true }).click();
  await expect(titel("Die Werkstatt öffnen")).toBeVisible();
  await bild(page, "einfuehrung_werk");
  await page.getByRole("button", { name: "Werk · Deutschland öffnen" }).click();
  await expect(titel("Die Anlage")).toBeVisible();
  await einfuehrung.getByRole("button", { name: "Weiter" }).click();
  await expect(titel("Einkauf")).toBeVisible();
  await page
    .getByRole("button", { name: /^Einkauf/ })
    .first()
    .click();
  await expect(titel("Draht einkaufen")).toBeVisible();
  await einfuehrung.getByRole("button", { name: "Weiter" }).click();
  await page
    .getByRole("button", { name: /^Verkauf/ })
    .first()
    .click();
  await expect(titel("Dein Preis")).toBeVisible();
  await bild(page, "einfuehrung_preis");
  // The preview takes no decisions: the price step is skipped.
  await einfuehrung.getByRole("button", { name: "Überspringen" }).click();
  await expect(titel("Personal")).toBeVisible();

  // Back by hand: the step waits even though it is done.
  await einfuehrung.getByRole("button", { name: "Zurück" }).click();
  await expect(titel("Dein Preis")).toBeVisible();
  await einfuehrung.getByRole("button", { name: "Überspringen" }).click();
  await page
    .getByRole("button", { name: /^Personal/ })
    .first()
    .click();

  // The round: world news, then the report with the first sale.
  await expect(titel("Runde beenden")).toBeVisible();
  await page.getByRole("button", { name: "Runde beenden" }).click();
  const ereignis = page.getByRole("dialog", { name: "Erster Weltkrieg" });
  await expect(ereignis).toBeVisible();
  await expect(titel("Dein erster Umsatz")).toBeVisible();
  await ereignis.getByRole("button", { name: "Weiter" }).click();
  const bericht = page.getByRole("dialog", { name: /Rundenbericht/ });
  await expect(einfuehrung).toContainText("29.954 USD");
  await bild(page, "einfuehrung_bericht");
  await bericht.getByRole("button", { name: "Weiter" }).click();
  await expect(titel("Deine Firma läuft")).toBeVisible();
  await expect(page.getByRole("button", { name: /Übersicht/ })).toHaveAttribute(
    "aria-current",
    "page",
  );

  // The goals after the introduction, then a short tour of the other views.
  const weiter = einfuehrung.getByRole("button", { name: "Weiter" });
  await weiter.click();
  await expect(titel("Etappen")).toBeVisible();
  await expect(page.getByRole("region", { name: /Etappen/ })).toBeVisible();
  await weiter.click();
  await expect(titel("Markt")).toBeVisible();
  await weiter.click();
  await expect(titel("Marke und Werbung")).toBeVisible();
  for (let i = 0; i < 3; i++) await weiter.click();
  await expect(page.getByRole("button", { name: "Weltkarte" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await einfuehrung.getByRole("button", { name: "Fertig" }).click();
  await expect(einfuehrung).toBeHidden();
  await expect(ring).toBeHidden();

  await page.keyboard.press("?");
  await page
    .getByRole("dialog", { name: "Tastaturkürzel" })
    .getByRole("button", { name: "Einführung starten" })
    .click();
  await expect(einfuehrung.getByText("Schritt 1 von 18")).toBeVisible();
  await einfuehrung.getByRole("button", { name: "Einführung beenden" }).click();
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
