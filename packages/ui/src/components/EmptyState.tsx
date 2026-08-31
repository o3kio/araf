import { Box, Button, SpaceBetween } from "@cloudscape-design/components";
import type { ReactNode } from "react";

export interface EmptyStateProps {
  readonly title?: string;
  readonly description?: string;
  readonly action?: { readonly label: string; readonly onPress: () => void };
  readonly children?: ReactNode;
}

export function EmptyState({ title, description, action, children }: EmptyStateProps) {
  return (
    <div role="status" aria-live="polite">
      <Box textAlign="center" color="text-body-secondary" padding={{ vertical: "xxl" }}>
        <SpaceBetween size="s" alignItems="center">
          {title ? <Box variant="h3">{title}</Box> : null}
          {description ? <Box variant="p">{description}</Box> : null}
          {action ? <Button onClick={action.onPress}>{action.label}</Button> : null}
          {children}
        </SpaceBetween>
      </Box>
    </div>
  );
}
