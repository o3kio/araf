import { useCallback, useState } from "react";
import type { ToastItem } from "./Toast";

/**
 * Convenience hook for managing toast state locally.
 */
export function useToastState() {
  const [toasts, setToasts] = useState<ToastItem[]>([]);
  const addToast = useCallback((item: Omit<ToastItem, "id"> & { readonly id?: string }) => {
    const id = item.id ?? `toast-${String(Date.now())}-${Math.random().toString(36).slice(2, 8)}`;
    setToasts((prev) => [...prev, { ...item, id }]);
  }, []);
  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);
  return { toasts, addToast, removeToast };
}
