# P4.5 production configuration contract

Araf production BFFs fail during startup when required configuration is
missing or invalid. They never fall back to the fixture adapter, temporary
session keys, or in-memory sessions.

Configuration precedence is intentionally simple: built-in defaults (only for
non-security-sensitive values) < environment variables. Secret values are
provided at runtime through the environment or a mounted secret file and are
never stored in an image or ConfigMap. A deployment must provide one complete
source for each required value; duplicate sources are an operational error and
must be resolved before rollout.

Required production values include `ARAF_UPSTREAM_ADAPTER` (`o3k` or
`openstack`), the selected backend endpoint, `ARAF_PUBLIC_URL`,
`ARAF_TRUSTED_ORIGINS`, OIDC client settings for the surface, and
`ARAF_SESSION_STORE_PATH` plus a base64-encoded 32-byte
`ARAF_SESSION_STORE_KEY`. OpenStack also requires
`ARAF_OPENSTACK_COMPATIBILITY_JOURNAL` and either an application token or
username/password credentials.

Production backend and public URLs must be HTTPS, host-qualified, and contain
no credentials, query, or fragment. Trusted origins must be clean HTTPS
origins (no path, query, or fragment). Fixture mode is limited to explicit
development/test profiles.

`GET /healthz` is process liveness. `GET /readyz` reports that startup
configuration and the selected dependency boundary were constructed; a
temporary cloud outage does not fabricate cloud state or force a process
restart. `GET /version` reports only version, git SHA, surface, and backend
metadata.
