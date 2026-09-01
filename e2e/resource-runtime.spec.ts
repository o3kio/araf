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

test.setTimeout(180_000);

test.describe("generic resource runtime", () => {
  let bffProcess: ChildProcess | undefined;
  let previewProcess: ChildProcess | undefined;
  let bffPort: number | undefined;
  let previewPort: number | undefined;
  let previewUrl: string | undefined;

  test.beforeAll(async () => {
    const repoRoot = process.cwd();

    bffPort = await getFreePort();
    bffProcess = spawn(join(repoRoot, "backend", "target", "debug", "tenant-bff"), {
      env: {
        ...process.env,
        ARAF_TENANT_BFF_PORT: String(bffPort),
        RUST_LOG: process.env.RUST_LOG ?? "info",
      },
      stdio: ["ignore", "pipe", "pipe"],
    });

    await waitForLog(bffProcess, /tenant-bff listening/);

    previewPort = await getFreePort();
    previewUrl = `http://127.0.0.1:${previewPort}`;

    const build = spawn("pnpm", ["-F", "@araf/tenant-console", "build"], {
      cwd: repoRoot,
      env: {
        ...process.env,
        VITE_TENANT_BFF_URL: `http://127.0.0.1:${bffPort}`,
      },
      stdio: ["ignore", "pipe", "pipe"],
    });

    await new Promise<void>((resolve, reject) => {
      build.once("exit", (code) => {
        if (code === 0) {
          resolve();
        } else {
          reject(new Error(`Tenant console build failed with exit code ${code ?? "unknown"}`));
        }
      });
      build.once("error", reject);
    });

    previewProcess = spawn(
      "pnpm",
      ["exec", "vite", "preview", "--port", String(previewPort), "--host", "127.0.0.1"],
      {
        cwd: join(repoRoot, "apps", "tenant-console"),
        env: {
          ...process.env,
          VITE_TENANT_BFF_URL: `http://127.0.0.1:${bffPort}`,
        },
        stdio: ["ignore", "pipe", "pipe"],
      },
    );

    await waitForPort("127.0.0.1", previewPort);
  });

  test.afterAll(async () => {
    await killProcess(previewProcess);
    await killProcess(bffProcess);
  });

  test("renders resource collection, detail, and relationships", async ({ page }) => {
    expect(previewUrl).toBeDefined();

    await page.goto(`${previewUrl ?? ""}/resources/compute.server`);

    // Collection heading and table rows are rendered.
    await expect(page.getByRole("heading", { name: /Servers|Compute/i })).toBeVisible();
    await expect(page.getByRole("table")).toBeVisible();
    await expect(page.getByRole("link", { name: "fixture-server-0" })).toBeVisible();

    // Pagination controls are visible.
    await expect(page.getByRole("navigation", { name: "Pagination" })).toBeVisible();
    await expect(page.getByText(/Page \d+ of \d+/)).toBeVisible();

    // Navigate to a server detail page and verify tabs.
    await page.getByRole("link", { name: "fixture-server-0" }).click();
    await expect(page).toHaveURL(/\/resources\/compute\.server\/resource-0000000000$/);
    await expect(page.getByRole("heading", { name: "fixture-server-0" })).toBeVisible();
    await expect(page.getByRole("tab", { name: "Overview" })).toBeVisible();
    await expect(page.getByRole("tab", { name: "Operations" })).toBeVisible();

    // Navigate to the volume collection and open a volume detail page.
    await page.goto(`${previewUrl ?? ""}/resources/storage.volume`);
    await expect(page.getByRole("heading", { name: /Volumes|Storage/i })).toBeVisible();
    await page.getByRole("link", { name: "fixture-volume-0" }).click();
    await expect(page).toHaveURL(/\/resources\/storage\.volume\/volume-00000000$/);
    await expect(page.getByRole("heading", { name: "fixture-volume-0" })).toBeVisible();

    // Relationship tab and link to the attached compute server are rendered.
    await expect(page.getByRole("tab", { name: "Relationships" })).toBeVisible();
    await page.getByRole("tab", { name: "Relationships" }).click();
    const serverLink = page.getByRole("link", { name: /fixture-server-0|resource-0000000000/ });
    await expect(serverLink).toBeVisible();
    await expect(serverLink).toHaveAttribute(
      "href",
      "/resources/compute.server/resource-0000000000",
    );
  });

  test("schema-driven create flow returns a Pending Operation", async ({ page }) => {
    expect(previewUrl).toBeDefined();

    await page.goto(`${previewUrl ?? ""}/resources/compute.server`);

    // Create button is visible and navigates to the create form.
    const createButton = page.getByRole("link", { name: /Create/i });
    await expect(createButton).toBeVisible();
    await createButton.click();
    await expect(page).toHaveURL(/\/resources\/compute\.server\/create$/);
    await expect(page.getByRole("heading", { name: "Create Server" })).toBeVisible();

    // Fill the schema-generated form.
    await page.getByLabel("name").fill("e2e-created-server");
    await page.getByLabel("regionId").selectOption("eu-west");
    await page.getByLabel("projectId").selectOption("project-1");

    // Submit and verify the Operation result shows Pending state.
    await page.getByRole("button", { name: "Create" }).click();
    await expect(page.getByText(/Operation .+ is pending/)).toBeVisible();
    await expect(page.getByText(/Correlation ID:/)).toBeVisible();
  });

  test("schema-driven action with input returns a Pending Operation", async ({ page }) => {
    expect(previewUrl).toBeDefined();

    await page.goto(`${previewUrl ?? ""}/resources/storage.volume/volume-00000000`);
    await expect(page.getByRole("heading", { name: "fixture-volume-0" })).toBeVisible();

    // Attach action has an input schema and opens a modal.
    await page.getByRole("button", { name: "Attach" }).click();
    await expect(page.getByRole("dialog")).toBeVisible();
    await expect(page.getByText(/Confirm attach for fixture-volume-0/)).toBeVisible();

    await page.getByLabel("serverId").fill("resource-0000000000");
    await page.getByRole("button", { name: "Confirm" }).click();

    await expect(page.getByText(/Operation .+ is pending/)).toBeVisible();
  });
});
