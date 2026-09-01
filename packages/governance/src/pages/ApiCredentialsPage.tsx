import {
  Header,
  Table,
  LoadingState,
  EmptyState,
  ErrorState,
  Button,
  ConfirmModal,
  type TableColumnDefinition,
} from "@araf/ui";
import { useSearchParams } from "react-router";
import { useCapabilities } from "@araf/resources";
import type { ApiCredential } from "@araf/api-client";
import { useApiCredentials } from "../hooks/useApiCredentials";
import { useCreateApiCredential } from "../hooks/useCreateApiCredential";
import { useDeleteApiCredential } from "../hooks/useDeleteApiCredential";
import { hasCapability } from "../capabilities";
import { PaginationControls } from "../components/PaginationControls";
import { errorMessage, errorCorrelationId } from "../errors";
import { clampPageSize, DEFAULT_PAGE, DEFAULT_PAGE_SIZE, parseIntOr } from "../pagination";
import { useCallback, useState } from "react";

const PAGE_PARAM = "page";
const PAGE_SIZE_PARAM = "pageSize";

function formatTimestamp(iso: string | null): string {
  if (!iso) return "—";
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

export function ApiCredentialsPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const { capabilities, loading: capabilitiesLoading } = useCapabilities();

  const page = Math.max(parseIntOr(searchParams.get(PAGE_PARAM), DEFAULT_PAGE), 1);
  const pageSize = clampPageSize(parseIntOr(searchParams.get(PAGE_SIZE_PARAM), DEFAULT_PAGE_SIZE));

  const {
    collection,
    loading: listLoading,
    error: listError,
    refresh,
  } = useApiCredentials({
    page: page - 1,
    pageSize,
  });
  const {
    create,
    loading: createLoading,
    error: createError,
    reset: resetCreate,
  } = useCreateApiCredential();
  const {
    deleteCredential,
    loading: deleteLoading,
    error: deleteError,
    reset: resetDelete,
  } = useDeleteApiCredential();

  const canList = hasCapability(capabilities, "tenant.api-credential", "list");
  const canCreate = hasCapability(capabilities, "tenant.api-credential", "create");
  const canDelete = hasCapability(capabilities, "tenant.api-credential", "delete");

  const [isCreating, setIsCreating] = useState(false);
  const [newCredential, setNewCredential] = useState<ApiCredential | undefined>(undefined);
  const [deleteTarget, setDeleteTarget] = useState<ApiCredential | undefined>(undefined);

  const setPage = (nextPage: number) => {
    setSearchParams((prev) => {
      const next = new URLSearchParams(prev);
      if (nextPage === DEFAULT_PAGE) {
        next.delete(PAGE_PARAM);
      } else {
        next.set(PAGE_PARAM, String(nextPage));
      }
      return next;
    });
  };

  const setPageSize = (nextPageSize: number) => {
    setSearchParams((prev) => {
      const next = new URLSearchParams(prev);
      next.delete(PAGE_PARAM);
      next.set(PAGE_SIZE_PARAM, String(clampPageSize(nextPageSize)));
      return next;
    });
  };

  const handleCreate = useCallback(
    async (payload: { name: string; kind: string; projectId: string }) => {
      resetCreate();
      const credential = await create({
        name: payload.name,
        kind: payload.kind,
        projectId: payload.projectId,
      });
      setNewCredential(credential);
      setIsCreating(false);
      refresh();
    },
    [create, refresh, resetCreate],
  );

  const dismissSecret = useCallback(() => {
    setNewCredential(undefined);
  }, []);

  const handleDelete = useCallback(
    async (credential: ApiCredential) => {
      resetDelete();
      await deleteCredential(credential.id);
      setDeleteTarget(undefined);
      refresh();
    },
    [deleteCredential, refresh, resetDelete],
  );

  const columns: TableColumnDefinition<ApiCredential>[] = [
    { id: "name", header: "Name", cell: (cred) => cred.name, isRowHeader: true },
    { id: "kind", header: "Kind", cell: (cred) => cred.kind },
    { id: "project", header: "Project", cell: (cred) => cred.projectId },
    {
      id: "created",
      header: "Created",
      cell: (cred) => formatTimestamp(cred.createdAt),
    },
    {
      id: "expires",
      header: "Expires",
      cell: (cred) => formatTimestamp(cred.expiresAt),
    },
    {
      id: "actions",
      header: "Actions",
      cell: (cred) =>
        canDelete ? (
          <Button
            variant="normal"
            onClick={() => {
              setDeleteTarget(cred);
            }}
            disabled={deleteLoading}
          >
            Delete
          </Button>
        ) : null,
    },
  ];

  const totalPages = collection ? Math.ceil(collection.total / collection.pageSize) : 0;

  const error = listError ?? createError ?? deleteError;

  return (
    <section aria-label="API credentials">
      <Header
        variant="h1"
        headingLevel="h1"
        actions={
          canCreate ? (
            <Button
              variant="primary"
              onClick={() => {
                setIsCreating(true);
              }}
            >
              Create credential
            </Button>
          ) : undefined
        }
      >
        API Credentials
      </Header>

      {capabilitiesLoading || listLoading ? (
        <LoadingState message="Loading API credentials..." />
      ) : null}

      {!canList && !capabilitiesLoading ? (
        <ErrorState
          title="Access denied"
          message="You do not have permission to list API credentials."
        />
      ) : null}

      {error ? (
        <ErrorState
          title="Could not process API credential request"
          message={errorMessage(error)}
          correlationId={errorCorrelationId(error)}
        />
      ) : null}

      {canList && !error && (
        <>
          <Table<ApiCredential>
            items={collection?.items ?? []}
            columnDefinitions={columns}
            trackingId="id"
            loading={listLoading}
            loadingText="Loading API credentials..."
            empty={
              <EmptyState
                title="No API credentials"
                description="There are no API credentials for the current scope."
              />
            }
            ariaLabels={{ tableLabel: "API credentials table" }}
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

      {isCreating && (
        <CreateCredentialModal
          onDismiss={() => {
            setIsCreating(false);
          }}
          onCreate={handleCreate}
          loading={createLoading}
        />
      )}

      {newCredential?.secret && (
        <SecretRevealModal credential={newCredential} onDismiss={dismissSecret} />
      )}

      {deleteTarget && (
        <ConfirmModal
          open
          title="Delete API credential"
          confirmLabel="Delete"
          cancelLabel="Cancel"
          onConfirm={() => {
            void handleDelete(deleteTarget);
          }}
          onCancel={() => {
            setDeleteTarget(undefined);
          }}
        >
          Delete the credential <strong>{deleteTarget.name}</strong>? This action cannot be undone.
        </ConfirmModal>
      )}
    </section>
  );
}

function CreateCredentialModal({
  onDismiss,
  onCreate,
  loading,
}: {
  onDismiss: () => void;
  onCreate: (payload: { name: string; kind: string; projectId: string }) => Promise<void>;
  loading: boolean;
}) {
  const [name, setName] = useState("");
  const [kind, setKind] = useState("service-account");
  const [projectId, setProjectId] = useState("");

  const isValid = name.trim() !== "" && kind.trim() !== "" && projectId.trim() !== "";

  return (
    <ConfirmModal
      open
      title="Create API credential"
      confirmLabel="Create"
      cancelLabel="Cancel"
      onConfirm={() => {
        if (!isValid) return;
        void onCreate({ name: name.trim(), kind: kind.trim(), projectId: projectId.trim() });
      }}
      onCancel={onDismiss}
      loading={loading}
      confirmDisabled={!isValid}
    >
      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (!isValid) return;
          void onCreate({
            name: name.trim(),
            kind: kind.trim(),
            projectId: projectId.trim(),
          });
        }}
        style={{ display: "flex", flexDirection: "column", gap: "1rem" }}
      >
        <label style={{ display: "flex", flexDirection: "column", gap: "0.25rem" }}>
          Name
          <input
            type="text"
            aria-label="Name"
            value={name}
            onChange={(e) => {
              setName(e.target.value);
            }}
            required
          />
        </label>
        <label style={{ display: "flex", flexDirection: "column", gap: "0.25rem" }}>
          Kind
          <input
            type="text"
            aria-label="Kind"
            value={kind}
            onChange={(e) => {
              setKind(e.target.value);
            }}
            required
          />
        </label>
        <label style={{ display: "flex", flexDirection: "column", gap: "0.25rem" }}>
          Project ID
          <input
            type="text"
            aria-label="Project ID"
            value={projectId}
            onChange={(e) => {
              setProjectId(e.target.value);
            }}
            required
          />
        </label>
      </form>
    </ConfirmModal>
  );
}

function SecretRevealModal({
  credential,
  onDismiss,
}: {
  credential: ApiCredential;
  onDismiss: () => void;
}) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    if (!credential.secret) return;
    try {
      await navigator.clipboard.writeText(credential.secret);
      setCopied(true);
    } catch {
      setCopied(false);
    }
  };

  return (
    <ConfirmModal
      open
      title="API credential created"
      confirmLabel="Done"
      onConfirm={onDismiss}
      onCancel={onDismiss}
    >
      <div role="status" aria-live="assertive">
        <p>Copy the secret now. It will not be shown again.</p>
        <div
          style={{
            display: "flex",
            gap: "0.5rem",
            alignItems: "center",
            margin: "1rem 0",
          }}
        >
          <input
            type="text"
            readOnly
            aria-label="API credential secret"
            value={credential.secret}
            style={{ flex: 1, fontFamily: "monospace" }}
            onFocus={(e) => {
              e.target.select();
            }}
          />
          <Button
            variant="primary"
            onClick={() => {
              void handleCopy();
            }}
          >
            Copy
          </Button>
        </div>
        {copied ? <p role="alert">Secret copied to clipboard.</p> : null}
      </div>
    </ConfirmModal>
  );
}
