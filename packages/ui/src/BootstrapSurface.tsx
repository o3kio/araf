import type { ReactNode } from "react";

/**
 * Minimal Araf-owned surface primitive used by the M0 bootstrap pages.
 *
 * This package is the only place a third-party enterprise component library
 * may be imported (ADR 0004). No underlying library is adopted yet; M1 owns
 * the design-system foundation. Application code must consume primitives from
 * here rather than reaching around the abstraction.
 */
export interface BootstrapSurfaceProps {
  /** Accessible page heading, e.g. "Araf Tenant Console". */
  readonly title: string;
  /** Short explanation of what this surface is. */
  readonly description: string;
  readonly children?: ReactNode;
}

export function BootstrapSurface({ title, description, children }: BootstrapSurfaceProps) {
  return (
    <main>
      <h1>{title}</h1>
      <p>{description}</p>
      {children}
    </main>
  );
}
