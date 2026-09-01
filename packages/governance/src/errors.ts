import type { ArafApiError } from "@araf/api-client";

export function errorMessage(error: Error | ArafApiError | undefined): string | undefined {
  if (!error) return undefined;
  if ("problem" in error) {
    return error.problem.detail;
  }
  return error.message;
}

export function errorCorrelationId(error: Error | ArafApiError | undefined): string | undefined {
  if (error && "correlationId" in error) {
    return error.correlationId;
  }
  return undefined;
}
