# Information architecture and UX contract

## 1. Design principle

Araf is a cloud-provider console, not a generic CRUD admin theme.

The product should be calm, dense when requested, predictable and task-oriented. Decorative dashboards, service-tile walls and provider jargon are not the default experience.

## 2. Permanent context

Tenant Console application chrome permanently exposes:

- organization/account context as applicable,
- project,
- region or `Global`,
- global search entry point,
- Operations,
- notifications/inbox entry point,
- current identity/help menu.

Actions must never silently execute against a hidden project or region.

## 3. Tenant navigation

Initial structure:

```text
Home

Services
  Compute
  Networking
  Storage
  Images
  [discovered entitled services]

Manage
  Operations
  Resources
  Usage & Cost

Organization
  Projects
  Users & Access
  Quotas
  Audit

Developer
  API / credentials where supported
  CLI / Terraform documentation links
```

Only deployed and entitled services appear. Structural navigation is stable; service entries are discovered.

## 4. Tenant home

The tenant home answers:

1. What do I have?
2. Is something wrong?
3. What is happening now?
4. What am I consuming?

Prototype cards/sections:

- quick create actions,
- needs attention,
- resource counts,
- usage/quota summary,
- active/recent Operations,
- recent resources.

Avoid graphs that do not lead to an actionable decision.

## 5. Operator navigation

Initial structure:

```text
Platform
  Overview
  Regions
  Health
  Capacity

Customers
  Accounts / Organizations
  Projects

Services
  Catalog
  Offerings (future-ready)
  Installed Services

Infrastructure
  Compute Providers
  Network Providers
  Storage Providers

Operations
  Operations
  Maintenance (post-MVP depth)

Governance
  IAM
  Quotas
  Metering
  Audit

System
  Identity configuration (only when upstream support exists)
  Certificates / feature management / upgrades (post-MVP depth)
```

The MVP may implement only the validated subset, but the shell must not require redesign to add the remaining sections.

## 6. Operator home

Focus on actionable platform state:

- region health,
- provider health,
- bounded capacity summaries,
- running/failed operations,
- services needing attention,
- explicit data freshness timestamps.

Araf must not invent capacity by summing provider-specific fields in the browser; use authoritative normalized O3K data.

## 7. Resource pages

Generic pattern:

```text
Breadcrumb / service / collection / resource

Resource name                         status
opaque ID / key context

primary actions / overflow

Overview | Configuration | Relationships | Metrics | Operations | Audit | Usage & Cost
```

Only meaningful tabs render.

## 8. Progressive disclosure

Create/edit flows show the minimum safe required choices first. Advanced provider-neutral controls live under an Advanced section.

Tenant flows do not expose KVM/Cloud Hypervisor/Ceph/LVM or controller names unless O3K intentionally models a user-facing offering/capability that requires such a choice.

## 9. Operations UX

Global Operations view provides:

- state,
- action,
- scope,
- resource,
- initiator when authorized,
- start/update time,
- correlation ID,
- structured failure state.

Operation details may show a timeline only from authoritative operation/event evidence. Araf must not create fake orchestration steps to make the UI appear detailed.

## 10. Audit UX

Audit and Operation are distinct:

- Operation = work O3K is performing/performed.
- Audit = who/what invoked or changed something.

A resource may link to both.

## 11. Error UX

Never use `Something went wrong` as the only actionable information when structured data exists.

Show safe:

- title/detail,
- operation link,
- correlation ID,
- retry guidance only when authoritative,
- troubleshooting/documentation link when trusted.

Secrets/internal stack traces never render to ordinary users.

## 12. Density and responsive behavior

Provide Comfortable and Compact density modes through design tokens/components, not per-page CSS hacks.

Desktop is the primary operational surface. Responsive behavior must keep critical tenant tasks usable on narrower screens, but a native/mobile-first administration product is not an MVP requirement.
