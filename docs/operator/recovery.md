# Backup, recovery and outage behavior

## Araf-owned durable state

| State | Durable owner | Backup | Reconstructible? |
| --- | --- | --- | --- |
| Encrypted Tenant/Operator sessions, revocations and auth state | matching BFF session store | yes, with the exact session key and permissions | No for active sessions; users can log in again after loss. |
| OpenStack CompatibilityOperation journal | OpenStack BFF journal path | yes | Partly; provider resource truth is authoritative, but correlation history is not recreated automatically. |
| Environment/configuration and secret references | deployment system | yes (metadata and references, never values in Git) | Reapply from reviewed release config and secret manager. |

O3K and OpenStack resources, images, volumes, networks, quotas and provider
databases are not Araf backups. Use the cloud platform's own backup/DR
procedures.

## Store unavailable or corrupt

The BFF fails closed: it must not use stale in-memory sessions when the durable
authority is missing, unreadable or corrupt. Check mount, UID/GID, permissions,
disk-full and lock behavior; restore the last known-good snapshot with the
matching key. If state cannot be recovered, plan session re-login and inspect
all pending CompatibilityOperations against authoritative provider state
before declaring an outcome.

## Encryption-key mismatch

Stop the rollout and restore the previous key from the secret manager. Do not
generate a replacement key in place or delete the store. After state is
readable, perform a planned rotation and verify both surfaces independently.

## Expired/cross-replica sessions

Expired sessions are rejected and reaped. Cross-replica login requires the
shared durable session/auth-state authority; check that both replicas mount
the same surface store and use the same key. Tenant and Operator stores are
never interchangeable.

## IdP outage

Existing unexpired sessions continue until their normal TTL or revocation
check requires the IdP; new login/callback and refresh flows fail closed. Do
not bypass OIDC or issue synthetic cookies. Check IdP discovery, token
endpoint, DNS/TLS, clock and client status, then verify a new login after
recovery.

## Backend outage

An O3K/OpenStack outage can make reads/mutations fail while the BFF process and
sessions remain healthy. Preserve the request/correlation and operation IDs,
wait for bounded timeout/error, and reconcile after recovery. Never blindly
repeat a mutation after a lost response; resource/Operation truth determines
whether it happened.
