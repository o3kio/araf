import {
  Header,
  Table,
  LoadingState,
  EmptyState,
  ErrorState,
  type TableColumnDefinition,
} from "@araf/ui";
import { Link } from "react-router";
import type { CustomerAccount } from "@araf/api-client";
import { useCustomerAccounts } from "../hooks/useCustomerAccounts";
import { errorMessage, errorCorrelationId } from "../errors";

function formatTimestamp(iso: string): string {
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

export function AccountsPage() {
  const { collection, loading, error, refresh } = useCustomerAccounts();

  const columns: TableColumnDefinition<CustomerAccount>[] = [
    {
      id: "name",
      header: "Name",
      cell: (account) => (
        <Link to={`/customers/accounts/${encodeURIComponent(account.id)}/projects`}>
          {account.name}
        </Link>
      ),
      isRowHeader: true,
    },
    { id: "id", header: "ID", cell: (account) => account.id },
    { id: "status", header: "Status", cell: (account) => account.status },
    {
      id: "created",
      header: "Created",
      cell: (account) => formatTimestamp(account.createdAt),
    },
  ];

  return (
    <section aria-label="Customer accounts">
      <Header variant="h1" headingLevel="h1">
        Accounts
      </Header>

      {error ? (
        <ErrorState
          title="Could not load accounts"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
          onRetry={refresh}
        />
      ) : null}

      {loading && !collection ? <LoadingState message="Loading accounts..." /> : null}

      {!error && (
        <Table<CustomerAccount>
          items={collection?.items ?? []}
          columnDefinitions={columns}
          trackingId="id"
          loading={loading}
          loadingText="Loading accounts..."
          empty={
            <EmptyState
              title="No accounts"
              description="There are no customer accounts visible to you."
            />
          }
          ariaLabels={{ tableLabel: "Customer accounts table" }}
        />
      )}
    </section>
  );
}
