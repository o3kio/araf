import { test, expect } from "@playwright/test";
import { spawn, type ChildProcess } from "node:child_process";
import { createServer, type AddressInfo } from "node:net";
import { join } from "node:path";

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

test.describe("operator platform", () => {
  let bffProcess: ChildProcess | undefined;
  let previewProcess: ChildProcess | undefined;
  let bffPort: number | undefined;
  let previewPort: number | undefined;
  let previewUrl: string | undefined;

  test.beforeAll(async () => {
    const repoRoot = process.cwd();

    bffPort = await getFreePort();
    bffProcess = spawn(join(repoRoot, "backend", "target", "debug", "operator-bff"), {
      env: {
        ...process.env,
        ARAF_OPERATOR_BFF_PORT: String(bffPort),
        RUST_LOG: process.env.RUST_LOG ?? "info",
      },
      stdio: ["ignore", "pipe", "pipe"],
    });

    await waitForLog(bffProcess, /operator-bff listening/);

    previewPort = await getFreePort();
    previewUrl = `http://127.0.0.1:${previewPort}`;

    const build = spawn("pnpm", ["-F", "@araf/operator-console", "build"], {
      cwd: repoRoot,
      env: {
        ...process.env,
        VITE_OPERATOR_BFF_URL: `http://127.0.0.1:${bffPort}`,
      },
      stdio: ["ignore", "pipe", "pipe"],
    });

    await new Promise<void>((resolve, reject) => {
      build.once("exit", (code) => {
        if (code === 0) {
          resolve();
        } else {
          reject(new Error(`Operator console build failed with exit code ${code ?? "unknown"}`));
        }
      });
      build.once("error", reject);
    });

    previewProcess = spawn(
      "pnpm",
      ["exec", "vite", "preview", "--port", String(previewPort), "--host", "127.0.0.1"],
      {
        cwd: join(repoRoot, "apps", "operator-console"),
        env: {
          ...process.env,
          VITE_OPERATOR_BFF_URL: `http://127.0.0.1:${bffPort}`,
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

  test("renders operator platform overview and navigation", async ({ page }) => {
    expect(previewUrl).toBeDefined();

    await page.goto(`${previewUrl ?? ""}/platform/overview`);
    await expect(page.getByRole("heading", { name: "Platform overview" })).toBeVisible();
    await expect(page.getByText("Active operations:")).toBeVisible();

    await page.goto(`${previewUrl ?? ""}/platform/regions`);
    await expect(page.getByRole("heading", { name: "Regions" })).toBeVisible();
    await expect(page.getByRole("table")).toBeVisible();

    await page.goto(`${previewUrl ?? ""}/platform/health`);
    await expect(page.getByRole("heading", { name: "Provider health" })).toBeVisible();

    await page.goto(`${previewUrl ?? ""}/platform/capacity`);
    await expect(page.getByRole("heading", { name: "Capacity" })).toBeVisible();
  });

  test("renders operator accounts and audit pages", async ({ page }) => {
    expect(previewUrl).toBeDefined();

    await page.goto(`${previewUrl ?? ""}/customers/accounts`);
    await expect(page.getByRole("heading", { name: "Accounts" })).toBeVisible();
    await expect(page.getByRole("table")).toBeVisible();

    await page.goto(`${previewUrl ?? ""}/governance/audit`);
    await expect(page.getByRole("heading", { name: "Audit" })).toBeVisible();
  });

  test("operator routes render on the operator console", async ({ page }) => {
    expect(previewUrl).toBeDefined();

    await page.goto(`${previewUrl ?? ""}/platform/overview`);
    await expect(page.getByRole("heading", { name: "Platform overview" })).toBeVisible();
    await expect(page.getByText("Operator routes are not available")).not.toBeVisible();
  });
});
