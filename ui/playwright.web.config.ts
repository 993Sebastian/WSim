import { defineConfig, devices } from "@playwright/test";

// The browser version with the real simulation core (WebAssembly in a worker), as it is
// published on GitHub Pages; once at desktop size and once at iPhone size.
export default defineConfig({
  testDir: "./e2e-web",
  forbidOnly: !!process.env.CI,
  reporter: process.env.CI ? "github" : "list",
  timeout: 180_000,
  use: { baseURL: "http://localhost:4174" },
  projects: [
    { name: "desktop", use: { ...devices["Desktop Chrome"] } },
    // WebKit is not installed here; Chromium with the iPhone's screen and touch.
    { name: "iphone", use: { ...devices["iPhone 13"], browserName: "chromium" } },
  ],
  webServer: {
    command:
      "pnpm build:web && pnpm exec vite preview --mode web --outDir dist-web --port 4174 --strictPort",
    port: 4174,
    reuseExistingServer: !process.env.CI,
    timeout: 900_000,
  },
});
