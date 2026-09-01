import { useEffect, useState, type ReactNode } from "react";

const OPERATOR_PATH_PREFIXES = ["/operator", "/platform", "/infrastructure", "/system"];

/**
 * Return true if the given path belongs to the operator route surface.
 *
 * This is a coarse client-side guard for bundle/route separation; real
 * authorization enforcement lives in the BFF and O3K.
 */
export function isOperatorRoute(pathname: string): boolean {
  return OPERATOR_PATH_PREFIXES.some((prefix) => pathname.startsWith(prefix));
}

export interface TenantRouteGuardProps {
  children: ReactNode;
  fallback?: ReactNode;
}

/**
 * Render children only when the current URL is not an operator route.
 */
export function TenantRouteGuard({ children, fallback = null }: TenantRouteGuardProps) {
  const [blocked, setBlocked] = useState(() => isOperatorRoute(window.location.pathname));

  useEffect(() => {
    const check = () => {
      setBlocked(isOperatorRoute(window.location.pathname));
    };
    window.addEventListener("popstate", check);
    return () => {
      window.removeEventListener("popstate", check);
    };
  }, []);

  if (blocked) {
    return <>{fallback}</>;
  }

  return <>{children}</>;
}
