# M8 implementation prompt — tenant governance, access, quota and audit

Implement **M8** after M2/M3/M4 and after confirming current upstream identity/governance APIs.

## Read first

- `AGENTS.md`
- `docs/architecture/o3k-integration-contract.md`
- `docs/security/threat-model.md`
- `docs/product/information-architecture.md`
- current O3K AuthContext/IAM/project/quota/audit contracts.

## Goal

Give delegated organization/project administrators enterprise-grade governance without exposing platform-operator authority.

## Required implementation

1. Inspect current O3K governance APIs; implement only confirmed capabilities.
2. Add tenant-side project/access views for supported:
   - current identity/AuthContext,
   - project membership/users/groups/roles as available,
   - effective capabilities,
   - quota and usage limits,
   - audit/event reads.
3. UI action availability must use effective server capabilities rather than hard-coded role-name comparisons.
4. Clearly distinguish Audit from Operations.
5. Add cross-project negative tests and stale-context tests.
6. Developer/API credentials section may expose supported service-account/application-credential flows, but must never display a long-lived secret again after creation unless the authoritative API explicitly supports secure retrieval and product requirements justify it.
7. Avoid building a second IAM policy language in Araf.
8. Document unsupported desired governance features as upstream dependencies.

## Non-goals

- provider/operator IAM,
- full reseller hierarchy,
- custom policy editor if O3K lacks an authoritative policy contract,
- hidden support impersonation.

## Acceptance

- delegated user can understand project access/quota/audit within authorized scope,
- forbidden project/resource access is rejected server-side,
- role names are not treated as permission truth,
- audit and operation histories remain distinct,
- secrets are not persisted in browser storage/logs.

Branch suggestion: `m8-tenant-governance`.
