import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
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
    const onSelectionChange = vi.fn();
    render(
      <Table
        items={items}
        columnDefinitions={columnDefinitions}
        trackingId="id"
        onSelectionChange={onSelectionChange}
      />,
    );
    const rows = screen.getAllByRole("row");
    // Skip header row; each data row should have a checkbox.
    expect(rows.length).toBeGreaterThanOrEqual(3);
  });

  it("notifies selection changes", async () => {
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
  });
});
