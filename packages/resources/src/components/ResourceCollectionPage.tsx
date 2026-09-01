import {
  Table,
  Header,
  LoadingState,
  EmptyState,
  ErrorState,
  StatusIndicator,
  type TableColumnDefinition,
} from "@araf/ui";
import { Link } from "react-router";
import { useResourceCollection } from "../hooks/useResourceCollection";
import { useResourceDescriptor } from "../hooks/useResourceDescriptor";
import { getResourceField, formatResourceField } from "../fields";
import { mapResourceStatus } from "../status";
import type { Resource, SortDirection } from "@araf/api-client";
import type { ResourceDescriptor } from "../descriptor";
import type { CollectionQuery } from "../hooks/useResourceCollection";
import { errorMessage, errorCorrelationId } from "../errors";

export interface ResourceCollectionPageProps {
  resourceType: string;
}

/**
 * Generic resource collection page rendered entirely from the resource descriptor.
 *
 * Supports server-side pagination, descriptor-driven filters, sorting, status
 * indicators, and row links to the resource detail page.
 */
export function ResourceCollectionPage({ resourceType }: ResourceCollectionPageProps) {
  const {
    descriptor,
    loading: descriptorLoading,
    error: descriptorError,
  } = useResourceDescriptor(resourceType);
  const {
    collection,
    query,
    loading: collectionLoading,
    error: collectionError,
    setPage,
    setPageSize,
    setSort,
    setFilter,
    clearFilters,
  } = useResourceCollection(resourceType, descriptor);

  const error = descriptorError ?? collectionError;
  const loading = descriptorLoading || collectionLoading;

  const columns: TableColumnDefinition<Resource>[] =
    descriptor?.columns.map((column) => ({
      id: column.id,
      header: column.header,
      width: column.width,
      cell: (resource) => {
        if (column.field === "status") {
          const { type, label } = mapResourceStatus(resource.status);
          return <StatusIndicator type={type}>{label}</StatusIndicator>;
        }

        const value = getResourceField(resource, column.field);
        const display = formatResourceField(value);

        if (column.field === "name") {
          return (
            <Link
              to={`/resources/${encodeURIComponent(resourceType)}/${encodeURIComponent(resource.id)}`}
            >
              {display}
            </Link>
          );
        }

        return <span>{display}</span>;
      },
    })) ?? [];

  const totalPages = collection ? Math.ceil(collection.total / collection.pageSize) : 0;

  return (
    <section aria-label={`${descriptor?.pluralName ?? resourceType} collection`}>
      <Header
        variant="h1"
        headingLevel="h1"
        description={`Manage ${descriptor?.pluralName ?? resourceType}`}
        counter={collection ? `(${String(collection.total)})` : undefined}
      >
        {descriptor?.pluralName ?? resourceType}
      </Header>

      {error ? (
        <ErrorState
          title="Could not load resources"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
        />
      ) : null}

      {!error && loading && !collection ? <LoadingState message="Loading resources..." /> : null}

      {!error && descriptor && (
        <>
          <CollectionControls
            descriptor={descriptor}
            query={query}
            onSetSort={setSort}
            onSetFilter={setFilter}
            onClearFilters={clearFilters}
          />

          <Table<Resource>
            items={collection?.items ?? []}
            columnDefinitions={columns}
            trackingId="id"
            loading={loading}
            loadingText="Loading resources..."
            empty={
              <EmptyState
                title="No resources"
                description={`There are no ${descriptor.pluralName} matching the current scope and filters.`}
              />
            }
            ariaLabels={{
              tableLabel: `${descriptor.pluralName} table`,
            }}
          />

          {collection && totalPages > 0 ? (
            <PaginationControls
              page={collection.page + 1}
              pageSize={collection.pageSize}
              totalPages={totalPages}
              hasMore={collection.hasMore}
              total={collection.total}
              onPageChange={setPage}
              onPageSizeChange={setPageSize}
            />
          ) : null}
        </>
      )}
    </section>
  );
}

interface CollectionControlsProps {
  descriptor: ResourceDescriptor;
  query: CollectionQuery;
  onSetSort: (field: string | undefined, direction: SortDirection) => void;
  onSetFilter: (filterId: string, value: string) => void;
  onClearFilters: () => void;
}

function CollectionControls({
  descriptor,
  query,
  onSetSort,
  onSetFilter,
  onClearFilters,
}: CollectionControlsProps) {
  return (
    <div
      className="araf-collection-controls"
      style={{ display: "flex", gap: "1rem", margin: "1rem 0", flexWrap: "wrap" }}
    >
      {descriptor.sortableFields.length > 0 ? (
        <label>
          Sort by:{" "}
          <select
            aria-label="Sort by"
            value={query.sortField ?? ""}
            onChange={(e) => {
              const field = e.target.value || undefined;
              onSetSort(field, query.sortDirection);
            }}
          >
            <option value="">Default</option>
            {descriptor.sortableFields.map((field) => (
              <option key={field} value={field}>
                {field}
              </option>
            ))}
          </select>
        </label>
      ) : null}

      {descriptor.sortableFields.length > 0 && query.sortField ? (
        <label>
          Direction:{" "}
          <select
            aria-label="Sort direction"
            value={query.sortDirection}
            onChange={(e) => {
              onSetSort(query.sortField, e.target.value as SortDirection);
            }}
          >
            <option value="asc">Ascending</option>
            <option value="desc">Descending</option>
          </select>
        </label>
      ) : null}

      {descriptor.filters.map((filter) => (
        <label key={filter.id}>
          {filter.label}:{" "}
          <input
            type={filter.kind === "text" ? "search" : "text"}
            aria-label={filter.label}
            value={query.filters[filter.id] ?? ""}
            onChange={(e) => {
              onSetFilter(filter.id, e.target.value);
            }}
            placeholder={filter.kind === "select" ? "Filter..." : "Search..."}
          />
        </label>
      ))}

      {descriptor.filters.length > 0 ? (
        <button type="button" onClick={onClearFilters}>
          Clear filters
        </button>
      ) : null}
    </div>
  );
}

interface PaginationControlsProps {
  page: number;
  pageSize: number;
  totalPages: number;
  hasMore: boolean;
  total: number;
  onPageChange: (page: number) => void;
  onPageSizeChange: (pageSize: number) => void;
}

function PaginationControls({
  page,
  pageSize,
  totalPages,
  hasMore,
  total,
  onPageChange,
  onPageSizeChange,
}: PaginationControlsProps) {
  return (
    <nav
      aria-label="Pagination"
      className="araf-pagination-controls"
      style={{ display: "flex", gap: "1rem", alignItems: "center", marginTop: "1rem" }}
    >
      <button
        type="button"
        disabled={page <= 1}
        onClick={() => {
          onPageChange(page - 1);
        }}
      >
        Previous
      </button>
      <span>
        Page {page} of {totalPages} ({total} total)
      </span>
      <button
        type="button"
        disabled={!hasMore}
        onClick={() => {
          onPageChange(page + 1);
        }}
      >
        Next
      </button>
      <label>
        Page size:{" "}
        <select
          aria-label="Page size"
          value={pageSize}
          onChange={(e) => {
            onPageSizeChange(Number.parseInt(e.target.value, 10));
          }}
        >
          <option value={10}>10</option>
          <option value={25}>25</option>
          <option value={50}>50</option>
          <option value={100}>100</option>
        </select>
      </label>
    </nav>
  );
}
