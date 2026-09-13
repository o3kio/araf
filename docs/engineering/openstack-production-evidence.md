# OpenStack production support-profile evidence

## Verdict

`NO-GO`

The repository now contains the backend boundary and compatibility adapter,
but no representative real OpenStack deployment was available in this
execution environment. A production support claim therefore remains blocked by
evidence, not by a fixture result.

The fail-closed gate is [`tests/p3-9-openstack-production-gate.sh`](../../tests/p3-9-openstack-production-gate.sh).
It was executed without a real-environment harness and returned the exact
`NO-GO` verdict, writing a redacted result artifact under
`target/p3-9-openstack-gate/result.env`.

## Implemented boundary

| Area | Adapter behavior | Evidence |
|---|---|---|
| Keystone | Server-side token, project discovery and scope validation | `openstack.rs`, contract tests |
| Nova | Server list/show/create/delete/start/stop mapping | `openstack.rs` |
| Glance | Image list/show mapping | `openstack.rs` |
| Neutron | Network/subnet/port/security-group list/show and generic mutations | `openstack.rs` |
| Cinder | Volume list/show/create/delete mapping | `openstack.rs` |
| Swift | Optional container capability, hidden unless endpoint is configured | `OPENSTACK_OBJECT_STORAGE_URL` |
| Operations | `openstack-compat-*` correlation objects persisted in a bounded journal and reconciled from resource reads; resource state remains authoritative | ADR 0005; `compatibility.rs` |

## Required real-run evidence (not executed)

- Keystone authentication with a federated or explicitly configured
  server-side credential and two projects;
- Nova/Glance/Neutron/Cinder lifecycle journeys through the same Playwright
  flows as O3K;
- real asynchronous failure and restart/reconciliation scenarios;
- quota responses and cross-project negative access;
- exact OpenStack release/service versions and endpoint microversions.

Until those checks are run against a representative deployment, the only
valid P3.9 verdict is `NO-GO`.
