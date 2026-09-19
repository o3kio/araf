#!/usr/bin/env bash
set -euo pipefail

root_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
python3 - "$root_dir" <<'PY'
import pathlib, re, sys

root = pathlib.Path(sys.argv[1])
required = [
    "docs/operator/README.md", "docs/operator/installation.md",
    "docs/operator/configuration.md", "docs/operator/secrets.md",
    "docs/operator/security.md", "docs/operator/observability.md",
    "docs/operator/troubleshooting.md", "docs/operator/upgrade-rollback.md",
    "docs/operator/recovery.md", "docs/operator/support-bundle.md",
    "docs/operator/limitations.md", "docs/operator/cold-operator-validation.md",
    "tests/support-bundle.sh",
]
missing = [p for p in required if not (root / p).exists()]
if missing:
    raise SystemExit("missing required operator artifacts: " + ", ".join(missing))

link_re = re.compile(r"!?(?:\[[^\]]*\])\(([^)]+)\)")
errors = []
for doc in (root / "docs").rglob("*.md"):
    for raw in link_re.findall(doc.read_text(encoding="utf-8")):
        target = raw.split("#", 1)[0].strip().strip("<>")
        if not target or "://" in target or target.startswith("mailto:"):
            continue
        candidate = (doc.parent / target).resolve()
        if not candidate.exists():
            errors.append(f"{doc.relative_to(root)} -> {raw}")
if errors:
    raise SystemExit("broken internal documentation links:\n" + "\n".join(errors))

text = "\n".join(p.read_text(encoding="utf-8") for p in (root / "docs/operator").glob("*.md"))
for phrase in ("#106 remains OPEN", "CompatibilityOperation", "canonical O3K Operation", "Terraform", "OpenTofu"):
    if phrase not in text:
        raise SystemExit(f"operator documentation is missing required phrase: {phrase}")

installation = (root / "docs/operator/installation.md").read_text(encoding="utf-8")
observability = (root / "docs/operator/observability.md").read_text(encoding="utf-8")
support_bundle = (root / "tests/support-bundle.sh").read_text(encoding="utf-8")
for surface in ("tenant", "operator"):
    for probe in ("healthz", "readyz", "version"):
        if f"https://{surface}.example/{probe}" in installation:
            raise SystemExit(f"browser-origin probe must not be documented: {surface}/{probe}")
if "kubectl -n araf port-forward svc/<release>-tenant-bff 18080:80" not in installation:
    raise SystemExit("installation docs must show the private Tenant BFF probe path")
if "kubectl -n araf port-forward svc/<release>-tenant-bff 18080:80" not in observability:
    raise SystemExit("observability docs must show the private BFF metrics path")
if "https://tenant.example/metrics" in observability:
    raise SystemExit("browser-origin metrics scrape must not be documented")
if "ARAF_SUPPORT_BUNDLE_LOG_FILES" in support_bundle:
    raise SystemExit("support bundle must not ingest arbitrary operator-selected log files")
print("docs link/path gate: PASS")
PY

# Commands and support tooling named by the operator entry point must exist.
for script in tests/support-bundle.sh tests/support-bundle-security.sh tests/package-upgrade-rollback.sh tests/prometheus-observability.sh tests/release-publish.sh; do
  [[ -x "$root_dir/$script" ]] || { echo "missing executable: $script" >&2; exit 1; }
done
echo 'docs command gate: PASS'
