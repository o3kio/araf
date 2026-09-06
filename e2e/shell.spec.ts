import { test, expect } from "@playwright/test";
import { spawn, type ChildProcess } from "node:child_process";
import { createServer, type AddressInfo } from "node:net";
import { join } from "node:path";

async function freePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const port = (server.address() as AddressInfo).port;
      server.close(() => resolve(port));
    });
  });
}

async function waitForLog(child: ChildProcess, pattern: RegExp): Promise<void> {
  await new Promise<void>((resolve, reject) => {
    const timer = setTimeout(() => {
      cleanup();
      reject(new Error(`Timed out waiting for ${pattern.source}`));
    }, 30_000);
    const onData = (data: Buffer) => {
      if (pattern.test(data.toString())) {
        clearTimeout(timer);
        cleanup();
        resolve();
      }
    };
    const cleanup = () => {
      child.stdout?.off("data", onData);
      child.stderr?.off("data", onData);
    };
    child.stdout?.on("data", onData);
    child.stderr?.on("data", onData);
    child.once("exit", (code) => {
      if (code !== null) reject(new Error(`Process exited with code ${code}`));
    });
  });
}

async function waitForPort(port: number): Promise<void> {
  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    try {
      await new Promise<void>((resolve, reject) => {
        const server = createServer();
        server.once("error", (error) => {
          server.close();
          if ((error as NodeJS.ErrnoException).code === "EADDRINUSE") resolve();
          else reject(error);
        });
        server.listen(port, "127.0.0.1", () => {
          server.close();
          reject(new Error("port is still free"));
        });
      });
      return;
    } catch {
      await new Promise((resolve) => setTimeout(resolve, 250));
    }
  }
  throw new Error(`Timed out waiting for port ${port}`);
}

async function stop(child: ChildProcess | undefined): Promise<void> {
  if (!child || child.exitCode !== null) return;
  await new Promise<void>((resolve) => {
    const timer = setTimeout(() => child.kill("SIGKILL"), 5_000);
    child.once("exit", () => {
      clearTimeout(timer);
      resolve();
    });
    child.kill("SIGTERM");
  });
}

async function buildPreview(
  root: string,
  packageName: "@araf/tenant-console" | "@araf/operator-console",
  directory: "tenant-console" | "operator-console",
  bffPort: number,
): Promise<{ process: ChildProcess; url: string }> {
  const variable =
    packageName === "@araf/tenant-console" ? "VITE_TENANT_BFF_URL" : "VITE_OPERATOR_BFF_URL";
  const env = { ...process.env, [variable]: `http://127.0.0.1:${bffPort}` };
  const build = spawn("pnpm", ["-F", packageName, "build"], { cwd: root, env, stdio: "ignore" });
  await new Promise<void>((resolve, reject) => {
    build.once("exit", (code) =>
      code === 0 ? resolve() : reject(new Error(`${packageName} build failed`)),
    );
    build.once("error", reject);
  });
  const previewPort = await freePort();
  const preview = spawn(
    "pnpm",
    ["exec", "vite", "preview", "--port", String(previewPort), "--host", "127.0.0.1"],
    { cwd: join(root, "apps", directory), env, stdio: "ignore" },
  );
  await waitForPort(previewPort);
  return { process: preview, url: `http://127.0.0.1:${previewPort}` };
}

test.setTimeout(180_000);

test.describe("console shell integration", () => {
  let tenantBff: ChildProcess | undefined;
  let operatorBff: ChildProcess | undefined;
  let tenantPreview: { process: ChildProcess; url: string } | undefined;
  let operatorPreview: { process: ChildProcess; url: string } | undefined;

  test.beforeAll(async () => {
    const root = process.cwd();
    const [tenantPort, operatorPort] = await Promise.all([freePort(), freePort()]);
    tenantBff = spawn(join(root, "backend", "target", "debug", "tenant-bff"), {
      env: { ...process.env, ARAF_TENANT_BFF_PORT: String(tenantPort) },
      stdio: ["ignore", "pipe", "pipe"],
    });
    operatorBff = spawn(join(root, "backend", "target", "debug", "operator-bff"), {
      env: { ...process.env, ARAF_OPERATOR_BFF_PORT: String(operatorPort) },
      stdio: ["ignore", "pipe", "pipe"],
    });
    await Promise.all([
      waitForLog(tenantBff, /tenant-bff listening/),
      waitForLog(operatorBff, /operator-bff listening/),
    ]);
    [tenantPreview, operatorPreview] = await Promise.all([
      buildPreview(root, "@araf/tenant-console", "tenant-console", tenantPort),
      buildPreview(root, "@araf/operator-console", "operator-console", operatorPort),
    ]);
  });

  test.afterAll(async () => {
    await Promise.all([
      stop(tenantPreview?.process),
      stop(operatorPreview?.process),
      stop(tenantBff),
      stop(operatorBff),
    ]);
  });

  test("tenant shell exposes scope, active navigation, styled controls, and keyboard focus", async ({
    page,
  }) => {
    await page.goto(`${tenantPreview?.url}/resources/compute.server`);
    const navigation = page.getByRole("navigation", { name: "Tenant navigation" });
    await expect(navigation).toBeVisible();
    await expect(page.getByLabel("Project")).toBeVisible();
    await expect(page.getByLabel("Region")).toBeVisible();
    await expect(navigation.getByRole("link", { name: "Servers" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    await expect(page.getByPlaceholder("Search resources")).toBeVisible();

    const activeLink = navigation.getByRole("link", { name: "Servers" });
    await expect(activeLink).toHaveCSS("min-height", "36px");
    await expect(activeLink).toHaveCSS("font-weight", "700");
    await page.keyboard.press("Tab");
    await expect(page.locator(":focus")).toHaveAttribute("aria-label", "Search resources");
  });

  test("operator shell keeps its platform context and responsive layout", async ({ page }) => {
    await page.goto(`${operatorPreview?.url}/platform/overview`);
    await expect(page.getByRole("navigation", { name: "Operator navigation" })).toBeVisible();
    await expect(page.getByTestId("operator-context")).toHaveText("Platform context");
    await expect(page.getByRole("link", { name: "Overview" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    await expect(page.getByRole("link", { name: "Overview" })).toHaveCSS("min-height", "36px");

    await page.setViewportSize({ width: 520, height: 800 });
    await expect(page.getByRole("navigation", { name: "Operator navigation" })).toBeVisible();
    await expect(page.getByTestId("operator-context")).toBeVisible();
  });
});
