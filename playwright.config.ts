import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright configuration for the Araf end-to-end test suite.
 *
 * The BFF and tenant console preview server are started by the test fixture
 * itself so that free ports can be discovered at runtime. Do not add a
 * `webServer` block here.
 */
export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1,
  reporter: "list",
  use: {
    baseURL: process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:4173",
    trace: "on-first-retry",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
