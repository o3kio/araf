import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { OperationStatus } from "./OperationStatus";
import type { OperationState } from "@araf/api-client";

describe("OperationStatus", () => {
  const cases: { state: OperationState; label: string }[] = [
    { state: "pending", label: "Pending" },
    { state: "running", label: "Running" },
    { state: "succeeded", label: "Succeeded" },
    { state: "failed", label: "Failed" },
    { state: "retryable", label: "Retryable" },
    { state: "unknownOutcome", label: "Unknown outcome" },
  ];

  it.each(cases)("renders $label for $state", ({ state, label }) => {
    render(<OperationStatus state={state} />);
    expect(screen.getByText(label)).toBeInTheDocument();
  });
});
