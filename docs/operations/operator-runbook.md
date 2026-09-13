# Araf operator runbook

This runbook is written for an engineer with no repository knowledge. Use the
versioned Compose or Helm artifacts; do not deploy from a source checkout.

## Install and verify

1. Supply digest-pinned images and external secrets (`ARAF_BFF_IMAGE`,
   `ARAF_BFF_DIGEST`, OIDC credentials, backend credentials and the durable
   session-store path).
2. Set `ARAF_RUNTIME_PROFILE=production`, an explicit `ARAF_UPSTREAM_ADAPTER`
   (`o3k` or `openstack`), HTTPS public/trusted origins and the documented
   backend endpoint variables.
3. Deploy `deploy/docker-compose.release.yml` or the Helm chart.
4. Verify `/healthz`, `/readyz` and `/version` on both BFF surfaces. A `ready`
   response means configuration and the adapter boundary are present; it does
   not claim that a cloud mutation succeeded.

## Diagnosis

Start with the browser Problem Details `correlationId` and
`X-Request-ID`/`X-Correlation-ID` response headers. Search BFF structured logs
for those values, then inspect `/metrics` for status/latency and backend
status-class counters. Follow the linked canonical O3K Operation or
OpenStack CompatibilityOperation to the authoritative provider read.

Common incidents:

- **BFF unavailable:** check pod/container health, `/healthz`, resource limits,
  certificate/trusted-proxy configuration and recent rollout events.
- **Ready but backend unavailable:** inspect `/readyz`, backend counters and
  authenticated operator health; provider state is never inferred from a 202.
- **Login/CSRF failures:** verify issuer discovery, redirect URI, HTTPS origin,
  surface-specific cookie and clock skew. Never request a token from the
  browser.
- **Operation stuck/failing:** use the canonical Operation timeline and
  correlation ID; retry only when the authoritative contract says retryable.
- **OpenStack CompatibilityOperation unknown:** inspect the durable journal,
  then read Nova/Neutron/Cinder authoritative state. Compatibility records do
  not override provider state and reconcile after BFF restart.
- **Quota/discovery issue:** preserve the structured code/detail and check
  project scope/capabilities; do not substitute fixture data.

## Recovery and change

Back up the encrypted/permissioned session store and compatibility journal
according to the deployment storage policy. Restore them only to the matching
surface and release. Roll out N+1 with the same external state, verify
`/version`, readiness, session continuity and operation recovery, then roll
back the image digest if required. Configuration errors must stop startup.

## Support bundle

Run `tests/support-bundle.sh /secure/path/bundle` from an operator host. The
script collects version/readiness/metrics and environment variable names only;
it never copies cookies, tokens, credentials, session files, journals or
request bodies. Inspect the archive before transferring it.

## Supported profile boundaries

O3K is the native backend. The certified OpenStack profile is Keystone, Nova,
Glance, Neutron and Cinder. Swift/Object Storage, floating IP and volume
attachment workflows are not advertised unless separately certified.
