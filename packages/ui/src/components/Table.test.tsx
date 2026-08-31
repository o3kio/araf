import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { runAxe } from "../test/axe-helper";
import { describe, expect, it, vi } from "vitest";
import { Table } from "./Table";

interface Resource {
  id: string;
  name: string;
  status: string;
}

describe("Table", () => {
  const items: Resource[] = [
    { id: "r1", name: "alpha", status: "ready" },
    { id: "r2", name: "beta", status: "busy" },
  ];

  const columnDefinitions = [
    { id: "name", header: "Name", cell: (item: Resource) => item.name },
    { id: "status", header: "Status", cell: (item: Resource) => item.status },
  ];

  it("renders column headers and rows", () => {
    render(<Table items={items} columnDefinitions={columnDefinitions} />);
    expect(screen.getByRole("columnheader", { name: "Name" })).toBeVisible();
    expect(screen.getByRole("cell", { name: "alpha" })).toBeVisible();
    expect(screen.getByRole("cell", { name: "beta" })).toBeVisible();
  });

  it("uses trackingId as trackBy for selection keys", () => {
    render(
      <Table
        items={items}
        columnDefinitions={columnDefinitions}
        trackingId="id"
        onSelectionChange={vi.fn()}
      />,
    );
    const rows = screen.getAllByRole("row");
    // Header + 2 data rows.
    expect(rows).toHaveLength(3);
  });

  it("notifies selection changes with the selected item", async () => {
    const onSelectionChange = vi.fn();
    render(
      <Table
        items={items}
        columnDefinitions={columnDefinitions}
        trackingId="id"
        onSelectionChange={onSelectionChange}
      />,
    );
    const checkboxes = screen.getAllByRole("checkbox");
    expect(checkboxes[0]).toBeDefined();
    if (checkboxes[0]) {
      await userEvent.click(checkboxes[0]);
    }
    expect(onSelectionChange).toHaveBeenCalled();
    const firstCall = onSelectionChange.mock.calls[0];
    expect(firstCall).toBeDefined();
    if (firstCall) {
      expect(firstCall[0]).toContainEqual(items[0]);
    }
  });

  it("has no accessibility violations", async () => {
    const { container } = render(
      <Table
        items={items}
        columnDefinitions={columnDefinitions}
        trackingId="id"
        ariaLabels={{
          selectionGroupLabel: "Resource selection",
          allItemsSelectionLabel: "Select all resources",
          itemSelectionLabel: () => "Select resource",
        }}
      />,
    );
    expect(await runAxe(container)).toHaveNoViolations();
  });
});
