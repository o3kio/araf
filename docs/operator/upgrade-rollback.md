# Upgrade and rollback

The supported change is an artifact/config rollout that preserves durable
state. Run in a disposable or maintenance window and record the exact old and
new image digests.

## Upgrade

1. **Preflight:** read the release compatibility matrix and [limitations](limitations.md);
   confirm O3K/OpenStack profile, IdP, ingress, storage capacity, backups,
   secret availability and a clean `git diff`/values review.
2. **Backup/state check:** snapshot each surface's encrypted session store;
   snapshot the OpenStack CompatibilityOperation journal when used. Confirm
   the matching session key is recoverable. Do not back up cloud resources as
   Araf-owned state.
3. **Verify artifacts:** check registry digest, SBOM and keyless provenance;
   run `./tests/package-upgrade-rollback.sh` and Helm lint.
4. **Roll out:** update only image/version metadata and reviewed compatible
   configuration through Helm or Compose. Preserve PVCs, journal paths,
   cookie names and keys. Use a rolling strategy only when the shared store
   contract is actually available.
5. **Verify:** wait for `/healthz`, `/readyz`, `/version` on both surfaces;
   complete login, scope selection, service discovery, bounded list, one
   read-only operation and journal/session continuity smoke.
6. **Observe:** watch error/latency/upstream metrics and logs for at least the
   deployment's normal observation window; retain correlation IDs for defects.

Mixed versions are supported only for the tested N→N+1 window in the release
   evidence. Do not mix versions across an untested schema or session-journal
   format.

## Rollback safety

Rollback is safe when the old artifact can read the unchanged durable state,
configuration is compatible, and no irreversible migration has run. Restore
the previous image digest and compatible non-secret config, keep the same
session key/PVC/journal, then repeat health and session/operation smoke.

Rollback is not safe as an automatic action after a state-format migration,
key rotation without the old key, or a mutation whose provider outcome is
ambiguous. First reconcile authoritative O3K/OpenStack state; never replay a
destructive action to “test” rollback. The release owner must not promise
rollback outside the tested P4.5 semantics.
