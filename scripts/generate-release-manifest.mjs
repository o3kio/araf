import fs from "node:fs";
import process from "node:process";
import crypto from "node:crypto";

const version = process.env.ARAF_VERSION;
const sourceSha = process.env.ARAF_GIT_SHA;
const digest = (name) => {
  const value = process.env[`ARAF_${name}_DIGEST`];
  if (!/^sha256:[a-f0-9]{64}$/.test(value ?? "")) throw new Error(`missing digest for ${name}`);
  return value;
};
if (!/^v\d+\.\d+\.\d+-rc\.\d+$/.test(version ?? "") || !/^[a-f0-9]{40}$/.test(sourceSha ?? "")) {
  throw new Error("ARAF_VERSION must be vX.Y.Z-rc.N and ARAF_GIT_SHA must be a full SHA");
}
const bff = digest("BFF");
const tenant = digest("TENANT_CONSOLE");
const operator = digest("OPERATOR_CONSOLE");
const artifacts = {
  tenant_console: { image: "ghcr.io/o3kio/araf-tenant-console", digest: tenant },
  operator_console: { image: "ghcr.io/o3kio/araf-operator-console", digest: operator },
  tenant_bff: { image: "ghcr.io/o3kio/araf-bff", digest: bff },
  operator_bff: { image: "ghcr.io/o3kio/araf-bff", digest: bff },
};
for (const artifact of Object.values(artifacts)) {
  Object.assign(artifact, { source_sha: sourceSha, version, platform: "linux/amd64" });
  artifact.sbom = `oci://${artifact.image}@${artifact.digest}`;
  artifact.provenance = `oci://${artifact.image}@${artifact.digest}`;
}
const manifest = {
  schema_version: 1,
  release: { version, source_sha: sourceSha, tag: version, prerelease: true },
  artifacts,
  deployment: {
    compose: "deploy/docker-compose.release.yml",
    helm: "deploy/helm/araf",
    runtime: "Docker Engine >=29.8.0 / Compose >=5.5.1",
    source_build_required: false,
  },
  contracts: {
    tenant_api: "araf-bff/v1",
    operator_api: "araf-bff/v1",
    backend_adapter: "o3k.io/v1 or OpenStack 2026.1-core",
    auth_session: "araf-oidc-session/v1",
    deployment: "araf-release-compose/v1",
  },
  security: { fixture_mode_allowed: false, provenance_required: true, sbom_required: true },
};
const output = JSON.stringify(manifest, null, 2) + "\n";
fs.writeFileSync(process.env.ARAF_MANIFEST_PATH ?? "release/manifest.json", output);
fs.writeFileSync(
  process.env.ARAF_MANIFEST_SHA_PATH ?? "release/manifest.sha256",
  `${crypto.createHash("sha256").update(output).digest("hex")}  manifest.json\n`,
);
