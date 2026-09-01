import { useCallback, useEffect, useState } from "react";
import type { Scope, RegionId } from "./types";

const PROJECT_PARAM = "project";
const REGION_PARAM = "region";
const ORGANIZATION_PARAM = "org";

function readScopeFromUrl(url: URL): Partial<Scope> {
  const projectId = url.searchParams.get(PROJECT_PARAM) ?? undefined;
  const regionIdRaw = url.searchParams.get(REGION_PARAM) ?? undefined;
  const organizationId = url.searchParams.get(ORGANIZATION_PARAM) ?? undefined;

  const regionId: RegionId | undefined =
    regionIdRaw === "global" ? "global" : (regionIdRaw ?? undefined);

  return {
    ...(organizationId ? { organizationId } : {}),
    ...(projectId ? { projectId } : {}),
    ...(regionId !== undefined ? { regionId } : {}),
  };
}

/**
 * Read the scope encoded in the current browser URL.
 */
export function getScopeFromUrl(): Partial<Scope> {
  return readScopeFromUrl(new URL(window.location.href));
}

function writeScopeToUrl(url: URL, scope: Scope): URL {
  const next = new URL(url.href);

  if (scope.organizationId) {
    next.searchParams.set(ORGANIZATION_PARAM, scope.organizationId);
  } else {
    next.searchParams.delete(ORGANIZATION_PARAM);
  }

  if (scope.projectId) {
    next.searchParams.set(PROJECT_PARAM, scope.projectId);
  } else {
    next.searchParams.delete(PROJECT_PARAM);
  }

  if (scope.regionId !== undefined && scope.regionId !== "") {
    next.searchParams.set(REGION_PARAM, scope.regionId);
  } else {
    next.searchParams.delete(REGION_PARAM);
  }

  return next;
}

/**
 * Update the browser URL to reflect the current scope without reloading.
 *
 * Uses `history.replaceState` so refresh/back navigation retain the scope.
 */
export function syncScopeToUrl(scope: Scope): void {
  const next = writeScopeToUrl(new URL(window.location.href), scope);
  const nextHref = next.pathname + next.search + next.hash;
  if (nextHref !== window.location.pathname + window.location.search + window.location.hash) {
    window.history.replaceState(null, "", nextHref);
  }
}

/**
 * React hook that returns the scope encoded in the current URL and re-renders
 * on `popstate` events.
 */
export function useScopeFromUrl(): Partial<Scope> {
  const [scope, setScope] = useState<Partial<Scope>>(() => getScopeFromUrl());

  useEffect(() => {
    const handleChange = () => {
      setScope(getScopeFromUrl());
    };
    window.addEventListener("popstate", handleChange);
    return () => {
      window.removeEventListener("popstate", handleChange);
    };
  }, []);

  return scope;
}

/**
 * React hook that synchronizes a controlled scope to the URL.
 */
export function useSyncScopeToUrl(scope: Scope): void {
  const sync = useCallback(() => {
    syncScopeToUrl(scope);
  }, [scope]);

  useEffect(() => {
    sync();
  }, [sync]);
}

/**
 * Merge URL-derived scope with an explicit default scope.
 */
export function mergeScopeWithUrl(defaultScope: Scope, urlScope: Partial<Scope>): Scope {
  return {
    ...defaultScope,
    ...urlScope,
  };
}

export { PROJECT_PARAM, REGION_PARAM, ORGANIZATION_PARAM };
