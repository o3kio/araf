import { readFile, writeFile } from "node:fs/promises";

const version = process.env.RELEASE_VERSION;
const sourceSha = process.env.SOURCE_SHA;
const outputPath = process.env.ARAF_MANIFEST_PATH ?? "dist/release/release-manifest.json";
const digestsPath = process.env.ARAF_DIGESTS_PATH ?? `dist/release/araf-${version}-digests.txt`;

if (!version || !/^v\d+\.\d+\.\d+-[A-Za-z0-9.-]+$/.test(version)) {
  throw new Error("RELEASE_VERSION must be a versioned release tag");
}
if (!sourceSha || !/^[a-f0-9]{40}$/.test(sourceSha)) {
  throw new Error("SOURCE_SHA must be the 40-character source commit SHA");
}

const digestLines = (await readFile(digestsPath, "utf8"))
  .trim()
  .split(/\r?\n/u)
  .map((line) => line.split(/\s+/u));
const indexByComponent = new Map(
  digestLines
    .filter(([component, ref]) => component && ref?.includes("@sha256:"))
    .map(([component, ref]) => [component, ref]),
);

function artifact(component) {
  const ref = indexByComponent.get(component);
  if (!ref) throw new Error(`missing digest for ${component}`);
  const separator = ref.indexOf("@");
  const image = ref.slice(0, separator);
  const digest = ref.slice(separator + 1);
  return {
    image: `${image}@${digest}`,
    tag: version,
    digest,
    source_sha: sourceSha,
    platform: "linux/amd64",
  };
}

const manifest = {
  schema_version: 1,
  release: { version, source_sha: sourceSha },
  artifacts: {
    tenant_console: artifact("tenant-console"),
    operator_console: artifact("operator-console"),
    // Both BFF surfaces are separate runtime components of the one canonical
    // multi-binary araf-bff image and intentionally share its digest.
    tenant_bff: artifact("bff"),
    operator_bff: artifact("bff"),
  },
  deployment: {
    compose: "deploy/docker-compose.release.yml",
    helm: "deploy/helm/araf",
  },
  compatibility: {
    supported_backend_modes: ["o3k", "openstack"],
    required_o3k_api_contract: "o3k-native-v1",
    tenant_bff_contract: "araf-tenant-bff-v1",
    operator_bff_contract: "araf-operator-bff-v1",
    auth_session_contract: "araf-session-csrf-v1",
    deployment_contract: "compose-digest-v1",
  },
  security: { fixture_mode_allowed: false },
};

await writeFile(outputPath, `${JSON.stringify(manifest, null, 2)}\n`);
