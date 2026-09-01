import { useCallback, useEffect, useRef, useState } from "react";

export interface OperationTransport {
  startPolling: () => void;
  stopPolling: () => void;
  isPolling: boolean;
}

const DEFAULT_INTERVAL_MS = 3_000;
const MAX_BACKOFF_MS = 60_000;

/**
 * Start polling the given refresh callback at a bounded interval.
 *
 * On success the interval resets to the default. On error the interval doubles
 * up to a maximum backoff to avoid hammering a struggling BFF. This abstraction
 * can later be replaced with an SSE-based transport without changing consumers.
 */
export function startPolling(
  refresh: () => Promise<void> | void,
  intervalMs = DEFAULT_INTERVAL_MS,
): { stop: () => void } {
  let currentInterval = intervalMs;
  let intervalId: ReturnType<typeof setInterval> | undefined;

  async function tick(): Promise<void> {
    try {
      await refresh();
      currentInterval = intervalMs;
    } catch {
      currentInterval = Math.min(currentInterval * 2, MAX_BACKOFF_MS);
    }

    if (intervalId) {
      clearInterval(intervalId);
    }
    intervalId = setInterval(() => {
      void tick();
    }, currentInterval);
  }

  intervalId = setInterval(() => {
    void tick();
  }, currentInterval);

  return {
    stop: () => {
      if (intervalId) {
        clearInterval(intervalId);
        intervalId = undefined;
      }
    },
  };
}

/**
 * Stop a polling session started by {@link startPolling}.
 *
 * Exported as a no-op helper for symmetry; callers typically keep the `stop`
 * function returned by `startPolling`.
 */
export function stopPolling(session: { stop: () => void } | undefined): void {
  session?.stop();
}

/**
 * Hook that polls a refresh callback while `enabled` is true.
 *
 * The refresh callback should return a Promise that resolves when fresh data is
 * available. Errors are swallowed and trigger exponential backoff up to 60s.
 */
export function useOperationTransport(
  refresh: () => Promise<void> | void,
  enabled: boolean,
  intervalMs = DEFAULT_INTERVAL_MS,
): OperationTransport {
  const [isPolling, setIsPolling] = useState(false);
  const sessionRef = useRef<{ stop: () => void } | undefined>(undefined);

  const start = useCallback(() => {
    if (sessionRef.current) return;
    setIsPolling(true);
    sessionRef.current = startPolling(refresh, intervalMs);
  }, [refresh, intervalMs]);

  const stop = useCallback(() => {
    stopPolling(sessionRef.current);
    sessionRef.current = undefined;
    setIsPolling(false);
  }, []);

  useEffect(() => {
    if (enabled) {
      start();
    } else {
      stop();
    }
    return stop;
  }, [enabled, start, stop]);

  return {
    startPolling: start,
    stopPolling: stop,
    isPolling,
  };
}
