# Post-O3K-stable Araf program

Status: Accepted sequencing

## Goal

Araf resumes active O3K integration work only after O3K completes its Production Phase and publishes the stable bounded O3K Core baseline.

This avoids release-candidate ping/pong and lets Araf certify against a target users can actually rely on.

## Dependency

Cross-repository prerequisite:

- `o3kio/o3k#976` complete;
- stable O3K release published;
- exact source SHA and release provenance known;
- supported O3K profile and API contracts frozen.

## Program order

```text
Stable O3K PP.7 baseline
        |
        v
Stage A — native Araf compatibility/certification
  #126 CSP-safe schema runtime
  #127 test-infrastructure cleanup
  #128 exact stable O3K integration certification
        |
        v
Stage B — HA/failure semantics
  #106
        |
        v
Stage C — live upgrade/rollback
  #113
        |
        v
Stage D — UX and product enhancements
  #112 and later enhancements
```

## Stage A output

Stage A produces one exact compatibility tuple:

- O3K version/source/profile/API contract;
- Araf version/source/OCI digests;
- OIDC/session/CSRF/security evidence;
- strict CSP;
- native browser create/inspect/action/delete;
- reboot/recovery;
- tenant/operator separation;
- durable release evidence.

## Stable-target rule

Do not use a moving O3K branch as the final compatibility authority.

Historical Araf integration candidates remain evidence but are not the new target.

Once stable compatibility is established, normal Araf releases should remain compatible with the same stable O3K contract unless the contract genuinely changes.

## Product authority

Araf is a client/BFF/dashboard.

It never becomes authority for:

- topology;
- Placement;
- CloudProfile;
- BuildingBlocks;
- cloud resources;
- IAM;
- O3K readiness.

## Principle

> Correct integration before HA. HA before broad enhancement. Production evidence before feature breadth.
