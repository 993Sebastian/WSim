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
  const haken = page.getByLabel(/Einführung zeigen/);
  await expect(haken).toBeChecked();
  if (!einfuehrung) await haken.uncheck();
  await page.getByRole("button", { name: "Spiel starten" }).click();
  // The person founds its company right after the start (PE3).
  await gruenden(page, "Rheinische Nagelwerke");
  await expect(page.locator(".kopfleiste")).toBeVisible();
}

async function gruenden(page: Page, name: string) {
  const dialog = page.getByRole("dialog", { name: "Firma gründen" });
  await dialog.getByLabel("Name der Firma").fill(name);
  await dialog.getByRole("button", { name: "Firma gründen" }).click();
  await expect(dialog).toBeHidden();
}

/** The commands the preview received after the founding at the start. */
const befehle = (page: Page) =>
  page.evaluate(() =>
    ((globalThis as { __wsimBefehle?: unknown[] }).__wsimBefehle ?? []).filter(
      (b) => !(typeof b === "object" && b !== null && "FoundCompany" in b),
    ),
  );

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
  await expect(maschine.getByText(/0,504 t Draht → 0,494 t Nägel/)).toBeVisible();
  await bild(page, "werk_anlagen");

  // Sales: last month's sales, the price beside market price and unit cost.
  await werk.getByRole("button", { name: "Verkauf" }).click();
  const naegel = werk.getByRole("article", { name: "Verkauf von Nägel" });
  await expect(naegel.getByText("15 t", { exact: true })).toBeVisible();
  await expect(naegel.getByText("Stückkosten", { exact: true }).first()).toBeVisible();
  await expect(naegel.getByText(/Marge -8 %/)).toBeVisible();
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
    markt.getByRole("cell", { name: "98 % Pittsburgh Machine Works & Sons" }).first(),
  ).toBeVisible();
  // Openings for a newcomer.
  await page.getByLabel("Chancen").check();
  await expect(markt.getByRole("button", { name: "Markt für Eisenerz öffnen" })).toBeVisible();
  await expect(markt.getByRole("button", { name: "Markt für Stabstahl öffnen" })).toBeHidden();
  await page.getByLabel("alle Produkte").check();
  await expect(markt.getByRole("button", { name: "Markt für Stabstahl öffnen" })).toBeVisible();
  await bild(page, "markt");
  // One product market: sellers with shares and who buys.
  await markt.getByRole("button", { name: "Markt für Nägel öffnen" }).click();
  const naegelMarkt = page.getByRole("region", { name: "Nägel in Deutschland" });
  await expect(
    naegelMarkt
      .getByRole("table", { name: "Anbieter im Vormonat" })
      .getByText("Pittsburgh Machine Works & Sons"),
  ).toBeVisible();
  await expect(naegelMarkt.getByText("Ärmstes Fünftel")).toBeVisible();
  // The last months: price against the reference price, sales and the own share (M24).
  const verlauf = naegelMarkt.getByRole("region", { name: "Verlauf" });
  await expect(verlauf.getByText("Gestrichelt: Richtpreis im Land.")).toBeVisible();
  await expect(
    verlauf.getByRole("img", {
      name: /^Bezahlter Preis je Monat: 1\.894 USD\/t \(Jan 1914\) bis 1\.999 USD\/t \(Jul 1914\)$/,
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
  await expect(page.getByRole("button", { name: "Wettbewerb" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await expect(page.getByRole("button", { name: /^Angebote/ })).toBeVisible();

  await page.keyboard.press("5");
  // Research by product (Q6): researchable products with their effort, details on click.
  const produkte = page.getByRole("table", { name: "Produkte" });
  await expect(produkte).toBeVisible();
  await produkte.getByRole("button", { name: "Traktor" }).click();
  const traktor = page.getByRole("region", { name: "Traktor" });
  await expect(traktor.getByText("Vorprodukte")).toBeVisible();
  await expect(traktor.getByText(/ab 1917 – vorher teurer/)).toBeVisible();
  await bild(page, "forschung");
  // Locked products name the input that does not exist yet.
  await page.getByLabel("Zeigen").selectOption("gesperrt");
  await produkte.getByRole("button", { name: "Fernseher", exact: true }).click();
  await expect(
    page.getByRole("region", { name: "Fernseher" }).getByText(/Erst erforschbar, wenn es/),
  ).toBeVisible();

  await page.keyboard.press("6");
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

  // The site's own name.
  await page.locator("summary", { hasText: "Umbenennen" }).click();
  const umbenennen = page.getByRole("form", { name: "Umbenennen" });
  await umbenennen.getByLabel("Name des Standorts").fill("  Drahtwerk Nord ");
  await umbenennen.getByRole("button", { name: "Namen speichern" }).click();
  await expect(umbenennen.getByRole("alert")).toHaveText(
    /Vorschau im Browser führt keine Befehle aus/,
  );

  // Closing the site: proceeds and severance shown, confirmed before the command.
  await page.locator("summary", { hasText: "Standort schließen" }).click();
  const schliessen = page.getByRole("form", { name: "Standort schließen" });
  await expect(schliessen.getByRole("row", { name: /Abfindung/ })).toBeVisible();
  const knopf = schliessen.getByRole("button", { name: "Standort schließen" });
  await expect(knopf).toBeDisabled();
  await schliessen.getByRole("checkbox").check();
  await knopf.click();
  await expect(schliessen.getByRole("alert")).toHaveText(
    /Vorschau im Browser führt keine Befehle aus/,
  );

  // How the quality of the product comes about (#191).
  await page.locator("summary", { hasText: "Produktqualität: 52 Punkte" }).click();
  await expect(page.getByRole("row", { name: /Qualität der Vorprodukte/ })).toContainText("+2,4");
  await expect(page.getByRole("row", { name: /Forschungsniveau/ })).toBeVisible();
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
  await expect(einkauf.getByLabel("Höchstpreis")).toHaveValue("2.500");
  await einkauf.getByRole("button", { name: /Einkauf (ändern|starten)/ }).click();
  // The market price of the last months beside the purchase.
  await page.locator("summary", { hasText: "Marktpreis der letzten 12 Monate" }).click();
  await expect(page.getByText("Spanne")).toBeVisible();
  // Sources by quality (#191): taking a market offer sets quality and price limit.
  await page.locator("summary", { hasText: "Bezugsquellen vergleichen" }).click();
  const quellen = page.getByRole("table").filter({ hasText: "Preis frei Werk" });
  await expect(quellen.getByRole("row", { name: /Svenska Järn/ })).toContainText("72");
  await page.getByLabel("Nach Qualität").check();
  await expect(quellen.getByRole("row").nth(1)).toContainText("Svenska Järn");
  await quellen
    .getByRole("row", { name: /Rheinische Hütten/ })
    .getByRole("button", { name: "Übernehmen" })
    .click();
  await expect(einkauf.getByLabel("Mindestqualität")).toHaveValue("45");
  await quellen
    .getByRole("row", { name: /Svenska Järn/ })
    .getByRole("button", { name: "Liefervertrag anbieten" })
    .click();
  const bezugsvertrag = page.getByRole("form", { name: "Liefervertrag mit Svenska Järn" });
  await expect(bezugsvertrag.getByLabel("Mindestqualität")).toHaveValue("67");
  await bezugsvertrag.getByRole("button", { name: "Vertrag anbieten" }).click();
  await expect(bezugsvertrag.getByRole("alert")).toHaveText(
    /Vorschau im Browser führt keine Befehle aus/,
  );
  // Drawing wire from an own works abroad, by need.
  await page.locator("summary", { hasText: "Von eigenem Standort beziehen" }).click();
  const intern = page.getByRole("form", { name: "Bezug von Draht festlegen" });
  await expect(intern).toContainText("Drahtzug Shanghai");
  await intern.getByLabel("Reichweite").fill("25");
  await intern.getByRole("button", { name: "Bezug starten" }).click();
  // A one-off purchase on top of the stock target.
  const pauschal = page.getByRole("form", { name: "Pauschalkauf von Draht" });
  await pauschal.getByLabel("Menge").fill("5.000");
  await pauschal.getByRole("button", { name: "Kaufen" }).click();
  await expect(pauschal.getByRole("alert")).toHaveText(
    /Vorschau im Browser führt keine Befehle aus/,
  );

  // The stock of the site with use, reach and values.
  await page.getByRole("button", { name: "Lager" }).click();
  const lager = page.getByRole("table", { name: "Lager: Werk · Deutschland" });
  await expect(lager.getByRole("row", { name: /Draht/ })).toContainText("10 Tage");
  // Once to another own site: freight and days of the way are shown first.
  const umlagern = page.getByRole("form", { name: "An eigenen Standort liefern" });
  await expect(umlagern).toContainText("38 Tage unterwegs");
  await umlagern.getByLabel("Menge").fill("4");
  await umlagern.getByRole("button", { name: "Liefern" }).click();

  await page.getByRole("button", { name: "Verkauf" }).click();
  const verkauf = page.getByRole("form", { name: "Preis für Nägel festlegen" });
  // The price follows the market: 5 % below it.
  await verkauf.getByLabel("Abweichung vom Marktpreis").fill("-5");
  await verkauf.getByRole("button", { name: "Übernehmen" }).click();
  await verkauf.getByLabel("Fester Preis").check();
  await verkauf.getByLabel("Preis", { exact: true }).fill("2.400,50");
  await expect(verkauf.getByLabel("Preis", { exact: true })).toHaveValue("2.400,50");
  await verkauf.getByRole("button", { name: "Übernehmen" }).click();
  // The price follows the production cost plus a margin.
  await verkauf.getByLabel("Aufschlag auf die Herstellkosten").check();
  await verkauf.getByLabel("Marge").fill("25");
  await verkauf.getByRole("button", { name: "Übernehmen" }).click();

  await page.getByRole("button", { name: "Personal" }).click();
  const lohn = page.getByRole("form", { name: "Lohnaufschlag" });
  await lohn.getByLabel("Lohnaufschlag").fill("12,5");
  await lohn.getByRole("button", { name: "Übernehmen" }).click();
  // Training (W1): neither the strategy nor the site has set a target yet.
  const schulung = page.getByRole("form", { name: "Schulung" });
  await expect(schulung).toContainText("Schulungsziel0 % (kein Ziel)");
  await schulung.getByLabel("Schulungsziel").fill("80");
  await schulung.getByRole("button", { name: "Übernehmen" }).click();

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
    { RenameSite: { site: 0, name: "Drahtwerk Nord" } },
    { CloseSite: { site: 0 } },
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
    // A contract with a source of the comparison: its free quantity a day, price and
    // quality − 5.
    {
      ProposeContract: {
        seller: 7,
        buyer: 0,
        product: "draht",
        per_day: 10,
        price: 1_260_000,
        months: 12,
        min_quality: 67,
        penalty: 0.2,
      },
    },
    {
      SetInternalSupply: { site: 0, product: "draht", source: 5, mode: { Need: { days: 25 } } },
    },
    // The limit suggested: the market price + 10 %.
    { BuyOnce: { site: 0, product: "draht", quantity: 5_000, max_price: 18_656_200 } },
    // 1 887,21 USD − 5 %, rounded to cents
    { TransferGoods: { from: 0, to: 5, product: "draht", quantity: 4 } },
    {
      SetSale: {
        site: 0,
        product: "naegel",
        mode: { Relative: { deviation: -0.05, floor: 0 } },
        keep: 0,
      },
    },
    { SetSale: { site: 0, product: "naegel", mode: { Fixed: 24_005_000 }, keep: 0 } },
    { SetSale: { site: 0, product: "naegel", mode: { CostPlus: { margin: 0.25 } }, keep: 0 } },
    { SetWagePremium: { site: 0, premium: 0.125 } },
    { SetTraining: { site: 0, target: 0.8 } },
    { TakeLoan: { amount: 200_000_000, years: 8 } },
    { SetAdvertising: { country: "DEU", group: "metallwaren", budget: 50_000_000 } },
  ]);
});

test("Standort auf einem gewählten Grundstück gründen und das Grundstück kaufen", async ({
  page,
}) => {
  await starten(page);
  await page.getByRole("button", { name: "Standorte" }).click();
  const gruenden = page.getByRole("form", { name: "Neuer Standort" });
  const wahl = gruenden.getByRole("group", { name: "Grundstück wählen" });
  await expect(wahl).toContainText("ha Gewerbefläche frei");
  const knopf = gruenden.getByRole("button", { name: "Gründen" });
  // No plot chosen yet.
  await expect(knopf).toBeDisabled();
  await expect(wahl).toContainText("Wähle oben ein Grundstück.");
  // All locations: the cheapest four of each.
  const liste = wahl.getByRole("table", { name: "Freie Grundstücke" });
  await expect(liste.getByRole("radio")).toHaveCount(12);
  await wahl.getByLabel("Lage", { exact: true }).selectOption("hafen");
  await expect(liste.getByRole("cell", { name: "Stadt" })).toHaveCount(0);
  await liste.getByRole("radio").first().check();
  await wahl.getByLabel("Pachten", { exact: true }).check();
  await expect(wahl).toContainText(/Gewählt: Hafen, [\d.,]+ (m²|ha) – Pacht [\d.]+ USD je Jahr/);
  await expect(gruenden).toContainText("Kosten jetzt:");
  await expect(knopf).toBeEnabled();
  await knopf.click();
  await bild(page, "grundstueck_waehlen");

  await page.getByRole("button", { name: "Werk · Deutschland öffnen" }).click();
  const grundstueck = page.getByRole("region", { name: "Grundstück" });
  await expect(grundstueck).toContainText(/[\d.,]+ (m²|ha) von [\d.,]+ (m²|ha) belegt/);
  await expect(grundstueck).toContainText("Gepachtet für");
  await grundstueck.getByRole("button", { name: /Grundstück kaufen/ }).click();
  const bauen = page.getByRole("form", { name: "Anlage bauen" });
  await expect(bauen).toContainText(/noch Platz für \d+/);
  // Sizes (M36): a very large unit costs more and makes four times as much.
  await bauen.getByLabel("Größe").selectOption("VeryLarge");
  await expect(bauen.getByLabel("Größe")).toHaveValue("VeryLarge");
  await bauen.scrollIntoViewIfNeeded();
  await bild(page, "bauen_groesse");
  // A facility that does not fit: marked in the list, the button stays off.
  await bauen.getByLabel("Anlage").selectOption("aluminiumhuette");
  await expect(bauen).toContainText("passt nicht mehr aufs Grundstück");
  await expect(bauen.getByRole("button", { name: "Bauen" })).toBeDisabled();

  const gesendet = await befehle(page);
  expect(gesendet).toEqual([
    { FoundSiteOnPlot: { plot: expect.any(Number), kind: "Factory", lease: true } },
    { BuyPlot: { site: 0 } },
  ]);
});

test("Zahlenfelder setzen beim Tippen Tausenderpunkte", async ({ page }) => {
  await starten(page);
  await page.getByRole("button", { name: "Finanzen", exact: true }).click();
  const kredit = page.getByRole("form", { name: "Kredit aufnehmen" });
  const betrag = kredit.getByLabel("Betrag");
  const cursor = () =>
    betrag.evaluate((e) => (e as { selectionStart: number | null }).selectionStart);
  await betrag.pressSequentially("1234567");
  await expect(betrag).toHaveValue("1.234.567");
  // Typing in the middle keeps the caret behind the typed digit.
  for (let i = 0; i < 3; i++) await betrag.press("ArrowLeft");
  await betrag.pressSequentially("09");
  await expect(betrag).toHaveValue("123.409.567");
  expect(await cursor()).toBe(7);
  // Backspace removes digits, not the separators.
  await betrag.press("Backspace");
  await betrag.press("Backspace");
  await expect(betrag).toHaveValue("1.234.567");
  expect(await cursor()).toBe(5);
  await betrag.press("End");
  await betrag.pressSequentially(",5");
  await expect(betrag).toHaveValue("1.234.567,5");
  await kredit.getByRole("button", { name: "Aufnehmen" }).click();
  expect((await befehle(page)).at(-1)).toEqual({
    TakeLoan: { amount: 12_345_675_000, years: 10 },
  });
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

test("Der Rang zeigt den Platz unter allen Firmen", async ({ page }) => {
  await starten(page);
  const wettbewerb = page.getByRole("region", { name: "Wettbewerb" });
  await expect(
    wettbewerb.getByRole("heading", { name: "Dein Rang unter 101 Firmen" }),
  ).toBeVisible();
  const eigenkapital = wettbewerb.locator(".kennzahl", { hasText: "Nach Eigenkapital" });
  const umsatz = wettbewerb.locator(".kennzahl", { hasText: "Nach Umsatz" });
  // At the start nobody has revenue yet: no place by revenue.
  await expect(eigenkapital).toContainText("Platz 101");
  await expect(umsatz).toContainText("noch kein Umsatz");
  await wettbewerb.getByLabel("Wie entsteht: Rang?").first().click();
  await expect(wettbewerb.getByRole("note")).toContainText("deine eingeschlossen");

  await page.keyboard.press("Control+Enter");
  await page
    .getByRole("dialog", { name: "Erster Weltkrieg" })
    .getByRole("button", { name: "Weiter" })
    .click();
  await page.keyboard.press("Escape");
  await expect(umsatz).toContainText("Platz 101");
  await bild(page, "rang");
});

test("Kaufangebote beantworten und selbst bieten", async ({ page }) => {
  await starten(page);
  // The preview's competitor bids for the workshop (hint and counter: core tests).
  await page.keyboard.press("4");
  const angebot = page.getByRole("article", {
    name: "Kaufangebot von Vogt AG",
  });
  await expect(angebot).toContainText("Werk in Deutschland (Nägel)");
  await expect(angebot).toContainText("33.981 USD");
  await expect(angebot).toContainText("01.04.1914");
  // The base value with its parts.
  await angebot.getByLabel("Wie entsteht: Grundwert?").first().click();
  await expect(angebot.getByRole("note")).toContainText("Restwert der Anlagen");
  await expect(angebot.getByRole("note")).toContainText("27.185 USD");
  await bild(page, "angebot");
  await angebot.getByLabel("Wie entsteht: Grundwert?").first().click();

  await angebot.getByRole("button", { name: "Annehmen" }).click();
  await angebot.getByLabel("Preis").fill("50000");
  await expect(angebot.getByLabel("Preis")).toHaveValue("50.000");
  await angebot.getByRole("button", { name: "Gegenangebot machen" }).click();
  const befehleNachAntwort = await befehle(page);
  expect(befehleNachAntwort.slice(-2)).toEqual([
    { AnswerOffer: { offer: 0, answer: "Accept" } },
    { AnswerOffer: { offer: 0, answer: { Counter: { price: 500_000_000 } } } },
  ]);

  // Bid for a competitor's site.
  await page.getByRole("button", { name: "Firmen", exact: true }).click();
  await page.getByRole("button", { name: "Vogt AG" }).click();
  await expect(page.getByRole("heading", { name: "Vogt AG" })).toBeVisible();
  const werk = page.getByRole("article", { name: "Werk in Deutschland" });
  await expect(werk).toContainText("Konserve, Wurstwaren, Schinken, Eier");
  await expect(werk).toContainText("Neubau heute: 12,43 Mrd. USD");
  // Its areas: the food with a brand known everywhere in Germany.
  const bereich = page.getByRole("article", { name: "Bereich Lebensmittel" });
  await expect(bereich).toContainText("Standorte (1): Werk in Deutschland");
  await expect(bereich).toContainText("Marke: Deutschland 100 %");
  await bereich.getByLabel("Wie entsteht: Grundwert?").first().click();
  await expect(bereich.getByRole("note")).toContainText("Marke (Werbung für dieselbe Bekanntheit)");
  await bereich.getByLabel("Wie entsteht: Grundwert?").first().click();
  await bereich.getByLabel("Preis").fill("250000000");
  await expect(bereich.getByLabel("Preis")).toHaveValue("250.000.000");
  await bereich.getByRole("button", { name: "Angebot abgeben" }).click();
  expect((await befehle(page)).at(-1)).toEqual({
    MakeOffer: { seller: 63, object: { Area: "lebensmittel" }, price: 2_500_000_000_000 },
  });
  await werk.getByLabel("Preis").fill("200000000");
  await expect(werk.getByLabel("Preis")).toHaveValue("200.000.000");
  await werk.getByRole("button", { name: "Angebot abgeben" }).click();
  expect((await befehle(page)).at(-1)).toEqual({
    MakeOffer: { seller: 63, object: { Site: 1153 }, price: 2_000_000_000_000 },
  });
  await bild(page, "firma");
});

test("An einem Start-up beteiligen und es fördern", async ({ page }) => {
  await starten(page);
  await page.keyboard.press("8");
  await expect(page.getByRole("heading", { name: "Erfinder und Gründungen" })).toBeVisible();
  await expect(page.getByText(/Du hältst noch keine Beteiligungen/)).toBeVisible();
  await page
    .getByRole("button", { name: "Patricia Lawrence: beteiligen, fördern, verkaufen" })
    .click();
  const detail = page.getByRole("region", { name: "Patricia Lawrence" });
  await expect(detail).toContainText("Armbanduhr mit Automatik (neu, 9 Jahre vor der Zeit)");
  await expect(detail).toContainText("Gründer 100 %");
  // An open round: a pledge of at most the rest.
  const zusage = detail.getByRole("form", { name: "In der Runde zusagen" });
  await expect(zusage).toContainText("Höchstens 12.900 USD");
  await zusage.getByLabel("Betrag").fill("10000");
  await expect(zusage.getByLabel("Betrag")).toHaveValue("10.000");
  await zusage.getByRole("button", { name: "Zusagen" }).click();
  await expect(zusage.getByRole("alert")).toHaveText(/Vorschau im Browser führt keine Befehle aus/);
  const foerdern = detail.getByRole("form", { name: "Fördergeld geben" });
  await foerdern.getByLabel("Betrag").fill("5.000");
  await foerdern.getByRole("button", { name: "Fördern" }).click();
  // Without a stake neither sale nor steering.
  await expect(detail.getByRole("form", { name: "An Investoren verkaufen" })).toHaveCount(0);
  await expect(detail.getByRole("group", { name: "Lenken" })).toHaveCount(0);
  await bild(page, "beteiligung");
  expect((await befehle(page)).slice(-2)).toEqual([
    { InvestInVenture: { venture: 6, amount: 100_000_000 } },
    { GrantVenture: { venture: 6, amount: 50_000_000 } },
  ]);
  await detail.getByRole("button", { name: "Zurück zur Liste" }).click();
  await expect(page.getByRole("table", { name: "Laufend" })).toBeVisible();
  // Without a research center there is nothing to spin off yet.
  await page.getByRole("button", { name: /^Ausgründen/ }).click();
  await expect(
    page.getByText(/Keines deiner Forschungszentren arbeitet an einem Projekt/),
  ).toBeVisible();
});

test("Produktionsketten zeigen Kosten, eigene Abdeckung und führen zum Markt", async ({ page }) => {
  await starten(page);
  await page.getByRole("button", { name: "Markt", exact: true }).click();
  await page.getByRole("button", { name: "Produktionsketten", exact: true }).click();
  const ketten = page.getByRole("region", { name: "Produktionsketten" });
  // The player's own chain comes first: nails from bought wire.
  await expect(ketten.getByLabel("Kette von")).toHaveValue("naegel");
  const naegel = ketten
    .locator("li")
    .filter({ has: page.getByRole("button", { name: "Markt für Nägel öffnen" }) })
    .first();
  await expect(naegel.getByText("stellst du her").first()).toBeVisible();
  await expect(naegel.getByText("verkaufst du").first()).toBeVisible();
  await expect(
    ketten.getByText(/Nagelmaschine · Stückkosten ≈ [\d.]+ USD\/t · Marktpreis [\d.]+ USD\/t/),
  ).toBeVisible();
  await expect(ketten.getByText("kaufst du ein")).toBeVisible();
  // Deeper levels open on demand.
  const stahl = ketten
    .locator("summary")
    .filter({ has: page.getByRole("button", { name: "Markt für Stahl öffnen" }) })
    .first();
  await expect(ketten.getByText(/Abbau: Erzbergwerk/).first()).toBeHidden();
  await stahl.click({ position: { x: 4, y: 8 } });
  await expect(ketten.getByText(/Abbau: Erzbergwerk/).first()).toBeVisible();
  await bild(page, "ketten");
  // Another chain, then a product's market.
  await ketten.getByLabel("Kette von").selectOption("automobil");
  await expect(ketten.getByRole("button", { name: "Markt für Motor öffnen" })).toBeVisible();
  await ketten.getByLabel("Kette von").selectOption("naegel");
  await ketten.getByRole("button", { name: "Markt für Draht öffnen" }).first().click();
  await expect(page.getByRole("region", { name: "Draht in Deutschland" })).toBeVisible();
});

test("Lieferverträge: laufender Vertrag und neues Angebot an einen Lieferanten", async ({
  page,
}) => {
  await starten(page);
  await page.getByRole("button", { name: "Markt", exact: true }).click();
  await page.getByRole("button", { name: "Lieferverträge", exact: true }).click();
  const liste = page.getByRole("table", { name: "Lieferverträge" });
  await expect(liste.getByRole("row").nth(1)).toContainText("Draht");
  await expect(liste.getByRole("row").nth(1)).toContainText("Läuft");
  await expect(liste.getByRole("button", { name: /^Kündigen/ })).toBeVisible();
  const neu = page.getByRole("region", { name: "Neuer Liefervertrag" });
  await neu.getByLabel("Produkt").selectOption("draht");
  await expect(neu.getByLabel("Lieferant", { exact: true })).toContainText("Barker Ltd.");
  // The example's workshop hardly needs wire (half of it rounds to 0): ask for 10 t a day.
  await neu.getByLabel("Menge je Tag").fill("10");
  await neu.getByRole("button", { name: "Vertrag anbieten" }).click();
  await expect
    .poll(async () => JSON.stringify((await befehle(page)).at(-1)))
    .toContain('"ProposeContract"');
  await bild(page, "vertraege");
});

test("Logistik: Weg der Ladungen, Flotte und Fahrzeugkauf", async ({ page }) => {
  await starten(page);
  await page.getByRole("button", { name: "Markt", exact: true }).click();
  await page.getByRole("button", { name: "Logistik", exact: true }).click();
  await expect(page.getByRole("radio", { name: /Eigene Flotte/ })).toBeChecked();
  const flotte = page.getByRole("table", { name: "Eigene Flotte" });
  await expect(flotte.getByRole("row").nth(1)).toContainText("Fuhrwerk");
  const kauf = page.getByRole("table", { name: "Fahrzeuge kaufen" });
  const bahn = kauf.getByRole("row").filter({ hasText: "Eisenbahn" });
  await bahn.getByLabel("Anzahl").fill("2");
  await bahn.getByRole("button", { name: "Kaufen" }).click();
  await expect
    .poll(async () => JSON.stringify((await befehle(page)).at(-1)))
    .toContain('"BuyVehicles":{"vehicle":"eisenbahn","count":2}');
  await bild(page, "logistik");
});

test("Tochterfirmen: Tochter, Gründung und Konzernbilanz", async ({ page }) => {
  await starten(page);
  await page.getByRole("button", { name: "Organisation", exact: true }).click();
  await page.getByRole("button", { name: "Tochterfirmen", exact: true }).click();
  const liste = page.getByRole("table", { name: "Tochterfirmen" });
  await expect(liste.getByRole("row").nth(1)).toContainText("Spedition");
  await expect(page.getByRole("table", { name: "Aktiva" })).toBeVisible();
  await page.getByLabel("Name", { exact: true }).fill("Werke Süd");
  await page.getByRole("button", { name: "Gründen" }).click();
  await expect
    .poll(async () => JSON.stringify((await befehle(page)).at(-1)))
    .toContain('"FoundSubsidiary"');
  await bild(page, "tochterfirmen");
});

test("Controlling: Ebenen aufklappen und Kostenarten sehen", async ({ page }) => {
  await starten(page);
  await page.getByRole("button", { name: "Finanzen", exact: true }).click();
  await page.getByRole("button", { name: "Controlling", exact: true }).click();
  const tabelle = page.getByRole("table", { name: "Ergebnis je Ebene" });
  await expect(tabelle).toContainText("Konzern");
  await tabelle.getByRole("button", { name: "Europa aufklappen" }).click();
  await expect(tabelle).toContainText("Deutschland");
  await tabelle.getByRole("button", { name: "Deutschland", exact: true }).click();
  await expect(page.getByRole("region", { name: "Einzelheiten: Deutschland" })).toBeVisible();
  await bild(page, "controlling");
});

test("Anleihen: Bonität und Hinweis für kleine Firmen", async ({ page }) => {
  await starten(page);
  await page.getByRole("button", { name: "Finanzen", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Anleihen" })).toBeVisible();
  await expect(page.getByText(/^Bonität:/)).toBeVisible();
  await expect(page.getByText(/erst von Firmen mit mindestens/)).toBeVisible();
});

test("Börse: Index, notierte Firmen und Aktienkauf", async ({ page }) => {
  await starten(page);
  await page.getByRole("button", { name: "Finanzen", exact: true }).click();
  await page.getByRole("button", { name: "Börse", exact: true }).click();
  await expect(page.getByText(/^Börsenindex:/)).toBeVisible();
  await expect(page.getByRole("heading", { name: "Deine Firma an der Börse" })).toBeVisible();
  const tabelle = page.getByRole("table", { name: "Börsennotierte Firmen" });
  const zeile = tabelle.getByRole("row").nth(1);
  await zeile.getByRole("button", { name: /^Kaufen:/ }).click();
  await expect
    .poll(async () => JSON.stringify((await befehle(page)).at(-1)))
    .toContain('"BuyShares"');
  await bild(page, "boerse");
});

test("Weiterentwicklung zeigt Niveau, Wirkung, Aufwand und das Niveau der Anbieter", async ({
  page,
}) => {
  await starten(page);
  await page.getByRole("button", { name: "Forschung", exact: true }).click();
  await page.getByRole("button", { name: "Weiterentwicklung", exact: true }).click();
  const wirkung = page.getByText(
    /\+30 Qualität, −20 % Arbeit, −15 % Vorprodukte, \+25 % Ausstoß je Stück/,
  );
  await expect(wirkung.first()).toBeVisible();
  const naegel = page.getByRole("article", { name: "Nägel" });
  await expect(naegel).toContainText("Dein Niveau");
  await expect(naegel).toContainText("Reife des Produkts");
  await expect(naegel).toContainText("mit einem voll besetzten Labor in Deutschland");
  await expect(naegel).toContainText(
    /Kostenetwa [\d.,]+ (Mio\. )?USD mit einem Labor: 20 Forscher/,
  );
  // Without a research center the card leads there.
  await expect(naegel).toContainText("Zum Forschen brauchst du ein Forschungszentrum");
  await bild(page, "weiterentwicklung");
  // The sellers of a market show their research level.
  await page.getByRole("button", { name: "Markt", exact: true }).click();
  await page.getByRole("button", { name: "Produktionsketten", exact: true }).click();
  await page
    .getByRole("region", { name: "Produktionsketten" })
    .getByRole("button", { name: "Markt für Draht öffnen" })
    .first()
    .click();
  const draht = page.getByRole("region", { name: "Draht in Deutschland" });
  const anbieter = draht.getByRole("table", { name: "Anbieter im Vormonat" });
  await expect(anbieter.getByRole("row").first()).toContainText("Forschung");
  // The quality each seller offers.
  await expect(anbieter.getByRole("row").first()).toContainText("Qualität");
  await expect(anbieter.getByRole("row").nth(1).getByRole("cell").nth(2)).toHaveText("58");
});

test("Erklärungen zerlegen Preis, Nachfrage und Stückkosten", async ({ page }) => {
  await starten(page);
  await page.getByRole("button", { name: "Runde beenden" }).click();
  await page
    .getByRole("dialog", { name: "Erster Weltkrieg" })
    .getByRole("button", { name: "Weiter" })
    .click();
  await page.keyboard.press("Escape");

  // The market price: reference price × price level × market situation.
  await page.getByRole("button", { name: "Markt", exact: true }).click();
  await page.getByRole("button", { name: "Markt für Nägel öffnen" }).click();
  await page.getByLabel("Wie entsteht: Marktpreis?").first().click();
  const preis = page.getByRole("note", { name: "Wie entsteht: Marktpreis?" });
  await expect(preis).toContainText("Richtpreis (Preisniveau 1)1.800 USD/t");
  await expect(preis).toContainText("Preisniveau Deutschland 0,8, davon wirkt 10 %× 0,98");
  await expect(preis).toContainText("Marktlage: 13 % über dem Richtpreis× 1,13");
  await expect(preis).toContainText("Marktpreis= 1.997 USD/t");
  await bild(page, "erklaerung_preis");
  await page.getByRole("button", { name: /Alle Produkte/ }).click();

  // The demand of a consumer good by income fifth.
  await page.getByRole("button", { name: "Markt für Möbel öffnen" }).click();
  await page.getByLabel("Wie entsteht: Nachfrage der Verbraucher?").first().click();
  const nachfrage = page.getByRole("note", { name: "Wie entsteht: Nachfrage der Verbraucher?" });
  await expect(nachfrage.getByText("Zielbesitz je Kopf")).toBeVisible();
  await expect(nachfrage.getByRole("row")).toHaveCount(6);
  await expect(nachfrage).toContainText("Die Haushalte vergleichen den Preis");
  await page.getByRole("button", { name: /Alle Produkte/ }).click();

  // The unit cost of the own nails.
  await page.getByRole("button", { name: "Standorte" }).click();
  await page.getByRole("button", { name: "Werk · Deutschland öffnen" }).click();
  await page.getByRole("button", { name: "Verkauf" }).click();
  await page.getByLabel("Wie entsteht: Stückkosten?").first().click();
  const kosten = page.getByRole("note", { name: "Wie entsteht: Stückkosten?" });
  await expect(kosten).toContainText("Material");
  await expect(kosten).toContainText(/Stückkosten= [\d.,]+ USD\/t/);
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
  await expect(einfuehrung.getByText("Schritt 1 von 23")).toBeVisible();
  await bild(page, "einfuehrung");
  // Folded to one line, to see more of the screen.
  await einfuehrung.getByRole("button", { name: "Einführung verkleinern" }).click();
  await expect(einfuehrung.getByRole("button", { name: "Weiter" })).toBeHidden();
  await expect(einfuehrung).toContainText("Schritt 1 von 23 · Willkommen bei");
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
  await expect(einfuehrung).toContainText("26.556 USD");
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
  await expect(titel("Standorte und Grundstücke")).toBeVisible();
  await expect(page.getByRole("group", { name: "Grundstück wählen" })).toBeVisible();
  await weiter.click();
  await expect(titel("Markt")).toBeVisible();
  await weiter.click();
  await expect(titel("Marke und Werbung")).toBeVisible();
  await weiter.click();
  await expect(titel("Wettbewerb")).toBeVisible();
  await expect(page.getByRole("button", { name: "Wettbewerb" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await weiter.click();
  await expect(titel("Forschung")).toBeVisible();
  await weiter.click();
  await expect(titel("Produkte weiterentwickeln")).toBeVisible();
  await expect(page.getByRole("button", { name: "Weiterentwicklung" })).toBeVisible();
  await weiter.click();
  await expect(titel("Finanzen")).toBeVisible();
  await weiter.click();
  await expect(titel("Organisation")).toBeVisible();
  await expect(page.getByRole("button", { name: "Organisation" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await weiter.click();
  await expect(titel("Beteiligungen")).toBeVisible();
  await expect(page.getByRole("button", { name: "Beteiligungen" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await expect(page.getByRole("heading", { name: "Erfinder und Gründungen" })).toBeVisible();
  await weiter.click();
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
  await expect(einfuehrung.getByText("Schritt 1 von 23")).toBeVisible();
  await einfuehrung.getByRole("button", { name: "Einführung beenden" }).click();
  await expect(einfuehrung).toBeHidden();
});

test("Stellen besetzen: Organigramm, Managermarkt und Entlassen", async ({ page }) => {
  await starten(page);
  await page.keyboard.press("7");
  await expect(page.getByRole("button", { name: "Organisation" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  const werk = page.getByRole("article", { name: "Werk · Deutschland" });
  await expect(werk).toContainText("Werksleitung");
  await expect(werk).toContainText("Arthur Morgan");
  await expect(werk).toContainText("71.522 USD");
  await expect(werk).toContainText("Nächste Prüfung: 04.01.1915");
  // Age and planned retirement (PE1).
  await expect(werk).toContainText("35 Jahre · Ruhestand am 01.02.1944 mit 64");
  // The head takes care of the functions without a specialist.
  await expect(werk).toContainText("Auslastung und Rezept, Überkapazität");
  // All skills as levels behind the ⓘ.
  await werk.getByLabel("Wie entsteht: Fähigkeiten?").first().click();
  await expect(werk.getByRole("note")).toContainText("Fachkompetenz Finanzen");
  await expect(werk.getByRole("note")).toContainText("solide");
  await werk.getByLabel("Wie entsteht: Fähigkeiten?").first().click();
  await bild(page, "organisation");

  // Fill the production position from the market.
  await werk.getByRole("button", { name: "Besetzen Produktion (Werk · Deutschland)" }).click();
  await expect(
    page.getByRole("heading", { name: "Produktion · Werk · Deutschland" }),
  ).toBeVisible();
  // The unit's continent comes first; Europe has no candidates in the example right now.
  await expect(page.getByLabel("Bewerber aus")).toHaveValue("europa");
  await expect(page.getByText(/Im Moment gibt es hier keine Bewerber/)).toBeVisible();
  // Candidates of other continents on request.
  await page.getByLabel("Bewerber aus").selectOption("alle");
  const bewerber = page.getByRole("table", { name: "Bewerber" });
  await expect(bewerber.getByRole("row").nth(0)).toContainText("Alter");
  await expect(bewerber.getByRole("row").nth(1)).toContainText("Antonio Ortega");
  await expect(bewerber.getByRole("row").nth(1)).toContainText("Peru, Bolivien und Chile");
  await expect(bewerber.getByRole("row").nth(1)).toContainText("stark");
  await bild(page, "managermarkt");
  await bewerber.getByRole("button", { name: "Einstellen: Antonio Ortega" }).click();
  expect((await befehle(page)).at(-1)).toEqual({
    HireManager: {
      manager: 626,
      position: { unit: { Site: 0 }, role: { Specialist: "produktion" } },
    },
  });
  // The own head could move here instead.
  const eigene = page.getByRole("table", { name: "Eigene Manager versetzen" });
  await eigene.getByRole("button", { name: "Hierher versetzen: Arthur Morgan" }).click();
  expect((await befehle(page)).at(-1)).toEqual({
    MoveManager: {
      manager: 300,
      position: { unit: { Site: 0 }, role: { Specialist: "produktion" } },
    },
  });

  // Dismissing asks first and names the severance.
  await page.getByRole("button", { name: "← Organisation" }).click();
  await werk.getByRole("button", { name: "Entlassen: Arthur Morgan, Werksleitung" }).click();
  await expect(werk).toContainText("Arthur Morgan entlassen? Abfindung 17.880 USD.");
  await werk.getByRole("button", { name: "Ja, entlassen" }).click();
  expect((await befehle(page)).at(-1)).toEqual({ DismissManager: { manager: 300 } });

  // The country's positions (MA3): its head, and a specialist from the market.
  const land = page.getByRole("article", { name: "Land · Deutschland" });
  await expect(land).toContainText("Landesleitung");
  await expect(land).toContainText("Carmen López");
  await expect(land).toContainText("Kraftwerk");
  await land.getByRole("button", { name: "Besetzen Produktion (Land · Deutschland)" }).click();
  await expect(
    page.getByRole("heading", { name: "Produktion · Land · Deutschland" }),
  ).toBeVisible();
  await page.getByLabel("Bewerber aus").selectOption("alle");
  await page
    .getByRole("table", { name: "Bewerber" })
    .getByRole("button", { name: "Einstellen: Antonio Ortega" })
    .click();
  expect((await befehle(page)).at(-1)).toEqual({
    HireManager: {
      manager: 626,
      position: { unit: { Country: "DEU" }, role: { Specialist: "produktion" } },
    },
  });
});

test("Anliegen beantworten und das Budget einer Stelle setzen", async ({ page }) => {
  await starten(page);
  await page.keyboard.press("7");
  const reiter = page.getByRole("navigation", { name: "Organisation" });
  await reiter.getByRole("button", { name: /Anliegen/ }).click();
  // The example game has no open concern (answering one: Organisation.test.tsx).
  const anliegen = page.getByRole("region", { name: "Anliegen" });
  await expect(anliegen).toContainText("Keine offenen Anliegen.");
  await bild(page, "anliegen");

  // The head's budget: it asks about every expense; the player gives it room again.
  await reiter.getByRole("button", { name: "Stellen" }).click();
  const werk = page.getByRole("article", { name: "Werk · Deutschland" });
  await werk.getByText("Werksleitung", { exact: true }).last().click();
  await expect(werk).toContainText("fragt bei jeder Ausgabe");
  await expect(werk).toContainText("Was die Stelle selbst entschieden hat");
  await expect(werk).toContainText("Noch keine eigenen Entscheidungen über ihr Budget.");
  const formular = werk.getByRole("form", { name: "Budget: Werksleitung" });
  await formular.getByLabel("Je Entscheidung").fill("3");
  await formular.getByLabel("Im Jahr").fill("8");
  await formular.getByRole("button", { name: "Budget übernehmen" }).click();
  expect((await befehle(page)).at(-1)).toEqual({
    SetBudget: { position: { unit: { Site: 0 }, role: "Head" }, shares: [0.03, 0.08] },
  });
  await bild(page, "budget");

  // Pay and satisfaction (MA6): the market value, and a head that fills its positions.
  await expect(werk).toContainText("Gehalt und Zufriedenheit");
  await expect(werk).toContainText("Marktwert (seine Forderung heute)");
  // The preview keeps its data: the box stays as it was, the command is sent.
  await werk.getByLabel(/Besetzt freie Fachstellen selbst/).click();
  expect((await befehle(page)).at(-1)).toEqual({
    SetHiringByHead: { position: { unit: { Site: 0 }, role: "Head" }, enabled: true },
  });

  // The headquarters (ZA1, W2): a move to another country and city.
  const sitz = page.getByRole("article", { name: "Hauptsitz" });
  await expect(sitz).toContainText("Hauptsitz: Berlin, Deutschland");
  await expect(sitz.getByRole("table", { name: "Städte in Deutschland" })).toContainText("Hamburg");
  await sitz.getByLabel("Neues Land").selectOption("FRA");
  await sitz.getByLabel(/^Stadt/).selectOption("marseille");
  await sitz.getByRole("button", { name: "Hauptsitz verlegen" }).click();
  expect((await befehle(page)).at(-1)).toEqual({
    SetHeadquarters: { country: "FRA", city: "marseille" },
  });

  // The central departments (ZA2): employees for the finance department.
  const abteilungen = page.getByRole("article", { name: "Zentralabteilungen" });
  await expect(abteilungen).toContainText("Noch keine Angestellten");
  const finanzen = abteilungen.getByRole("form", { name: "Angestellte der Abteilung Finanzen" });
  await finanzen.getByRole("textbox").fill("2");
  await finanzen.getByRole("button", { name: "Festlegen" }).click();
  expect((await befehle(page)).at(-1)).toEqual({
    StaffDepartment: { department: "Finance", staff: 2 },
  });

  // Which concerns halt a run of rounds: a choice in the menu, kept in the browser.
  await page.getByRole("button", { name: "Menü" }).click();
  const halt = page.getByRole("group", { name: "Bei Anliegen anhalten" });
  await expect(halt.getByRole("menuitemradio", { name: "bei wichtigen" })).toHaveAttribute(
    "aria-checked",
    "true",
  );
  await halt.getByRole("menuitemradio", { name: "bei allen" }).click();
  expect(await page.evaluate(() => localStorage.getItem("wsim-anhalten"))).toBe("alle");
});

test("Vorgaben je Ebene in der Strategie", async ({ page }) => {
  await starten(page);
  await page.keyboard.press("7");
  const reiter = page.getByRole("navigation", { name: "Organisation" });
  await reiter.getByRole("button", { name: "Strategie" }).click();
  const tabelle = page.getByRole("table", { name: "Wo gilt was" });
  await expect(tabelle).toContainText("Premium");
  await expect(tabelle).toContainText("von Europa");
  await expect(tabelle).toContainText("hier festgelegt");
  await page.getByRole("button", { name: "Bearbeiten: Deutschland" }).click();
  const preis = page.getByRole("form", { name: "Preis: Deutschland" });
  await expect(preis).toContainText("Gilt: Premium · von Europa");
  await preis.getByLabel("Preisstrategie").selectOption("kampfpreis");
  await preis.getByRole("button", { name: "Hier festlegen" }).click();
  expect((await befehle(page)).at(-1)).toEqual({
    SetStrategy: { scope: { Country: "DEU" }, field: "Price", value: { Price: "Fight" } },
  });
  await bild(page, "strategie");
});

test("Vorstand, Strategieauftrag und Rücksprache", async ({ page }) => {
  await starten(page);
  await page.keyboard.press("7");
  const reiter = page.getByRole("navigation", { name: "Organisation" });
  await reiter.getByRole("button", { name: "Stellen" }).click();
  const vorstand = page.getByRole("region", { name: "Vorstand" });
  await expect(vorstand).toContainText("CEO");
  await expect(vorstand).toContainText("Ressort Finanzen");
  await reiter.getByRole("button", { name: "Rücksprache" }).click();
  const bericht = page.getByRole("article", { name: "Rücksprache 01.01.1914 – 31.03.1914" });
  await expect(bericht.getByRole("region", { name: "Chancen" })).toContainText("Antrag");
  await expect(bericht.getByRole("region", { name: "Risiken" })).toContainText("Ziel verfehlt");
  const auftrag = page.getByRole("form", { name: "Strategieauftrag an den Vorstand" });
  await auftrag.getByLabel("Leitlinie").selectOption("sicherheit");
  await auftrag.getByRole("button", { name: "Auftrag übernehmen" }).click();
  const gesendet = (await befehle(page)).at(-1) as {
    SetMandate: { mandate: { guideline: unknown } };
  };
  expect(gesendet.SetMandate.mandate.guideline).toBe("Safety");
  await bild(page, "ruecksprache");
});

test("Person: Steckbrief, Familie, Rollen und Lebenslauf", async ({ page }) => {
  await starten(page);
  await page.getByRole("button", { name: /^Person: / }).click();
  const steckbrief = page.getByRole("region", { name: "Steckbrief" });
  await expect(steckbrief.getByRole("heading", { name: "Jürgen Albrecht" })).toBeVisible();
  await expect(steckbrief).toContainText("30 Jahre");
  const familie = page.getByRole("region", { name: "Familie" });
  // The example person is married without children yet (children: Person.test.tsx).
  await expect(familie).toContainText("Noch keine Kinder.");
  await expect(familie).toContainText("Weitere Kinder sind möglich");
  await expect(page.getByRole("region", { name: "Rollen" })).toContainText(
    "CEO – du führst die Firma selbst",
  );
  await expect(page.getByRole("region", { name: "Lebenslauf" })).toContainText(
    "01.01.1914Beginn mit der Firma Neue Firma",
  );
  await bild(page, "person");
});

test("Mehrere Monate am Stück bis Jahresende", async ({ page }) => {
  await starten(page);
  await page.getByLabel("Rundenlänge").selectOption("jahresende");
  await page.getByRole("button", { name: "Runde beenden" }).click();
  await expect(page.getByText(/^Runde 2 · Tag \d+ von 31$/)).toBeVisible();
  await page
    .getByRole("dialog", { name: "Erster Weltkrieg" })
    .getByRole("button", { name: "Weiter" })
    .click();
  const bericht = page.getByRole("dialog", { name: "Rundenbericht" });
  await expect(bericht).toContainText("2 Runden am Stück");
  await expect(bericht.getByText("Das Jahr ist zu Ende.")).toBeVisible();
  await bild(page, "mehrere_runden");
});

test("Berichte sammeln die Runden der Sitzung", async ({ page }) => {
  await starten(page);
  // Berichte are the tenth view: key 0.
  await page.keyboard.press("0");
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
