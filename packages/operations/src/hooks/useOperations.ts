import { useCallback, useEffect, useMemo, useState } from "react";
import { useSearchParams } from "react-router";
import type {
  ArafApiError,
  ListOperationsQuery,
  Operation,
  OperationState,
  PaginatedCollection,
} from "@araf/api-client";
import { useOperationsClient } from "../client/context";

export interface OperationsFilters {
  state: OperationState | undefined;
  action: string;
  resourceType: string;
  resourceId: string;
  projectId: string;
  regionId: string;
}

export interface UseOperationsResult {
  collection: PaginatedCollection<Operation> | undefined;
  filters: OperationsFilters;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  setPage: (page: number) => void;
  setPageSize: (pageSize: number) => void;
  setFilter: <K extends keyof OperationsFilters>(key: K, value: OperationsFilters[K]) => void;
  clearFilters: () => void;
  refresh: () => void;
}

const PAGE_PARAM = "page";
const PAGE_SIZE_PARAM = "pageSize";
const STATE_PARAM = "state";
const ACTION_PARAM = "action";
const RESOURCE_TYPE_PARAM = "resourceType";
const RESOURCE_ID_PARAM = "resourceId";
const PROJECT_ID_PARAM = "projectId";
const REGION_ID_PARAM = "regionId";

const FILTER_KEYS: (keyof OperationsFilters)[] = [
  "state",
  "action",
  "resourceType",
  "resourceId",
  "projectId",
  "regionId",
];

function parseIntOr(value: string | null, fallback: number): number {
  if (value === null) return fallback;
  const parsed = Number.parseInt(value, 10);
  return Number.isNaN(parsed) ? fallback : parsed;
}

function clampPageSize(pageSize: number): number {
  return Math.min(Math.max(pageSize, 1), 100);
}

function isOperationState(value: string): value is OperationState {
  return value === "pending" || value === "running" || value === "succeeded" || value === "failed";
}

function buildFilters(params: URLSearchParams): OperationsFilters {
  const rawState = params.get(STATE_PARAM);
  return {
    state: rawState && isOperationState(rawState) ? rawState : undefined,
    action: params.get(ACTION_PARAM) ?? "",
    resourceType: params.get(RESOURCE_TYPE_PARAM) ?? "",
    resourceId: params.get(RESOURCE_ID_PARAM) ?? "",
    projectId: params.get(PROJECT_ID_PARAM) ?? "",
    regionId: params.get(REGION_ID_PARAM) ?? "",
  };
}

function buildSearchParams(
  page: number,
  pageSize: number,
  filters: OperationsFilters,
): URLSearchParams {
  const params = new URLSearchParams();
  if (page !== 1) {
    params.set(PAGE_PARAM, String(page));
  }
  if (pageSize !== 25) {
    params.set(PAGE_SIZE_PARAM, String(pageSize));
  }
  for (const key of FILTER_KEYS) {
    const value = filters[key];
    if (value) {
      params.set(key, value);
    }
  }
  return params;
}

/**
 * Fetch a paginated, filterable list of canonical O3K Operations.
 *
 * Pagination and filter state are synced to URL search params so the list view
 * is bookmarkable. The returned `refresh` function can be used to poll for
 * state updates.
 */
export function useOperations(baseFilters?: Partial<OperationsFilters>): UseOperationsResult {
  const client = useOperationsClient();
  const [searchParams, setSearchParams] = useSearchParams();
  const [collection, setCollection] = useState<PaginatedCollection<Operation> | undefined>(
    undefined,
  );
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<ArafApiError | Error | undefined>(undefined);
  const [refreshToken, setRefreshToken] = useState(0);

  const page = useMemo(
    () => Math.max(parseIntOr(searchParams.get(PAGE_PARAM), 1), 1),
    [searchParams],
  );
  const pageSize = useMemo(
    () => clampPageSize(parseIntOr(searchParams.get(PAGE_SIZE_PARAM), 25)),
    [searchParams],
  );

  const filters = useMemo((): OperationsFilters => {
    const fromUrl = buildFilters(searchParams);
    const overrides = baseFilters ?? {};
    return {
      state: overrides.state ?? fromUrl.state,
      action: overrides.action ?? fromUrl.action,
      resourceType: overrides.resourceType ?? fromUrl.resourceType,
      resourceId: overrides.resourceId ?? fromUrl.resourceId,
      projectId: overrides.projectId ?? fromUrl.projectId,
      regionId: overrides.regionId ?? fromUrl.regionId,
    };
  }, [searchParams, baseFilters]);

  const applyFilters = useCallback(
    (updater: (prev: OperationsFilters) => OperationsFilters) => {
      setSearchParams((prev) => {
        const current = buildFilters(prev);
        const next = updater(current);
        return buildSearchParams(1, pageSize, next);
      });
    },
    [pageSize, setSearchParams],
  );

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(undefined);

    const query: ListOperationsQuery = {
      page: page - 1,
      pageSize,
      ...(filters.state ? { state: filters.state } : {}),
      ...(filters.action ? { action: filters.action } : {}),
      ...(filters.resourceType ? { resourceType: filters.resourceType } : {}),
      ...(filters.resourceId ? { resourceId: filters.resourceId } : {}),
      ...(filters.projectId ? { projectId: filters.projectId } : {}),
      ...(filters.regionId ? { regionId: filters.regionId } : {}),
    };

    client
      .listOperations(query)
      .then((result) => {
        if (!cancelled) setCollection(result);
      })
      .catch((err: unknown) => {
        if (!cancelled) setError(err instanceof Error ? err : new Error(String(err)));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [client, page, pageSize, filters, refreshToken]);

  const setPage = useCallback(
    (nextPage: number) => {
      setSearchParams((prev) => {
        const currentFilters = buildFilters(prev);
        return buildSearchParams(Math.max(nextPage, 1), pageSize, currentFilters);
      });
    },
    [pageSize, setSearchParams],
  );

  const setPageSize = useCallback(
    (nextPageSize: number) => {
      setSearchParams((prev) => {
        const currentFilters = buildFilters(prev);
        return buildSearchParams(1, clampPageSize(nextPageSize), currentFilters);
      });
    },
    [setSearchParams],
  );

  const setFilter = useCallback(
    <K extends keyof OperationsFilters>(key: K, value: OperationsFilters[K]) => {
      applyFilters((prev) => ({ ...prev, [key]: value }));
    },
    [applyFilters],
  );

  const clearFilters = useCallback(() => {
    applyFilters(() => ({
      state: baseFilters?.state,
      action: baseFilters?.action ?? "",
      resourceType: baseFilters?.resourceType ?? "",
      resourceId: baseFilters?.resourceId ?? "",
      projectId: baseFilters?.projectId ?? "",
      regionId: baseFilters?.regionId ?? "",
    }));
  }, [applyFilters, baseFilters]);

  const refresh = useCallback(() => {
    setRefreshToken((t) => t + 1);
  }, []);

  return {
    collection,
    filters,
    loading,
    error,
    setPage,
    setPageSize,
    setFilter,
    clearFilters,
    refresh,
  };
}
