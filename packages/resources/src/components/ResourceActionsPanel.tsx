import { Button, ConfirmModal, FormField, LoadingState, SpaceBetween } from "@araf/ui";
import { useState, useCallback, useMemo } from "react";
import { validateFormData, type ValidationError } from "@araf/schema-runtime";
import { useCapabilities } from "../hooks/useCapabilities";
import { useResourceAction } from "../hooks/useResourceAction";
import type { ResourceDescriptor } from "../descriptor";
import type { Resource, Operation, ActionDescriptor } from "@araf/api-client";

export interface ResourceActionsPanelProps {
  readonly resource: Resource;
  readonly descriptor: ResourceDescriptor;
  readonly onOperation?: (operation: Operation) => void;
}

interface ModalState {
  open: boolean;
  action: ActionDescriptor | undefined;
  confirmOnly: boolean;
  inputValues: Record<string, unknown>;
  inputErrors: Record<string, string>;
}

const INITIAL_MODAL_STATE: ModalState = {
  open: false,
  action: undefined,
  confirmOnly: false,
  inputValues: {},
  inputErrors: {},
};

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function hasCapability(
  capabilities: readonly { readonly resourceType: string; readonly action: string }[],
  required: { readonly resourceType: string; readonly action: string },
): boolean {
  return capabilities.some(
    (c) => c.resourceType === required.resourceType && c.action === required.action,
  );
}

function needsConfirmation(action: ActionDescriptor): boolean {
  return action.riskClass === "destructive" || action.riskClass === "disruptive";
}

function collectErrors(errors: readonly ValidationError[]): Record<string, string> {
  const map: Record<string, string> = {};
  for (const error of errors) {
    const missingProperty =
      error.keyword === "required" && typeof error.params?.missingProperty === "string"
        ? error.params.missingProperty
        : undefined;
    const key = missingProperty ?? error.instancePath.replace(/^\//u, "");
    if (key && !(key in map)) {
      map[key] = error.message;
    }
  }
  return map;
}

function buildInputSchemaFields(
  schema: unknown,
): { key: string; label: string; required: boolean }[] {
  if (!isPlainObject(schema)) return [];
  const properties = schema.properties;
  if (!isPlainObject(properties)) return [];
  const required = new Set(
    Array.isArray(schema.required)
      ? schema.required.filter((r): r is string => typeof r === "string")
      : [],
  );

  const fields: { key: string; label: string; required: boolean }[] = [];
  for (const [key, property] of Object.entries(properties)) {
    if (!isPlainObject(property)) continue;
    const label = typeof property.title === "string" ? property.title : key;
    fields.push({ key, label, required: required.has(key) });
  }
  return fields;
}

export function ResourceActionsPanel({
  resource,
  descriptor,
  onOperation,
}: ResourceActionsPanelProps) {
  const { capabilities, loading: capabilitiesLoading } = useCapabilities();
  const {
    submit,
    loading: actionLoading,
    error: actionError,
    operation,
  } = useResourceAction(resource.resourceType, resource.id);
  const [modal, setModal] = useState<ModalState>(INITIAL_MODAL_STATE);

  const visibleActions = useMemo(
    () =>
      descriptor.supportedActions.filter((action) =>
        hasCapability(capabilities, action.requiredCapability),
      ),
    [descriptor.supportedActions, capabilities],
  );

  const openModal = useCallback((action: ActionDescriptor) => {
    const hasInputSchema = action.inputSchema !== undefined;
    setModal({
      open: true,
      action,
      confirmOnly: !hasInputSchema,
      inputValues: {},
      inputErrors: {},
    });
  }, []);

  const closeModal = useCallback(() => {
    setModal(INITIAL_MODAL_STATE);
  }, []);

  const executeAction = useCallback(
    async (action: ActionDescriptor, payload?: unknown) => {
      const result = await submit(action.id, payload);
      if (result) {
        onOperation?.(result);
        closeModal();
      }
    },
    [submit, onOperation, closeModal],
  );

  const handleConfirm = useCallback(async () => {
    if (!modal.action) return;

    if (modal.confirmOnly) {
      await executeAction(modal.action);
      return;
    }

    const errors = validateFormData(modal.action.inputSchema, modal.inputValues);
    if (errors.length > 0) {
      setModal((prev) => ({ ...prev, inputErrors: collectErrors(errors) }));
      return;
    }

    await executeAction(modal.action, modal.inputValues);
  }, [modal, executeAction]);

  const setInputValue = useCallback((key: string, value: unknown) => {
    setModal((prev) => ({
      ...prev,
      inputValues: { ...prev.inputValues, [key]: value },
      inputErrors: { ...prev.inputErrors, [key]: "" },
    }));
  }, []);

  if (capabilitiesLoading) {
    return <LoadingState message="Loading actions..." />;
  }

  if (visibleActions.length === 0) {
    return null;
  }

  const modalTitle = modal.action ? modal.action.name : "Confirm action";
  const destructive = modal.action?.riskClass === "destructive";

  return (
    <div style={{ display: "flex", gap: "0.5rem", flexWrap: "wrap" }}>
      {visibleActions.map((action) => (
        <Button
          key={action.id}
          formAction="none"
          variant={action.riskClass === "destructive" ? "normal" : "primary"}
          onClick={() => {
            if (needsConfirmation(action) || action.inputSchema !== undefined) {
              openModal(action);
            } else {
              void executeAction(action);
            }
          }}
        >
          {action.name}
        </Button>
      ))}

      {operation ? (
        <div role="status" style={{ marginTop: "0.5rem" }}>
          Operation <strong>{operation.id}</strong> is <strong>{operation.state}</strong>.
          Correlation ID: {operation.correlationId}
        </div>
      ) : null}

      <ConfirmModal
        open={modal.open}
        title={modalTitle}
        confirmLabel={destructive ? "Delete" : "Confirm"}
        onConfirm={() => {
          void handleConfirm();
        }}
        onCancel={closeModal}
        loading={actionLoading}
      >
        <SpaceBetween size="m" direction="vertical">
          {modal.action ? (
            <p>
              {destructive
                ? `Are you sure you want to ${modal.action.name.toLowerCase()} ${resource.name}? This action cannot be undone.`
                : `Confirm ${modal.action.name.toLowerCase()} for ${resource.name}.`}
            </p>
          ) : null}

          {modal.action?.inputSchema ? (
            <SpaceBetween size="m" direction="vertical">
              {buildInputSchemaFields(modal.action.inputSchema).map((field) => (
                <FormField
                  key={field.key}
                  id={`action-input-${field.key}`}
                  label={field.label}
                  errorText={modal.inputErrors[field.key]}
                >
                  <input
                    id={`action-input-${field.key}`}
                    type="text"
                    value={
                      typeof modal.inputValues[field.key] === "string"
                        ? String(modal.inputValues[field.key])
                        : ""
                    }
                    onChange={(event) => {
                      setInputValue(field.key, event.target.value);
                    }}
                    required={field.required}
                    aria-required={field.required}
                  />
                </FormField>
              ))}
            </SpaceBetween>
          ) : null}

          {actionError ? (
            <div role="alert" style={{ color: "red" }}>
              {actionError instanceof Error ? actionError.message : String(actionError)}
            </div>
          ) : null}
        </SpaceBetween>
      </ConfirmModal>
    </div>
  );
}
