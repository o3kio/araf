import { StatusIndicator } from "@araf/ui";
import type { OperationState } from "@araf/api-client";

const stateToStatus: Record<
  OperationState,
  { type: "pending" | "info" | "success" | "error" | "warning"; label: string }
> = {
  pending: { type: "pending", label: "Pending" },
  running: { type: "info", label: "Running" },
  succeeded: { type: "success", label: "Succeeded" },
  failed: { type: "error", label: "Failed" },
  retryable: { type: "warning", label: "Retryable" },
  unknownOutcome: { type: "warning", label: "Unknown outcome" },
};

export interface OperationStatusProps {
  state: OperationState;
}

/**
 * Render a canonical Operation state as a status indicator.
 */
export function OperationStatus({ state }: OperationStatusProps) {
  const { type, label } = stateToStatus[state];
  return <StatusIndicator type={type}>{label}</StatusIndicator>;
}
