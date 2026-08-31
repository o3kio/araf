import { Alert, Button, SpaceBetween } from "@cloudscape-design/components";

export interface ErrorStateProps {
  readonly title?: string;
  readonly message?: string;
  readonly onRetry?: () => void;
  readonly correlationId?: string;
}

export function ErrorState({ title = "Error", message, onRetry, correlationId }: ErrorStateProps) {
  return (
    <Alert
      type="error"
      header={title}
      action={onRetry ? <Button onClick={onRetry}>Retry</Button> : undefined}
    >
      <SpaceBetween size="xs" direction="vertical">
        {message ? <span>{message}</span> : null}
        {correlationId ? (
          <span style={{ fontFamily: "monospace", fontSize: "0.75rem" }}>
            Correlation ID: {correlationId}
          </span>
        ) : null}
      </SpaceBetween>
    </Alert>
  );
}
