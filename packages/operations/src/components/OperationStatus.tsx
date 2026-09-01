import { StatusIndicator } from "@araf/ui";
import type { OperationState } from "@araf/api-client";

const stateToStatus: Record<
  OperationState,
  { type: "pending" | "info" | "success" | "error"; label: string }
> = {
  pending: { type: "pending", label: "Pending" },
  running: { type: "info", label: "Running" },
  succeeded: { type: "success", label: "Succeeded" },
  failed: { type: "error", label: "Failed" },
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
