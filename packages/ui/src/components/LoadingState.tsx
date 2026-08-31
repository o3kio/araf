import { Box, Spinner, SpaceBetween } from "@cloudscape-design/components";

export interface LoadingStateProps {
  readonly message?: string;
}

export function LoadingState({ message = "Loading..." }: LoadingStateProps) {
  return (
    <div role="status" aria-live="polite">
      <Box textAlign="center" padding={{ vertical: "xxl" }}>
        <SpaceBetween size="m" alignItems="center">
          <Spinner size="big" aria-label={message} />
          <Box variant="p" color="text-body-secondary">
            {message}
          </Box>
        </SpaceBetween>
      </Box>
    </div>
  );
}
