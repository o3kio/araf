# Secrets, custody and rotation

The deployment secret manager owns secret values. Kubernetes Secrets or
Compose environment injection are transport mechanisms, not a reason to put
values in Git, Helm values, images, support bundles or browser storage.

| Secret | Custodian | Used by | Rotation |
| --- | --- | --- | --- |
| `ARAF_SESSION_STORE_KEY` (32-byte AES key, base64) | deployment/security operator | matching BFF surface's durable session store | Add the new key only through a planned migration; retain the old key until all state is re-encrypted/expired. Never regenerate during rollback. |
| Tenant/Operator OIDC client secret | IdP/deployment operator | only the matching confidential BFF client | Rotate at IdP, update the external secret, restart one surface, verify login, then revoke the old secret. |
| `O3K_TOKEN` (if configured) | O3K/deployment operator | O3K adapter server-side | Issue a replacement, update secret, restart/roll, revoke old token; prefer per-session server-side tokens where supported. |
| `OPENSTACK_TOKEN` | Keystone/deployment operator | OpenStack adapter server-side | Issue/revoke through Keystone and roll the BFF. |
| `OPENSTACK_USERNAME` / `OPENSTACK_PASSWORD` | Keystone/deployment operator | OpenStack adapter server-side | Create a replacement service identity, update both values atomically, verify catalog access, revoke old password. |
| TLS private key/certificate | ingress/Kubernetes operator | browser-facing ingress | Rotate at the ingress secret; verify certificate chain and HSTS. It never enters a BFF env or bundle. |

Inject only the keys named by the chart (`secrets.name` and its key fields) or
an equivalent external-secret mapping. Restrict read access to the matching
surface and storage. File-backed session/journal volumes must be encrypted at
rest and permissioned to the non-root BFF UID.

Never place access/refresh/O3K/Keystone tokens, passwords, client secrets,
session cookies, session keys, PKCE verifiers or private keys in a browser,
ConfigMap, values file, image, log, issue or support archive.
