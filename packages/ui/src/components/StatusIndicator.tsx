import { StatusIndicator as CloudscapeStatusIndicator } from "@cloudscape-design/components";
import type { StatusIndicatorProps as CloudscapeStatusIndicatorProps } from "@cloudscape-design/components";

export type ArafStatusType = "success" | "error" | "warning" | "info" | "pending";

export interface StatusIndicatorProps {
  readonly type: ArafStatusType;
  readonly children: string;
}

const typeMap: Record<ArafStatusType, CloudscapeStatusIndicatorProps["type"]> = {
  success: "success",
  error: "error",
  warning: "warning",
  info: "info",
  pending: "pending",
};

export function StatusIndicator({ type, children }: StatusIndicatorProps) {
  return <CloudscapeStatusIndicator type={typeMap[type]}>{children}</CloudscapeStatusIndicator>;
}
