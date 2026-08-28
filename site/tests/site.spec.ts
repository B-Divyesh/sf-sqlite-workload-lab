import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("has a clean accessible document and no console errors", async ({ page }) => {
  const errors: string[] = [];
  page.on("console", (message) => { if (message.type() === "error") errors.push(message.text()); });
  await page.goto("/");
  await expect(page).toHaveTitle(/SQLite Workload Lab/);
  await expect(page.locator("h1")).toHaveCount(1);
  await expect(page.locator("main")).toBeVisible();
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations.filter((violation) => ["serious", "critical"].includes(violation.impact ?? ""))).toEqual([]);
  expect(errors).toEqual([]);
});

test("report tabs work with pointer and arrow keys", async ({ page }) => {
  await page.goto("/#demo");
  const run = page.getByRole("tab", { name: "Run", exact: true });
  const context = page.getByRole("tab", { name: "Context", exact: true });
  await run.focus();
  await page.keyboard.press("ArrowRight");
  await expect(context).toBeFocused();
  await expect(context).toHaveAttribute("aria-selected", "true");
  await expect(page.getByRole("tabpanel", { name: "Context", exact: true })).toContainText("profile_match");
  await page.getByRole("tab", { name: "CI diff" }).click();
  await expect(page.getByRole("cell", { name: /18.18% regression/ })).toBeVisible();
});

test("390px layout has no horizontal page overflow", async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== "mobile-390", "mobile-only assertion");
  await page.goto("/");
  const sizes = await page.evaluate(() => ({ scroll: document.documentElement.scrollWidth, client: document.documentElement.clientWidth }));
  expect(sizes.scroll).toBeLessThanOrEqual(sizes.client);
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  await expect(page.getByRole("link", { name: "Run your first profile" })).toBeVisible();
});

test("cached documentation explains offline state", async ({ page, context }) => {
  await page.goto("/");
  await page.evaluate(async () => { await navigator.serviceWorker.ready; });
  await page.reload();
  await context.setOffline(true);
  await page.reload();
  await expect(page.getByRole("status")).toContainText("Offline mode");
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  await context.setOffline(false);
});
