# Production configuration reference

Configuration is environment-based and validated before the BFF binds its
listener. Production defaults to fail-closed: an omitted adapter, insecure
URL, missing durable store or missing credential stops startup. Values below
are the supported production API; internal test variables are intentionally
omitted.

| Setting | Required | Values/default | Applies | Restart | Security/notes |
| --- | --- | --- | --- | --- | --- |
| `ARAF_RUNTIME_PROFILE` | yes | `production` | both | yes | `development`/`test` are fixture-only profiles. |
| `ARAF_UPSTREAM_ADAPTER` | yes | `o3k` or `openstack` | both | yes | No implicit fallback. |
| `ARAF_VERSION` | no | package version | both | yes | Release metadata only. |
| `ARAF_GIT_SHA` | no | unset | both | yes | Release metadata only. |
| `ARAF_O3K_API_CONTRACT` | when adapter=`o3k` | `o3k.io/v1` | both | yes | Required by the release profile; a mismatch stops startup. |
| `ARAF_PUBLIC_URL` | yes | HTTPS absolute URL | matching surface | yes | No credentials/query/fragment. |
| `ARAF_TRUSTED_ORIGINS` | yes | comma-separated HTTPS origins | matching surface | yes | Origins have no path/query/fragment; controls CORS/CSRF trust. |
| `ARAF_SESSION_STORE_PATH` | yes | writable durable path | matching surface | yes | Must be persistent; local-only storage is not HA-safe. |
| `ARAF_SESSION_STORE_KEY` | yes | base64-encoded exactly 32-byte AES key | matching surface | yes | Secret-manager only; losing it makes encrypted state unreadable. |
| `O3K_URL` | when adapter=`o3k` | HTTPS host-qualified URL | matching surface | yes | Compose maps this from the separate tenant/operator upstream settings; no credentials/query/fragment. |
| `O3K_TOKEN` | optional | server-side bootstrap token | both | yes | Development fallback; authenticated production requests use the server-side session token. Never browser-visible. |
| `SSL_CERT_FILE` | when private CA is needed | readable PEM bundle | both | yes | Explicit trust bundle for upstream TLS; verification is never disabled. |
| `OPENSTACK_AUTH_URL` | when adapter=`openstack` | HTTPS Keystone URL | both | yes | Credential-free, host-qualified. |
| `OPENSTACK_TOKEN` | one auth mode | server-side Keystone token | both | yes | Mutually sufficient with username/password; never browser-visible. |
| `OPENSTACK_USERNAME` + `OPENSTACK_PASSWORD` | one auth mode | non-empty pair | both | yes | Password is never persisted or logged; prefer token/federated deployment. |
| `OPENSTACK_USER_DOMAIN_NAME` | no | `Default` | both | yes | Keystone user domain. |
| `OPENSTACK_PROJECT_NAME` / `OPENSTACK_PROJECT_ID` | no | unset | both | yes | Optional service-account scope hints; user scope is validated by Keystone. |
| `OPENSTACK_REGION` | no | unset | both | yes | Optional catalog region. |
| `OPENSTACK_COMPUTE_URL` | no | URL | both | yes | Pin only when catalog discovery is unsuitable; capability is absent when unavailable. |
| `OPENSTACK_IMAGE_URL` | no | URL | both | yes | Glance endpoint; absence hides image capability. |
| `OPENSTACK_NETWORK_URL` | no | URL | both | yes | Neutron endpoint; absence hides network capability. |
| `OPENSTACK_VOLUME_URL` | no | URL | both | yes | Cinder endpoint; absence hides volume capability. |
| `OPENSTACK_OBJECT_STORAGE_URL` | no | URL | both | yes | Optional; Swift/object storage is not in the certified profile. |
| `ARAF_OPENSTACK_COMPATIBILITY_JOURNAL` | yes for OpenStack production | durable file path | both | yes | Correlation/reconciliation state, not cloud truth; back it up. |
| `ARAF_TENANT_OIDC_CLIENT_ID` / `_CLIENT_SECRET` / `_ISSUER_URL` / `_REDIRECT_URI` | yes | confidential client and HTTPS URLs | Tenant BFF | yes | Secret only in secret manager; issuer is discovered and validated. |
| `ARAF_OPERATOR_OIDC_CLIENT_ID` / `_CLIENT_SECRET` / `_ISSUER_URL` / `_REDIRECT_URI` | yes | confidential client and HTTPS URLs | Operator BFF | yes | Separate client, origin and cookie namespace from Tenant. |
| `ARAF_TENANT_BFF_PORT` | no | `8080` | Tenant BFF | yes | Container listener; expose only through private service/ingress. |
| `ARAF_OPERATOR_BFF_PORT` | no | `8081` | Operator BFF | yes | Container listener; expose only through private service/ingress. |
| `RUST_LOG` | no | subscriber default | both | no | Use an allowlisted level; structured logs never include secrets. |

All endpoint URLs are absolute and credential-free. In production, every
explicit OpenStack service URL and every selected Keystone catalog endpoint
must use the same HTTPS scheme as `OPENSTACK_AUTH_URL`; invalid/insecure
catalog entries are ignored, and the adapter does not follow HTTP redirects.
`ARAF_TRUSTED_ORIGINS` must contain origins, not callback paths. Changing any
setting requires a controlled restart/rollout; changing the session key
without restoring the matching old key invalidates encrypted sessions.
