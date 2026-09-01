import { useCallback, useEffect, useMemo, useState } from "react";
import { useSearchParams } from "react-router";
import type { ArafApiError, PaginatedCollection, Resource, SortDirection } from "@araf/api-client";
import { useScope } from "@araf/shell";
import { useResourceClient } from "../client/context";
import type { ResourceDescriptor } from "../descriptor";

export interface CollectionQuery {
  page: number;
  pageSize: number;
  sortField: string | undefined;
  sortDirection: SortDirection;
  filters: Record<string, string>;
}

export interface UseResourceCollectionResult {
  collection: PaginatedCollection<Resource> | undefined;
  query: CollectionQuery;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  setPage: (page: number) => void;
  setPageSize: (pageSize: number) => void;
  setSort: (field: string | undefined, direction: SortDirection) => void;
  setFilter: (filterId: string, value: string) => void;
  clearFilters: () => void;
  refresh: () => void;
}

const PAGE_PARAM = "page";
const PAGE_SIZE_PARAM = "pageSize";
const SORT_FIELD_PARAM = "sortField";
const SORT_DIRECTION_PARAM = "sortDirection";

function clampPageSize(pageSize: number): number {
  return Math.min(Math.max(pageSize, 1), 100);
}

function parseIntOr(value: string | null, fallback: number): number {
  if (value === null) return fallback;
  const parsed = Number.parseInt(value, 10);
  return Number.isNaN(parsed) ? fallback : parsed;
}

/**
 * Build the initial query from URL search params and descriptor defaults.
 */
function buildQuery(
  params: URLSearchParams,
  descriptor: ResourceDescriptor | undefined,
): CollectionQuery {
  const page = Math.max(parseIntOr(params.get(PAGE_PARAM), 1), 1);
  const pageSize = clampPageSize(parseIntOr(params.get(PAGE_SIZE_PARAM), 25));
  const sortField = params.get(SORT_FIELD_PARAM) ?? undefined;
  const sortDirection: SortDirection = params.get(SORT_DIRECTION_PARAM) === "desc" ? "desc" : "asc";

  const filters: Record<string, string> = {};
  for (const filter of descriptor?.filters ?? []) {
    const value = params.get(filter.id);
    if (value) {
      filters[filter.id] = value;
    }
  }

  return { page, pageSize, sortField, sortDirection, filters };
}

/**
 * Sync the current collection query back to URL search params so filters and
 * pagination are bookmarkable.
 */
function buildSearchParams(
  query: CollectionQuery,
  descriptor: ResourceDescriptor | undefined,
): URLSearchParams {
  const params = new URLSearchParams();

  if (query.page !== 1) {
    params.set(PAGE_PARAM, String(query.page));
  }
  if (query.pageSize !== 25) {
    params.set(PAGE_SIZE_PARAM, String(query.pageSize));
  }
  if (query.sortField) {
    params.set(SORT_FIELD_PARAM, query.sortField);
  }
  if (query.sortDirection === "desc") {
    params.set(SORT_DIRECTION_PARAM, "desc");
  }
  for (const filter of descriptor?.filters ?? []) {
    const value = query.filters[filter.id];
    if (value) {
      params.set(filter.id, value);
    }
  }

  return params;
}

/**
 * Fetch a paginated resource collection using the current scope and
 * bookmarkable URL state. Filter values are sent as query params keyed by
 * filter field so the BFF can apply server-side filtering.
 */
export function useResourceCollection(
  resourceType: string,
  descriptor: ResourceDescriptor | undefined,
): UseResourceCollectionResult {
  const client = useResourceClient();
  const { scope } = useScope();
  const [searchParams, setSearchParams] = useSearchParams();
  const [collection, setCollection] = useState<PaginatedCollection<Resource> | undefined>(
    undefined,
  );
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<ArafApiError | Error | undefined>(undefined);
  const [refreshToken, setRefreshToken] = useState(0);

  const query = useMemo(() => buildQuery(searchParams, descriptor), [searchParams, descriptor]);

  const applyQuery = useCallback(
    (updater: (prev: CollectionQuery) => CollectionQuery) => {
      setSearchParams((prev) => {
        const next = updater(buildQuery(prev, descriptor));
        return buildSearchParams(next, descriptor);
      });
    },
    [descriptor, setSearchParams],
  );

  useEffect(() => {
    if (!descriptor) return;

    let cancelled = false;
    setLoading(true);
    setError(undefined);

    const filterParams: Record<string, string> = {};
    for (const filter of descriptor.filters) {
      const value = query.filters[filter.id];
      if (value) {
        filterParams[filter.field] = value;
      }
    }

    client
      .listResources(resourceType, {
        page: query.page - 1,
        pageSize: query.pageSize,
        projectId: scope.projectId,
        regionId: scope.regionId,
        sortField: query.sortField,
        sortDirection: query.sortDirection,
        ...filterParams,
      })
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
  }, [client, resourceType, descriptor, query, scope, refreshToken]);

  const setPage = useCallback(
    (page: number) => {
      applyQuery((prev) => ({ ...prev, page: Math.max(page, 1) }));
    },
    [applyQuery],
  );

  const setPageSize = useCallback(
    (pageSize: number) => {
      applyQuery((prev) => ({ ...prev, pageSize: clampPageSize(pageSize), page: 1 }));
    },
    [applyQuery],
  );

  const setSort = useCallback(
    (field: string | undefined, direction: SortDirection) => {
      applyQuery((prev) => ({ ...prev, sortField: field, sortDirection: direction }));
    },
    [applyQuery],
  );

  const setFilter = useCallback(
    (filterId: string, value: string) => {
      applyQuery((prev) => ({
        ...prev,
        page: 1,
        filters: {
          ...prev.filters,
          ...(value ? { [filterId]: value } : {}),
        },
      }));
    },
    [applyQuery],
  );

  const clearFilters = useCallback(() => {
    applyQuery((prev) => ({ ...prev, page: 1, filters: {} }));
  }, [applyQuery]);

  const refresh = useCallback(() => {
    setRefreshToken((t) => t + 1);
  }, []);

  return {
    collection,
    query,
    loading,
    error,
    setPage,
    setPageSize,
    setSort,
    setFilter,
    clearFilters,
    refresh,
  };
}
