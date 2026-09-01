import { ArafApiError } from "@araf/api-client";

export function errorMessage(error: unknown): string {
  if (error instanceof ArafApiError) {
    return error.problem.detail || error.message;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

export function errorCorrelationId(error: unknown): string | undefined {
  if (error instanceof ArafApiError) {
    return error.correlationId;
  }
  return undefined;
}
