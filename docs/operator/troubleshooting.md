# Troubleshooting and incident runbooks

Start every incident by recording UTC time, surface, release/SHA,
`X-Request-ID`, `X-Correlation-ID`, structured error code and (if present)
Operation/CompatibilityOperation ID. Never retry a mutation while its outcome
is ambiguous.

## Decision tree: failed VM create

```text
Create unavailable
  -> Is Compute advertised and ready?
       no -> capability discovery/readiness; distinguish unsupported from outage
       yes
  -> Did POST reach the matching BFF (request/correlation log)?
       no -> browser session, origin or CSRF path
       yes
  -> BFF response
       4xx -> scope, authorization, validation or quota; do not retry blindly
       5xx/timeout -> backend/provider incident; inspect upstream status
       async -> inspect canonical Operation (O3K) or CompatibilityOperation (OpenStack)
```

## O3K runbooks

### O3K unreachable or backend outage

**Symptoms:** timeout/5xx, rising `upstream_calls` failures, resource pages
degraded; existing browser sessions remain valid. `readyz` can remain green
because it describes configured construction, not a live mutation.

**Checks:** check BFF health/readiness, `O3K_URL` DNS/TLS from the BFF network,
O3K `/healthz`/`/readyz`, status-class metrics and the correlation chain.
Check the O3K control-plane/provider health as an operator. Do not switch to
fixtures or edit browser requests.

**Safe remediation:** restore route/TLS/DNS or O3K capacity; fix the upstream
incident. Do not replay a mutation after a timeout until the authoritative
Operation/resource is inspected.

**Verify:** a read-only list/show succeeds, then follow one known Operation;
record recovery time and IDs.

### OIDC/authentication failure

Check issuer discovery and issuer-origin match, client ID/secret, exact HTTPS
redirect URI, surface host, clock skew, callback reachability and cookie
flags. Then check server-side session lookup, expiry and AuthContext/scope
selection. A 401 after login is not fixed by requesting a token in the
browser; rotate the client secret or restore the IdP/config and restart.

### Mutation or Operation stuck/failing

For O3K, locate the canonical Operation ID and timeline; inspect structured
error/code, scope, actor, attempt and provider/controller evidence. A `202`
only means accepted. Retry only if the O3K contract marks the action
retryable; otherwise reconcile by read.

### NoValidHost/capacity

If the request reaches O3K and the Operation fails with `NoValidHost` or an
equivalent scheduling/capacity code, inspect provider registration,
availability zone/region, host inventory, image/flavor constraints, quota and
placement capacity in the O3K operator diagnostics. This is a provider or
capacity defect, not evidence that the Araf UI fabricated a failure.

### Capability absent

Compare the authenticated service/capability discovery response with the
configured backend and readiness. Missing service/endpoint or `ready=false`
means unsupported/unavailable capability. If discovery itself times out or
returns 5xx, it is an outage. Never re-enable a hidden action by editing the
client.

## OpenStack runbooks

For every case, OpenStack services remain authoritative; Araf's
`openstack-compat-*` CompatibilityOperation is a durable correlation and
reconciliation record only. Read Nova/Glance/Neutron/Cinder/Keystone after an
uncertain response.

- **Keystone 401/403/catalog:** verify BFF auth mode, token expiry, project
  scope, domain and service catalog/region. Re-authenticate or rotate the
  server credential; do not expose it to the browser.
- **Nova scheduling:** inspect flavor/image constraints, host/placement
  capacity, AZ and quota. `NoValidHost` is upstream capacity/scheduling.
- **Glance image:** verify image visibility/owner, status and endpoint; a
  missing image is not an Araf resource-state success.
- **Neutron networking:** verify network/subnet/port/security-group scope and
  project filters; re-read authoritative Neutron state after 409/5xx.
- **Cinder volume:** verify backend availability, quota and attachment state;
  attachment UX is outside the certified profile unless separately enabled.
- **CompatibilityOperation unknown:** inspect journal readability/permissions,
  correlation ID and last observed state, then read the authoritative service.
  Mark unknown until provider state establishes a result; never convert it to
  success from HTTP acceptance.
- **409 conflict:** inspect generation/state and provider resource before any
  retry; use the documented precondition/idempotency contract.
- **5xx/temporary outage:** check bounded timeout and upstream recovery;
  retain the operation record and reconcile, without automatic replay.

## BFF unavailable

Check ingress routing and TLS, pod/container restart events, resource limits,
`/healthz`, readiness, filesystem permissions and recent digest/config
changes. Restore the previous artifact only under the rollback rules; do not
delete durable state.
