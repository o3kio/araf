# M11 implementation prompt — usage, quota and bounded cost awareness

Implement **M11** only against authoritative upstream usage/quota/metering data that exists at implementation time.

## Read first

- `AGENTS.md`
- `docs/product/mvp-prototype.md`
- `docs/architecture/o3k-integration-contract.md`
- current O3K metering/quota/pricing contracts and roadmap.

## Goal

Make resource consumption understandable without turning Araf into an invoicing/accounting system or fabricating cost data.

## Required implementation

1. Inspect upstream metering/usage/quota APIs before designing production adapters.
2. Implement tenant usage/quota summaries using authoritative data.
3. Where a trustworthy price/catalog contract exists, show clearly labeled estimated cost with currency/time basis.
4. If pricing is not upstream-ready, ship usage/quota without fake currency estimates and record the dependency.
5. Add resource-level Usage & Cost section only for meters actually available.
6. Implement bounded date-range querying and loading/error/empty states.
7. Keep billing/invoice/tax/payment semantics outside MVP.
8. Operator view may show aggregate metering health/usage only if server authorization and aggregation contracts support it.
9. Ensure a user cannot query another project's usage by manipulating IDs/ranges.

## Acceptance

- tenant can understand quota and available usage metrics,
- cost is shown only from authoritative price + usage data and is labeled as estimate when appropriate,
- no invoice/billing ledger has been invented,
- cross-project usage access is rejected,
- large date ranges are bounded server-side.

Branch suggestion: `m11-metering-usage`.
