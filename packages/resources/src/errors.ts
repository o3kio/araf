/**
 * Safely extract a display message and correlation id from an unknown error.
 */
export function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (
    typeof error === "object" &&
    error !== null &&
    "message" in error &&
    typeof error.message === "string"
  ) {
    return (error as { message: string }).message;
  }
  return String(error);
}

export function errorCorrelationId(error: unknown): string | undefined {
  if (
    error instanceof Error &&
    "correlationId" in error &&
    typeof (error as { correlationId: unknown }).correlationId === "string"
  ) {
    return (error as { correlationId: string }).correlationId;
  }
  return undefined;
}
