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

test.describe("service catalog and installed services UX", () => {
  let tenantBffProcess: ChildProcess | undefined;
  let operatorBffProcess: ChildProcess | undefined;
  let tenantPreview: { process: ChildProcess; url: string } | undefined;
  let operatorPreview: { process: ChildProcess; url: string } | undefined;

  test.beforeAll(async () => {
    const repoRoot = process.cwd();

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

  test("tenant service catalog page lists discovered services", async ({ page }) => {
    expect(tenantPreview).toBeDefined();
    const previewUrl = tenantPreview?.url ?? "";

    await page.goto(`${previewUrl}/services/catalog`);

    await expect(page.getByRole("heading", { name: "Service catalog" })).toBeVisible();
    await expect(page.getByRole("table")).toBeVisible();

    // Fixture catalog includes Compute and Object Storage services.
    await expect(page.getByText("Compute", { exact: true })).toBeVisible();
    await expect(page.getByText("Object Storage", { exact: true })).toBeVisible();
  });

  test("tenant navigation is derived from the BFF service catalog", async ({ page }) => {
    expect(tenantPreview).toBeDefined();
    const previewUrl = tenantPreview?.url ?? "";

    await page.goto(`${previewUrl}/`);

    const navigation = page.getByRole("navigation", { name: "Tenant navigation" });
    await expect(navigation).toBeVisible();

    // Dynamic resource links from the fixture catalog.
    await expect(navigation.getByRole("link", { name: "Servers" })).toBeVisible();
    await expect(navigation.getByRole("link", { name: "Buckets" })).toBeVisible();
  });

  test("operator installed services page lists services and resource types", async ({ page }) => {
    expect(operatorPreview).toBeDefined();
    const previewUrl = operatorPreview?.url ?? "";

    await page.goto(`${previewUrl}/services/installed`);

    await expect(page.getByRole("heading", { name: "Installed services" })).toBeVisible();
    await expect(page.getByRole("table", { name: "Installed services" })).toBeVisible();

    // Fixture installed services include compute and object storage.
    await expect(
      page.getByRole("table", { name: "Installed services" }).getByText("Compute", { exact: true }),
    ).toBeVisible();
    await expect(page.getByText("object.storage.bucket")).toBeVisible();
  });
});
