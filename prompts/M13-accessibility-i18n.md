# M13 implementation prompt — accessibility, i18n, density and cross-browser closure

Implement/finalize **M13** after core tenant/operator flows exist. Earlier phases must already follow accessible component practices.

## Read first

- `AGENTS.md`
- `docs/product/information-architecture.md`
- `docs/engineering/quality-gates.md`

## Goal

Close the MVP as an enterprise console usable with keyboard/assistive technology, ready for localization, and consistent across supported desktop browsers and density modes.

## Required implementation

1. Audit critical tenant/operator workflows against WCAG 2.2 AA requirements relevant to the product.
2. Fix:
   - focus order/visibility,
   - accessible names/descriptions,
   - form errors,
   - modal/drawer focus trapping/restoration,
   - table/action keyboard behavior,
   - status semantics not conveyed by color alone,
   - target sizing where applicable.
3. Add automated accessibility tests and manual-check evidence for critical flows.
4. Externalize user-facing strings through a lightweight i18n framework/contract; English can remain the only shipped locale for MVP.
5. Centralize date/time/number/currency/unit formatting and make timezone context explicit.
6. Verify Comfortable/Compact density across canonical pages.
7. Run release-critical Playwright flows on Chromium, Firefox and WebKit/Safari-compatible engine.
8. Verify narrow-width/responsive usability for core tenant tasks without claiming native mobile parity.
9. Record known bounded accessibility limitations only if genuinely unavoidable and tracked.

## Acceptance

- critical flows meet the project WCAG 2.2 AA target with documented automated/manual evidence,
- no hard-coded unextractable user-facing strings in core runtime/pages,
- dates/numbers/currency are locale-safe,
- compact/comfortable modes do not break interactions,
- release-critical cross-browser tests pass.

Branch suggestion: `m13-accessibility-i18n`.
