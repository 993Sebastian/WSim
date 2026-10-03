import { expect, test } from "@playwright/test";

test("Startbildschirm lädt im Browser", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveTitle("WSim – Wirtschaftssimulation 1900–2100");
  await expect(page.getByRole("heading", { name: "WSim" })).toBeVisible();
  await expect(page.getByRole("status")).toHaveText(/Vorschau im Browser/);
});
