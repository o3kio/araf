import type { ReactNode } from "react";

export interface OperationsNavItemProps {
  href?: string;
  children?: ReactNode;
}

/**
 * Reusable Operations navigation entry point.
 */
export function OperationsNavItem({
  href = "/operations",
  children = "Operations",
}: OperationsNavItemProps) {
  return (
    <a href={href} className="araf-operations-nav-item">
      {children}
    </a>
  );
}
