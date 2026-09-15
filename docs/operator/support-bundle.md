# Redacted support bundle

Run the repository-provided collector from an operator host with access to a
BFF health endpoint:

```bash
ARAF_TENANT_BFF_URL=https://tenant-bff.internal \
  ./tests/support-bundle.sh /secure/path/araf-support-bundle
```

The command writes a mode-700 temporary directory, creates a compressed
archive, prints its path, and removes the temporary directory. It collects:

- UTC timestamp and runtime/kernel metadata;
- `/version`, `/readyz` and bounded `/metrics` output;
- environment variable names only (`NAME=<redacted>`), never values;
- a README describing the redaction boundary.

It deliberately does not collect process environments, cookies, session
files, compatibility journals, request bodies, Authorization headers, OIDC or
provider credentials, client secrets, encryption keys or private keys. Attach
logs and journal excerpts only after a human secret review; the collector does
not accept arbitrary log-file paths. Prefer correlation IDs and structured
error fields.

Before transfer:

```bash
tar -tzf /secure/path/araf-support-bundle.tar.gz
tar --wildcards -xOzf /secure/path/araf-support-bundle.tar.gz \
  'araf-support-bundle*/diagnostics.txt' | less
```

Verify the archive permissions and scan it for local secret markers. The
synthetic regression test is `./tests/support-bundle-security.sh`; it proves
OIDC token-like values, bearer tokens, passwords, client secrets and session
keys supplied in the environment are absent from the archive.
