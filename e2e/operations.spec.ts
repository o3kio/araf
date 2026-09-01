import { test, expect } from "@playwright/test";
import { spawn, type ChildProcess } from "node:child_process";
import { createServer, type AddressInfo } from "node:net";
import { join } from "node:path";

/**
 * Return a free TCP port on 127.0.0.1.
 */
async function getFreePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.unref();
    server.on("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address() as AddressInfo;
      const port = address.port;
      server.close(() => resolve(port));
    });
  });
}

/**
 * Wait for a child process to emit a log line matching the given pattern.
 */
async function waitForLog(
  child: ChildProcess,
  pattern: RegExp,
  timeoutMs = 10_000,
): Promise<string> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      cleanup();
      reject(new Error(`Timed out waiting for log matching ${pattern.source}`));
    }, timeoutMs);

    const onData = (data: Buffer) => {
      const line = data.toString();
      if (pattern.test(line)) {
        clearTimeout(timer);
        cleanup();
        resolve(line);
      }
    };

    const cleanup = () => {
      child.stdout?.off("data", onData);
      child.stderr?.off("data", onData);
    };

    child.stdout?.on("data", onData);
    child.stderr?.on("data", onData);
  });
}

/**
 * Wait until something is listening on the given host:port.
 */
async function waitForPort(host: string, port: number, timeoutMs = 30_000): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  let lastError: Error | undefined;

  while (Date.now() < deadline) {
    try {
      await new Promise<void>((resolve, reject) => {
        const socket = createServer();
        socket.once("error", (err) => {
          socket.close();
          if ((err as NodeJS.ErrnoException).code === "EADDRINUSE") {
            resolve();
          } else {
            reject(err);
          }
        });
        socket.listen(port, host, () => {
          socket.close();
          reject(new Error(`Port ${host}:${port} is still free`));
        });
      });
      return;
    } catch (err) {
      lastError = err as Error;
      await new Promise((r) => setTimeout(r, 250));
    }
  }

  throw new Error(
    `Timed out waiting for ${host}:${port} to be listening: ${lastError?.message ?? "unknown error"}`,
  );
}

/**
 * Kill a child process and wait for it to exit.
 */
async function killProcess(child: ChildProcess | undefined): Promise<void> {
  if (!child || child.exitCode !== null) return;

  return new Promise((resolve) => {
    const timer = setTimeout(() => {
      child.kill("SIGKILL");
    }, 5_000);

    child.once("exit", () => {
      clearTimeout(timer);
      resolve();
    });

    child.kill("SIGTERM");
  });
}

/**
 * Build and preview a console app, returning the preview URL.
 */
async function buildAndPreviewConsole(
  repoRoot: string,
  appName: "@araf/tenant-console" | "@araf/operator-console",
  appDir: string,
  env: Record<string, string>,
): Promise<{ process: ChildProcess; url: string }> {
  const build = spawn("pnpm", ["-F", appName, "build"], {
    cwd: repoRoot,
    env: { ...process.env, ...env },
    stdio: ["ignore", "pipe", "pipe"],
  });

  await new Promise<void>((resolve, reject) => {
    build.once("exit", (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`${appName} build failed with exit code ${code ?? "unknown"}`));
      }
    });
    build.once("error", reject);
  });

  const previewPort = await getFreePort();
  const previewUrl = `http://127.0.0.1:${previewPort}`;

  const previewProcess = spawn(
    "pnpm",
    ["exec", "vite", "preview", "--port", String(previewPort), "--host", "127.0.0.1"],
    {
      cwd: join(repoRoot, "apps", appDir),
      env: { ...process.env, ...env },
      stdio: ["ignore", "pipe", "pipe"],
    },
  );

  await waitForPort("127.0.0.1", previewPort);

  return { process: previewProcess, url: previewUrl };
}

test.setTimeout(240_000);

test.describe("operations UX prototype gate", () => {
  let tenantBffProcess: ChildProcess | undefined;
  let operatorBffProcess: ChildProcess | undefined;
  let tenantPreview: { process: ChildProcess; url: string } | undefined;
  let operatorPreview: { process: ChildProcess; url: string } | undefined;

  test.beforeAll(async () => {
    const repoRoot = process.cwd();

    // Discover free ports and start both BFFs in parallel.
    const [tenantBffPort, operatorBffPort] = await Promise.all([getFreePort(), getFreePort()]);

    tenantBffProcess = spawn(join(repoRoot, "backend", "target", "debug", "tenant-bff"), {
      env: {
        ...process.env,
        ARAF_TENANT_BFF_PORT: String(tenantBffPort),
        RUST_LOG: process.env.RUST_LOG ?? "info",
      },
      stdio: ["ignore", "pipe", "pipe"],
    });

    operatorBffProcess = spawn(join(repoRoot, "backend", "target", "debug", "operator-bff"), {
      env: {
        ...process.env,
        ARAF_OPERATOR_BFF_PORT: String(operatorBffPort),
        RUST_LOG: process.env.RUST_LOG ?? "info",
      },
      stdio: ["ignore", "pipe", "pipe"],
    });

    await Promise.all([
      waitForLog(tenantBffProcess, /tenant-bff listening/),
      waitForLog(operatorBffProcess, /operator-bff listening/),
    ]);

    // Build and preview both consoles in parallel.
    [tenantPreview, operatorPreview] = await Promise.all([
      buildAndPreviewConsole(repoRoot, "@araf/tenant-console", "tenant-console", {
        VITE_TENANT_BFF_URL: `http://127.0.0.1:${tenantBffPort}`,
      }),
      buildAndPreviewConsole(repoRoot, "@araf/operator-console", "operator-console", {
        VITE_OPERATOR_BFF_URL: `http://127.0.0.1:${operatorBffPort}`,
      }),
    ]);
  }, 180_000);

  test.afterAll(async () => {
    await killProcess(tenantPreview?.process);
    await killProcess(operatorPreview?.process);
    await killProcess(tenantBffProcess);
    await killProcess(operatorBffProcess);
  });

  test("tenant journey: create server, inspect operation, and watch terminal transition", async ({
    page,
  }) => {
    expect(tenantPreview).toBeDefined();
    const previewUrl = tenantPreview?.url ?? "";

    await page.goto(`${previewUrl}/resources/compute.server`);

    // Create a server.
    await page.getByRole("link", { name: /Create/i }).click();
    await expect(page).toHaveURL(/\/resources\/compute\.server\/create$/);

    await page.getByLabel("name").fill("e2e-operations-server");
    await page.getByLabel("regionId").selectOption("eu-west");
    await page.getByLabel("projectId").selectOption("project-1");

    await page.getByRole("button", { name: "Create" }).click();

    // Success screen is truthful: it does not claim the resource is created.
    await expect(page.getByText(/Request accepted/i)).toBeVisible();
    await expect(page.getByText(/Operation .+ is pending/)).toBeVisible();

    // Extract the operation id from the page.
    const operationText = await page.locator("text=/Operation op-\\S+ is pending/").textContent();
    expect(operationText).toBeTruthy();
    const operationId = operationText?.match(/Operation (op-[\w-]+) is pending/)?.[1];
    expect(operationId).toBeDefined();

    // Navigate to the operation detail page and wait for a terminal state.
    await page.getByRole("link", { name: /View operation/i }).click();
    await expect(page).toHaveURL(new RegExp(`/operations/${operationId ?? ""}$`));

    await expect(page.getByText(/Operation op-\S+/)).toBeVisible();

    // Poll for terminal state (fixture transitions within a few seconds).
    await expect
      .poll(
        async () => {
          const stateText = await page
            .locator("text=/Succeeded|Failed/")
            .first()
            .textContent()
            .catch(() => null);
          return stateText;
        },
        { timeout: 15_000, intervals: [500, 1_000, 1_000] },
      )
      .toMatch(/Succeeded|Failed/);

    // Timeline is rendered from authoritative events.
    await expect(page.getByRole("list", { name: "Operation timeline" })).toBeVisible();
  });

  test("reload survival: operation detail survives a page reload", async ({ page }) => {
    expect(tenantPreview).toBeDefined();
    const previewUrl = tenantPreview?.url ?? "";

    await page.goto(`${previewUrl}/resources/compute.server/create`);

    await page.getByLabel("name").fill("e2e-reload-server");
    await page.getByLabel("regionId").selectOption("eu-west");
    await page.getByLabel("projectId").selectOption("project-1");

    await page.getByRole("button", { name: "Create" }).click();
    await expect(page.getByText(/Request accepted/i)).toBeVisible();

    const operationLink = page.getByRole("link", { name: /View operation/i });
    const href = await operationLink.getAttribute("href");
    expect(href).toBeTruthy();

    await operationLink.click();
    await expect(page).toHaveURL(new RegExp(`${href ?? ""}$`));

    // Reload and verify the operation is still present.
    await page.reload();
    await expect(page.getByText(/Operation op-\S+/)).toBeVisible();
  });

  test("operator journey: open platform, view operations list, and open a failed operation", async ({
    page,
  }) => {
    expect(operatorPreview).toBeDefined();
    const previewUrl = operatorPreview?.url ?? "";

    await page.goto(`${previewUrl}/platform/overview`);
    await expect(page.getByRole("heading", { name: /Platform overview/i })).toBeVisible();

    await page.goto(`${previewUrl}/operations`);

    // Operations list renders with server-bounded table and pagination.
    await expect(page.getByRole("heading", { name: "Operations" })).toBeVisible();
    await expect(page.getByRole("table")).toBeVisible();
    await expect(page.getByRole("navigation", { name: "Pagination" })).toBeVisible();

    // Filter to failed state to find a deterministic failed operation in the fixture universe.
    await page.getByLabel("State").selectOption("failed");
    const tableBody = page.locator("table tbody");
    await expect
      .poll(() => tableBody.getByRole("link").count(), { timeout: 10_000 })
      .toBeGreaterThan(0);

    const firstFailedLink = tableBody.getByRole("link").first();
    await expect(firstFailedLink).toBeVisible();
    const href = await firstFailedLink.getAttribute("href");
    expect(href).toMatch(/^\/operations\/op-/);

    await firstFailedLink.click();
    await expect(page).toHaveURL(/\/operations\/op-\S+$/);

    // Failed operation detail shows structured error and timeline section.
    await expect(page.getByText("Failed").first()).toBeVisible();
    await expect(page.getByRole("heading", { name: "Error" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Timeline" })).toBeVisible();
  });
});
