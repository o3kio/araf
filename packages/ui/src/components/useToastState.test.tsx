import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useToastState } from "./useToastState";

describe("useToastState", () => {
  it("starts with an empty toast list", () => {
    const { result } = renderHook(() => useToastState());
    expect(result.current.toasts).toHaveLength(0);
  });

  it("adds a toast with a generated id", () => {
    const { result } = renderHook(() => useToastState());
    act(() => {
      result.current.addToast({ type: "info", message: "Hello" });
    });
    expect(result.current.toasts).toHaveLength(1);
    expect(result.current.toasts[0]?.message).toBe("Hello");
    expect(result.current.toasts[0]?.id).toBeDefined();
  });

  it("respects a provided id", () => {
    const { result } = renderHook(() => useToastState());
    act(() => {
      result.current.addToast({ id: "my-id", type: "success", message: "Done" });
    });
    expect(result.current.toasts[0]?.id).toBe("my-id");
  });

  it("removes a toast by id", () => {
    const { result } = renderHook(() => useToastState());
    act(() => {
      result.current.addToast({ id: "a", type: "warning", message: "Careful" });
    });
    act(() => {
      result.current.removeToast("a");
    });
    expect(result.current.toasts).toHaveLength(0);
  });
});
