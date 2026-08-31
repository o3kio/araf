# Security Policy

## Project status

Araf is in architecture/prototype development and must not yet be used as a production cloud control-plane console. The authentication, authorization, session, extension, and tenant/operator trust boundaries are being implemented and validated through the MVP roadmap.

## Reporting

Do not open a public issue for a suspected vulnerability that could expose credentials, tenant data, operator access, provider access, session material, or destructive cloud operations. Report it privately to `security@kubedo.io` with:

- affected version or commit;
- reproduction steps;
- expected impact;
- suggested mitigation when available.

Do not include real secrets or customer data.

## Security principles

- separate Tenant and Operator console/BFF/session trust surfaces;
- no O3K access or refresh tokens in browser storage;
- least privilege and explicit organization/project/region scope;
- server-side authorization remains authoritative;
- strict CSRF and CSP protections;
- no arbitrary remotely executed JavaScript service extensions;
- untrusted resource and descriptor content is treated as data, never executable markup;
- provider and infrastructure credentials remain outside browser/public API handling;
- no secret values in logs, traces, metrics, errors, or audit events;
- bounded input sizes and strict schema/descriptor validation;
- signed releases, SBOMs, provenance, dependency and license review before production release.

See `docs/security/threat-model.md` for the architecture threat model and `docs/engineering/quality-gates.md` for required evidence.

## Supported versions

No version is currently supported for production security fixes. This policy will be updated before the first production-supported release.
