import fs from "node:fs";
import process from "node:process";
const [manifestPath = "release/manifest.json"] = process.argv.slice(2);
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
const sourceSha = /^[a-f0-9]{40}$/;
const imageDigest = /^sha256:[a-f0-9]{64}$/;
if (
  manifest.schema_version !== 1 ||
  manifest.release?.prerelease !== true ||
  manifest.security?.fixture_mode_allowed !== false
)
  throw new Error("manifest schema or security flags invalid");
if (
  !/^v\d+\.\d+\.\d+-rc\.\d+$/.test(manifest.release.version) ||
  manifest.release.version !== manifest.release.tag ||
  !sourceSha.test(manifest.release.source_sha)
)
  throw new Error("release identity invalid");
if (manifest.deployment.source_build_required !== false)
  throw new Error("release deployment must not require a source build");
for (const [name, artifact] of Object.entries(manifest.artifacts)) {
  if (artifact.source_sha !== manifest.release.source_sha)
    throw new Error(`${name}: source SHA mismatch`);
  if (artifact.version !== manifest.release.version) throw new Error(`${name}: version mismatch`);
  if (!imageDigest.test(artifact.digest) || !/^ghcr\.io\/o3kio\/araf-[a-z-]+$/.test(artifact.image))
    throw new Error(`${name}: image identity invalid`);
  if (
    artifact.sbom !== `oci://${artifact.image}@${artifact.digest}` ||
    artifact.provenance !== artifact.sbom
  )
    throw new Error(`${name}: SBOM/provenance subject mismatch`);
}
for (const name of ["tenant_console", "operator_console", "tenant_bff", "operator_bff"])
  if (!Object.hasOwn(manifest.artifacts, name)) throw new Error(`missing artifact: ${name}`);
console.log(`release manifest valid: ${manifest.release.version} ${manifest.release.source_sha}`);
