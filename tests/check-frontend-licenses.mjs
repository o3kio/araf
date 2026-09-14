/* global console */

import { readdir, readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { URL } from "node:url";

const root = path.resolve(new URL("..", import.meta.url).pathname);
const allowlist = new Set(
  (await readFile(path.join(root, "policy/frontend-licenses.txt"), "utf8"))
    .split(/\r?\n/)
    .map((line) => line.replace(/#.*/, "").trim())
    .filter(Boolean),
);

const packageFiles = [];
const pnpmStore = path.join(root, "node_modules/.pnpm");
for (const entry of await readdir(pnpmStore, { withFileTypes: true })) {
  if (!entry.isDirectory()) continue;
  const dependencies = path.join(pnpmStore, entry.name, "node_modules");
  let children;
  try {
    children = await readdir(dependencies, { withFileTypes: true });
  } catch {
    continue;
  }
  for (const child of children) {
    const childPath = path.join(dependencies, child.name);
    if (!child.isDirectory()) continue;
    if (child.name.startsWith("@")) {
      for (const scoped of await readdir(childPath, { withFileTypes: true })) {
        if (scoped.isDirectory()) {
          packageFiles.push(path.relative(root, path.join(childPath, scoped.name, "package.json")));
        }
      }
    } else {
      packageFiles.push(path.relative(root, path.join(childPath, "package.json")));
    }
  }
}

const violations = [];
for (const relative of packageFiles) {
  const packageJson = JSON.parse(await readFile(path.join(root, relative), "utf8"));
  const license = packageJson.license;
  const identifiers =
    typeof license === "string"
      ? (license.match(/[A-Za-z0-9.-]+/g) ?? [])
      : license && typeof license === "object" && typeof license.type === "string"
        ? [license.type]
        : [];
  if (identifiers.length === 0 || identifiers.some((id) => !allowlist.has(id))) {
    violations.push(`${packageJson.name ?? relative}: ${String(license ?? "missing")}`);
  }
}

if (violations.length > 0) {
  console.error("frontend license policy violations:");
  for (const violation of violations.sort()) console.error(`- ${violation}`);
  process.exit(1);
}

console.log(`frontend license policy: PASS (${packageFiles.length} packages)`);
