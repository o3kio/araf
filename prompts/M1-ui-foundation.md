# M1 implementation prompt — Araf design system foundation

Implement **M1** after M0.

## Read first

- `AGENTS.md`
- `docs/product/information-architecture.md`
- `docs/architecture/overview.md`
- ADR 0004
- `docs/engineering/quality-gates.md`

## Goal

Create the Araf-owned UI foundation that can support an enterprise cloud console without leaking the underlying component-library API through the product.

## Required implementation

1. Evaluate and integrate the selected Cloudscape-compatible enterprise primitives **only inside `packages/ui`**.
2. Define Araf design tokens for:
   - typography,
   - spacing,
   - semantic status colors,
   - focus,
   - radius/borders,
   - comfortable/compact density,
   - motion constraints.
3. Create stable Araf components needed by later phases, initially including:
   - application page/header primitives,
   - resource status,
   - table wrapper,
   - empty/loading/error states,
   - form field/section primitives,
   - confirmation modal,
   - tabs,
   - notification/toast primitive.
4. Build a component showcase/dev route or Storybook-like equivalent only if it remains lightweight and helps verify accessibility/theming. Do not introduce a large tool without need.
5. Establish light/dark-ready token architecture; dark mode may be implemented if straightforward but visual polish is secondary to stable semantics.
6. Add accessible component tests and initial visual regression snapshots for canonical primitives.
7. Ensure application packages contain **zero direct imports** from the underlying component library; enforce via lint/module-boundary rule where practical.

## Design requirements

- calm enterprise aesthetic,
- high information density without dashboard clutter,
- provider-neutral vocabulary,
- visible keyboard focus,
- no glassmorphism/neon/animation-heavy theme,
- O3K/Araf identity must not look like a reskinned AWS console.

## Non-goals

- no resource runtime,
- no full page implementations,
- no branding editor,
- no arbitrary theme CSS injection.

## Acceptance

- `packages/ui` is the sole third-party UI import boundary,
- comfortable/compact mode is token-driven,
- semantic status and focus behavior is documented/tested,
- tenant/operator apps can consume the same primitives,
- automated accessibility checks pass for the implemented primitives.

Branch suggestion: `m1-ui-foundation`.
