import { readFile } from "node:fs/promises";

const path = process.argv[2] ?? "dist/release/release-manifest.json";
const manifest = JSON.parse(await readFile(path, "utf8"));
const fail = (message) => {
  throw new Error(`release manifest: ${message}`);
};
if (manifest.schema_version !== 1) fail("unsupported schema_version");
if (!/^v\d+\.\d+\.\d+-[A-Za-z0-9.-]+$/.test(manifest.release?.version ?? ""))
  fail("invalid version");
if (!/^[a-f0-9]{40}$/.test(manifest.release?.source_sha ?? "")) fail("invalid source_sha");
for (const component of ["tenant_console", "operator_console", "tenant_bff", "operator_bff"]) {
  const artifact = manifest.artifacts?.[component];
  if (!artifact || !/^ghcr\.io\/.+@sha256:[a-f0-9]{64}$/.test(artifact.image))
    fail(`invalid ${component} image`);
  if (artifact.tag !== manifest.release.version) fail(`${component} tag/version mismatch`);
  const imageTag = artifact.image.slice(0, artifact.image.indexOf("@")).split(":").at(-1);
  if (imageTag !== manifest.release.version) fail(`${component} image tag/version mismatch`);
  if (artifact.digest !== artifact.image.slice(artifact.image.indexOf("@") + 1))
    fail(`${component} digest mismatch`);
  if (artifact.source_sha !== manifest.release.source_sha) fail(`${component} source mismatch`);
  if (artifact.platform !== "linux/amd64") fail(`${component} platform mismatch`);
}
if (manifest.security?.fixture_mode_allowed !== false) fail("fixture mode must be disabled");
if (manifest.compatibility?.required_o3k_api_contract !== "o3k-native-iam-v1")
  fail("O3K contract mismatch");
console.log(`release manifest valid: ${path}`);
