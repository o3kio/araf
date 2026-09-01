import { spawn, type ChildProcess } from "node:child_process";
import { createServer } from "node:net";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it, beforeAll, afterAll } from "vitest";
import { createArafClient, ArafApiError } from "./index.js";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const bffBinary = resolve(repoRoot, "backend/target/debug/tenant-bff");

function getFreePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      const port = typeof address === "object" && address !== null ? address.port : 0;
      server.close((error) => {
        if (error) {
          reject(error);
        } else {
          resolve(port);
        }
      });
    });
    server.on("error", reject);
  });
}

function startTenantBff(port: number): Promise<ChildProcess> {
  return new Promise((resolve, reject) => {
    const child = spawn(bffBinary, [], {
      cwd: repoRoot,
      env: {
        ...process.env,
        ARAF_TENANT_BFF_PORT: String(port),
        RUST_LOG: "info",
      },
      stdio: ["ignore", "pipe", "pipe"],
    });

    let resolved = false;
    const buffer: string[] = [];

    function handleData(data: Buffer): void {
      const text = data.toString();
      buffer.push(text);
      if (!resolved && text.includes("tenant-bff listening")) {
        resolved = true;
        resolve(child);
      }
    }

    child.stdout.on("data", handleData);
    child.stderr.on("data", handleData);

    child.on("error", (error) => {
      if (!resolved) {
        reject(error);
      }
    });

    child.on("exit", (code) => {
      if (!resolved) {
        reject(
          new Error(
            `tenant-bff exited before listening (code ${String(code)}). Output:\n${buffer.join("")}`,
          ),
        );
      }
    });

    setTimeout(() => {
      if (!resolved) {
        child.kill("SIGTERM");
        reject(new Error(`timed out waiting for tenant-bff to start. Output:\n${buffer.join("")}`));
      }
    }, 10_000);
  });
}

describe("tenant-bff fixture contract", () => {
  let port: number;
  let bff: ChildProcess | undefined;

  beforeAll(async () => {
    port = await getFreePort();
    bff = await startTenantBff(port);
  }, 15_000);

  afterAll(() => {
    if (bff && !bff.killed) {
      bff.kill("SIGTERM");
      setTimeout(() => {
        if (bff && !bff.killed) {
          bff.kill("SIGKILL");
        }
      }, 2_000);
    }
  });

  it("returns a healthy healthz response", async () => {
    const client = createArafClient(`http://127.0.0.1:${String(port)}`);
    const health = await client.healthz();

    expect(health.status).toBe("ok");
    expect(health.service).toBe("tenant-bff");
  });

  it("agrees with the fixture on the resource collection total", async () => {
    const client = createArafClient(`http://127.0.0.1:${String(port)}`);
    const collection = await client.listResources("compute.server", {
      page: 0,
      pageSize: 5,
    });

    expect(collection.total).toBe(100_000);
    expect(collection.items).toHaveLength(5);
    expect(collection.hasMore).toBe(true);
    expect(collection.items[0]?.resourceType).toBe("compute.server");
  });

  it("accepts a resource action and returns a Pending Operation", async () => {
    const client = createArafClient(`http://127.0.0.1:${String(port)}`);
    const operation = await client.submitAction("compute.server", "resource-0000000001", {
      actionId: "start",
    });

    expect(operation.action).toBe("start");
    expect(operation.state).toBe("pending");
    expect(operation.resourceId).toBe("resource-0000000001");
    expect(operation.resourceType).toBe("compute.server");
  });

  it("surfaces ProblemDetails for an unsupported resource type", async () => {
    const client = createArafClient(`http://127.0.0.1:${String(port)}`);

    await expect(
      client.listResources("unsupported.type", { page: 0, pageSize: 1 }),
    ).rejects.toBeInstanceOf(ArafApiError);

    try {
      await client.listResources("unsupported.type", { page: 0, pageSize: 1 });
      expect.fail("expected an error");
    } catch (error) {
      const apiError = error as ArafApiError;
      expect(apiError.status).toBe(404);
      expect(apiError.problem.type).toBe("https://araf.o3k.io/errors/404");
      expect(apiError.problem.status).toBe(404);
    }
  });
});
